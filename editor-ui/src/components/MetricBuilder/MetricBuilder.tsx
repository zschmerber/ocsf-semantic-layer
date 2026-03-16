/**
 * MetricBuilder component - main container for metric editing functionality.
 * 
 * Combines MetricForm, MetricList, DimensionSelector, and GranularitySelector.
 * 
 * Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7
 */

import { useState, useCallback, useMemo } from 'react';
import { useEditorStore } from '../../store';
import type { SemanticMetric, TimeGranularity } from '../../types';
import { MetricForm } from './MetricForm';
import { MetricList } from './MetricList';
import { DimensionSelector } from './DimensionSelector';
import { GranularitySelector } from './GranularitySelector';
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
          <h2>Metrics</h2>
          <p className="metric-builder-description">
            Define aggregations over your semantic entities. Metrics use <strong>dimensions</strong> (attributes marked as "Dimension" in entities) for GROUP BY operations, enabling drill-down analysis by source IP, user, action, etc.
          </p>
        </div>
        <button className="btn primary" onClick={handleCreateNew}>
          + New Metric
        </button>
      </div>
      
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
              <h4>Dimensions</h4>
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
              <h4>Time Granularities</h4>
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
