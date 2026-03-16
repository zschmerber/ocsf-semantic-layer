/**
 * MetricForm component for creating and editing semantic metrics.
 * 
 * Provides form fields for name, caption, description, aggregation type,
 * and measure configuration (field or expression).
 * 
 * Requirements: 3.1, 3.2, 3.3, 3.6
 */

import { useState, useCallback } from 'react';
import type { SemanticMetric, Aggregation, OCSFMapping } from '../../types';
import { createDefaultMetric } from '../../types';

// ============================================
// Types
// ============================================

export interface MetricFormProps {
  metric?: SemanticMetric;
  onSave: (metric: SemanticMetric) => void;
  onCancel: () => void;
}

const AGGREGATION_OPTIONS: { value: Aggregation; label: string; description: string }[] = [
  { value: 'count', label: 'Count', description: 'Count of records' },
  { value: 'count_distinct', label: 'Count Distinct', description: 'Count of unique values' },
  { value: 'sum', label: 'Sum', description: 'Sum of numeric values' },
  { value: 'avg', label: 'Average', description: 'Average of numeric values' },
  { value: 'min', label: 'Minimum', description: 'Minimum value' },
  { value: 'max', label: 'Maximum', description: 'Maximum value' },
];

type MeasureType = 'field' | 'expression';

// ============================================
// Component
// ============================================

export function MetricForm({
  metric,
  onSave,
  onCancel,
}: MetricFormProps) {
  const isEditing = !!metric;
  
  // Form state
  const [name, setName] = useState(metric?.name ?? '');
  const [caption, setCaption] = useState(metric?.caption ?? '');
  const [description, setDescription] = useState(metric?.description ?? '');
  const [aggregation, setAggregation] = useState<Aggregation>(metric?.aggregation ?? 'count');
  const [measureType, setMeasureType] = useState<MeasureType>(
    metric?.measure?.expression ? 'expression' : 'field'
  );
  const [measureField, setMeasureField] = useState(metric?.measure?.field ?? '');
  const [measureExpression, setMeasureExpression] = useState(metric?.measure?.expression ?? '');
  const [isHotPath, setIsHotPath] = useState(metric?.is_hot_path ?? false);
  
  // Validation
  const [errors, setErrors] = useState<Record<string, string>>({});
  
  // Determine if aggregation needs a measure field/expression
  const needsMeasure = aggregation !== 'count';
  
  const validateForm = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      newErrors.name = 'Name must start with lowercase letter and contain only lowercase letters, numbers, and underscores';
    }
    
    if (!caption.trim()) {
      newErrors.caption = 'Caption is required';
    }
    
    if (needsMeasure) {
      if (measureType === 'field' && !measureField.trim()) {
        newErrors.measure = 'Measure field is required for this aggregation';
      } else if (measureType === 'expression' && !measureExpression.trim()) {
        newErrors.measure = 'Measure expression is required';
      }
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [name, caption, needsMeasure, measureType, measureField, measureExpression]);
  
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      
      if (!validateForm()) return;
      
      // Build measure configuration
      const measure: OCSFMapping = {};
      if (needsMeasure) {
        if (measureType === 'field') {
          measure.field = measureField.trim();
        } else {
          measure.expression = measureExpression.trim();
        }
      }
      
      const newMetric: SemanticMetric = metric
        ? {
            ...metric,
            name: name.trim(),
            caption: caption.trim(),
            description: description.trim(),
            aggregation,
            measure,
            is_hot_path: isHotPath,
          }
        : {
            ...createDefaultMetric(name.trim()),
            caption: caption.trim(),
            description: description.trim(),
            aggregation,
            measure,
            is_hot_path: isHotPath,
          };
      
      onSave(newMetric);
    },
    [metric, name, caption, description, aggregation, needsMeasure, measureType, measureField, measureExpression, isHotPath, validateForm, onSave]
  );
  
  const clearFieldError = useCallback((field: string) => {
    setErrors((prev) => {
      const { [field]: _, ...rest } = prev;
      return rest;
    });
  }, []);

  return (
    <form className="metric-form" onSubmit={handleSubmit}>
      <div className="form-header">
        <h3>{isEditing ? 'Edit Metric' : 'Create Metric'}</h3>
      </div>
      
      <div className="form-body">
        {/* Name Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="metric-name">
            Name <span className="required">*</span>
          </label>
          <input
            id="metric-name"
            type="text"
            className={`form-input ${errors.name ? 'error' : ''}`}
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              if (errors.name) clearFieldError('name');
            }}
            placeholder="e.g., dns_query_count"
            disabled={isEditing}
          />
          {errors.name && <span className="form-error">{errors.name}</span>}
          {isEditing && (
            <span className="form-hint">Name cannot be changed after creation</span>
          )}
        </div>
        
        {/* Caption Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="metric-caption">
            Caption <span className="required">*</span>
          </label>
          <input
            id="metric-caption"
            type="text"
            className={`form-input ${errors.caption ? 'error' : ''}`}
            value={caption}
            onChange={(e) => {
              setCaption(e.target.value);
              if (errors.caption) clearFieldError('caption');
            }}
            placeholder="e.g., DNS Query Count"
          />
          {errors.caption && <span className="form-error">{errors.caption}</span>}
        </div>
        
        {/* Description Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="metric-description">
            Description
          </label>
          <textarea
            id="metric-description"
            className="form-textarea"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe what this metric measures..."
            rows={3}
          />
        </div>
        
        {/* Aggregation Type Dropdown - Requirement 3.2 */}
        <div className="form-group">
          <label className="form-label" htmlFor="metric-aggregation">
            Aggregation Type <span className="required">*</span>
          </label>
          <select
            id="metric-aggregation"
            className="form-select"
            value={aggregation}
            onChange={(e) => setAggregation(e.target.value as Aggregation)}
          >
            {AGGREGATION_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label} - {opt.description}
              </option>
            ))}
          </select>
        </div>
        
        {/* Measure Configuration - Requirement 3.3 */}
        {needsMeasure && (
          <div className="form-group">
            <label className="form-label">
              Measure Configuration <span className="required">*</span>
            </label>
            
            {/* Measure Type Toggle */}
            <div className="measure-type-toggle">
              <button
                type="button"
                className={`toggle-btn ${measureType === 'field' ? 'active' : ''}`}
                onClick={() => setMeasureType('field')}
              >
                Field
              </button>
              <button
                type="button"
                className={`toggle-btn ${measureType === 'expression' ? 'active' : ''}`}
                onClick={() => setMeasureType('expression')}
              >
                Expression
              </button>
            </div>
            
            {measureType === 'field' ? (
              <input
                type="text"
                className={`form-input mono ${errors.measure ? 'error' : ''}`}
                value={measureField}
                onChange={(e) => {
                  setMeasureField(e.target.value);
                  if (errors.measure) clearFieldError('measure');
                }}
                placeholder="e.g., metadata.count or actor.user.uid"
              />
            ) : (
              <textarea
                className={`form-textarea mono ${errors.measure ? 'error' : ''}`}
                value={measureExpression}
                onChange={(e) => {
                  setMeasureExpression(e.target.value);
                  if (errors.measure) clearFieldError('measure');
                }}
                placeholder="e.g., CASE WHEN status = 'success' THEN 1 ELSE 0 END"
                rows={3}
              />
            )}
            {errors.measure && <span className="form-error">{errors.measure}</span>}
            <span className="form-hint">
              {measureType === 'field' 
                ? 'Enter the OCSF field path to aggregate'
                : 'Enter a SQL expression for computed measures'}
            </span>
          </div>
        )}
        
        {/* Hot Path Toggle */}
        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={isHotPath}
              onChange={(e) => setIsHotPath(e.target.checked)}
            />
            <span>Hot Path Metric</span>
          </label>
          <span className="form-hint">
            Enable for high-frequency, low-latency analytics
          </span>
        </div>
      </div>
      
      <div className="form-footer">
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="btn primary">
          {isEditing ? 'Save Changes' : 'Create Metric'}
        </button>
      </div>
    </form>
  );
}

export default MetricForm;
