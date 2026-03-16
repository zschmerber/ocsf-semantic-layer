/**
 * MetricList component for displaying and managing semantic metrics.
 * 
 * Displays existing metrics in a list with edit and delete actions.
 * 
 * Requirements: 3.7
 */

import { useState, useCallback } from 'react';
import type { SemanticMetric } from '../../types';

// ============================================
// Types
// ============================================

export interface MetricListProps {
  metrics: SemanticMetric[];
  selectedMetric: string | null;
  onSelect: (name: string) => void;
  onEdit: (metric: SemanticMetric) => void;
  onDelete: (name: string) => void;
}

// ============================================
// Component
// ============================================

export function MetricList({
  metrics,
  selectedMetric,
  onSelect,
  onEdit,
  onDelete,
}: MetricListProps) {
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const handleDelete = useCallback(
    (name: string, e: React.MouseEvent) => {
      e.stopPropagation();
      if (confirmDelete === name) {
        onDelete(name);
        setConfirmDelete(null);
      } else {
        setConfirmDelete(name);
      }
    },
    [confirmDelete, onDelete]
  );

  const handleCancelDelete = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setConfirmDelete(null);
  }, []);

  const handleEdit = useCallback(
    (metric: SemanticMetric, e: React.MouseEvent) => {
      e.stopPropagation();
      onEdit(metric);
    },
    [onEdit]
  );

  if (metrics.length === 0) {
    return (
      <div className="metric-list-empty">
        <div className="empty-icon">📊</div>
        <p>No metrics defined yet</p>
        <p className="text-muted">Create a metric to define KPIs and aggregations</p>
      </div>
    );
  }

  return (
    <div className="metric-list">
      {metrics.map((metric) => (
        <div
          key={metric.name}
          className={`metric-list-item ${selectedMetric === metric.name ? 'selected' : ''}`}
          onClick={() => onSelect(metric.name)}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              onSelect(metric.name);
            }
          }}
        >
          <div className="metric-item-content">
            <div className="metric-item-header">
              <span className="metric-item-name">
                {metric.caption || metric.name}
              </span>
              <span className="metric-item-badge">
                {metric.aggregation}
              </span>
              {metric.is_hot_path && (
                <span className="metric-hot-path-badge">🔥 Hot Path</span>
              )}
            </div>
            <div className="metric-item-meta">
              <span className="metric-item-id">{metric.name}</span>
              {metric.dimensions.length > 0 && (
                <span className="metric-dimensions-count">
                  {metric.dimensions.length} dimension{metric.dimensions.length !== 1 ? 's' : ''}
                </span>
              )}
            </div>
            {metric.description && (
              <p className="metric-item-description">{metric.description}</p>
            )}
          </div>

          <div className="metric-item-actions">
            {confirmDelete === metric.name ? (
              <div className="delete-confirm">
                <span className="delete-confirm-text">Delete?</span>
                <button
                  className="btn-icon confirm"
                  onClick={(e) => handleDelete(metric.name, e)}
                  title="Confirm delete"
                >
                  ✓
                </button>
                <button
                  className="btn-icon cancel"
                  onClick={handleCancelDelete}
                  title="Cancel"
                >
                  ✕
                </button>
              </div>
            ) : (
              <>
                <button
                  className="btn-icon"
                  onClick={(e) => handleEdit(metric, e)}
                  title="Edit metric"
                >
                  ✏️
                </button>
                <button
                  className="btn-icon danger"
                  onClick={(e) => handleDelete(metric.name, e)}
                  title="Delete metric"
                >
                  🗑️
                </button>
              </>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}

export default MetricList;
