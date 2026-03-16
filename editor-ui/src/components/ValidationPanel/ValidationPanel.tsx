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
import type { ValidationError, ValidationWarning } from '../../types';
import './ValidationPanel.css';

// Debounce delay for real-time validation (500ms per requirements)
const VALIDATION_DEBOUNCE_MS = 500;

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
  
  const validationErrors = useEditorStore((state) => state.validationErrors);
  const validationWarnings = useEditorStore((state) => state.validationWarnings);
  const model = useEditorStore((state) => state.model);
  const setValidationResults = useEditorStore((state) => state.setValidationResults);
  const selectEntity = useEditorStore((state) => state.selectEntity);
  const selectMetric = useEditorStore((state) => state.selectMetric);
  
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
  const isValid = !hasErrors && !hasWarnings;
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
          <span className={`summary-icon ${isValid ? 'success' : hasErrors ? 'error' : 'warning'}`}>
            {isValid ? '✓' : hasErrors ? '✕' : '⚠'}
          </span>
          <span className={`summary-count ${hasErrors ? 'error' : ''}`}>
            {combinedErrors.length}
          </span>
          <span className="summary-label">Errors</span>
        </div>
        <div className="summary-item">
          <span className={`summary-icon ${hasWarnings ? 'warning' : 'success'}`}>
            {hasWarnings ? '⚠' : '✓'}
          </span>
          <span className={`summary-count ${hasWarnings ? 'warning' : ''}`}>
            {combinedWarnings.length}
          </span>
          <span className="summary-label">Warnings</span>
        </div>
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
        ) : isValid ? (
          <div className="validation-empty">
            <span className="empty-icon">✓</span>
            <h3>Model is Valid</h3>
            <p>
              No validation errors or warnings found. Your {validationMode === 'combined' ? 'semantic and index models are' : validationMode === 'semantic' ? 'semantic model is' : 'index model is'} ready for export.
            </p>
          </div>
        ) : (
          <>
            {/* Errors Section */}
            {hasErrors && (
              <ValidationSection
                title="Errors"
                type="error"
                items={combinedErrors}
                expanded={errorsExpanded}
                onToggle={() => setErrorsExpanded(!errorsExpanded)}
                onItemClick={handleItemClick}
              />
            )}

            {/* Warnings Section */}
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
