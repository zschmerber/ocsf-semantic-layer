import type {
  ReferenceEvent,
  InterpretedMapping,
  SuggestedModel,
  SuggestedAttribute,
  SuggestedMetric,
  AttributeTier,
} from '../types/referenceEvent';
import type { SchemaTree, AttributeNode } from '../types';

// ============================================
// Helpers
// ============================================

/**
 * Derive an entity name from the event class name.
 * e.g. "DNS Activity" → "dns_activity_entity"
 */
function deriveEntityName(className: string): string {
  return (
    className
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '_')
      .replace(/^_|_$/g, '') + '_entity'
  );
}

/**
 * Get the last segment of a dot-notation path.
 */
function lastSegment(path: string): string {
  const parts = path.split('.');
  return parts[parts.length - 1];
}

/**
 * Build a flat map of field path → AttributeNode from the schema class attributes.
 * Recursively walks children to produce dot-notation paths.
 */
function flattenSchemaAttributes(
  attributes: AttributeNode[],
  prefix: string = ''
): Map<string, AttributeNode> {
  const result = new Map<string, AttributeNode>();
  for (const attr of attributes) {
    const path = prefix ? `${prefix}.${attr.name}` : attr.name;
    result.set(path, attr);
    if (attr.children && attr.children.length > 0) {
      const childMap = flattenSchemaAttributes(attr.children, path);
      for (const [childPath, childNode] of childMap) {
        result.set(childPath, childNode);
      }
    }
  }
  return result;
}

/**
 * Find the class node in the schema tree by class UID.
 */
function findClassAttributes(
  schemaTree: SchemaTree,
  classUid: number
): AttributeNode[] | null {
  for (const category of schemaTree.categories) {
    for (const cls of category.classes) {
      if (cls.uid === classUid) {
        return cls.attributes;
      }
    }
  }
  return null;
}

/**
 * Determine the tier for an attribute that IS present in the reference event.
 * - Core: required or recommended in schema (or confirmed by mapping)
 * - Extended: optional in schema with weak/no mapping support
 */
function classifyObservedAttribute(
  schemaAttr: AttributeNode | undefined,
  mapping: InterpretedMapping | null,
  fieldPath: string
): AttributeTier {
  // If mapping confirms this field with High confidence → Core
  if (mapping) {
    const entry = mapping.entries.find((e) => e.ocsfField === fieldPath);
    if (entry && entry.confidence === 'High') {
      return 'core';
    }
  }

  if (schemaAttr) {
    if (schemaAttr.requirement === 'required' || schemaAttr.requirement === 'recommended') {
      return 'core';
    }
    // optional in schema → extended
    return 'extended';
  }

  // Not in schema at all but present in event → extended
  return 'extended';
}

// ============================================
// Metric suggestion helpers
// ============================================

const STATUS_FIELD_PATTERNS = ['status_id', 'rcode_id', 'disposition_id'];

function isStatusField(fieldPath: string): boolean {
  const name = lastSegment(fieldPath);
  return STATUS_FIELD_PATTERNS.some((p) => name.includes(p));
}

function humanize(fieldPath: string): string {
  return lastSegment(fieldPath).replace(/_/g, ' ').replace(/\bid\b/g, '').trim();
}

function suggestNumericMetrics(fieldPath: string): SuggestedMetric[] {
  const label = humanize(fieldPath);
  return [
    {
      name: `${lastSegment(fieldPath)}_count`,
      description: `Total count of events with ${label}`,
      aggregation: 'count',
      fieldMeasure: fieldPath,
      dimensions: [],
      timeGranularities: ['hour', 'day'],
      confidence: 'High',
    },
    {
      name: `${lastSegment(fieldPath)}_sum`,
      description: `Sum of ${label} values`,
      aggregation: 'sum',
      fieldMeasure: fieldPath,
      dimensions: [],
      timeGranularities: ['hour', 'day'],
      confidence: 'High',
    },
    {
      name: `${lastSegment(fieldPath)}_avg`,
      description: `Average ${label} per time window`,
      aggregation: 'avg',
      fieldMeasure: fieldPath,
      dimensions: [],
      timeGranularities: ['hour', 'day'],
      confidence: 'High',
    },
  ];
}

function suggestTimestampMetrics(fieldPath: string): SuggestedMetric[] {
  const label = humanize(fieldPath) || 'event';
  return [
    {
      name: `${lastSegment(fieldPath)}_by_granularity`,
      description: `${capitalize(label)} count by time granularity (minute, hour, day)`,
      aggregation: 'count',
      fieldMeasure: fieldPath,
      dimensions: [],
      timeGranularities: ['minute', 'hour', 'day'],
      confidence: 'Medium',
    },
    {
      name: `event_rate`,
      description: `Events per time window based on ${label}`,
      aggregation: 'count',
      fieldMeasure: fieldPath,
      dimensions: [],
      timeGranularities: ['minute', 'hour'],
      confidence: 'Medium',
    },
  ];
}

function suggestCardinalityMetric(fieldPath: string): SuggestedMetric {
  const label = humanize(fieldPath);
  return {
    name: `unique_${lastSegment(fieldPath)}`,
    description: `Unique ${label} values per hour`,
    aggregation: 'count_distinct',
    fieldMeasure: fieldPath,
    dimensions: [],
    timeGranularities: ['hour', 'day'],
    confidence: 'Medium',
  };
}

function suggestRatioMetric(fieldPath: string): SuggestedMetric {
  const label = humanize(fieldPath);
  return {
    name: `${lastSegment(fieldPath)}_ratio`,
    description: `Ratio distribution of ${label} values`,
    aggregation: 'ratio',
    fieldMeasure: fieldPath,
    dimensions: [],
    timeGranularities: ['hour', 'day'],
    confidence: 'Medium',
  };
}

function suggestTopNMetric(fieldPath: string): SuggestedMetric {
  const label = humanize(fieldPath);
  return {
    name: `top_${lastSegment(fieldPath)}`,
    description: `Top values for ${label} by frequency`,
    aggregation: 'top_n',
    fieldMeasure: fieldPath,
    dimensions: [],
    timeGranularities: ['hour', 'day'],
    confidence: 'Low',
  };
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// ============================================
// Public API
// ============================================

/**
 * Generate a suggested semantic model from a reference event, schema tree,
 * and optional mapping interpretation.
 *
 * - Classifies attributes into Core / Extended / Potential tiers
 * - Suggests metrics based on field types and observable status
 * - Includes lineage info when mapping is available
 */
export function generateSuggestedModel(
  referenceEvent: ReferenceEvent,
  schemaTree: SchemaTree,
  mapping: InterpretedMapping | null
): SuggestedModel {
  const classUid = referenceEvent.classUid ?? 0;
  const entityName = referenceEvent.className
    ? deriveEntityName(referenceEvent.className)
    : 'unknown_entity';

  // Build observable lookup set
  const observableFieldPaths = new Set(
    referenceEvent.observables.map((o) => o.fieldPath)
  );

  // Build schema attribute map for the detected class
  const classAttrs = classUid ? findClassAttributes(schemaTree, classUid) : null;
  const schemaMap = classAttrs ? flattenSchemaAttributes(classAttrs) : new Map<string, AttributeNode>();

  // Build mapping lookup by ocsf field
  const mappingByOcsf = new Map<string, InterpretedMapping['entries'][number]>();
  if (mapping) {
    for (const entry of mapping.entries) {
      mappingByOcsf.set(entry.ocsfField, entry);
    }
  }

  // Track which event field paths we've seen (for Potential tier)
  const eventFieldPaths = new Set(referenceEvent.fields.map((f) => f.path));

  // ---- Build attributes ----
  const attributes: SuggestedAttribute[] = [];

  // 1. Attributes from the reference event
  for (const field of referenceEvent.fields) {
    const schemaAttr = schemaMap.get(field.path);
    const tier = classifyObservedAttribute(schemaAttr, mapping, field.path);
    const mappingEntry = mappingByOcsf.get(field.path);

    const attr: SuggestedAttribute = {
      name: lastSegment(field.path),
      fieldPath: field.path,
      type: field.type,
      sampleValue: field.value,
      tier,
      isObservable: observableFieldPaths.has(field.path),
    };

    if (mappingEntry) {
      attr.lineage = {
        rawField: mappingEntry.rawField,
        transformation: mappingEntry.transformation ?? 'direct',
      };
    }

    attributes.push(attr);
  }

  // 2. Potential attributes: in schema but NOT in event
  for (const [schemaPath, schemaAttr] of schemaMap) {
    if (!eventFieldPaths.has(schemaPath)) {
      attributes.push({
        name: lastSegment(schemaPath),
        fieldPath: schemaPath,
        type: schemaAttr.type,
        sampleValue: '',
        tier: 'potential',
        isObservable: false,
      });
    }
  }

  // ---- Suggest metrics ----
  const metrics: SuggestedMetric[] = [];
  const seenMetricNames = new Set<string>();

  function addMetric(m: SuggestedMetric): void {
    if (!seenMetricNames.has(m.name)) {
      seenMetricNames.add(m.name);
      metrics.push(m);
    }
  }

  for (const field of referenceEvent.fields) {
    // Numeric fields → count, sum, avg
    if (field.type === 'integer' || field.type === 'float') {
      for (const m of suggestNumericMetrics(field.path)) {
        addMetric(m);
      }
    }

    // Timestamp fields → time-granularity + event rate
    if (field.type === 'timestamp' || lastSegment(field.path) === 'time' || lastSegment(field.path).endsWith('_dt')) {
      for (const m of suggestTimestampMetrics(field.path)) {
        addMetric(m);
      }
    }

    // Observable string fields → count_distinct
    if (field.type === 'string' && observableFieldPaths.has(field.path)) {
      addMetric(suggestCardinalityMetric(field.path));
    }

    // Status/result fields → ratio
    if (isStatusField(field.path)) {
      addMetric(suggestRatioMetric(field.path));
    }

    // High-cardinality categorical strings → top-N
    if (field.type === 'string' && !observableFieldPaths.has(field.path) && !isStatusField(field.path)) {
      addMetric(suggestTopNMetric(field.path));
    }
  }

  return {
    entityName,
    classUid,
    attributes,
    metrics,
  };
}
