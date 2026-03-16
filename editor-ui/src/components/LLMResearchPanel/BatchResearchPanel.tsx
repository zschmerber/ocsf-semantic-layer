/**
 * BatchResearchPanel component for researching multiple attributes at once.
 * 
 * Supports batch research for multiple targets in a single API call.
 * Uses TanStack Query for API calls (Requirement 7.4)
 * 
 * Requirements: 4.6, 4.7, 7.4, 8.5
 */

import { useState, useCallback, useMemo } from 'react';
import type {
  SemanticEntity,
  SemanticAttribute,
  ResearchField,
  LLMSuggestion,
  OCSFContext,
  BatchResearchRequest,
  ResearchTargetRequest,
} from '../../types';
import { useEditorStore } from '../../store';
import { useLLMBatchResearch, getErrorMessage } from '../../api';
import './LLMResearchPanel.css';

// ============================================
// Types
// ============================================

export interface BatchResearchPanelProps {
  entity: SemanticEntity;
  onClose?: () => void;
}

type ResearchStatus = 'idle' | 'loading' | 'success' | 'error';

interface AttributeSuggestions {
  attributeName: string;
  suggestions: Array<{
    suggestion: LLMSuggestion;
    status: 'pending' | 'accepted' | 'rejected';
  }>;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Build OCSF context for an attribute.
 */
function buildOCSFContext(
  entity: SemanticEntity,
  attribute: SemanticAttribute,
  schema?: ReturnType<typeof useEditorStore.getState>['schema']
): OCSFContext {
  const classUid = entity.source_event_classes[0];
  let className = '';
  let classCaption = '';
  let classDescription = '';
  let categoryName = '';

  if (schema && classUid) {
    for (const category of schema.categories) {
      const eventClass = category.classes.find((c) => c.uid === classUid);
      if (eventClass) {
        className = eventClass.name;
        classCaption = eventClass.caption;
        classDescription = eventClass.description;
        categoryName = category.name;
        break;
      }
    }
  }

  return {
    class_name: className,
    class_caption: classCaption,
    class_description: classDescription,
    category_name: categoryName,
    attribute_name: attribute.ocsf_mapping.field || attribute.name,
    attribute_type: typeof attribute.attr_type === 'string' 
      ? attribute.attr_type 
      : 'array',
    attribute_description: attribute.description,
  };
}

/**
 * Format field name for display.
 */
function formatFieldName(field: ResearchField): string {
  switch (field) {
    case 'description':
      return 'Description';
    case 'synonyms':
      return 'Synonyms';
    case 'security_context':
      return 'Security Context';
    case 'sample_values':
      return 'Sample Values';
    default:
      return field;
  }
}

/**
 * Get confidence level class.
 */
function getConfidenceClass(confidence: number): string {
  if (confidence >= 0.8) return 'high';
  if (confidence >= 0.6) return 'medium';
  return 'low';
}

// ============================================
// Component
// ============================================

export function BatchResearchPanel({ entity, onClose }: BatchResearchPanelProps) {
  // Store state
  const schema = useEditorStore((state) => state.schema);
  const updateEntityAttribute = useEditorStore((state) => state.updateEntityAttribute);
  
  // Global loading state (Requirement: 8.5)
  const isGlobalLoading = useEditorStore((state) => state.isLoading);
  const startLoading = useEditorStore((state) => state.startLoading);
  const stopLoading = useEditorStore((state) => state.stopLoading);

  // Use TanStack Query mutation for batch LLM research (Requirement 7.4)
  const { 
    mutate: batchResearchMutation, 
    isPending: isResearching,
  } = useLLMBatchResearch();

  // Local state
  const [status, setStatus] = useState<ResearchStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [selectedAttributes, setSelectedAttributes] = useState<Set<string>>(
    () => new Set(entity.attributes.map((a) => a.name))
  );
  const [selectedFields, setSelectedFields] = useState<Set<ResearchField>>(
    () => new Set(['description', 'synonyms', 'security_context'])
  );
  const [results, setResults] = useState<AttributeSuggestions[]>([]);
  const [totalTokensUsed, setTotalTokensUsed] = useState<number>(0);
  const [expandedAttributes, setExpandedAttributes] = useState<Set<string>>(new Set());
  const [additionalContext, setAdditionalContext] = useState<string>('');
  const [showContextInput, setShowContextInput] = useState(false);

  // Derived state
  const availableFields: ResearchField[] = ['description', 'synonyms', 'security_context', 'sample_values'];

  const pendingSuggestionsCount = useMemo(() => {
    return results.reduce(
      (count, r) => count + r.suggestions.filter((s) => s.status === 'pending').length,
      0
    );
  }, [results]);

  // Handlers
  const handleAttributeToggle = useCallback((name: string) => {
    setSelectedAttributes((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  }, []);

  const handleFieldToggle = useCallback((field: ResearchField) => {
    setSelectedFields((prev) => {
      const next = new Set(prev);
      if (next.has(field)) {
        next.delete(field);
      } else {
        next.add(field);
      }
      return next;
    });
  }, []);

  const handleSelectAllAttributes = useCallback(() => {
    setSelectedAttributes(new Set(entity.attributes.map((a) => a.name)));
  }, [entity.attributes]);

  const handleDeselectAllAttributes = useCallback(() => {
    setSelectedAttributes(new Set());
  }, []);

  const handleBatchResearch = useCallback(async () => {
    if (selectedAttributes.size === 0) {
      setError('Please select at least one attribute');
      return;
    }
    if (selectedFields.size === 0) {
      setError('Please select at least one field to research');
      return;
    }

    setStatus('loading');
    setError(null);
    setResults([]);
    
    // Start global loading indicator (Requirement: 8.5)
    startLoading('llm', `Researching ${selectedAttributes.size} attributes...`);

    // Build batch request with all selected attributes
    const targets: ResearchTargetRequest[] = entity.attributes
      .filter((attr) => selectedAttributes.has(attr.name))
      .map((attr) => ({
        target_type: 'attribute' as const,
        entity_name: entity.name,
        attribute_name: attr.name,
        ocsf_context: buildOCSFContext(entity, attr, schema),
        requested_fields: Array.from(selectedFields),
        is_observable: attr.is_observable,
      }));

    const request: BatchResearchRequest = { 
      targets,
      shared_context: additionalContext.trim() || undefined,
    };

    // Single API call for all targets using TanStack Query (Property 12, Requirement 7.4)
    batchResearchMutation(request, {
      onSuccess: (data) => {
        console.log('Batch research raw response:', data);
        
        // Map results to attributes
        const attributeResults: AttributeSuggestions[] = targets.map((target, index) => {
          const response = data[index];
          console.log(`Response for ${target.attribute_name}:`, response);
          return {
            attributeName: target.attribute_name!,
            suggestions: (response?.suggestions || []).map((s) => ({
              suggestion: s,
              status: 'pending' as const,
            })),
          };
        });

        console.log('Mapped attribute results:', attributeResults);
        setResults(attributeResults);
        setTotalTokensUsed(data.reduce((sum, r) => sum + (r?.tokens_used || 0), 0));
        setStatus('success');
        stopLoading();
      },
      onError: (err) => {
        console.error('Batch research error:', err);
        setError(getErrorMessage(err));
        setStatus('error');
        stopLoading();
      },
    });
  }, [entity, selectedAttributes, selectedFields, schema, startLoading, stopLoading, batchResearchMutation]);

  const handleAcceptSuggestion = useCallback(
    (attributeName: string, suggestionId: string) => {
      const attrResult = results.find((r) => r.attributeName === attributeName);
      const suggestionState = attrResult?.suggestions.find(
        (s) => s.suggestion.id === suggestionId
      );
      if (!suggestionState || suggestionState.status !== 'pending') return;

      const { suggestion } = suggestionState;

      // Apply the suggestion to the model
      const updates: Partial<SemanticAttribute> = {};
      switch (suggestion.field) {
        case 'description':
          updates.description = suggestion.value as string;
          break;
        case 'synonyms':
          updates.synonyms = suggestion.value as string[];
          break;
        case 'security_context':
          updates.security_context = suggestion.value as string;
          break;
        case 'sample_values':
          updates.sample_values = suggestion.value as string[];
          break;
      }

      if (Object.keys(updates).length > 0) {
        updateEntityAttribute(entity.name, attributeName, updates);
      }

      // Update suggestion status
      setResults((prev) =>
        prev.map((r) =>
          r.attributeName === attributeName
            ? {
                ...r,
                suggestions: r.suggestions.map((s) =>
                  s.suggestion.id === suggestionId ? { ...s, status: 'accepted' } : s
                ),
              }
            : r
        )
      );
    },
    [entity.name, results, updateEntityAttribute]
  );

  const handleRejectSuggestion = useCallback(
    (attributeName: string, suggestionId: string) => {
      setResults((prev) =>
        prev.map((r) =>
          r.attributeName === attributeName
            ? {
                ...r,
                suggestions: r.suggestions.map((s) =>
                  s.suggestion.id === suggestionId ? { ...s, status: 'rejected' } : s
                ),
              }
            : r
        )
      );
    },
    []
  );

  const handleAcceptAll = useCallback(() => {
    results.forEach((attrResult) => {
      attrResult.suggestions
        .filter((s) => s.status === 'pending')
        .forEach((s) => handleAcceptSuggestion(attrResult.attributeName, s.suggestion.id));
    });
  }, [results, handleAcceptSuggestion]);

  const handleRetry = useCallback(() => {
    setStatus('idle');
    setError(null);
    setResults([]);
    setExpandedAttributes(new Set());
  }, []);

  const toggleAttributeExpanded = useCallback((attributeName: string) => {
    setExpandedAttributes((prev) => {
      const next = new Set(prev);
      if (next.has(attributeName)) {
        next.delete(attributeName);
      } else {
        next.add(attributeName);
      }
      return next;
    });
  }, []);

  const expandAllAttributes = useCallback(() => {
    setExpandedAttributes(new Set(results.map((r) => r.attributeName)));
  }, [results]);

  const collapseAllAttributes = useCallback(() => {
    setExpandedAttributes(new Set());
  }, []);

  return (
    <div className="llm-research-panel batch-panel">
      <div className="research-panel-header">
        <div className="research-panel-title">
          <span className="research-icon">🔬</span>
          <span>Batch Research</span>
        </div>
        {onClose && (
          <button
            className="btn-icon"
            onClick={onClose}
            aria-label="Close batch research panel"
          >
            ✕
          </button>
        )}
      </div>

      <div className="research-panel-body">
        {/* Entity Info */}
        <div className="research-target-info">
          <span className="target-label">Entity:</span>
          <span className="target-name">{entity.caption || entity.name}</span>
          <span className="target-count">
            ({entity.attributes.length} attributes)
          </span>
        </div>

        {/* Selection Phase */}
        {status === 'idle' && (
          <>
            {/* Attribute Selection */}
            <div className="batch-selection-section">
              <div className="selection-header">
                <label className="field-selection-label">
                  Select attributes to research:
                </label>
                <div className="selection-actions">
                  <button
                    className="btn small"
                    onClick={handleSelectAllAttributes}
                  >
                    Select All
                  </button>
                  <button
                    className="btn small"
                    onClick={handleDeselectAllAttributes}
                  >
                    Deselect All
                  </button>
                </div>
              </div>
              <div className="attribute-checkboxes">
                {entity.attributes.map((attr) => (
                  <label key={attr.name} className="attribute-checkbox">
                    <input
                      type="checkbox"
                      checked={selectedAttributes.has(attr.name)}
                      onChange={() => handleAttributeToggle(attr.name)}
                    />
                    <span className="attr-checkbox-name">
                      {attr.caption || attr.name}
                    </span>
                    {attr.is_observable && (
                      <span className="attr-checkbox-badge observable">IOC</span>
                    )}
                  </label>
                ))}
              </div>
            </div>

            {/* Field Selection */}
            <div className="research-field-selection">
              <label className="field-selection-label">
                Select fields to research:
              </label>
              <div className="field-checkboxes">
                {availableFields.map((field) => (
                  <label key={field} className="field-checkbox">
                    <input
                      type="checkbox"
                      checked={selectedFields.has(field)}
                      onChange={() => handleFieldToggle(field)}
                    />
                    <span>{formatFieldName(field)}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* Additional Context Section */}
            <div className="research-context-section">
              <button
                className={`context-toggle-btn ${showContextInput ? 'expanded' : ''}`}
                onClick={() => setShowContextInput(!showContextInput)}
              >
                <span className="toggle-icon">{showContextInput ? '▼' : '▶'}</span>
                <span>Add Context</span>
                {additionalContext && !showContextInput && (
                  <span className="context-indicator">✓ Context added</span>
                )}
              </button>
              
              {showContextInput && (
                <div className="context-input-area">
                  <p className="context-help-text">
                    Provide additional context to help generate better suggestions. 
                    You can paste detection rules, documentation, field descriptions, 
                    or any relevant information about how this entity is used.
                  </p>
                  <textarea
                    className="context-textarea"
                    value={additionalContext}
                    onChange={(e) => setAdditionalContext(e.target.value)}
                    placeholder="Examples:
• Detection rules that use these fields
• Documentation about the data source
• Sample log entries or events
• Business context for how this entity is used
• Related MITRE ATT&CK techniques
• Field naming conventions from your organization"
                    rows={8}
                  />
                  <div className="context-actions">
                    <span className="context-char-count">
                      {additionalContext.length} characters
                    </span>
                    {additionalContext && (
                      <button
                        className="btn small"
                        onClick={() => setAdditionalContext('')}
                      >
                        Clear
                      </button>
                    )}
                  </div>
                </div>
              )}
            </div>
          </>
        )}

        {/* Loading State */}
        {(status === 'loading' || isResearching) && (
          <div className="research-loading">
            <div className="loading-spinner" />
            <span>
              Researching {selectedAttributes.size} attribute
              {selectedAttributes.size !== 1 ? 's' : ''}...
            </span>
          </div>
        )}

        {/* Error State */}
        {status === 'error' && error && (
          <div className="research-error">
            <div className="error-icon">⚠️</div>
            <div className="error-message">{error}</div>
            <button className="btn" onClick={handleRetry}>
              Retry
            </button>
          </div>
        )}

        {/* Results */}
        {status === 'success' && results.length > 0 && (
          <div className="batch-results">
            <div className="suggestions-header">
              <span className="suggestions-count">
                {results.reduce((sum, r) => sum + r.suggestions.length, 0)} suggestions
                for {results.length} attributes
              </span>
              <div className="suggestions-header-actions">
                {totalTokensUsed > 0 && (
                  <span className="tokens-used">{totalTokensUsed} tokens</span>
                )}
                <button className="btn small" onClick={expandAllAttributes}>
                  Expand All
                </button>
                <button className="btn small" onClick={collapseAllAttributes}>
                  Collapse All
                </button>
              </div>
            </div>

            <div className="batch-results-list">
              {results.map((attrResult) => {
                const isExpanded = expandedAttributes.has(attrResult.attributeName);
                const pendingCount = attrResult.suggestions.filter(
                  (s) => s.status === 'pending'
                ).length;
                const acceptedCount = attrResult.suggestions.filter(
                  (s) => s.status === 'accepted'
                ).length;

                return (
                  <div key={attrResult.attributeName} className="batch-result-item collapsible">
                    <button
                      className={`batch-result-header clickable ${isExpanded ? 'expanded' : ''}`}
                      onClick={() => toggleAttributeExpanded(attrResult.attributeName)}
                    >
                      <span className="expand-icon">{isExpanded ? '▼' : '▶'}</span>
                      <span className="batch-result-attr-name">
                        {entity.attributes.find(
                          (a) => a.name === attrResult.attributeName
                        )?.caption || attrResult.attributeName}
                      </span>
                      <span className="batch-result-badges">
                        {pendingCount > 0 && (
                          <span className="badge pending">{pendingCount} pending</span>
                        )}
                        {acceptedCount > 0 && (
                          <span className="badge accepted">{acceptedCount} accepted</span>
                        )}
                        {attrResult.suggestions.length === 0 && (
                          <span className="badge empty">No suggestions</span>
                        )}
                      </span>
                    </button>

                    {isExpanded && attrResult.suggestions.length > 0 && (
                      <div className="batch-result-suggestions">
                        {attrResult.suggestions.map(({ suggestion, status: suggStatus }) => (
                          <div
                            key={suggestion.id}
                            className={`suggestion-item ${suggStatus}`}
                          >
                            <div className="suggestion-header">
                              <span className="suggestion-field">
                                {formatFieldName(suggestion.field)}
                              </span>
                              <span
                                className={`suggestion-confidence ${getConfidenceClass(
                                  suggestion.confidence
                                )}`}
                              >
                                {Math.round(suggestion.confidence * 100)}%
                              </span>
                            </div>

                            <div className="suggestion-value">
                              {Array.isArray(suggestion.value) ? (
                                <ul className="suggestion-list">
                                  {suggestion.value.map((v, i) => (
                                    <li key={i}>{v}</li>
                                  ))}
                                </ul>
                              ) : (
                                <p>{suggestion.value}</p>
                              )}
                            </div>

                            {suggStatus === 'pending' && (
                              <div className="suggestion-actions">
                                <button
                                  className="btn primary small"
                                  onClick={() =>
                                    handleAcceptSuggestion(
                                      attrResult.attributeName,
                                      suggestion.id
                                    )
                                  }
                                >
                                  ✓ Accept
                                </button>
                                <button
                                  className="btn small"
                                  onClick={() =>
                                    handleRejectSuggestion(
                                      attrResult.attributeName,
                                      suggestion.id
                                    )
                                  }
                                >
                                  ✕ Reject
                                </button>
                              </div>
                            )}

                            {suggStatus === 'accepted' && (
                              <div className="suggestion-status accepted">
                                ✓ Applied
                              </div>
                            )}

                            {suggStatus === 'rejected' && (
                              <div className="suggestion-status rejected">
                                ✕ Rejected
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}

                    {isExpanded && attrResult.suggestions.length === 0 && (
                      <div className="batch-result-empty">
                        No suggestions generated for this attribute
                      </div>
                    )}
                  </div>
                );
              })}
            </div>

            {pendingSuggestionsCount > 1 && (
              <div className="suggestions-bulk-actions">
                <button className="btn primary" onClick={handleAcceptAll}>
                  Accept All ({pendingSuggestionsCount})
                </button>
              </div>
            )}
          </div>
        )}

        {/* No Results */}
        {status === 'success' && results.length === 0 && (
          <div className="research-empty">
            <div className="empty-icon">🤷</div>
            <p>No suggestions generated.</p>
            <button className="btn" onClick={handleRetry}>
              Try Again
            </button>
          </div>
        )}
      </div>

      {/* Footer Actions */}
      {status === 'idle' && (
        <div className="research-panel-footer">
          <button
            className="btn primary"
            onClick={handleBatchResearch}
            disabled={selectedAttributes.size === 0 || selectedFields.size === 0 || isGlobalLoading || isResearching}
          >
            🔬 Research {selectedAttributes.size} Attribute
            {selectedAttributes.size !== 1 ? 's' : ''}
          </button>
        </div>
      )}
    </div>
  );
}

export default BatchResearchPanel;
