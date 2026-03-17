/**
 * AttributeMappingZone component with drag-drop support for OCSF attributes.
 * 
 * Allows users to drag OCSF attributes from the schema browser and drop them
 * to create semantic attribute mappings.
 * 
 * Requirements: 2.3, 2.4, 2.5, 16.5, 16.6
 */

import { useState, useCallback, useMemo } from 'react';
import { useDrop } from 'react-dnd';
import type { SemanticEntity, SemanticAttribute, SemanticType, ResearchTarget, FieldMapping } from '../../types';
import { createDefaultAttribute } from '../../types';
import { useEditorStore } from '../../store';
import { useIndexStore } from '../../store/indexStore';
import { AttributeEditor, MappingSuggestion } from './AttributeEditor';
import { LLMResearchPanel } from '../LLMResearchPanel';

// ============================================
// Types
// ============================================

export interface AttributeMappingZoneProps {
  entity: SemanticEntity;
}

// Drag item type for OCSF attributes
export const OCSF_ATTRIBUTE_TYPE = 'ocsf-attribute';

export interface OCSFAttributeDragItem {
  type: typeof OCSF_ATTRIBUTE_TYPE;
  path: string;
  name: string;
  caption: string;
  description: string;
  attrType: string;
  isArray: boolean;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Converts OCSF type string to SemanticType.
 */
function ocsfTypeToSemanticType(ocsfType: string, isArray: boolean): SemanticType {
  let baseType: SemanticType;
  
  const lowerType = ocsfType.toLowerCase();
  if (lowerType.includes('integer') || lowerType.includes('int') || lowerType.includes('long')) {
    baseType = 'integer';
  } else if (lowerType.includes('float') || lowerType.includes('double')) {
    baseType = 'float';
  } else if (lowerType.includes('boolean') || lowerType.includes('bool')) {
    baseType = 'boolean';
  } else if (lowerType.includes('timestamp') || lowerType.includes('datetime')) {
    baseType = 'timestamp';
  } else if (lowerType.includes('json') || lowerType.includes('object')) {
    baseType = 'json';
  } else {
    baseType = 'string';
  }
  
  return isArray ? { array: baseType } : baseType;
}

/**
 * Generates a unique attribute name from the OCSF path.
 */
function generateAttributeName(path: string, existingNames: Set<string>): string {
  // Extract the last part of the path as the base name
  const parts = path.split('.');
  let baseName = parts[parts.length - 1];
  
  // Convert to snake_case if needed
  baseName = baseName.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
  
  // Ensure uniqueness
  let name = baseName;
  let counter = 1;
  while (existingNames.has(name)) {
    name = `${baseName}_${counter}`;
    counter++;
  }
  
  return name;
}

/**
 * Calculate similarity between two strings.
 * Returns a score between 0 and 1.
 */
function calculateStringSimilarity(str1: string, str2: string): number {
  const s1 = str1.toLowerCase();
  const s2 = str2.toLowerCase();
  
  if (s1 === s2) return 1;
  if (s1.length === 0 || s2.length === 0) return 0;
  
  // Check for substring match
  if (s1.includes(s2) || s2.includes(s1)) {
    return 0.8;
  }
  
  // Check for word overlap
  const words1 = s1.split(/[_.\-\s]+/);
  const words2 = s2.split(/[_.\-\s]+/);
  const commonWords = words1.filter(w => words2.some(w2 => w2.includes(w) || w.includes(w2)));
  if (commonWords.length > 0) {
    return 0.6 + (commonWords.length / Math.max(words1.length, words2.length)) * 0.3;
  }
  
  return 0;
}

/**
 * Find the best mapping suggestion for an attribute.
 * Requirements: 16.1, 16.6
 */
function findBestMappingSuggestion(
  attributeName: string,
  fieldMappings: FieldMapping[]
): MappingSuggestion | null {
  if (fieldMappings.length === 0) return null;
  
  let bestSuggestion: MappingSuggestion | null = null;
  let bestScore = 0;
  
  for (const mapping of fieldMappings) {
    const targetSimilarity = calculateStringSimilarity(attributeName, mapping.target_field);
    const sourceSimilarity = calculateStringSimilarity(attributeName, mapping.source_field);
    const similarity = Math.max(targetSimilarity, sourceSimilarity);
    
    if (similarity > bestScore && similarity >= 0.5) {
      bestScore = similarity;
      bestSuggestion = {
        sourceField: mapping.source_field,
        targetField: mapping.target_field,
        confidence: similarity,
        transformation: mapping.transformation,
        matchReason: targetSimilarity >= sourceSimilarity 
          ? 'Match with OCSF target field' 
          : 'Match with source field',
      };
    }
  }
  
  return bestSuggestion;
}

// ============================================
// Component
// ============================================

export function AttributeMappingZone({ entity }: AttributeMappingZoneProps) {
  const [editingAttribute, setEditingAttribute] = useState<string | null>(null);
  const [showNewAttributeForm, setShowNewAttributeForm] = useState(false);
  const [researchTarget, setResearchTarget] = useState<ResearchTarget | null>(null);
  
  // Store actions
  const addEntityAttribute = useEditorStore((state) => state.addEntityAttribute);
  const updateEntityAttribute = useEditorStore((state) => state.updateEntityAttribute);
  const removeEntityAttribute = useEditorStore((state) => state.removeEntityAttribute);
  
  // Get field mappings from index store (Requirements: 16.1, 16.6)
  const fieldMappings = useIndexStore((state) => state.fieldMappings);
  
  // Calculate unmapped attributes and available suggestions (Requirements: 16.5, 16.6)
  const { unmappedAttributes, suggestionsMap, totalSuggestions } = useMemo(() => {
    const unmapped: SemanticAttribute[] = [];
    const suggestions: Map<string, MappingSuggestion> = new Map();
    let count = 0;
    
    for (const attr of entity.attributes) {
      const hasMapping = !!(attr.ocsf_mapping.field || attr.ocsf_mapping.expression);
      if (!hasMapping) {
        unmapped.push(attr);
        const suggestion = findBestMappingSuggestion(attr.name, fieldMappings);
        if (suggestion) {
          suggestions.set(attr.name, suggestion);
          count++;
        }
      }
    }
    
    return { 
      unmappedAttributes: unmapped, 
      suggestionsMap: suggestions,
      totalSuggestions: count 
    };
  }, [entity.attributes, fieldMappings]);
  
  // Group attributes by folder for list view organization (Requirement 8.9)
  const groupedAttributes = useMemo(() => {
    const ungrouped: SemanticAttribute[] = [];
    const folders = new Map<string, SemanticAttribute[]>();
    let hasFolders = false;

    for (const attr of entity.attributes) {
      if (attr.folder) {
        hasFolders = true;
        const list = folders.get(attr.folder) || [];
        list.push(attr);
        folders.set(attr.folder, list);
      } else {
        ungrouped.push(attr);
      }
    }

    return { ungrouped, folders, hasFolders };
  }, [entity.attributes]);

  // Apply all suggestions handler (Requirement 16.6)
  const handleApplyAllSuggestions = useCallback(() => {
    for (const [attrName, suggestion] of suggestionsMap) {
      updateEntityAttribute(entity.name, attrName, {
        ocsf_mapping: {
          field: suggestion.targetField,
          ...(suggestion.transformation && { expression: suggestion.transformation }),
        },
      });
    }
  }, [entity.name, suggestionsMap, updateEntityAttribute]);
  
  // Drop handler for OCSF attributes
  const handleDrop = useCallback(
    (item: OCSFAttributeDragItem) => {
      const existingNames = new Set(entity.attributes.map((a) => a.name));
      const name = generateAttributeName(item.path, existingNames);
      
      const newAttribute: SemanticAttribute = {
        ...createDefaultAttribute(name),
        caption: item.caption || item.name,
        description: item.description,
        attr_type: ocsfTypeToSemanticType(item.attrType, item.isArray),
        ocsf_mapping: {
          field: item.path,
        },
        is_dimension: false,
      };
      
      addEntityAttribute(entity.name, newAttribute);
    },
    [entity, addEntityAttribute]
  );
  
  // React DnD drop configuration
  const [{ isOver, canDrop }, dropRef] = useDrop(
    () => ({
      accept: OCSF_ATTRIBUTE_TYPE,
      drop: (item: OCSFAttributeDragItem) => {
        handleDrop(item);
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
        canDrop: monitor.canDrop(),
      }),
    }),
    [handleDrop]
  );
  
  // Handlers
  const handleEditAttribute = useCallback((name: string) => {
    setEditingAttribute(name);
    setShowNewAttributeForm(false);
  }, []);
  
  const handleSaveAttribute = useCallback(
    (originalName: string, updates: Partial<SemanticAttribute>) => {
      updateEntityAttribute(entity.name, originalName, updates);
      setEditingAttribute(null);
    },
    [entity.name, updateEntityAttribute]
  );
  
  const handleDeleteAttribute = useCallback(
    (name: string) => {
      removeEntityAttribute(entity.name, name);
      if (editingAttribute === name) {
        setEditingAttribute(null);
      }
    },
    [entity.name, removeEntityAttribute, editingAttribute]
  );
  
  const handleAddManual = useCallback(() => {
    setShowNewAttributeForm(true);
    setEditingAttribute(null);
  }, []);
  
  const handleCreateAttribute = useCallback(
    (attribute: SemanticAttribute) => {
      addEntityAttribute(entity.name, attribute);
      setShowNewAttributeForm(false);
    },
    [entity.name, addEntityAttribute]
  );
  
  const handleCancelNew = useCallback(() => {
    setShowNewAttributeForm(false);
  }, []);

  const handleResearchAttribute = useCallback(
    (attribute: SemanticAttribute) => {
      setResearchTarget({ type: 'attribute', entity, attribute });
    },
    [entity]
  );

  const handleCloseResearch = useCallback(() => {
    setResearchTarget(null);
  }, []);

  const dropClassName = `attribute-drop-zone ${isOver ? 'drag-over' : ''} ${canDrop ? 'can-drop' : ''}`;

  return (
    <div className="attribute-mapping-zone">
      <div className="mapping-zone-header">
        <h4>Attributes</h4>
        <div className="mapping-zone-actions">
          {/* Apply All Suggestions Button (Requirement 16.6) */}
          {totalSuggestions > 0 && (
            <button 
              className="apply-all-suggestions-btn"
              onClick={handleApplyAllSuggestions}
              title={`Apply ${totalSuggestions} mapping suggestion${totalSuggestions > 1 ? 's' : ''} to unmapped attributes`}
            >
              <span className="btn-icon-spark">✨</span>
              Apply {totalSuggestions} Suggestion{totalSuggestions > 1 ? 's' : ''}
            </button>
          )}
          {/* Unmapped count indicator (Requirement 16.5) */}
          {unmappedAttributes.length > 0 && (
            <span className="unmapped-count-badge" title={`${unmappedAttributes.length} attribute${unmappedAttributes.length > 1 ? 's' : ''} without OCSF mapping`}>
              {unmappedAttributes.length} unmapped
            </span>
          )}
          <button className="btn" onClick={handleAddManual}>
            + Add Attribute
          </button>
        </div>
      </div>
      
      {/* Drop Zone */}
      <div ref={dropRef} className={dropClassName}>
        {entity.attributes.length === 0 && !showNewAttributeForm ? (
          <div className="drop-zone-empty">
            <div className="drop-icon">📥</div>
            <p>Drag OCSF attributes here</p>
            <p className="text-muted">or click "Add Attribute" to create manually</p>
          </div>
        ) : (
          <div className="attribute-list">
            {/* Render ungrouped attributes first (Requirement 8.9) */}
            {groupedAttributes.ungrouped.map((attr) => (
              <div key={attr.name} className="attribute-item-wrapper">
                {editingAttribute === attr.name ? (
                  <AttributeEditor
                    attribute={attr}
                    existingNames={new Set(
                      entity.attributes
                        .filter((a) => a.name !== attr.name)
                        .map((a) => a.name)
                    )}
                    onSave={(updates) => handleSaveAttribute(attr.name, updates)}
                    onCancel={() => setEditingAttribute(null)}
                    onDelete={() => handleDeleteAttribute(attr.name)}
                  />
                ) : researchTarget?.type === 'attribute' && 
                     researchTarget.attribute.name === attr.name ? (
                  <div className="attribute-research-wrapper">
                    <AttributeItem
                      attribute={attr}
                      onEdit={() => handleEditAttribute(attr.name)}
                      onDelete={() => handleDeleteAttribute(attr.name)}
                      onResearch={() => handleResearchAttribute(attr)}
                      isResearching
                      hasSuggestion={suggestionsMap.has(attr.name)}
                    />
                    <LLMResearchPanel
                      target={researchTarget}
                      onClose={handleCloseResearch}
                    />
                  </div>
                ) : (
                  <AttributeItem
                    attribute={attr}
                    onEdit={() => handleEditAttribute(attr.name)}
                    onDelete={() => handleDeleteAttribute(attr.name)}
                    onResearch={() => handleResearchAttribute(attr)}
                    hasSuggestion={suggestionsMap.has(attr.name)}
                  />
                )}
              </div>
            ))}

            {/* Render folder-grouped attributes with headers (Requirement 8.9) */}
            {groupedAttributes.hasFolders && Array.from(groupedAttributes.folders.entries()).map(([folder, attrs]) => (
              <div key={`folder-${folder}`} className="folder-group">
                <div className="folder-group-header">
                  <span className="folder-group-icon">📁</span>
                  <span>{folder}</span>
                  <span className="folder-group-count">({attrs.length})</span>
                </div>
                {attrs.map((attr) => (
                  <div key={attr.name} className="attribute-item-wrapper">
                    {editingAttribute === attr.name ? (
                      <AttributeEditor
                        attribute={attr}
                        existingNames={new Set(
                          entity.attributes
                            .filter((a) => a.name !== attr.name)
                            .map((a) => a.name)
                        )}
                        onSave={(updates) => handleSaveAttribute(attr.name, updates)}
                        onCancel={() => setEditingAttribute(null)}
                        onDelete={() => handleDeleteAttribute(attr.name)}
                      />
                    ) : researchTarget?.type === 'attribute' && 
                         researchTarget.attribute.name === attr.name ? (
                      <div className="attribute-research-wrapper">
                        <AttributeItem
                          attribute={attr}
                          onEdit={() => handleEditAttribute(attr.name)}
                          onDelete={() => handleDeleteAttribute(attr.name)}
                          onResearch={() => handleResearchAttribute(attr)}
                          isResearching
                          hasSuggestion={suggestionsMap.has(attr.name)}
                        />
                        <LLMResearchPanel
                          target={researchTarget}
                          onClose={handleCloseResearch}
                        />
                      </div>
                    ) : (
                      <AttributeItem
                        attribute={attr}
                        onEdit={() => handleEditAttribute(attr.name)}
                        onDelete={() => handleDeleteAttribute(attr.name)}
                        onResearch={() => handleResearchAttribute(attr)}
                        hasSuggestion={suggestionsMap.has(attr.name)}
                      />
                    )}
                  </div>
                ))}
              </div>
            ))}
            
            {showNewAttributeForm && (
              <div className="attribute-item-wrapper">
                <AttributeEditor
                  existingNames={new Set(entity.attributes.map((a) => a.name))}
                  onSave={(attr) => handleCreateAttribute(attr as SemanticAttribute)}
                  onCancel={handleCancelNew}
                  isNew
                />
              </div>
            )}
          </div>
        )}
        
        {isOver && canDrop && (
          <div className="drop-indicator">
            <span>Drop to add attribute</span>
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================
// AttributeItem Sub-component
// ============================================

interface AttributeItemProps {
  attribute: SemanticAttribute;
  onEdit: () => void;
  onDelete: () => void;
  onResearch: () => void;
  isResearching?: boolean;
  hasSuggestion?: boolean;
}

function AttributeItem({ attribute, onEdit, onDelete, onResearch, isResearching, hasSuggestion }: AttributeItemProps) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  
  const typeDisplay = typeof attribute.attr_type === 'object' 
    ? `${(attribute.attr_type as { array: SemanticType }).array}[]`
    : attribute.attr_type;
  
  // Check if attribute is unmapped (Requirement 16.5)
  const isUnmapped = !(attribute.ocsf_mapping.field || attribute.ocsf_mapping.expression);

  return (
    <div className={`attribute-item ${isResearching ? 'researching' : ''} ${isUnmapped ? 'unmapped' : ''}`}>
      <div className="attribute-item-content">
        <div className="attribute-item-header">
          <span className="attribute-name">{attribute.caption || attribute.name}</span>
          <span className="attribute-type-badge">{typeDisplay}</span>
          {attribute.is_dimension && (
            <span className="attribute-dimension-badge">Dimension</span>
          )}
          {attribute.is_observable && (
            <span className="attribute-observable-badge">Observable</span>
          )}
          {isUnmapped && (
            <span className="attribute-unmapped-badge">Unmapped</span>
          )}
          {hasSuggestion && isUnmapped && (
            <span className="attribute-suggestion-badge" title="Mapping suggestion available">💡</span>
          )}
        </div>
        <div className="attribute-item-meta">
          <code className="attribute-id">{attribute.name}</code>
          {attribute.ocsf_mapping.field && (
            <span className="attribute-mapping">
              → <code>{attribute.ocsf_mapping.field}</code>
            </span>
          )}
        </div>
        {attribute.description && (
          <p className="attribute-description">{attribute.description}</p>
        )}
      </div>
      
      <div className="attribute-item-actions">
        {showDeleteConfirm ? (
          <div className="delete-confirm">
            <span className="delete-confirm-text">Delete?</span>
            <button
              className="btn-icon confirm"
              onClick={() => {
                onDelete();
                setShowDeleteConfirm(false);
              }}
              title="Confirm delete"
            >
              ✓
            </button>
            <button
              className="btn-icon cancel"
              onClick={() => setShowDeleteConfirm(false)}
              title="Cancel"
            >
              ✕
            </button>
          </div>
        ) : (
          <>
            <button
              className="attribute-research-btn"
              onClick={onResearch}
              title="Research with LLM"
              disabled={isResearching}
            >
              🔬
            </button>
            <button className="btn-icon" onClick={onEdit} title="Edit attribute">
              ✏️
            </button>
            <button
              className="btn-icon danger"
              onClick={() => setShowDeleteConfirm(true)}
              title="Delete attribute"
            >
              🗑️
            </button>
          </>
        )}
      </div>
    </div>
  );
}

export default AttributeMappingZone;
