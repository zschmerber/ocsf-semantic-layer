/**
 * MappingBuilder component for defining source-to-OCSF field mappings.
 * 
 * Provides a visual interface for:
 * - Selecting source fields from parsed log data
 * - Selecting target OCSF fields from the schema
 * - Creating and managing field mappings
 * - Adding transformation expressions
 * 
 * Requirements: 14.4, 14.5, 14.6, 14.7, 14.8
 */

import { useCallback, useMemo, useState } from 'react';
import { useIndexStore } from '../../store/indexStore';
import { useSchema } from '../../api/hooks';
import { useCatalogSearch } from '../../api/catalogHooks';
import type { FieldMapping, SchemaTree, AttributeNode, ClassNode, CatalogEntry } from '../../types';
import './MappingBuilder.css';

// ============================================
// Types
// ============================================

/**
 * Flattened OCSF field for selection.
 */
export interface OCSFField {
  path: string;
  name: string;
  caption: string;
  description: string;
  type: string;
  requirement: 'required' | 'recommended' | 'optional';
  classUid?: number;
  className?: string;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Flattens OCSF schema attributes into a list of selectable fields.
 * Traverses categories, classes, and nested attributes.
 */
function flattenSchemaAttributes(schema: SchemaTree): OCSFField[] {
  const fields: OCSFField[] = [];
  
  for (const category of schema.categories) {
    for (const cls of category.classes) {
      flattenClassAttributes(cls, '', fields);
    }
  }
  
  // Also include object types
  for (const obj of schema.objects) {
    flattenAttributeList(obj.attributes, obj.name, fields, undefined, obj.caption);
  }
  
  return fields;
}

/**
 * Flattens attributes from a class.
 */
function flattenClassAttributes(
  cls: ClassNode,
  prefix: string,
  fields: OCSFField[]
): void {
  flattenAttributeList(cls.attributes, prefix, fields, cls.uid, cls.caption);
}

/**
 * Flattens a list of attributes recursively.
 */
function flattenAttributeList(
  attributes: AttributeNode[],
  prefix: string,
  fields: OCSFField[],
  classUid?: number,
  className?: string
): void {
  for (const attr of attributes) {
    const path = prefix ? `${prefix}.${attr.name}` : attr.name;
    
    fields.push({
      path,
      name: attr.name,
      caption: attr.caption,
      description: attr.description,
      type: attr.type,
      requirement: attr.requirement,
      classUid,
      className,
    });
    
    // Recursively flatten nested attributes
    if (attr.children && attr.children.length > 0) {
      flattenAttributeList(attr.children, path, fields, classUid, className);
    }
  }
}

/**
 * Suggests OCSF target fields based on source field name.
 * Uses simple string matching for suggestions.
 * 
 * Requirement 14.6: Suggest OCSF target fields based on source field names
 */
function suggestTargetFields(sourceField: string, ocsfFields: OCSFField[]): OCSFField[] {
  const sourceLower = sourceField.toLowerCase();
  const sourceWords = sourceLower.split(/[._\-\s]+/);
  
  return ocsfFields
    .map(field => {
      const fieldLower = field.path.toLowerCase();
      const fieldWords = fieldLower.split(/[._\-\s]+/);
      
      // Calculate match score
      let score = 0;
      
      // Exact match
      if (fieldLower === sourceLower) {
        score += 100;
      }
      
      // Contains source name
      if (fieldLower.includes(sourceLower)) {
        score += 50;
      }
      
      // Word overlap
      for (const word of sourceWords) {
        if (word.length > 2 && fieldWords.some(fw => fw.includes(word))) {
          score += 10;
        }
      }
      
      // Common field name patterns
      const commonMappings: Record<string, string[]> = {
        'src_ip': ['src_endpoint.ip', 'src.ip', 'source_ip'],
        'dst_ip': ['dst_endpoint.ip', 'dst.ip', 'destination_ip'],
        'user': ['actor.user.name', 'user.name'],
        'username': ['actor.user.name', 'user.name'],
        'timestamp': ['time', 'metadata.logged_time'],
        'action': ['activity_name', 'action'],
        'status': ['status', 'status_code'],
        'message': ['message', 'raw_data'],
        'hostname': ['device.hostname', 'src_endpoint.hostname'],
        'ip': ['src_endpoint.ip', 'dst_endpoint.ip'],
        'port': ['src_endpoint.port', 'dst_endpoint.port'],
      };
      
      for (const [pattern, targets] of Object.entries(commonMappings)) {
        if (sourceLower.includes(pattern)) {
          if (targets.some(t => fieldLower.includes(t))) {
            score += 30;
          }
        }
      }
      
      return { field, score };
    })
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 10)
    .map(({ field }) => field);
}

// ============================================
// Sub-Components
// ============================================

interface OCSFFieldSelectorProps {
  fields: OCSFField[];
  selectedField: string | null;
  onSelect: (field: string | null) => void;
  suggestedFields?: OCSFField[];
}

/**
 * OCSFFieldSelector sub-component for selecting target OCSF fields.
 * Displays a searchable list of OCSF fields with suggestions.
 */
function OCSFFieldSelector({ 
  fields, 
  selectedField, 
  onSelect,
  suggestedFields = []
}: OCSFFieldSelectorProps) {
  const [searchQuery, setSearchQuery] = useState('');
  
  const filteredFields = useMemo(() => {
    if (!searchQuery) {
      // Show suggestions first, then all fields
      const suggestionPaths = new Set(suggestedFields.map(f => f.path));
      const otherFields = fields.filter(f => !suggestionPaths.has(f.path));
      return [...suggestedFields, ...otherFields.slice(0, 50)];
    }
    
    const query = searchQuery.toLowerCase();
    return fields
      .filter(f => 
        f.path.toLowerCase().includes(query) ||
        f.caption.toLowerCase().includes(query) ||
        f.description.toLowerCase().includes(query)
      )
      .slice(0, 50);
  }, [fields, searchQuery, suggestedFields]);
  
  return (
    <div className="ocsf-field-selector">
      <input
        type="text"
        className="field-search"
        placeholder="Search OCSF fields..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
      />
      
      {suggestedFields.length > 0 && !searchQuery && (
        <div className="suggestions-header">
          <span className="suggestions-label">Suggested</span>
        </div>
      )}
      
      <div className="field-list">
        {filteredFields.map((field, index) => {
          const isSuggested = suggestedFields.some(s => s.path === field.path);
          const showDivider = !searchQuery && 
            index === suggestedFields.length && 
            suggestedFields.length > 0;
          
          return (
            <div key={field.path}>
              {showDivider && (
                <div className="field-divider">
                  <span>All Fields</span>
                </div>
              )}
              <div
                className={`ocsf-field ${selectedField === field.path ? 'selected' : ''} ${isSuggested ? 'suggested' : ''}`}
                onClick={() => onSelect(selectedField === field.path ? null : field.path)}
                title={field.description}
              >
                <div className="field-info">
                  <span className="field-path">{field.path}</span>
                  <span className="field-caption">{field.caption}</span>
                </div>
                <div className="field-meta">
                  <span className={`requirement-badge ${field.requirement}`}>
                    {field.requirement}
                  </span>
                  <span className="field-type-badge">{field.type}</span>
                </div>
              </div>
            </div>
          );
        })}
        
        {filteredFields.length === 0 && (
          <div className="no-fields">
            {searchQuery ? `No fields matching "${searchQuery}"` : 'No fields available'}
          </div>
        )}
      </div>
    </div>
  );
}

interface MappingRowProps {
  mapping: FieldMapping;
  onUpdate: (updates: Partial<FieldMapping>) => void;
  onRemove: () => void;
}

/**
 * MappingRow sub-component for displaying and editing a single field mapping.
 * Includes transformation input for field value conversion.
 * 
 * Requirement 14.7: Support transformation expressions for field value conversion
 */
function MappingRow({ mapping, onUpdate, onRemove }: MappingRowProps) {
  const [showTransform, setShowTransform] = useState(!!mapping.transformation);
  
  return (
    <div className="mapping-row">
      <div className="mapping-fields">
        <span className="source-field">
          <code>{mapping.source_field}</code>
        </span>
        <span className="mapping-arrow">→</span>
        <span className="target-field">
          <code>{mapping.target_field}</code>
        </span>
      </div>
      
      <div className="mapping-controls">
        <button
          type="button"
          className={`transform-toggle ${showTransform ? 'active' : ''}`}
          onClick={() => setShowTransform(!showTransform)}
          title="Add transformation"
        >
          ƒ(x)
        </button>
        <button
          type="button"
          className="remove-btn"
          onClick={onRemove}
          title="Remove mapping"
        >
          ×
        </button>
      </div>
      
      {showTransform && (
        <div className="transformation-input">
          <label>Transformation:</label>
          <input
            type="text"
            placeholder="e.g., UPPER(value), CAST(value AS INTEGER)"
            value={mapping.transformation || ''}
            onChange={(e) => onUpdate({ transformation: e.target.value || undefined })}
          />
          <span className="transform-help">
            Use SQL expressions. Reference source value as <code>value</code>.
          </span>
        </div>
      )}
    </div>
  );
}

// ============================================
// Main Component
// ============================================

export function MappingBuilder() {
  const { 
    parsedFields, 
    fieldMappings, 
    selectedSourceField, 
    selectedTargetField,
    setSelectedSourceField, 
    setSelectedTargetField,
    addFieldMapping, 
    removeFieldMapping, 
    updateFieldMapping
  } = useIndexStore();
  
  const { data: schema, isLoading: schemaLoading } = useSchema();
  
  // Flatten OCSF schema attributes for target field selection
  const ocsfFields = useMemo(() => {
    if (!schema) return [];
    return flattenSchemaAttributes(schema);
  }, [schema]);
  
  // Get suggested target fields based on selected source field
  const suggestedTargetFields = useMemo(() => {
    if (!selectedSourceField || ocsfFields.length === 0) return [];
    return suggestTargetFields(selectedSourceField, ocsfFields);
  }, [selectedSourceField, ocsfFields]);
  
  // Handle creating a new mapping
  const handleCreateMapping = useCallback(() => {
    if (!selectedSourceField || !selectedTargetField) return;
    
    addFieldMapping({
      source_field: selectedSourceField,
      target_field: selectedTargetField,
    });
    
    setSelectedSourceField(null);
    setSelectedTargetField(null);
  }, [selectedSourceField, selectedTargetField, addFieldMapping, setSelectedSourceField, setSelectedTargetField]);
  
  // Get unmapped source fields
  const unmappedSourceFields = useMemo(() => 
    parsedFields.filter(f => !fieldMappings.some(m => m.source_field === f.path)),
    [parsedFields, fieldMappings]
  );
  
  // Get mapped source fields
  const mappedSourceFields = useMemo(() =>
    parsedFields.filter(f => fieldMappings.some(m => m.source_field === f.path)),
    [parsedFields, fieldMappings]
  );
  
  // Calculate mapping progress
  const mappingProgress = parsedFields.length > 0 
    ? Math.round((fieldMappings.length / parsedFields.length) * 100)
    : 0;

  // Semantic search state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchEnabled, setSearchEnabled] = useState(false);
  const { data: searchResults = [], isFetching: searchFetching } = useCatalogSearch(
    searchQuery, 5, searchEnabled && searchQuery.trim().length > 0
  );
  
  if (parsedFields.length === 0) {
    return (
      <div className="mapping-builder empty-state">
        <div className="empty-message">
          <h4>No Source Fields</h4>
          <p>Import a log sample first to see source fields for mapping.</p>
        </div>
      </div>
    );
  }
  
  return (
    <div className="mapping-builder">
      <div className="mapping-header">
        <h3>Field Mappings</h3>
        <div className="mapping-progress">
          <span className="mapping-count">
            {fieldMappings.length} / {parsedFields.length} fields mapped
          </span>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${mappingProgress}%` }}
            />
          </div>
        </div>
      </div>

      <div className="catalog-search-bar">
        <input
          className="catalog-search-input"
          type="search"
          placeholder="🔍 Semantic search catalog entries…"
          value={searchQuery}
          onChange={(e) => { setSearchQuery(e.target.value); setSearchEnabled(true); }}
          onBlur={() => { if (!searchQuery) setSearchEnabled(false); }}
        />
        {searchFetching && <span className="catalog-search-spinner">⏳</span>}
        {searchResults.length > 0 && searchQuery && (
          <div className="catalog-search-results">
            {searchResults.map((entry: CatalogEntry, i: number) => (
              <div
                key={i}
                className="catalog-search-result"
                onClick={() => {
                  setSelectedTargetField(entry.entity_name);
                  setSearchQuery('');
                  setSearchEnabled(false);
                }}
              >
                <span className="result-name">{entry.entity_name}</span>
                <code className="result-version">{entry.ocsf_version}</code>
                {entry.caption && <span className="result-caption">{entry.caption}</span>}
              </div>
            ))}
          </div>
        )}
      </div>
      
      <div className="mapping-workspace">
        <div className="source-fields-panel">
          <h4>Source Fields</h4>
          <div className="source-fields-list">
            {unmappedSourceFields.length === 0 ? (
              <div className="all-mapped">
                <span className="check-icon">✓</span>
                All fields mapped
              </div>
            ) : (
              unmappedSourceFields.map(field => (
                <div
                  key={field.path}
                  className={`source-field ${selectedSourceField === field.path ? 'selected' : ''}`}
                  onClick={() => setSelectedSourceField(
                    selectedSourceField === field.path ? null : field.path
                  )}
                >
                  <span className="field-name">
                    <code>{field.path}</code>
                  </span>
                  <span className={`field-type type-${field.type}`}>
                    {field.type}
                  </span>
                </div>
              ))
            )}
          </div>
          
          {mappedSourceFields.length > 0 && (
            <div className="mapped-fields-section">
              <h5>Already Mapped ({mappedSourceFields.length})</h5>
              <div className="mapped-fields-list">
                {mappedSourceFields.map(field => (
                  <div key={field.path} className="source-field mapped">
                    <span className="field-name">
                      <code>{field.path}</code>
                    </span>
                    <span className="mapped-indicator">✓</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
        
        <div className="mapping-actions-panel">
          <button 
            className="btn primary create-mapping-btn"
            disabled={!selectedSourceField || !selectedTargetField}
            onClick={handleCreateMapping}
          >
            Create Mapping →
          </button>
          
          {selectedSourceField && !selectedTargetField && (
            <p className="action-hint">Select a target OCSF field</p>
          )}
          {!selectedSourceField && (
            <p className="action-hint">Select a source field to map</p>
          )}
        </div>
        
        <div className="target-fields-panel">
          <h4>OCSF Fields</h4>
          {schemaLoading ? (
            <div className="loading-fields">Loading schema...</div>
          ) : (
            <OCSFFieldSelector 
              fields={ocsfFields}
              selectedField={selectedTargetField}
              onSelect={setSelectedTargetField}
              suggestedFields={suggestedTargetFields}
            />
          )}
        </div>
      </div>
      
      <div className="current-mappings">
        <h4>Current Mappings</h4>
        {fieldMappings.length === 0 ? (
          <div className="no-mappings">
            <p>No mappings created yet. Select source and target fields above.</p>
          </div>
        ) : (
          <div className="mappings-list">
            {fieldMappings.map(mapping => (
              <MappingRow 
                key={mapping.source_field}
                mapping={mapping}
                onUpdate={(updates) => updateFieldMapping(mapping.source_field, updates)}
                onRemove={() => removeFieldMapping(mapping.source_field)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default MappingBuilder;
