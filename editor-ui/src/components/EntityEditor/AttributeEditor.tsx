/**
 * AttributeEditor component for editing semantic attribute properties.
 * 
 * Provides form fields for name, caption, type, dimension flag, and OCSF mapping.
 * Integrates with index store for mapping suggestions from field mappings.
 * 
 * Requirements: 2.5, 16.1, 16.2, 16.3, 16.4, 16.5, 16.6
 */

import { useState, useCallback, useMemo } from 'react';
import type { SemanticAttribute, SemanticType, ThreatRelevance, FieldMapping } from '../../types';
import { createDefaultAttribute } from '../../types';
import { useIndexStore } from '../../store/indexStore';

// ============================================
// Types
// ============================================

export interface AttributeEditorProps {
  attribute?: SemanticAttribute;
  existingNames: Set<string>;
  onSave: (attribute: Partial<SemanticAttribute>) => void;
  onCancel: () => void;
  onDelete?: () => void;
  isNew?: boolean;
}

/**
 * A mapping suggestion with confidence score.
 * Requirements: 16.1, 16.3
 */
export interface MappingSuggestion {
  sourceField: string;
  targetField: string;
  confidence: number;
  transformation?: string;
  matchReason: string;
}

const SEMANTIC_TYPES: SemanticType[] = [
  'string',
  'integer',
  'float',
  'boolean',
  'timestamp',
  'json',
];

// Helper to parse comma-separated string to array
const parseCommaSeparated = (value: string): string[] => {
  return value
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
};

// Helper to format array as comma-separated string
const formatCommaSeparated = (arr: string[] | undefined): string => {
  return arr?.join(', ') ?? '';
};

// ============================================
// Mapping Suggestion Utilities
// ============================================

/**
 * Calculate similarity between two strings using Levenshtein distance.
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
  
  // Levenshtein distance
  const matrix: number[][] = [];
  for (let i = 0; i <= s1.length; i++) {
    matrix[i] = [i];
  }
  for (let j = 0; j <= s2.length; j++) {
    matrix[0][j] = j;
  }
  for (let i = 1; i <= s1.length; i++) {
    for (let j = 1; j <= s2.length; j++) {
      const cost = s1[i - 1] === s2[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,
        matrix[i][j - 1] + 1,
        matrix[i - 1][j - 1] + cost
      );
    }
  }
  
  const maxLen = Math.max(s1.length, s2.length);
  return 1 - matrix[s1.length][s2.length] / maxLen;
}

/**
 * Find mapping suggestions for an attribute based on field mappings.
 * Requirements: 16.1, 16.3
 */
function findMappingSuggestions(
  attributeName: string,
  fieldMappings: FieldMapping[]
): MappingSuggestion[] {
  if (fieldMappings.length === 0) return [];
  
  const suggestions: MappingSuggestion[] = [];
  
  for (const mapping of fieldMappings) {
    // Check similarity with target field (OCSF field)
    const targetSimilarity = calculateStringSimilarity(attributeName, mapping.target_field);
    
    // Check similarity with source field
    const sourceSimilarity = calculateStringSimilarity(attributeName, mapping.source_field);
    
    // Use the higher similarity
    const similarity = Math.max(targetSimilarity, sourceSimilarity);
    
    if (similarity >= 0.4) {
      let matchReason = '';
      if (targetSimilarity >= sourceSimilarity) {
        if (targetSimilarity === 1) {
          matchReason = 'Exact match with OCSF target field';
        } else if (targetSimilarity >= 0.8) {
          matchReason = 'Strong match with OCSF target field';
        } else {
          matchReason = 'Partial match with OCSF target field';
        }
      } else {
        if (sourceSimilarity === 1) {
          matchReason = 'Exact match with source field';
        } else if (sourceSimilarity >= 0.8) {
          matchReason = 'Strong match with source field';
        } else {
          matchReason = 'Partial match with source field';
        }
      }
      
      suggestions.push({
        sourceField: mapping.source_field,
        targetField: mapping.target_field,
        confidence: similarity,
        transformation: mapping.transformation,
        matchReason,
      });
    }
  }
  
  // Sort by confidence descending
  return suggestions.sort((a, b) => b.confidence - a.confidence);
}

/**
 * Get confidence level label and color class.
 * Requirements: 16.3
 */
function getConfidenceDisplay(confidence: number): { label: string; className: string } {
  if (confidence >= 0.9) {
    return { label: 'High', className: 'confidence-high' };
  } else if (confidence >= 0.7) {
    return { label: 'Medium', className: 'confidence-medium' };
  } else {
    return { label: 'Low', className: 'confidence-low' };
  }
}

// ============================================
// Component
// ============================================

export function AttributeEditor({
  attribute,
  existingNames,
  onSave,
  onCancel,
  onDelete,
  isNew = false,
}: AttributeEditorProps) {
  // Get field mappings from index store
  const fieldMappings = useIndexStore((state) => state.fieldMappings);
  
  // Form state - Basic
  const [name, setName] = useState(attribute?.name ?? '');
  const [caption, setCaption] = useState(attribute?.caption ?? '');
  const [description, setDescription] = useState(attribute?.description ?? '');
  const [attrType, setAttrType] = useState<SemanticType>(
    typeof attribute?.attr_type === 'object' 
      ? (attribute.attr_type as { array: SemanticType }).array 
      : (attribute?.attr_type ?? 'string')
  );
  const [isArray, setIsArray] = useState(
    typeof attribute?.attr_type === 'object'
  );
  const [isDimension, setIsDimension] = useState(attribute?.is_dimension ?? false);
  const [isObservable, setIsObservable] = useState(attribute?.is_observable ?? false);
  const [ocsfField, setOcsfField] = useState(attribute?.ocsf_mapping.field ?? '');
  const [ocsfExpression, setOcsfExpression] = useState(attribute?.ocsf_mapping.expression ?? '');
  
  // Mapping suggestion state (Requirements: 16.1, 16.4)
  const [showSuggestions, setShowSuggestions] = useState(true);
  const [selectedSuggestion, setSelectedSuggestion] = useState<MappingSuggestion | null>(null);
  
  // Form state - Advanced/Security
  const [showAdvanced, setShowAdvanced] = useState(
    // Auto-expand if any advanced fields have values
    !!(attribute?.synonyms?.length || attribute?.security_context || 
       attribute?.value_pattern || attribute?.sample_values?.length ||
       attribute?.threat_relevance?.use_cases?.length || 
       attribute?.threat_relevance?.mitre_techniques?.length)
  );
  const [synonyms, setSynonyms] = useState(formatCommaSeparated(attribute?.synonyms));
  const [sampleValues, setSampleValues] = useState(formatCommaSeparated(attribute?.sample_values));
  const [securityContext, setSecurityContext] = useState(attribute?.security_context ?? '');
  const [valuePattern, setValuePattern] = useState(attribute?.value_pattern ?? '');
  const [useCases, setUseCases] = useState(formatCommaSeparated(attribute?.threat_relevance?.use_cases));
  const [mitreTechniques, setMitreTechniques] = useState(formatCommaSeparated(attribute?.threat_relevance?.mitre_techniques));
  
  // Validation
  const [errors, setErrors] = useState<Record<string, string>>({});
  
  // Calculate mapping suggestions based on attribute name (Requirements: 16.1, 16.3)
  const mappingSuggestions = useMemo(() => {
    if (!name.trim()) return [];
    return findMappingSuggestions(name, fieldMappings);
  }, [name, fieldMappings]);
  
  // Check if attribute has a mapping (for highlighting unmapped - Requirement 16.5)
  const hasMapping = useMemo(() => {
    return !!(ocsfField.trim() || ocsfExpression.trim());
  }, [ocsfField, ocsfExpression]);
  
  // Apply a mapping suggestion (Requirements: 16.2, 16.4)
  const applySuggestion = useCallback((suggestion: MappingSuggestion) => {
    setOcsfField(suggestion.targetField);
    if (suggestion.transformation) {
      setOcsfExpression(suggestion.transformation);
    }
    setSelectedSuggestion(suggestion);
    // Clear mapping error if present
    if (errors.mapping) {
      setErrors((prev) => {
        const { mapping, ...rest } = prev;
        return rest;
      });
    }
  }, [errors.mapping]);
  
  const validateForm = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      newErrors.name = 'Name must be lowercase with underscores';
    } else if (isNew && existingNames.has(name)) {
      newErrors.name = 'An attribute with this name already exists';
    }
    
    if (!caption.trim()) {
      newErrors.caption = 'Caption is required';
    }
    
    if (!ocsfField.trim() && !ocsfExpression.trim()) {
      newErrors.mapping = 'Either field path or expression is required';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [name, caption, ocsfField, ocsfExpression, isNew, existingNames]);
  
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      
      if (!validateForm()) return;
      
      const finalType: SemanticType = isArray ? { array: attrType } : attrType;
      
      // Build threat_relevance if any values present
      const parsedUseCases = parseCommaSeparated(useCases);
      const parsedMitreTechniques = parseCommaSeparated(mitreTechniques);
      const threatRelevance: ThreatRelevance | undefined = 
        (parsedUseCases.length > 0 || parsedMitreTechniques.length > 0)
          ? { use_cases: parsedUseCases, mitre_techniques: parsedMitreTechniques }
          : undefined;
      
      const updates: Partial<SemanticAttribute> = {
        name: name.trim(),
        caption: caption.trim(),
        description: description.trim(),
        attr_type: finalType,
        is_dimension: isDimension,
        is_observable: isObservable,
        ocsf_mapping: {
          ...(ocsfField.trim() && { field: ocsfField.trim() }),
          ...(ocsfExpression.trim() && { expression: ocsfExpression.trim() }),
        },
        // Advanced fields
        synonyms: parseCommaSeparated(synonyms),
        sample_values: parseCommaSeparated(sampleValues),
        security_context: securityContext.trim() || undefined,
        value_pattern: valuePattern.trim() || undefined,
        threat_relevance: threatRelevance,
      };
      
      if (isNew) {
        // For new attributes, include all default fields
        const newAttr: SemanticAttribute = {
          ...createDefaultAttribute(name.trim()),
          ...updates,
        };
        onSave(newAttr);
      } else {
        onSave(updates);
      }
    },
    [
      name,
      caption,
      description,
      attrType,
      isArray,
      isDimension,
      isObservable,
      ocsfField,
      ocsfExpression,
      synonyms,
      sampleValues,
      securityContext,
      valuePattern,
      useCases,
      mitreTechniques,
      isNew,
      validateForm,
      onSave,
    ]
  );

  return (
    <form className="attribute-editor" onSubmit={handleSubmit}>
      <div className="attribute-editor-header">
        <span className="editor-title">
          {isNew ? 'New Attribute' : 'Edit Attribute'}
        </span>
      </div>
      
      <div className="attribute-editor-body">
        {/* Name and Caption Row */}
        <div className="form-row">
          <div className="form-group half">
            <label className="form-label" htmlFor="attr-name">
              Name <span className="required">*</span>
            </label>
            <input
              id="attr-name"
              type="text"
              className={`form-input ${errors.name ? 'error' : ''}`}
              value={name}
              onChange={(e) => {
                setName(e.target.value);
                if (errors.name) {
                  setErrors((prev) => {
                    const { name, ...rest } = prev;
                    return rest;
                  });
                }
              }}
              placeholder="e.g., source_ip"
              disabled={!isNew}
            />
            {errors.name && <span className="form-error">{errors.name}</span>}
          </div>
          
          <div className="form-group half">
            <label className="form-label" htmlFor="attr-caption">
              Caption <span className="required">*</span>
            </label>
            <input
              id="attr-caption"
              type="text"
              className={`form-input ${errors.caption ? 'error' : ''}`}
              value={caption}
              onChange={(e) => {
                setCaption(e.target.value);
                if (errors.caption) {
                  setErrors((prev) => {
                    const { caption, ...rest } = prev;
                    return rest;
                  });
                }
              }}
              placeholder="e.g., Source IP Address"
            />
            {errors.caption && <span className="form-error">{errors.caption}</span>}
          </div>
        </div>
        
        {/* Type Row */}
        <div className="form-row">
          <div className="form-group half">
            <label className="form-label" htmlFor="attr-type">
              Type
            </label>
            <select
              id="attr-type"
              className="form-select"
              value={attrType as string}
              onChange={(e) => setAttrType(e.target.value as SemanticType)}
            >
              {SEMANTIC_TYPES.map((type) => (
                <option key={type as string} value={type as string}>
                  {type as string}
                </option>
              ))}
            </select>
          </div>
          
          <div className="form-group half">
            <label className="form-label">&nbsp;</label>
            <div className="checkbox-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={isArray}
                  onChange={(e) => setIsArray(e.target.checked)}
                />
                <span>Array</span>
              </label>
            </div>
          </div>
        </div>
        
        {/* Flags Row */}
        <div className="form-group">
          <label className="form-label">Flags</label>
          <div className="checkbox-group vertical-with-hints">
            <label className="checkbox-label with-hint">
              <div className="checkbox-main">
                <input
                  type="checkbox"
                  checked={isDimension}
                  onChange={(e) => setIsDimension(e.target.checked)}
                />
                <span>Dimension</span>
              </div>
              <span className="checkbox-hint">
                Use for GROUP BY in metrics. Mark attributes you'll filter or aggregate by (e.g., source_ip, user_name, action).
              </span>
            </label>
            <label className="checkbox-label with-hint">
              <div className="checkbox-main">
                <input
                  type="checkbox"
                  checked={isObservable}
                  onChange={(e) => setIsObservable(e.target.checked)}
                />
                <span>Observable (IOC)</span>
              </div>
              <span className="checkbox-hint">
                Mark as Indicator of Compromise for threat intel. Enables extraction to observable tables for fast lookups.
              </span>
            </label>
          </div>
        </div>
        
        {/* OCSF Mapping */}
        <div className={`form-group ${!hasMapping && mappingSuggestions.length > 0 ? 'unmapped-highlight' : ''}`}>
          <label className="form-label" htmlFor="attr-ocsf-field">
            OCSF Field Path
            {!hasMapping && <span className="unmapped-indicator"> (unmapped)</span>}
          </label>
          <input
            id="attr-ocsf-field"
            type="text"
            className={`form-input mono ${errors.mapping ? 'error' : ''}`}
            value={ocsfField}
            onChange={(e) => {
              setOcsfField(e.target.value);
              setSelectedSuggestion(null); // Clear suggestion when manually editing
              if (errors.mapping) {
                setErrors((prev) => {
                  const { mapping, ...rest } = prev;
                  return rest;
                });
              }
            }}
            placeholder="e.g., src_endpoint.ip"
          />
        </div>
        
        {/* Mapping Suggestions Panel (Requirements: 16.1, 16.3, 16.4) */}
        {mappingSuggestions.length > 0 && showSuggestions && (
          <div className="mapping-suggestions-panel">
            <div className="suggestions-header">
              <span className="suggestions-title">
                💡 Mapping Suggestions ({mappingSuggestions.length})
              </span>
              <button
                type="button"
                className="btn-icon suggestions-toggle"
                onClick={() => setShowSuggestions(false)}
                title="Hide suggestions"
              >
                ×
              </button>
            </div>
            <div className="suggestions-list">
              {mappingSuggestions.slice(0, 5).map((suggestion, index) => {
                const { label, className } = getConfidenceDisplay(suggestion.confidence);
                const isSelected = selectedSuggestion?.targetField === suggestion.targetField;
                return (
                  <div
                    key={index}
                    className={`suggestion-item ${isSelected ? 'selected' : ''}`}
                    onClick={() => applySuggestion(suggestion)}
                  >
                    <div className="suggestion-main">
                      <span className="suggestion-field">{suggestion.targetField}</span>
                      <span className={`suggestion-confidence ${className}`}>
                        {label} ({Math.round(suggestion.confidence * 100)}%)
                      </span>
                    </div>
                    <div className="suggestion-meta">
                      <span className="suggestion-reason">{suggestion.matchReason}</span>
                      <span className="suggestion-source">from: {suggestion.sourceField}</span>
                    </div>
                    {suggestion.transformation && (
                      <div className="suggestion-transform">
                        <code>{suggestion.transformation}</code>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
            {mappingSuggestions.length > 5 && (
              <div className="suggestions-more">
                +{mappingSuggestions.length - 5} more suggestions
              </div>
            )}
          </div>
        )}
        
        {/* Show suggestions toggle when hidden */}
        {mappingSuggestions.length > 0 && !showSuggestions && (
          <button
            type="button"
            className="btn-show-suggestions"
            onClick={() => setShowSuggestions(true)}
          >
            Show {mappingSuggestions.length} mapping suggestion{mappingSuggestions.length > 1 ? 's' : ''}
          </button>
        )}
        
        <div className="form-group">
          <label className="form-label" htmlFor="attr-ocsf-expr">
            Or Expression
          </label>
          <input
            id="attr-ocsf-expr"
            type="text"
            className={`form-input mono ${errors.mapping ? 'error' : ''}`}
            value={ocsfExpression}
            onChange={(e) => {
              setOcsfExpression(e.target.value);
              if (errors.mapping) {
                setErrors((prev) => {
                  const { mapping, ...rest } = prev;
                  return rest;
                });
              }
            }}
            placeholder="e.g., COALESCE(src_endpoint.ip, unmapped.src_ip)"
          />
          {errors.mapping && <span className="form-error">{errors.mapping}</span>}
        </div>
        
        {/* Description */}
        <div className="form-group">
          <label className="form-label" htmlFor="attr-description">
            Description
          </label>
          <textarea
            id="attr-description"
            className="form-textarea"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe this attribute..."
            rows={2}
          />
        </div>
        
        {/* Advanced / Security Section */}
        <div className="form-section-toggle">
          <button
            type="button"
            className="section-toggle-btn"
            onClick={() => setShowAdvanced(!showAdvanced)}
          >
            <span className={`toggle-icon ${showAdvanced ? 'expanded' : ''}`}>▶</span>
            Security &amp; Threat Intelligence
          </button>
        </div>
        
        {showAdvanced && (
          <div className="form-section advanced-section">
            {/* Synonyms */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-synonyms">
                Synonyms
                <span className="form-hint">Comma-separated alternative names for LLM queries</span>
              </label>
              <input
                id="attr-synonyms"
                type="text"
                className="form-input"
                value={synonyms}
                onChange={(e) => setSynonyms(e.target.value)}
                placeholder="e.g., client_ip, origin_ip, src_ip"
              />
            </div>
            
            {/* Sample Values */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-sample-values">
                Sample Values
                <span className="form-hint">Comma-separated example values</span>
              </label>
              <input
                id="attr-sample-values"
                type="text"
                className="form-input"
                value={sampleValues}
                onChange={(e) => setSampleValues(e.target.value)}
                placeholder="e.g., 10.0.0.1, 192.168.1.100"
              />
            </div>
            
            {/* Value Pattern */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-value-pattern">
                Value Pattern
                <span className="form-hint">Regex pattern for valid values</span>
              </label>
              <input
                id="attr-value-pattern"
                type="text"
                className="form-input mono"
                value={valuePattern}
                onChange={(e) => setValuePattern(e.target.value)}
                placeholder="e.g., ^(?:[0-9]{1,3}\\.){3}[0-9]{1,3}$"
              />
            </div>
            
            {/* Security Context */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-security-context">
                Security Context
                <span className="form-hint">How this attribute is used in threat detection</span>
              </label>
              <textarea
                id="attr-security-context"
                className="form-textarea"
                value={securityContext}
                onChange={(e) => setSecurityContext(e.target.value)}
                placeholder="e.g., Track query patterns per source IP to detect compromised hosts."
                rows={2}
              />
            </div>
            
            {/* Threat Relevance - Use Cases */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-use-cases">
                Threat Use Cases
                <span className="form-hint">Comma-separated security use cases</span>
              </label>
              <input
                id="attr-use-cases"
                type="text"
                className="form-input"
                value={useCases}
                onChange={(e) => setUseCases(e.target.value)}
                placeholder="e.g., DGA detection, DNS tunneling, C2 beaconing"
              />
            </div>
            
            {/* Threat Relevance - MITRE ATT&CK */}
            <div className="form-group">
              <label className="form-label" htmlFor="attr-mitre">
                MITRE ATT&amp;CK Techniques
                <span className="form-hint">Comma-separated technique IDs</span>
              </label>
              <input
                id="attr-mitre"
                type="text"
                className="form-input mono"
                value={mitreTechniques}
                onChange={(e) => setMitreTechniques(e.target.value)}
                placeholder="e.g., T1071.004, T1568.002, T1048.003"
              />
            </div>
          </div>
        )}
      </div>
      
      <div className="attribute-editor-footer">
        {onDelete && !isNew && (
          <button
            type="button"
            className="btn danger"
            onClick={onDelete}
          >
            Delete
          </button>
        )}
        <div className="footer-spacer" />
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="btn primary">
          {isNew ? 'Add' : 'Save'}
        </button>
      </div>
    </form>
  );
}

export default AttributeEditor;
