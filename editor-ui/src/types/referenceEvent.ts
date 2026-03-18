/**
 * TypeScript types for the Analyst Data-First Workflow.
 * These types support the Reference Event parsing, class detection,
 * observable flagging, mapping interpretation, and suggested model generation.
 */

// ============================================
// Parsed Event Types
// ============================================

/**
 * A single extracted field from a pasted OCSF JSON event.
 * Fields are flattened to dot-notation paths.
 */
export interface ParsedField {
  path: string;
  type: string;
  value: string;
  rawValue: unknown;
}

/**
 * The complete parsed event stored in ReferenceEventStore.
 */
export interface ReferenceEvent {
  rawJson: string;
  fields: ParsedField[];
  classUid: number | null;
  categoryUid: number | null;
  className: string | null;
  categoryName: string | null;
  classConfidence: 'Definitive' | 'Low';
  observables: ObservableFlag[];
  typeMismatches: TypeMismatch[];
  parsedAt: string;
}

// ============================================
// Observable Detection Types
// ============================================

/**
 * Supported observable type classifications.
 */
export type ObservableType = 'ip' | 'hostname' | 'hash' | 'url' | 'email' | 'mac' | 'process';

/**
 * A field flagged as containing observable data.
 */
export interface ObservableFlag {
  fieldPath: string;
  observableType: ObservableType;
  confidence: 'High' | 'Medium';
  matchedBy: 'name' | 'value' | 'both';
}

// ============================================
// Class Detection Types
// ============================================

/**
 * Result of class_uid / category_uid detection from a parsed event.
 */
export interface ClassDetectionResult {
  classUid: number | null;
  categoryUid: number | null;
  className: string | null;
  categoryName: string | null;
  confidence: 'Definitive' | 'Low';
  error: string | null;
}

// ============================================
// Type Comparison Types
// ============================================

/**
 * A mismatch between an observed field type and the schema-defined type.
 */
export interface TypeMismatch {
  fieldPath: string;
  observedType: string;
  schemaType: string;
}

// ============================================
// Mapping Interpretation Types
// ============================================

/**
 * Verification status of a mapping entry against the reference event.
 */
export type VerificationStatus = 'Verified' | 'Unverified' | 'Conflict';

/**
 * A single field-to-field mapping entry from LLM interpretation.
 */
export interface MappingEntry {
  rawField: string;
  ocsfField: string;
  transformation: string | null;
  confidence: 'High' | 'Medium' | 'Low';
  explanation: string | null;
  verificationStatus: VerificationStatus;
  conflictDetail: string | null;
}

/**
 * Normalized result from the LLM mapping interpretation.
 */
export interface InterpretedMapping {
  entries: MappingEntry[];
  sourceSystem: {
    logType: string | null;
    vendor: string | null;
  };
  issues: string[];
  interpretedAt: string;
}

// ============================================
// Mapping Coverage Types
// ============================================

/**
 * Computed metrics showing mapping completeness relative to the reference event.
 */
export interface MappingCoverage {
  percentEventFieldsMapped: number;
  percentMappingFieldsUnobserved: number;
  percentEventFieldsUnmapped: number;
  unmappedFieldPaths: string[];
  unobservedMappingFields: string[];
}

// ============================================
// Suggested Model Types
// ============================================

/**
 * Attribute classification tier based on schema presence and event observation.
 */
export type AttributeTier = 'core' | 'extended' | 'potential';

/**
 * A suggested attribute for the auto-generated entity.
 */
export interface SuggestedAttribute {
  name: string;
  fieldPath: string;
  type: string;
  sampleValue: string;
  tier: AttributeTier;
  isObservable: boolean;
  lineage?: {
    rawField: string;
    transformation: string;
  };
}

/**
 * A suggested metric candidate derived from event fields.
 */
export interface SuggestedMetric {
  name: string;
  description: string;
  aggregation: string;
  fieldMeasure: string;
  dimensions: string[];
  timeGranularities: string[];
  confidence: 'High' | 'Medium' | 'Low';
}

/**
 * The complete auto-generated suggested model.
 */
export interface SuggestedModel {
  entityName: string;
  classUid: number;
  attributes: SuggestedAttribute[];
  metrics: SuggestedMetric[];
}

// ============================================
// Persistence Types
// ============================================

/**
 * Serialized state for localStorage persistence.
 * On load, derived state is re-computed from rawJson.
 */
export interface ReferenceEventSaveState {
  version: 1;
  rawJson: string;
  mappingRawText: string | null;
  interpretedMapping: InterpretedMapping | null;
  parsedAt: string;
}
