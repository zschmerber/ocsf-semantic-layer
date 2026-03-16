/**
 * LLMResearchPanel component for LLM-assisted semantic enrichment.
 * 
 * Provides a research panel accessible from entity and attribute editors
 * with suggestion display and accept/reject actions.
 * 
 * Uses TanStack Query for API calls (Requirement 7.4)
 * 
 * Requirements: 4.1, 4.5, 4.8, 7.4, 8.5
 */

import { useState, useCallback, useMemo } from 'react';
import type {
  ResearchTarget,
  ResearchField,
  LLMSuggestion,
  OCSFContext,
  ResearchRequest,
  SemanticEntity,
  SemanticAttribute,
} from '../../types';
import { useEditorStore } from '../../store';
import { useLLMResearch, getErrorMessage } from '../../api';
import './LLMResearchPanel.css';

// ============================================
// Types
// ============================================

export interface LLMResearchPanelProps {
  target: ResearchTarget;
  onClose?: () => void;
}

type ResearchStatus = 'idle' | 'loading' | 'success' | 'error';

interface SuggestionState {
  suggestion: LLMSuggestion;
  status: 'pending' | 'accepted' | 'rejected';
}

// ============================================
// Helper Functions
// ============================================

/**
 * Build OCSF context from entity and schema.
 */
function buildOCSFContext(
  entity: SemanticEntity,
  attribute?: SemanticAttribute,
  schema?: ReturnType<typeof useEditorStore.getState>['schema']
): OCSFContext {
  // Find the first event class for context
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

  const context: OCSFContext = {
    class_name: className,
    class_caption: classCaption,
    class_description: classDescription,
    category_name: categoryName,
  };

  if (attribute) {
    context.attribute_name = attribute.ocsf_mapping.field || attribute.name;
    context.attribute_type = typeof attribute.attr_type === 'string' 
      ? attribute.attr_type 
      : 'array';
    context.attribute_description = attribute.description;
  }

  return context;
}

/**
 * Get available research fields based on target type.
 */
function getAvailableFields(target: ResearchTarget): ResearchField[] {
  if (target.type === 'entity') {
    return ['description', 'synonyms', 'security_context'];
  }
  return ['description', 'synonyms', 'security_context', 'sample_values'];
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
 * Get confidence level label.
 */
function getConfidenceLabel(confidence: number): string {
  if (confidence >= 0.8) return 'High';
  if (confidence >= 0.6) return 'Medium';
  return 'Low';
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

export function LLMResearchPanel({ target, onClose }: LLMResearchPanelProps) {
  // Store state
  const schema = useEditorStore((state) => state.schema);
  const updateEntity = useEditorStore((state) => state.updateEntity);
  const updateEntityAttribute = useEditorStore((state) => state.updateEntityAttribute);
  
  // Global loading state (Requirement: 8.5)
  const isGlobalLoading = useEditorStore((state) => state.isLoading);
  const startLoading = useEditorStore((state) => state.startLoading);
  const stopLoading = useEditorStore((state) => state.stopLoading);

  // Use TanStack Query mutation for LLM research (Requirement 7.4)
  const { 
    mutate: researchMutation, 
    isPending: isResearching,
  } = useLLMResearch();

  // Local state
  const [status, setStatus] = useState<ResearchStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [suggestions, setSuggestions] = useState<SuggestionState[]>([]);
  const [selectedFields, setSelectedFields] = useState<Set<ResearchField>>(
    () => new Set(getAvailableFields(target))
  );
  const [tokensUsed, setTokensUsed] = useState<number>(0);

  // Derived state
  const availableFields = useMemo(() => getAvailableFields(target), [target]);
  const targetName = useMemo(() => {
    if (target.type === 'entity') {
      return target.entity.caption || target.entity.name;
    }
    return `${target.entity.name}.${target.attribute.caption || target.attribute.name}`;
  }, [target]);

  const pendingSuggestions = useMemo(
    () => suggestions.filter((s) => s.status === 'pending'),
    [suggestions]
  );

  // Handlers
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

  const handleResearch = useCallback(async () => {
    if (selectedFields.size === 0) {
      setError('Please select at least one field to research');
      return;
    }

    setStatus('loading');
    setError(null);
    setSuggestions([]);
    
    // Start global loading indicator (Requirement: 8.5)
    startLoading('llm', 'Researching with LLM...');

    const entity = target.type === 'entity' ? target.entity : target.entity;
    const attribute = target.type === 'attribute' ? target.attribute : undefined;
    const context = buildOCSFContext(entity, attribute, schema);

    const request: ResearchRequest = {
      target_type: target.type,
      entity_name: entity.name,
      attribute_name: attribute?.name,
      ocsf_context: context,
      requested_fields: Array.from(selectedFields),
      is_observable: attribute?.is_observable,
    };

    researchMutation(request, {
      onSuccess: (data) => {
        setSuggestions(
          data.suggestions.map((s) => ({
            suggestion: s,
            status: 'pending' as const,
          }))
        );
        setTokensUsed(data.tokens_used);
        setStatus('success');
        stopLoading();
      },
      onError: (err) => {
        setError(getErrorMessage(err));
        setStatus('error');
        stopLoading();
      },
    });
  }, [target, selectedFields, schema, startLoading, stopLoading, researchMutation]);

  const handleAccept = useCallback(
    (suggestionId: string) => {
      const suggestionState = suggestions.find(
        (s) => s.suggestion.id === suggestionId
      );
      if (!suggestionState || suggestionState.status !== 'pending') return;

      const { suggestion } = suggestionState;
      const entity = target.type === 'entity' ? target.entity : target.entity;

      // Apply the suggestion to the model
      if (target.type === 'entity') {
        const updates: Partial<SemanticEntity> = {};
        switch (suggestion.field) {
          case 'description':
            updates.description = suggestion.value as string;
            break;
          // Note: Entity-level synonyms would need to be added to the model
        }
        if (Object.keys(updates).length > 0) {
          updateEntity(entity.name, updates);
        }
      } else {
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
          updateEntityAttribute(entity.name, target.attribute.name, updates);
        }
      }

      // Update suggestion status
      setSuggestions((prev) =>
        prev.map((s) =>
          s.suggestion.id === suggestionId ? { ...s, status: 'accepted' } : s
        )
      );
    },
    [target, suggestions, updateEntity, updateEntityAttribute]
  );

  const handleReject = useCallback((suggestionId: string) => {
    setSuggestions((prev) =>
      prev.map((s) =>
        s.suggestion.id === suggestionId ? { ...s, status: 'rejected' } : s
      )
    );
  }, []);

  const handleAcceptAll = useCallback(() => {
    pendingSuggestions.forEach((s) => handleAccept(s.suggestion.id));
  }, [pendingSuggestions, handleAccept]);

  const handleRetry = useCallback(() => {
    setStatus('idle');
    setError(null);
    setSuggestions([]);
  }, []);

  return (
    <div className="llm-research-panel">
      <div className="research-panel-header">
        <div className="research-panel-title">
          <span className="research-icon">🔬</span>
          <span>LLM Research</span>
        </div>
        {onClose && (
          <button
            className="btn-icon"
            onClick={onClose}
            aria-label="Close research panel"
          >
            ✕
          </button>
        )}
      </div>

      <div className="research-panel-body">
        {/* Target Info */}
        <div className="research-target-info">
          <span className="target-label">
            {target.type === 'entity' ? 'Entity' : 'Attribute'}:
          </span>
          <span className="target-name">{targetName}</span>
        </div>

        {/* Field Selection */}
        {status === 'idle' && (
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
        )}

        {/* Loading State */}
        {(status === 'loading' || isResearching) && (
          <div className="research-loading">
            <div className="loading-spinner" />
            <span>Researching with LLM...</span>
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

        {/* Suggestions */}
        {status === 'success' && suggestions.length > 0 && (
          <div className="research-suggestions">
            <div className="suggestions-header">
              <span className="suggestions-count">
                {suggestions.length} suggestion{suggestions.length !== 1 ? 's' : ''}
              </span>
              {tokensUsed > 0 && (
                <span className="tokens-used">{tokensUsed} tokens used</span>
              )}
            </div>

            <div className="suggestions-list">
              {suggestions.map(({ suggestion, status: suggestionStatus }) => (
                <div
                  key={suggestion.id}
                  className={`suggestion-item ${suggestionStatus}`}
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
                      {getConfidenceLabel(suggestion.confidence)} confidence
                    </span>
                  </div>

                  <div className="suggestion-value">
                    {Array.isArray(suggestion.value) ? (
                      <ul className="suggestion-list-value">
                        {suggestion.value.map((v, i) => (
                          <li key={i}>{v}</li>
                        ))}
                      </ul>
                    ) : (
                      <p>{suggestion.value}</p>
                    )}
                  </div>

                  {suggestionStatus === 'pending' && (
                    <div className="suggestion-actions">
                      <button
                        className="btn primary small"
                        onClick={() => handleAccept(suggestion.id)}
                      >
                        Accept
                      </button>
                      <button
                        className="btn small"
                        onClick={() => handleReject(suggestion.id)}
                      >
                        Reject
                      </button>
                    </div>
                  )}

                  {suggestionStatus === 'accepted' && (
                    <div className="suggestion-status accepted">
                      ✓ Applied
                    </div>
                  )}

                  {suggestionStatus === 'rejected' && (
                    <div className="suggestion-status rejected">
                      ✗ Rejected
                    </div>
                  )}
                </div>
              ))}
            </div>

            {pendingSuggestions.length > 1 && (
              <div className="suggestions-bulk-actions">
                <button className="btn primary" onClick={handleAcceptAll}>
                  Accept All ({pendingSuggestions.length})
                </button>
              </div>
            )}
          </div>
        )}

        {/* No Suggestions */}
        {status === 'success' && suggestions.length === 0 && (
          <div className="research-empty">
            <div className="empty-icon">🤷</div>
            <p>No suggestions generated. Try selecting different fields.</p>
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
            onClick={handleResearch}
            disabled={selectedFields.size === 0 || isGlobalLoading || isResearching}
          >
            🔬 Research
          </button>
        </div>
      )}
    </div>
  );
}

export default LLMResearchPanel;
