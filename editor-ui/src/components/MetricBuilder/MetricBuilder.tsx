/**
 * MetricBuilder component - main container for metric editing functionality.
 * 
 * Combines MetricForm, MetricList, DimensionSelector, and GranularitySelector.
 * When a Reference_Event is loaded, shows a "Suggested Metrics" section with
 * candidates from suggestedModelGenerator.
 * 
 * Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 10.1, 10.2, 10.3, 10.4,
 *               10.5, 10.6, 10.7, 10.8, 10.9, 10.10, 13.2
 */

import { useState, useCallback, useMemo } from 'react';
import { useEditorStore } from '../../store';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import { generateSuggestedModel } from '../../utils/suggestedModelGenerator';
import type { SemanticMetric, TimeGranularity } from '../../types';
import type { SuggestedMetric } from '../../types/referenceEvent';
import { MetricForm } from './MetricForm';
import { MetricList } from './MetricList';
import { DimensionSelector } from './DimensionSelector';
import { GranularitySelector } from './GranularitySelector';
import { ContextualTooltip } from '../ContextualTooltip';
import './MetricBuilder.css';

// ============================================
// Types
// ============================================

type EditorMode = 'list' | 'create' | 'edit';

// ============================================
// Component
// ============================================

export function MetricBuilder() {
  const [mode, setMode] = useState<EditorMode>('list');
  const [editingMetric, setEditingMetric] = useState<SemanticMetric | null>(null);
  
  // Store state
  const metrics = useEditorStore((state) => state.model.metrics);
  const entities = useEditorStore((state) => state.model.entities);
  const selectedMetric = useEditorStore((state) => state.selectedMetric);
  const selectMetric = useEditorStore((state) => state.selectMetric);
  const addMetric = useEditorStore((state) => state.addMetric);
  const updateMetric = useEditorStore((state) => state.updateMetric);
  const removeMetric = useEditorStore((state) => state.removeMetric);
  const schema = useEditorStore((state) => state.schema);

  // Reference event state
  const rawJson = useReferenceEventStore((s) => s.rawJson);
  const parsedFields = useReferenceEventStore((s) => s.parsedFields);
  const classDetection = useReferenceEventStore((s) => s.classDetection);
  const observableFlags = useReferenceEventStore((s) => s.observableFlags);
  const typeMismatches = useReferenceEventStore((s) => s.typeMismatches);
  const interpretedMapping = useReferenceEventStore((s) => s.interpretedMapping);

  // Suggested metrics state
  const [suggestionsExpanded, setSuggestionsExpanded] = useState(true);
  const [addedSuggestions, setAddedSuggestions] = useState<Set<string>>(new Set());

  // Generate suggested metrics from reference event
  const suggestedMetrics = useMemo<SuggestedMetric[]>(() => {
    if (!rawJson || !schema || !classDetection) return [];
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
    const model = generateSuggestedModel(refEvent, schema, interpretedMapping ?? null);
    return model.metrics;
  }, [rawJson, schema, classDetection, parsedFields, observableFlags, typeMismatches, interpretedMapping]);

  const hasReferenceEvent = rawJson !== null;

  // Add a suggested metric to the EditorStore
  const handleAddSuggestion = useCallback(
    (suggested: SuggestedMetric) => {
      const agg = (['count', 'sum', 'avg', 'min', 'max', 'count_distinct'].includes(suggested.aggregation)
        ? suggested.aggregation
        : 'count') as SemanticMetric['aggregation'];

      const metric: SemanticMetric = {
        name: suggested.name,
        caption: suggested.name.replace(/_/g, ' '),
        description: suggested.description,
        aggregation: agg,
        measure: { field: suggested.fieldMeasure },
        dimensions: suggested.dimensions,
        time_granularities: suggested.timeGranularities.filter(
          (g): g is TimeGranularity =>
            ['minute', 'hour', 'day', 'week', 'month'].includes(g)
        ),
        is_hot_path: false,
      };
      addMetric(metric);
      setAddedSuggestions((prev) => new Set(prev).add(suggested.name));
    },
    [addMetric]
  );
  
  // Get currently selected metric object
  const currentMetric = useMemo(() => {
    if (!selectedMetric) return null;
    return metrics.find((m) => m.name === selectedMetric) ?? null;
  }, [metrics, selectedMetric]);
  
  // Handlers
  const handleCreateNew = useCallback(() => {
    setEditingMetric(null);
    setMode('create');
  }, []);
  
  const handleEdit = useCallback((metric: SemanticMetric) => {
    setEditingMetric(metric);
    setMode('edit');
  }, []);
  
  const handleSave = useCallback(
    (metric: SemanticMetric) => {
      if (mode === 'create') {
        addMetric(metric);
        selectMetric(metric.name);
      } else if (mode === 'edit' && editingMetric) {
        updateMetric(editingMetric.name, metric);
      }
      setMode('list');
      setEditingMetric(null);
    },
    [mode, editingMetric, addMetric, updateMetric, selectMetric]
  );
  
  const handleCancel = useCallback(() => {
    setMode('list');
    setEditingMetric(null);
  }, []);
  
  const handleDelete = useCallback(
    (name: string) => {
      removeMetric(name);
    },
    [removeMetric]
  );
  
  const handleSelect = useCallback(
    (name: string) => {
      selectMetric(name);
    },
    [selectMetric]
  );
  
  // Dimension and granularity handlers for the detail panel
  const handleDimensionsChange = useCallback(
    (dimensions: string[]) => {
      if (currentMetric) {
        updateMetric(currentMetric.name, { dimensions });
      }
    },
    [currentMetric, updateMetric]
  );
  
  const handleGranularitiesChange = useCallback(
    (time_granularities: TimeGranularity[]) => {
      if (currentMetric) {
        updateMetric(currentMetric.name, { time_granularities });
      }
    },
    [currentMetric, updateMetric]
  );

  // Render form mode
  if (mode === 'create' || mode === 'edit') {
    return (
      <div className="metric-builder">
        <MetricForm
          metric={editingMetric ?? undefined}
          onSave={handleSave}
          onCancel={handleCancel}
        />
      </div>
    );
  }

  // Render list mode with detail panel
  return (
    <div className="metric-builder">
      <div className="metric-builder-header">
        <div className="metric-builder-header-content">
          <h2>
            <ContextualTooltip term="metric">
              <span>Metrics</span>
            </ContextualTooltip>
          </h2>
          <p className="metric-builder-description">
            Define aggregations over your semantic entities. Metrics use <strong>dimensions</strong> (attributes marked as "Dimension" in entities) for GROUP BY operations, enabling drill-down analysis by source IP, user, action, etc.
          </p>
          {hasReferenceEvent && (
            <span className="sample-event-indicator" title="Multi-event support is planned for a future version">Based on 1 sample event</span>
          )}
        </div>
        <button className="btn primary" onClick={handleCreateNew}>
          + New Metric
        </button>
      </div>

      {/* Suggested Metrics from Reference Event */}
      {hasReferenceEvent && suggestedMetrics.length > 0 && (
        <div className="suggested-metrics-section">
          <div
            className="suggested-metrics-header"
            onClick={() => setSuggestionsExpanded(!suggestionsExpanded)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') setSuggestionsExpanded(!suggestionsExpanded);
            }}
          >
            <span className="suggested-metrics-icon">✨</span>
            <span className="suggested-metrics-title">Suggested Metrics</span>
            <span className="suggested-metrics-count">{suggestedMetrics.length} candidates</span>
            <span className={`suggested-metrics-toggle ${suggestionsExpanded ? 'expanded' : ''}`}>▶</span>
          </div>
          {suggestionsExpanded && (
            <div className="suggested-metrics-list">
              {suggestedMetrics.map((sm) => {
                const isAdded = addedSuggestions.has(sm.name);
                return (
                  <div key={sm.name} className={`suggested-metric-item ${isAdded ? 'added' : ''}`}>
                    <div className="suggested-metric-info">
                      <div className="suggested-metric-top">
                        <span className="suggested-metric-name">{sm.name}</span>
                        <span className={`suggested-metric-confidence confidence-${sm.confidence.toLowerCase()}`}>
                          {sm.confidence}
                        </span>
                        <span className="suggested-metric-agg">{sm.aggregation}</span>
                      </div>
                      <span className="suggested-metric-desc">{sm.description}</span>
                      <span className="suggested-metric-field">
                        Field: <code>{sm.fieldMeasure}</code>
                      </span>
                    </div>
                    <button
                      className={`btn suggested-metric-add ${isAdded ? 'added' : ''}`}
                      onClick={() => !isAdded && handleAddSuggestion(sm)}
                      disabled={isAdded}
                      aria-label={isAdded ? `${sm.name} already added` : `Add ${sm.name}`}
                    >
                      {isAdded ? '✓ Added' : '+ Add'}
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
      
      <div className="metric-builder-content">
        <div className="metric-list-panel">
          <MetricList
            metrics={metrics}
            selectedMetric={selectedMetric}
            onSelect={handleSelect}
            onEdit={handleEdit}
            onDelete={handleDelete}
          />
        </div>
        
        {currentMetric && (
          <div className="metric-detail-panel">
            <div className="metric-detail-header">
              <h3>{currentMetric.caption || currentMetric.name}</h3>
              <button
                className="btn"
                onClick={() => handleEdit(currentMetric)}
              >
                Edit Details
              </button>
            </div>
            
            {currentMetric.description && (
              <p className="metric-detail-description">
                {currentMetric.description}
              </p>
            )}
            
            <div className="metric-detail-meta">
              <span className="meta-item">
                <span className="meta-label">Name:</span>
                <code>{currentMetric.name}</code>
              </span>
              <span className="meta-item">
                <span className="meta-label">Aggregation:</span>
                <span className="aggregation-badge">{currentMetric.aggregation}</span>
              </span>
              {currentMetric.is_hot_path && (
                <span className="meta-item">
                  <span className="hot-path-indicator">🔥 Hot Path</span>
                </span>
              )}
            </div>
            
            {/* Measure Display */}
            {(currentMetric.measure?.field || currentMetric.measure?.expression) && (
              <div className="metric-measure-section">
                <h4>Measure</h4>
                {currentMetric.measure.field && (
                  <div className="measure-display">
                    <span className="measure-type">Field:</span>
                    <code>{currentMetric.measure.field}</code>
                  </div>
                )}
                {currentMetric.measure.expression && (
                  <div className="measure-display">
                    <span className="measure-type">Expression:</span>
                    <pre className="measure-expression">{currentMetric.measure.expression}</pre>
                  </div>
                )}
              </div>
            )}
            
            {/* Dimension Selector - Requirement 3.4 */}
            <div className="metric-dimensions-section">
              <h4>
                <ContextualTooltip term="dimension">
                  <span>Dimensions</span>
                </ContextualTooltip>
              </h4>
              <p className="section-hint">
                Select which attributes to use for GROUP BY. Only attributes marked as "Dimension" in your entities appear here. These enable filtering and drill-down in your analytics.
              </p>
              <DimensionSelector
                entities={entities}
                selectedDimensions={currentMetric.dimensions}
                onChange={handleDimensionsChange}
              />
            </div>
            
            {/* Granularity Selector - Requirement 3.5 */}
            <div className="metric-granularities-section">
              <h4>
                <ContextualTooltip term="time_granularity">
                  <span>Time Granularities</span>
                </ContextualTooltip>
              </h4>
              <GranularitySelector
                selectedGranularities={currentMetric.time_granularities}
                onChange={handleGranularitiesChange}
              />
            </div>
          </div>
        )}
        
        {!currentMetric && metrics.length > 0 && (
          <div className="metric-detail-panel empty">
            <div className="empty-state">
              <div className="empty-icon">👈</div>
              <p>Select a metric to configure dimensions and granularities</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default MetricBuilder;
