/**
 * SuggestedModelPanel — displays an auto-generated suggested entity model
 * after Step 1 is complete with a Reference_Event loaded.
 *
 * Attributes are organized into three tiers:
 *   Core (pre-selected), Extended (shown, not selected), Potential (collapsed).
 * Analysts can promote/demote between tiers, toggle selection, and accept or discard.
 *
 * Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9
 */

import { useState, useMemo, useCallback } from 'react';
import { useEditorStore } from '../../store';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import { generateSuggestedModel } from '../../utils/suggestedModelGenerator';
import type {
  SuggestedAttribute,
  SuggestedMetric,
  AttributeTier,
} from '../../types/referenceEvent';
import type { SemanticEntity, SemanticAttribute, SemanticMetric } from '../../types';

// ============================================
// Types
// ============================================

interface AttributeState {
  attr: SuggestedAttribute;
  selected: boolean;
  tier: AttributeTier;
}

interface MetricState {
  metric: SuggestedMetric;
  added: boolean;
}

// ============================================
// Helpers
// ============================================

const TIER_LABELS: Record<AttributeTier, string> = {
  core: 'Core (High Confidence)',
  extended: 'Extended (Medium Confidence)',
  potential: 'Potential (Low Confidence)',
};

const TIER_ORDER: AttributeTier[] = ['core', 'extended', 'potential'];

function mapSemanticType(type: string): SemanticAttribute['attr_type'] {
  switch (type) {
    case 'integer': return 'integer';
    case 'float': return 'float';
    case 'boolean': return 'boolean';
    case 'timestamp': return 'timestamp';
    default: return 'string';
  }
}

// ============================================
// Component
// ============================================

export function SuggestedModelPanel({ onDismiss }: { onDismiss: () => void }) {
  const schema = useEditorStore((s) => s.schema);
  const addEntity = useEditorStore((s) => s.addEntity);
  const addMetric = useEditorStore((s) => s.addMetric);
  const selectEntity = useEditorStore((s) => s.selectEntity);

  const rawJson = useReferenceEventStore((s) => s.rawJson);
  const parsedFields = useReferenceEventStore((s) => s.parsedFields);
  const classDetection = useReferenceEventStore((s) => s.classDetection);
  const observableFlags = useReferenceEventStore((s) => s.observableFlags);
  const typeMismatches = useReferenceEventStore((s) => s.typeMismatches);
  const interpretedMapping = useReferenceEventStore((s) => s.interpretedMapping);

  const [potentialExpanded, setPotentialExpanded] = useState(false);

  // Generate the suggested model
  const suggestedModel = useMemo(() => {
    if (!rawJson || !schema || !classDetection) return null;
    const refEvent = {
      rawJson,
      fields: parsedFields,
      classUid: classDetection.classUid,
      categoryUid: classDetection.categoryUid,
      className: classDetection.className,
      categoryName: classDetection.categoryName,
      classConfidence: classDetection.confidence,
      observables: observableFlags,
      typeMismatches,
      parsedAt: new Date().toISOString(),
    };
    return generateSuggestedModel(refEvent, schema, interpretedMapping ?? null);
  }, [rawJson, schema, classDetection, parsedFields, observableFlags, typeMismatches, interpretedMapping]);

  // Attribute selection state — Core pre-selected, Extended/Potential not
  const [attrStates, setAttrStates] = useState<AttributeState[]>(() => {
    if (!suggestedModel) return [];
    return suggestedModel.attributes.map((attr) => ({
      attr,
      selected: attr.tier === 'core',
      tier: attr.tier,
    }));
  });

  // Metric state
  const [metricStates, setMetricStates] = useState<MetricState[]>(() => {
    if (!suggestedModel) return [];
    return suggestedModel.metrics.map((metric) => ({
      metric,
      added: false,
    }));
  });

  // Group attributes by tier
  const grouped = useMemo(() => {
    const groups: Record<AttributeTier, AttributeState[]> = {
      core: [], extended: [], potential: [],
    };
    for (const s of attrStates) {
      groups[s.tier].push(s);
    }
    return groups;
  }, [attrStates]);

  // Toggle attribute selection
  const toggleAttr = useCallback((fieldPath: string) => {
    setAttrStates((prev) =>
      prev.map((s) =>
        s.attr.fieldPath === fieldPath ? { ...s, selected: !s.selected } : s
      )
    );
  }, []);

  // Promote/demote attribute between tiers
  const changeTier = useCallback((fieldPath: string, newTier: AttributeTier) => {
    setAttrStates((prev) =>
      prev.map((s) =>
        s.attr.fieldPath === fieldPath ? { ...s, tier: newTier } : s
      )
    );
  }, []);

  // Accept: create entity + metrics in EditorStore
  const handleAccept = useCallback(() => {
    if (!suggestedModel) return;

    const selectedAttrs = attrStates.filter((s) => s.selected);
    const observableSet = new Set(
      observableFlags.map((o) => o.fieldPath)
    );

    const entityAttributes: SemanticAttribute[] = selectedAttrs.map((s) => ({
      name: s.attr.name,
      caption: s.attr.name.replace(/_/g, ' '),
      description: s.attr.lineage
        ? `Source: ${s.attr.lineage.rawField} (${s.attr.lineage.transformation})`
        : '',
      attr_type: mapSemanticType(s.attr.type),
      ocsf_mapping: { field: s.attr.fieldPath },
      is_dimension: s.attr.type === 'string',
      sample_values: s.attr.sampleValue ? [s.attr.sampleValue] : [],
      synonyms: [],
      is_observable: observableSet.has(s.attr.fieldPath),
    }));

    const entity: SemanticEntity = {
      name: suggestedModel.entityName,
      caption: suggestedModel.entityName.replace(/_/g, ' '),
      description: `Auto-generated from reference event (class_uid: ${suggestedModel.classUid})`,
      source_event_classes: suggestedModel.classUid ? [suggestedModel.classUid] : [],
      attributes: entityAttributes,
      relationships: [],
      covers_observables: [],
    };

    addEntity(entity);
    selectEntity(entity.name);

    // Add any metrics that were marked as added
    for (const ms of metricStates) {
      if (ms.added) {
        addMetricToStore(ms.metric);
      }
    }

    onDismiss();
  }, [suggestedModel, attrStates, metricStates, observableFlags, addEntity, selectEntity, onDismiss]);

  // Helper to add a metric to the store
  const addMetricToStore = useCallback((m: SuggestedMetric) => {
    const agg = (['count', 'sum', 'avg', 'min', 'max', 'count_distinct'].includes(m.aggregation)
      ? m.aggregation
      : 'count') as SemanticMetric['aggregation'];

    const metric: SemanticMetric = {
      name: m.name,
      caption: m.name.replace(/_/g, ' '),
      description: m.description,
      aggregation: agg,
      measure: { field: m.fieldMeasure },
      dimensions: m.dimensions,
      time_granularities: m.timeGranularities.filter(
        (g): g is SemanticMetric['time_granularities'][number] =>
          ['minute', 'hour', 'day', 'week', 'month'].includes(g)
      ),
      is_hot_path: false,
    };
    addMetric(metric);
  }, [addMetric]);

  // Toggle a metric as "added"
  const toggleMetric = useCallback((name: string) => {
    setMetricStates((prev) =>
      prev.map((ms) =>
        ms.metric.name === name ? { ...ms, added: !ms.added } : ms
      )
    );
  }, []);

  if (!suggestedModel) return null;

  const hasMapping = interpretedMapping !== null;

  return (
    <div className="suggested-model-panel">
      <div className="smp-header">
        <div className="smp-header-left">
          <span className="smp-icon">✨</span>
          <h3 className="smp-title">Suggested Model: {suggestedModel.entityName}</h3>
        </div>
        <div className="smp-header-actions">
          <button className="btn primary smp-accept-btn" onClick={handleAccept}>
            Accept Selected
          </button>
          <button className="btn smp-discard-btn" onClick={onDismiss}>
            Discard
          </button>
        </div>
      </div>

      <div className="smp-body">
        {/* Attribute tiers */}
        {TIER_ORDER.map((tier) => {
          const items = grouped[tier];
          if (items.length === 0) return null;

          const isPotential = tier === 'potential';
          const isCollapsed = isPotential && !potentialExpanded;

          return (
            <div key={tier} className={`smp-tier smp-tier-${tier}`}>
              <div
                className="smp-tier-header"
                onClick={isPotential ? () => setPotentialExpanded(!potentialExpanded) : undefined}
                role={isPotential ? 'button' : undefined}
                tabIndex={isPotential ? 0 : undefined}
                onKeyDown={isPotential ? (e) => { if (e.key === 'Enter' || e.key === ' ') setPotentialExpanded(!potentialExpanded); } : undefined}
              >
                <span className="smp-tier-label">{TIER_LABELS[tier]}</span>
                <span className="smp-tier-count">{items.length} attributes</span>
                {isPotential && (
                  <span className={`smp-tier-toggle ${potentialExpanded ? 'expanded' : ''}`}>▶</span>
                )}
              </div>

              {!isCollapsed && (
                <table className="smp-attr-table">
                  <thead>
                    <tr>
                      <th className="smp-col-check"></th>
                      <th className="smp-col-name">Name</th>
                      <th className="smp-col-type">Type</th>
                      <th className="smp-col-value">Sample Value</th>
                      <th className="smp-col-badges"></th>
                      {hasMapping && <th className="smp-col-lineage">Lineage</th>}
                      <th className="smp-col-actions">Tier</th>
                    </tr>
                  </thead>
                  <tbody>
                    {items.map((s) => (
                      <tr key={s.attr.fieldPath} className="smp-attr-row">
                        <td>
                          <input
                            type="checkbox"
                            checked={s.selected}
                            onChange={() => toggleAttr(s.attr.fieldPath)}
                            aria-label={`Select ${s.attr.name}`}
                          />
                        </td>
                        <td>
                          <code className="smp-field-path">{s.attr.fieldPath}</code>
                        </td>
                        <td><span className="smp-type-badge">{s.attr.type}</span></td>
                        <td className="smp-sample-value">
                          {s.attr.sampleValue
                            ? (s.attr.sampleValue.length > 40
                                ? s.attr.sampleValue.slice(0, 40) + '…'
                                : s.attr.sampleValue)
                            : <span className="smp-no-value">—</span>}
                        </td>
                        <td>
                          {s.attr.isObservable && (
                            <span className="smp-observable-badge" title="Observable field">🔍</span>
                          )}
                        </td>
                        {hasMapping && (
                          <td className="smp-lineage-cell">
                            {s.attr.lineage ? (
                              <span className="smp-lineage" title={`${s.attr.lineage.rawField} → ${s.attr.fieldPath} (${s.attr.lineage.transformation})`}>
                                <code>{s.attr.lineage.rawField}</code>
                                <span className="smp-lineage-arrow">→</span>
                                <span className="smp-lineage-transform">{s.attr.lineage.transformation}</span>
                              </span>
                            ) : (
                              <span className="smp-no-value">—</span>
                            )}
                          </td>
                        )}
                        <td>
                          <TierSelector
                            current={s.tier}
                            onChange={(newTier) => changeTier(s.attr.fieldPath, newTier)}
                          />
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          );
        })}

        {/* Suggested Metrics */}
        {metricStates.length > 0 && (
          <div className="smp-metrics-section">
            <h4 className="smp-metrics-title">Suggested Metrics</h4>
            <div className="smp-metrics-list">
              {metricStates.map((ms) => (
                <div key={ms.metric.name} className={`smp-metric-item ${ms.added ? 'added' : ''}`}>
                  <div className="smp-metric-info">
                    <span className="smp-metric-name">{ms.metric.name}</span>
                    <span className="smp-metric-desc">{ms.metric.description}</span>
                    <span className={`smp-metric-confidence confidence-${ms.metric.confidence.toLowerCase()}`}>
                      {ms.metric.confidence}
                    </span>
                  </div>
                  <button
                    className={`btn smp-metric-toggle ${ms.added ? 'added' : ''}`}
                    onClick={() => toggleMetric(ms.metric.name)}
                  >
                    {ms.added ? '✓ Added' : '+ Add'}
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================
// TierSelector sub-component
// ============================================

function TierSelector({
  current,
  onChange,
}: {
  current: AttributeTier;
  onChange: (tier: AttributeTier) => void;
}) {
  return (
    <select
      className="smp-tier-select"
      value={current}
      onChange={(e) => onChange(e.target.value as AttributeTier)}
      aria-label="Change attribute tier"
    >
      <option value="core">Core</option>
      <option value="extended">Extended</option>
      <option value="potential">Potential</option>
    </select>
  );
}

export default SuggestedModelPanel;
