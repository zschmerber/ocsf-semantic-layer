/**
 * EntityEditor component - main container for entity editing functionality.
 * 
 * Combines EntityForm, EntityList, and AttributeMappingZone components.
 * 
 * Requirements: 2.1, 2.2, 2.7, 2.8
 */

import { useState, useCallback, useMemo } from 'react';
import { useEditorStore } from '../../store';
import type { SemanticEntity, ClassNode, ResearchTarget } from '../../types';
import { EntityForm } from './EntityForm';
import { EntityList } from './EntityList';
import { AttributeMappingZone } from './AttributeMappingZone';
import { LLMResearchPanel, BatchResearchPanel } from '../LLMResearchPanel';
import './EntityEditor.css';

// ============================================
// Types
// ============================================

type EditorMode = 'list' | 'create' | 'edit';

// ============================================
// Component
// ============================================

export function EntityEditor() {
  const [mode, setMode] = useState<EditorMode>('list');
  const [editingEntity, setEditingEntity] = useState<SemanticEntity | null>(null);
  const [researchTarget, setResearchTarget] = useState<ResearchTarget | null>(null);
  const [showBatchResearch, setShowBatchResearch] = useState(false);
  
  // Store state
  const entities = useEditorStore((state) => state.model.entities);
  const schema = useEditorStore((state) => state.schema);
  const selectedEntity = useEditorStore((state) => state.selectedEntity);
  const selectEntity = useEditorStore((state) => state.selectEntity);
  const addEntity = useEditorStore((state) => state.addEntity);
  const updateEntity = useEditorStore((state) => state.updateEntity);
  const removeEntity = useEditorStore((state) => state.removeEntity);
  
  // Get all available event classes from schema
  const availableClasses = useMemo((): ClassNode[] => {
    if (!schema) return [];
    return schema.categories.flatMap((cat) => cat.classes);
  }, [schema]);
  
  // Get currently selected entity object
  const currentEntity = useMemo(() => {
    if (!selectedEntity) return null;
    return entities.find((e) => e.name === selectedEntity) ?? null;
  }, [entities, selectedEntity]);
  
  // Handlers
  const handleCreateNew = useCallback(() => {
    setEditingEntity(null);
    setMode('create');
  }, []);
  
  const handleEdit = useCallback((entity: SemanticEntity) => {
    setEditingEntity(entity);
    setMode('edit');
  }, []);
  
  const handleSave = useCallback(
    (entity: SemanticEntity) => {
      if (mode === 'create') {
        addEntity(entity);
        selectEntity(entity.name);
      } else if (mode === 'edit' && editingEntity) {
        updateEntity(editingEntity.name, entity);
      }
      setMode('list');
      setEditingEntity(null);
    },
    [mode, editingEntity, addEntity, updateEntity, selectEntity]
  );
  
  const handleCancel = useCallback(() => {
    setMode('list');
    setEditingEntity(null);
  }, []);
  
  const handleDelete = useCallback(
    (name: string) => {
      removeEntity(name);
    },
    [removeEntity]
  );
  
  const handleSelect = useCallback(
    (name: string) => {
      selectEntity(name);
    },
    [selectEntity]
  );

  const handleResearchEntity = useCallback(
    (entity: SemanticEntity) => {
      setResearchTarget({ type: 'entity', entity });
      setShowBatchResearch(false);
    },
    []
  );

  const handleBatchResearch = useCallback(() => {
    setShowBatchResearch(true);
    setResearchTarget(null);
  }, []);

  const handleCloseResearch = useCallback(() => {
    setResearchTarget(null);
    setShowBatchResearch(false);
  }, []);

  // Render form mode
  if (mode === 'create' || mode === 'edit') {
    return (
      <div className="entity-editor">
        <EntityForm
          entity={editingEntity ?? undefined}
          availableClasses={availableClasses}
          onSave={handleSave}
          onCancel={handleCancel}
        />
      </div>
    );
  }

  // Render list mode with attribute mapping
  return (
    <div className="entity-editor">
      <div className="entity-editor-header">
        <h2>Entities</h2>
        <button className="btn primary" onClick={handleCreateNew}>
          + New Entity
        </button>
      </div>
      
      <div className="entity-editor-content">
        <div className="entity-list-panel">
          <EntityList
            entities={entities}
            selectedEntity={selectedEntity}
            onSelect={handleSelect}
            onEdit={handleEdit}
            onDelete={handleDelete}
          />
        </div>
        
        {currentEntity && (
          <div className="entity-detail-panel">
            <div className="entity-detail-header">
              <h3>{currentEntity.caption || currentEntity.name}</h3>
              <div className="entity-detail-actions">
                <button
                  className="research-button"
                  onClick={() => handleResearchEntity(currentEntity)}
                  title="Research entity with LLM"
                >
                  🔬 Research Entity
                </button>
                {currentEntity.attributes.length > 0 && (
                  <button
                    className="research-button batch"
                    onClick={handleBatchResearch}
                    title="Research all attributes with LLM"
                  >
                    🔬 Batch Research
                  </button>
                )}
                <button
                  className="btn"
                  onClick={() => handleEdit(currentEntity)}
                >
                  Edit Details
                </button>
              </div>
            </div>
            
            {currentEntity.description && (
              <p className="entity-detail-description">
                {currentEntity.description}
              </p>
            )}
            
            <div className="entity-detail-meta">
              <span className="meta-item">
                <span className="meta-label">Name:</span>
                <code>{currentEntity.name}</code>
              </span>
              <span className="meta-item">
                <span className="meta-label">Event Classes:</span>
                <span>{currentEntity.source_event_classes.length}</span>
              </span>
            </div>

            {/* LLM Research Panel */}
            {researchTarget && researchTarget.type === 'entity' && 
             researchTarget.entity.name === currentEntity.name && (
              <div className="entity-research-panel">
                <LLMResearchPanel
                  target={researchTarget}
                  onClose={handleCloseResearch}
                />
              </div>
            )}
            
            <AttributeMappingZone
              entity={currentEntity}
            />
          </div>
        )}

        {/* Batch Research Modal - Full screen for better readability */}
        {showBatchResearch && currentEntity && (
          <div className="batch-research-modal-overlay" onClick={handleCloseResearch}>
            <div className="batch-research-modal" onClick={(e) => e.stopPropagation()}>
              <BatchResearchPanel
                entity={currentEntity}
                onClose={handleCloseResearch}
              />
            </div>
          </div>
        )}
        
        {!currentEntity && entities.length > 0 && (
          <div className="entity-detail-panel empty">
            <div className="empty-state">
              <div className="empty-icon">👈</div>
              <p>Select an entity to view and edit its attributes</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default EntityEditor;
