/**
 * MetricEditor component for editing semantic metric properties.
 *
 * Provides controls for metric_type, formula, non_additive_dimensions,
 * is_hidden toggle, and folder input.
 *
 * Requirements: 8.5, 8.8
 */

import { useState, useCallback } from 'react';
import type { SemanticMetric, MetricType } from '../../types';
import './MetricEditor.css';

// ============================================
// Types
// ============================================

export interface MetricEditorProps {
  metric: SemanticMetric;
  onChange: (updates: Partial<SemanticMetric>) => void;
  allDimensions: string[];
}

const METRIC_TYPES: MetricType[] = ['Additive', 'SemiAdditive', 'NonAdditive'];

// ============================================
// Component
// ============================================

export function MetricEditor({ metric, onChange, allDimensions }: MetricEditorProps) {
  const [showFormula, setShowFormula] = useState(!!metric.formula);

  const metricType = metric.metric_type ?? 'Additive';
  const nonAdditiveDimensions = metric.non_additive_dimensions ?? [];

  const handleMetricTypeChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const newType = e.target.value as MetricType;
      const updates: Partial<SemanticMetric> = { metric_type: newType };
      // Clear non_additive_dimensions when switching away from SemiAdditive
      if (newType !== 'SemiAdditive') {
        updates.non_additive_dimensions = [];
      }
      onChange(updates);
    },
    [onChange]
  );

  const handleFormulaChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange({ formula: e.target.value || undefined });
    },
    [onChange]
  );

  const handleToggleFormula = useCallback(() => {
    if (showFormula) {
      // Hiding formula — clear it
      setShowFormula(false);
      onChange({ formula: undefined });
    } else {
      setShowFormula(true);
    }
  }, [showFormula, onChange]);

  const handleDimensionToggle = useCallback(
    (dim: string) => {
      const current = new Set(nonAdditiveDimensions);
      if (current.has(dim)) {
        current.delete(dim);
      } else {
        current.add(dim);
      }
      onChange({ non_additive_dimensions: Array.from(current) });
    },
    [nonAdditiveDimensions, onChange]
  );

  const handleHiddenChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange({ is_hidden: e.target.checked });
    },
    [onChange]
  );

  const handleFolderChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange({ folder: e.target.value || undefined });
    },
    [onChange]
  );

  return (
    <div className="metric-editor">
      <div className="metric-editor-header">
        <span className="metric-editor-title">Metric Properties</span>
        {metric.is_hidden && <span className="hidden-badge">hidden</span>}
      </div>

      <div className="metric-editor-body">
        {/* Metric Type */}
        <div className="form-group">
          <label className="form-label" htmlFor="metric-type">
            Metric Type
          </label>
          <select
            id="metric-type"
            className="form-select"
            value={metricType}
            onChange={handleMetricTypeChange}
          >
            {METRIC_TYPES.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
          <span className="form-hint">
            {metricType === 'Additive' && 'Can be summed across all dimensions'}
            {metricType === 'SemiAdditive' && 'Cannot be summed across certain dimensions'}
            {metricType === 'NonAdditive' && 'Cannot be summed across any dimension'}
          </span>
        </div>

        {/* Formula */}
        <div className="form-group">
          <div className="formula-header">
            <label className="form-label" htmlFor="metric-formula">
              Formula
            </label>
            <label className="checkbox-label formula-toggle">
              <input
                type="checkbox"
                checked={showFormula}
                onChange={handleToggleFormula}
              />
              <span>Calculated metric</span>
            </label>
          </div>
          {showFormula && (
            <textarea
              id="metric-formula"
              className="form-textarea mono"
              value={metric.formula ?? ''}
              onChange={handleFormulaChange}
              placeholder="e.g., success_count / total_count"
              rows={3}
            />
          )}
          {showFormula && (
            <span className="form-hint">
              Reference other metric names. The measure field is ignored for calculated metrics.
            </span>
          )}
        </div>

        {/* Non-Additive Dimensions (shown only for SemiAdditive) */}
        {metricType === 'SemiAdditive' && (
          <div className="form-group">
            <label className="form-label">
              Non-Additive Dimensions
              <span className="form-hint">Dimensions across which this metric cannot be summed</span>
            </label>
            <div className="dimension-checklist">
              {allDimensions.length === 0 && (
                <span className="dimension-empty">No dimensions available</span>
              )}
              {allDimensions.map((dim) => (
                <label key={dim} className="checkbox-label dimension-item">
                  <input
                    type="checkbox"
                    checked={nonAdditiveDimensions.includes(dim)}
                    onChange={() => handleDimensionToggle(dim)}
                  />
                  <span className="dimension-name">{dim}</span>
                </label>
              ))}
            </div>
          </div>
        )}

        {/* Visibility & Folder */}
        <div className="form-row">
          <div className="form-group half">
            <label className="form-label">Visibility</label>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={metric.is_hidden ?? false}
                onChange={handleHiddenChange}
              />
              <span>Hidden</span>
            </label>
          </div>
          <div className="form-group half">
            <label className="form-label" htmlFor="metric-folder">
              Folder
            </label>
            <input
              id="metric-folder"
              type="text"
              className="form-input"
              value={metric.folder ?? ''}
              onChange={handleFolderChange}
              placeholder="e.g., Session Metrics"
            />
          </div>
        </div>
      </div>
    </div>
  );
}
