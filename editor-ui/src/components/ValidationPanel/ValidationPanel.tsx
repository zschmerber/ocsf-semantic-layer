/**
 * ValidationPanel component for displaying validation errors and warnings.
 * 
 * Features:
 * - Display validation errors and warnings in collapsible sections
 * - Clickable error links for navigation to entities/metrics
 * - Real-time validation on model changes (debounced)
 * - Manual validation trigger
 * - Global loading state integration (Requirement: 8.5)
 * - Uses TanStack Query for API calls (Requirement: 7.2)
 * - Index model validation (Requirements: 17.1-17.7)
 * 
 * Requirements: 5.5, 5.6, 7.2, 8.5, 17.1-17.7
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { useEditorStore } from '../../store';
import { useValidation, getErrorMessage, useTables, useSourceLineage } from '../../api';
import { validateIndexModel, type IndexValidationResult } from '../../utils/indexValidation';
import { useGuideStore } from '../../store/guideStore';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import type { ValidationError, ValidationWarning } from '../../types';
import type { ParsedField } from '../../types/referenceEvent';
import './ValidationPanel.css';

// Debounce delay for real-time validation (500ms per requirements)
const VALIDATION_DEBOUNCE_MS = 500;

// ============================================
// Sample-Event Validation Types & Helpers
// ============================================

interface SampleEventValidationItem {
  severity: 'warning' | 'error';
  message: string;
  path: string;
}

interface DryRunResult {
  metricName: string;
  aggregation: string;
  field: string;
  simulatedValue: string;
  notes: string;
}

/**
 * Cross-reference entity attributes and metric fields against parsed event fields.
 * Requirements: 11.1, 11.2, 11.3, 11.5
 */
function computeSampleEventValidation(
  entities: { name: string; attributes: { name: string; ocsf_mapping: { field?: string } }[] }[],
  metrics: { name: string; measure: { field?: string } }[],
  parsedFields: ParsedField[]
): SampleEventValidationItem[] {
  const items: SampleEventValidationItem[] = [];
  const fieldPaths = new Set(parsedFields.map((f) => f.path));
  const fieldTypeMap = new Map(parsedFields.map((f) => [f.path, f.type]));

  // Check entity attributes
  for (const entity of entities) {
    for (const attr of entity.attributes) {
      const mappedField = attr.ocsf_mapping?.field;
      if (mappedField && !fieldPaths.has(mappedField)) {
        items.push({
          severity: 'warning',
          message: `Attribute "${attr.name}" references field "${mappedField}" which is not present in the sample event`,
          path: `entities.${entity.name}.${attr.name}`,
        });
      }
    }
  }

  // Check metric measure fields
  for (const metric of metrics) {
    const measureField = metric.measure?.field;
    if (!measureField) continue;

    if (!fieldPaths.has(measureField)) {
      items.push({
        severity: 'warning',
        message: `Metric "${metric.name}" measures field "${measureField}" which is not present in the sample event`,
        path: `metrics.${metric.name}`,
      });
    } else {
      const fieldType = fieldTypeMap.get(measureField);
      if (fieldType && fieldType !== 'integer' && fieldType !== 'float') {
        items.push({
          severity: 'error',
          message: `Metric "${metric.name}" measures field "${measureField}" which is type "${fieldType}" (expected numeric)`,
          path: `metrics.${metric.name}`,
        });
      }
    }
  }

  return items;
}

/**
 * Simulate each metric against the single reference event.
 * Requirements: 11.6, 11.7, 11.8
 */
function computeDryRunSimulation(
  metrics: { name: string; aggregation: string; measure: { field?: string } }[],
  parsedFields: ParsedField[]
): DryRunResult[] {
  const fieldMap = new Map(parsedFields.map((f) => [f.path, f]));

  return metrics.map((metric) => {
    const measureField = metric.measure?.field ?? '';
    const field = fieldMap.get(measureField);
    const aggregation = metric.aggregation;

    // count always returns 1 for a single event
    if (aggregation === 'count') {
      return {
        metricName: metric.name,
        aggregation,
        field: measureField || '(event)',
        simulatedValue: '1',
        notes: 'Count of 1 sample event',
      };
    }

    // count_distinct always returns 1 for a single event
    if (aggregation === 'count_distinct') {
      if (!field) {
        return {
          metricName: metric.name,
          aggregation,
          field: measureField,
          simulatedValue: 'N/A',
          notes: 'Field not present in sample event',
        };
      }
      return {
        metricName: metric.name,
        aggregation,
        field: measureField,
        simulatedValue: '1',
        notes: 'Single event — 1 distinct value',
      };
    }

    // sum, avg, min, max — need a numeric field
    if (!field) {
      return {
        metricName: metric.name,
        aggregation,
        field: measureField,
        simulatedValue: 'N/A',
        notes: 'Field not present in sample event',
      };
    }

    if (field.type !== 'integer' && field.type !== 'float') {
      return {
        metricName: metric.name,
        aggregation,
        field: measureField,
        simulatedValue: 'N/A',
        notes: `Non-numeric field (type: ${field.type})`,
      };
    }

    const numericValue = Number(field.rawValue);
    if (isNaN(numericValue)) {
      return {
        metricName: metric.name,
        aggregation,
        field: measureField,
        simulatedValue: 'N/A',
        notes: 'Value is not a valid number',
      };
    }

    return {
      metricName: metric.name,
      aggregation,
      field: measureField,
      simulatedValue: String(numericValue),
      notes: `${aggregation} of single value`,
    };
  });
}

/**
 * Validation mode for the panel.
 */
type ValidationMode = 'semantic' | 'index' | 'combined';

interface ValidationPanelProps {
  onNavigateToEntity?: (entityName: string) => void;
  onNavigateToMetric?: (metricName: string) => void;
  /** Enable index model validation (Requirements 17.1-17.7) */
  enableIndexValidation?: boolean;
}

/**
 * Parse a validation path to extract the target type and name.
 * Paths follow format: "entities[0].attributes[1].field" or "metrics[0].dimensions"
 */
function parseValidationPath(path: string): { type: 'entity' | 'metric' | 'unknown'; name?: string; detail?: string } {
  const entityMatch = path.match(/^entities\[(\d+)\](?:\.(.+))?$/);
  if (entityMatch) {
    return { type: 'entity', detail: entityMatch[2] };
  }
  
  const metricMatch = path.match(/^metrics\[(\d+)\](?:\.(.+))?$/);
  if (metricMatch) {
    return { type: 'metric', detail: metricMatch[2] };
  }
  
  return { type: 'unknown' };
}

export function ValidationPanel({ onNavigateToEntity, onNavigateToMetric, enableIndexValidation = false }: ValidationPanelProps) {
  const [errorsExpanded, setErrorsExpanded] = useState(true);
  const [warningsExpanded, setWarningsExpanded] = useState(true);
  const [autoValidate, setAutoValidate] = useState(true);
  const [lastValidated, setLastValidated] = useState<Date | null>(null);
  const [validationMode, setValidationMode] = useState<ValidationMode>(
    enableIndexValidation ? 'combined' : 'semantic'
  );
  
  // Index validation state (Requirements 17.1-17.7)
  const [indexValidationResult, setIndexValidationResult] = useState<IndexValidationResult | null>(null);
  
  // Sample-event validation state (Requirements 11.1-11.8)
  const [sampleEventExpanded, setSampleEventExpanded] = useState(true);
  const [dryRunExpanded, setDryRunExpanded] = useState(true);
  
  const validationErrors = useEditorStore((state) => state.validationErrors);
  const validationWarnings = useEditorStore((state) => state.validationWarnings);
  const model = useEditorStore((state) => state.model);
  const setValidationResults = useEditorStore((state) => state.setValidationResults);
  const selectEntity = useEditorStore((state) => state.selectEntity);
  const selectMetric = useEditorStore((state) => state.selectMetric);
  
  // Reference event store (Requirements 11.1-11.8, 13.2)
  const rawJson = useReferenceEventStore((state) => state.rawJson);
  const parsedFields = useReferenceEventStore((state) => state.parsedFields);
  const hasReferenceEvent = rawJson !== null;
  
  // Global loading state (Requirement: 8.5)
  const isGlobalLoading = useEditorStore((state) => state.isLoading);
  const startLoading = useEditorStore((state) => state.startLoading);
  const stopLoading = useEditorStore((state) => state.stopLoading);
  
  // Use TanStack Query mutation for validation (Requirement 7.2)
  const { 
    mutate: validateMutation, 
    isPending: isValidating,
  } = useValidation();
  
  // Fetch index data for validation (Requirements 17.1-17.7)
  const { data: tables = [], isLoading: isLoadingTables } = useTables();
  const { data: sourceLineage = [], isLoading: isLoadingLineage } = useSourceLineage();
  
  // Track model changes for debounced validation
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const previousModelRef = useRef<string>('');
  
  /**
   * Perform index model validation.
   * Requirements 17.1-17.7: Validate index model structure
   */
  const performIndexValidation = useCallback(() => {
    const result = validateIndexModel(tables, sourceLineage);
    setIndexValidationResult(result);
    return result;
  }, [tables, sourceLineage]);
  
  /**
   * Combined validation errors and warnings based on mode.
   */
  const combinedErrors = useMemo((): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (validationMode === 'semantic' || validationMode === 'combined') {
      errors.push(...validationErrors);
    }
    
    if ((validationMode === 'index' || validationMode === 'combined') && indexValidationResult) {
      errors.push(...indexValidationResult.errors);
    }
    
    return errors;
  }, [validationMode, validationErrors, indexValidationResult]);
  
  const combinedWarnings = useMemo((): ValidationWarning[] => {
    const warnings: ValidationWarning[] = [];
    
    if (validationMode === 'semantic' || validationMode === 'combined') {
      warnings.push(...validationWarnings);
    }
    
    if ((validationMode === 'index' || validationMode === 'combined') && indexValidationResult) {
      warnings.push(...indexValidationResult.warnings);
    }
    
    return warnings;
  }, [validationMode, validationWarnings, indexValidationResult]);

  /**
   * Sample-event validation items — computed when reference event is loaded.
   * Requirements: 11.1, 11.2, 11.3, 11.5
   */
  const sampleEventItems = useMemo((): SampleEventValidationItem[] => {
    if (!hasReferenceEvent || parsedFields.length === 0) return [];
    return computeSampleEventValidation(model.entities, model.metrics, parsedFields);
  }, [hasReferenceEvent, parsedFields, model.entities, model.metrics]);

  /**
   * Dry-run simulation results — computed when reference event is loaded.
   * Requirements: 11.6, 11.7, 11.8
   */
  const dryRunResults = useMemo((): DryRunResult[] => {
    if (!hasReferenceEvent || parsedFields.length === 0 || model.metrics.length === 0) return [];
    return computeDryRunSimulation(model.metrics, parsedFields);
  }, [hasReferenceEvent, parsedFields, model.metrics]);

  const sampleEventErrors = sampleEventItems.filter((i) => i.severity === 'error');
  const sampleEventWarnings = sampleEventItems.filter((i) => i.severity === 'warning');
  const dryRunIssues = dryRunResults.filter((r) => r.simulatedValue === 'N/A');

  /**
   * Perform validation against the backend API.
   * Uses TanStack Query mutation with global loading state (Requirement: 7.2, 8.5)
   * Also performs index validation when enabled (Requirements 17.1-17.7)
   */
  const performValidation = useCallback(async () => {
    startLoading('validation', 'Validating model...');
    
    // Perform index validation if enabled (Requirements 17.1-17.7)
    if (enableIndexValidation && (validationMode === 'index' || validationMode === 'combined')) {
      performIndexValidation();
    }
    
    // Perform semantic validation if needed
    if (validationMode === 'semantic' || validationMode === 'combined') {
      validateMutation(model, {
        onSuccess: (result) => {
          setValidationResults(result.errors || [], result.warnings || []);
          setLastValidated(new Date());
          useGuideStore.getState().markValidationRun();
          stopLoading();
        },
        onError: (error) => {
          console.error('Validation error:', getErrorMessage(error));
          stopLoading();
        },
      });
    } else {
      // Index-only validation
      setLastValidated(new Date());
      useGuideStore.getState().markValidationRun();
      stopLoading();
    }
  }, [model, validateMutation, setValidationResults, startLoading, stopLoading, enableIndexValidation, validationMode, performIndexValidation]);

  /**
   * Debounced validation triggered by model changes.
   * Requirement 5.1: Validate within 500ms of field modification
   */
  useEffect(() => {
    if (!autoValidate) return;
    
    const modelJson = JSON.stringify(model);
    
    // Skip if model hasn't changed
    if (modelJson === previousModelRef.current) return;
    previousModelRef.current = modelJson;
    
    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }
    
    // Set new debounced validation
    debounceTimerRef.current = setTimeout(() => {
      performValidation();
    }, VALIDATION_DEBOUNCE_MS);
    
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [model, autoValidate, performValidation]);

  /**
   * Handle clicking on a validation item to navigate to the source.
   * Requirement 5.6: Clickable error links for navigation
   */
  const handleItemClick = useCallback((path: string) => {
    const parsed = parseValidationPath(path);
    
    // Extract index from path to get the actual entity/metric name
    const indexMatch = path.match(/\[(\d+)\]/);
    if (!indexMatch) return;
    
    const index = parseInt(indexMatch[1], 10);
    
    if (parsed.type === 'entity' && model.entities[index]) {
      const entityName = model.entities[index].name;
      selectEntity(entityName);
      onNavigateToEntity?.(entityName);
    } else if (parsed.type === 'metric' && model.metrics[index]) {
      const metricName = model.metrics[index].name;
      selectMetric(metricName);
      onNavigateToMetric?.(metricName);
    }
    // Index validation paths (tables, source_lineage) don't have navigation yet
  }, [model, selectEntity, selectMetric, onNavigateToEntity, onNavigateToMetric]);

  // Re-run index validation when tables or lineage data changes
  useEffect(() => {
    if (enableIndexValidation && (validationMode === 'index' || validationMode === 'combined')) {
      performIndexValidation();
    }
  }, [tables, sourceLineage, enableIndexValidation, validationMode, performIndexValidation]);

  const hasErrors = combinedErrors.length > 0;
  const hasWarnings = combinedWarnings.length > 0;
  const hasSampleEventItems = sampleEventItems.length > 0;
  const hasDryRunResults = dryRunResults.length > 0;
  const isValid = !hasErrors && !hasWarnings && !hasSampleEventItems;
  const isLoadingIndex = isLoadingTables || isLoadingLineage;

  return (
    <div className="validation-panel">
      <div className="validation-panel-header">
        <h2>Validation</h2>
        <div className="validation-header-actions">
          {enableIndexValidation && (
            <select
              className="validation-mode-select"
              value={validationMode}
              onChange={(e) => setValidationMode(e.target.value as ValidationMode)}
            >
              <option value="combined">Combined</option>
              <option value="semantic">Semantic Only</option>
              <option value="index">Index Only</option>
            </select>
          )}
          <label className="auto-validate-toggle">
            <input
              type="checkbox"
              checked={autoValidate}
              onChange={(e) => setAutoValidate(e.target.checked)}
            />
            Auto-validate
          </label>
          <button
            className="btn primary validate-btn"
            onClick={performValidation}
            disabled={isValidating || isGlobalLoading || isLoadingIndex}
          >
            {isValidating || isLoadingIndex ? (
              <>
                <span className="spinner" />
                Validating...
              </>
            ) : (
              'Validate Now'
            )}
          </button>
        </div>
      </div>

      {/* Validation Summary */}
      <div className="validation-summary">
        <div className="summary-item">
          <span className={`summary-icon ${isValid ? 'success' : hasErrors || sampleEventErrors.length > 0 ? 'error' : 'warning'}`}>
            {isValid ? '✓' : hasErrors || sampleEventErrors.length > 0 ? '✕' : '⚠'}
          </span>
          <span className={`summary-count ${hasErrors || sampleEventErrors.length > 0 ? 'error' : ''}`}>
            {combinedErrors.length + sampleEventErrors.length}
          </span>
          <span className="summary-label">Errors</span>
        </div>
        <div className="summary-item">
          <span className={`summary-icon ${hasWarnings || sampleEventWarnings.length > 0 ? 'warning' : 'success'}`}>
            {hasWarnings || sampleEventWarnings.length > 0 ? '⚠' : '✓'}
          </span>
          <span className={`summary-count ${hasWarnings || sampleEventWarnings.length > 0 ? 'warning' : ''}`}>
            {combinedWarnings.length + sampleEventWarnings.length}
          </span>
          <span className="summary-label">Warnings</span>
        </div>
        {hasReferenceEvent && dryRunIssues.length > 0 && (
          <div className="summary-item">
            <span className="summary-icon warning">⚠</span>
            <span className="summary-count warning">{dryRunIssues.length}</span>
            <span className="summary-label">Dry-run issues</span>
          </div>
        )}
        {lastValidated && (
          <div className="summary-item">
            <span className="last-validated">
              Last validated: {lastValidated.toLocaleTimeString()}
            </span>
          </div>
        )}
      </div>

      {/* Validation Content */}
      <div className="validation-content">
        {(isValidating || isLoadingIndex) && !hasErrors && !hasWarnings ? (
          <div className="validation-loading">
            <span className="spinner" />
            <p>Validating model...</p>
          </div>
        ) : isValid && !hasDryRunResults ? (
          <div className="validation-empty">
            <span className="empty-icon">✓</span>
            <h3>Model is Valid</h3>
            <p>
              No validation errors or warnings found. Your {validationMode === 'combined' ? 'semantic and index models are' : validationMode === 'semantic' ? 'semantic model is' : 'index model is'} ready for export.
            </p>
          </div>
        ) : (
          <>
            {/* Schema-Based Errors Section */}
            {hasErrors && (
              <>
                <div className="validation-section-label">Schema-based validation</div>
                <ValidationSection
                  title="Errors"
                  type="error"
                  items={combinedErrors}
                  expanded={errorsExpanded}
                  onToggle={() => setErrorsExpanded(!errorsExpanded)}
                  onItemClick={handleItemClick}
                />
              </>
            )}

            {/* Schema-Based Warnings Section */}
            {hasWarnings && (
              <ValidationSection
                title="Warnings"
                type="warning"
                items={combinedWarnings}
                expanded={warningsExpanded}
                onToggle={() => setWarningsExpanded(!warningsExpanded)}
                onItemClick={handleItemClick}
              />
            )}

            {/* Sample-Event Validation Section (Requirements 11.1-11.5) */}
            {hasReferenceEvent && hasSampleEventItems && (
              <>
                <div className="validation-section-label sample-event-label">
                  Sample-event validation
                  <span className="sample-event-badge" title="Multi-event support is planned for a future version">Based on 1 sample event</span>
                </div>
                <div className="validation-section">
                  <div
                    className="validation-section-header"
                    onClick={() => setSampleEventExpanded(!sampleEventExpanded)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => e.key === 'Enter' && setSampleEventExpanded(!sampleEventExpanded)}
                  >
                    <span className={`section-icon ${sampleEventExpanded ? 'expanded' : ''}`}>▶</span>
                    <span className={`section-title ${sampleEventErrors.length > 0 ? 'error' : 'warning'}`}>
                      Sample-Event Issues
                    </span>
                    <span className="section-count">{sampleEventItems.length}</span>
                  </div>
                  {sampleEventExpanded && (
                    <div className="validation-items">
                      {sampleEventItems.map((item, index) => (
                        <div
                          key={`sample-${index}`}
                          className={`validation-item ${item.severity}`}
                          role="listitem"
                        >
                          <span className={`validation-item-icon ${item.severity}`}>
                            {item.severity === 'error' ? '✕' : '⚠'}
                          </span>
                          <div className="validation-item-content">
                            <div className="validation-item-message">{item.message}</div>
                            <div>
                              <span className="validation-item-path">{item.path}</span>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </>
            )}

            {/* Dry-Run Simulation Section (Requirements 11.6-11.8) */}
            {hasReferenceEvent && hasDryRunResults && (
              <>
                <div className="validation-section-label dry-run-label">
                  Dry-run simulation
                  <span className="sample-event-badge" title="Multi-event support is planned for a future version">Simulated against 1 sample event</span>
                </div>
                <div className="validation-section">
                  <div
                    className="validation-section-header"
                    onClick={() => setDryRunExpanded(!dryRunExpanded)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => e.key === 'Enter' && setDryRunExpanded(!dryRunExpanded)}
                  >
                    <span className={`section-icon ${dryRunExpanded ? 'expanded' : ''}`}>▶</span>
                    <span className="section-title">
                      Metric Simulation
                    </span>
                    <span className="section-count">{dryRunResults.length}</span>
                  </div>
                  {dryRunExpanded && (
                    <div className="dry-run-table-wrapper">
                      <table className="dry-run-table">
                        <thead>
                          <tr>
                            <th>Metric Name</th>
                            <th>Aggregation</th>
                            <th>Field</th>
                            <th>Simulated Value</th>
                            <th>Notes</th>
                          </tr>
                        </thead>
                        <tbody>
                          {dryRunResults.map((result, index) => (
                            <tr key={`dryrun-${index}`} className={result.simulatedValue === 'N/A' ? 'dry-run-issue' : ''}>
                              <td>{result.metricName}</td>
                              <td><code>{result.aggregation}</code></td>
                              <td><code>{result.field}</code></td>
                              <td className={result.simulatedValue === 'N/A' ? 'dry-run-na' : 'dry-run-value'}>
                                {result.simulatedValue}
                              </td>
                              <td className="dry-run-notes">{result.notes}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                      <div className="dry-run-footer">
                        Simulated against 1 sample event — values represent a single data point
                      </div>
                    </div>
                  )}
                </div>
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
}

interface ValidationSectionProps {
  title: string;
  type: 'error' | 'warning';
  items: (ValidationError | ValidationWarning)[];
  expanded: boolean;
  onToggle: () => void;
  onItemClick: (path: string) => void;
}

function ValidationSection({ title, type, items, expanded, onToggle, onItemClick }: ValidationSectionProps) {
  return (
    <div className="validation-section">
      <div
        className="validation-section-header"
        onClick={onToggle}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === 'Enter' && onToggle()}
      >
        <span className={`section-icon ${expanded ? 'expanded' : ''}`}>▶</span>
        <span className={`section-title ${type}`}>{title}</span>
        <span className="section-count">{items.length}</span>
      </div>
      
      {expanded && (
        <div className="validation-items">
          {items.map((item, index) => (
            <ValidationItem
              key={`${item.path}-${index}`}
              item={item}
              type={type}
              onClick={() => onItemClick(item.path)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface ValidationItemProps {
  item: ValidationError | ValidationWarning;
  type: 'error' | 'warning';
  onClick: () => void;
}

function ValidationItem({ item, type, onClick }: ValidationItemProps) {
  return (
    <div
      className={`validation-item ${type}`}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && onClick()}
    >
      <span className={`validation-item-icon ${type}`}>
        {type === 'error' ? '✕' : '⚠'}
      </span>
      <div className="validation-item-content">
        <div className="validation-item-message">{item.message}</div>
        <div>
          <span className="validation-item-path">{item.path}</span>
          {item.code && <span className="validation-item-code">[{item.code}]</span>}
        </div>
      </div>
    </div>
  );
}

export default ValidationPanel;
