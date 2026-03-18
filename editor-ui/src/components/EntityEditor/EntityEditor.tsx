/**
 * EntityEditor component - main container for entity editing functionality.
 * 
 * Combines EntityForm, EntityList, and AttributeMappingZone components.
 * 
 * Requirements: 2.1, 2.2, 2.7, 2.8
 */

import { useState, useCallback, useMemo } from 'react';
import { useEditorStore } from '../../store';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import type { SemanticEntity, ClassNode, ResearchTarget, EntityRelationship } from '../../types';
import type { ObservableFlag, TypeMismatch } from '../../types/referenceEvent';
import { EntityForm } from './EntityForm';
import { EntityList } from './EntityList';
import { AttributeMappingZone } from './AttributeMappingZone';
import { LLMResearchPanel, BatchResearchPanel } from '../LLMResearchPanel';
import { ContextualTooltip } from '../ContextualTooltip';
import { SuggestedModelPanel } from './SuggestedModelPanel';
import { useGuideStore } from '../../store/guideStore';
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
  const [showAllFields, setShowAllFields] = useState(false);
  const [suggestedModelDismissed, setSuggestedModelDismissed] = useState(false);
  
  // Reference event store state (Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 13.2)
  const rawJson = useReferenceEventStore((state) => state.rawJson);
  const parsedFields = useReferenceEventStore((state) => state.parsedFields);
  const observableFlags = useReferenceEventStore((state) => state.observableFlags);
  const typeMismatches = useReferenceEventStore((state) => state.typeMismatches);
  
  // Guide store — check Step 1 completion (Requirements: 9.1, 9.2)
  const stepStatuses = useGuideStore((state) => state.stepStatuses);
  const step1Complete = stepStatuses[1] === 'complete';
  
  const hasReferenceEvent = rawJson !== null;
  
  // Show SuggestedModelPanel when Step 1 is complete, reference event loaded, and not dismissed
  const showSuggestedModel = step1Complete && hasReferenceEvent && !suggestedModelDismissed;
  
  // Build lookup maps for quick access
  const observableFlagMap = useMemo(() => {
    const map = new Map<string, ObservableFlag>();
    for (const flag of observableFlags) {
      map.set(flag.fieldPath, flag);
    }
    return map;
  }, [observableFlags]);
  
  const typeMismatchMap = useMemo(() => {
    const map = new Map<string, TypeMismatch>();
    for (const mismatch of typeMismatches) {
      map.set(mismatch.fieldPath, mismatch);
    }
    return map;
  }, [typeMismatches]);
  
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
        <h2>
          <ContextualTooltip term="semantic_entity">
            <span>Entities</span>
          </ContextualTooltip>
        </h2>
        <button className="btn primary" onClick={handleCreateNew}>
          + New Entity
        </button>
      </div>

      {/* Suggested Model Panel (Requirements: 9.1-9.9) */}
      {showSuggestedModel && (
        <SuggestedModelPanel onDismiss={() => setSuggestedModelDismissed(true)} />
      )}
      
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
                <ContextualTooltip term="event_class">
                  <span className="meta-label">Event Classes:</span>
                </ContextualTooltip>
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

            {/* Reference Event Data Overlay (Requirements: 8.1-8.6, 13.2) */}
            {hasReferenceEvent && (
              <div className="reference-event-overlay">
                <div className="reference-event-indicator">
                  <span className="sample-event-badge" title="Multi-event support is planned for a future version">📊 Based on 1 sample event</span>
                  <div className="field-view-toggle">
                    <button
                      className={`toggle-btn ${!showAllFields ? 'active' : ''}`}
                      onClick={() => setShowAllFields(false)}
                    >
                      Sample fields only
                    </button>
                    <button
                      className={`toggle-btn ${showAllFields ? 'active' : ''}`}
                      onClick={() => setShowAllFields(true)}
                    >
                      All schema fields
                    </button>
                  </div>
                </div>

                {!showAllFields && (
                  <div className="sample-fields-list">
                    {parsedFields.map((field) => {
                      const observable = observableFlagMap.get(field.path);
                      const mismatch = typeMismatchMap.get(field.path);
                      const truncatedValue = field.value.length > 60
                        ? field.value.slice(0, 60) + '…'
                        : field.value;

                      return (
                        <div key={field.path} className="sample-field-item">
                          <div className="sample-field-header">
                            <code className="sample-field-path">{field.path}</code>
                            <span className="sample-field-type">{field.type}</span>
                            {observable && (
                              <span className="sample-field-observable" title={`Observable: ${observable.observableType} (${observable.confidence} confidence)`}>
                                🔍
                              </span>
                            )}
                            {mismatch && (
                              <span className="sample-field-mismatch" title={`Type mismatch: observed ${mismatch.observedType}, schema expects ${mismatch.schemaType}`}>
                                ⚠️ {mismatch.observedType} → {mismatch.schemaType}
                              </span>
                            )}
                          </div>
                          <div className="sample-field-value" title={field.value}>
                            {truncatedValue}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            )}
            
            <AttributeMappingZone
              entity={currentEntity}
            />

            {/* Relationships Section */}
            {currentEntity.relationships.length > 0 && (
              <RelationshipsSection
                entity={currentEntity}
                onUpdateEntity={updateEntity}
              />
            )}
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

// ============================================
// RelationshipsSection Component
// ============================================

function RelationshipsSection({
  entity,
  onUpdateEntity,
}: {
  entity: SemanticEntity;
  onUpdateEntity: (name: string, updates: Partial<SemanticEntity>) => void;
}) {
  const handleRoleAliasChange = useCallback(
    (index: number, value: string) => {
      const updatedRelationships: EntityRelationship[] = entity.relationships.map(
        (rel, i) =>
          i === index
            ? { ...rel, role_alias: value || undefined }
            : rel
      );
      onUpdateEntity(entity.name, { relationships: updatedRelationships });
    },
    [entity, onUpdateEntity]
  );

  return (
    <div className="relationships-section">
      <h4 className="relationships-section-title">Relationships</h4>
      <div className="relationships-list">
        {entity.relationships.map((rel, index) => (
          <div key={`${rel.name}-${index}`} className="relationship-item">
            <div className="relationship-item-header">
              <span className="relationship-name">{rel.name}</span>
              <span className="relationship-cardinality-badge">{rel.cardinality}</span>
            </div>
            <div className="relationship-item-meta">
              <span className="relationship-target">
                → <code>{rel.target_entity}</code>
              </span>
              {rel.join_condition && (
                <span className="relationship-join">
                  <code>{rel.join_condition}</code>
                </span>
              )}
            </div>
            <div className="relationship-role-alias">
              <label className="relationship-role-alias-label" htmlFor={`role-alias-${index}`}>
                Role Alias
              </label>
              <input
                id={`role-alias-${index}`}
                type="text"
                className="form-input relationship-role-alias-input"
                placeholder="e.g. src_user"
                value={rel.role_alias ?? ''}
                onChange={(e) => handleRoleAliasChange(index, e.target.value)}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default EntityEditor;
