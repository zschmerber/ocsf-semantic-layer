/**
 * EntityList component for displaying existing entities with edit/delete actions.
 * 
 * Requirements: 2.7, 2.8
 */

import { useState, useCallback } from 'react';
import type { SemanticEntity } from '../../types';

// ============================================
// Types
// ============================================

export interface EntityListProps {
  entities: SemanticEntity[];
  selectedEntity: string | null;
  onSelect: (name: string) => void;
  onEdit: (entity: SemanticEntity) => void;
  onDelete: (name: string) => void;
}

// ============================================
// Component
// ============================================

export function EntityList({
  entities,
  selectedEntity,
  onSelect,
  onEdit,
  onDelete,
}: EntityListProps) {
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  
  const handleDeleteClick = useCallback((name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDeleteConfirm(name);
  }, []);
  
  const handleConfirmDelete = useCallback((name: string) => {
    onDelete(name);
    setDeleteConfirm(null);
  }, [onDelete]);
  
  const handleCancelDelete = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setDeleteConfirm(null);
  }, []);
  
  const handleEditClick = useCallback((entity: SemanticEntity, e: React.MouseEvent) => {
    e.stopPropagation();
    onEdit(entity);
  }, [onEdit]);

  if (entities.length === 0) {
    return (
      <div className="entity-list-empty">
        <div className="empty-icon">📦</div>
        <p>No entities defined</p>
        <p className="text-muted">Create your first entity to get started</p>
      </div>
    );
  }

  return (
    <div className="entity-list">
      {entities.map((entity) => (
        <div
          key={entity.name}
          className={`entity-list-item ${selectedEntity === entity.name ? 'selected' : ''}`}
          onClick={() => onSelect(entity.name)}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              onSelect(entity.name);
            }
          }}
        >
          <div className="entity-item-content">
            <div className="entity-item-header">
              <span className="entity-item-name">{entity.caption || entity.name}</span>
              <span className="entity-item-badge">
                {entity.attributes.length} attr
              </span>
            </div>
            <div className="entity-item-meta">
              <span className="entity-item-id">{entity.name}</span>
              <span className="entity-item-classes">
                {entity.source_event_classes.length} class{entity.source_event_classes.length !== 1 ? 'es' : ''}
              </span>
            </div>
            {entity.description && (
              <p className="entity-item-description">{entity.description}</p>
            )}
          </div>
          
          <div className="entity-item-actions">
            {deleteConfirm === entity.name ? (
              <div className="delete-confirm">
                <span className="delete-confirm-text">Delete?</span>
                <button
                  className="btn-icon confirm"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleConfirmDelete(entity.name);
                  }}
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
                  onClick={(e) => handleEditClick(entity, e)}
                  title="Edit entity"
                >
                  ✏️
                </button>
                <button
                  className="btn-icon danger"
                  onClick={(e) => handleDeleteClick(entity.name, e)}
                  title="Delete entity"
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

export default EntityList;
