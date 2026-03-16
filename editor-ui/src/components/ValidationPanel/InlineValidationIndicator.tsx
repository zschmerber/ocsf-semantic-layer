/**
 * InlineValidationIndicator component for showing validation errors next to invalid fields.
 * 
 * Features:
 * - Display validation errors/warnings inline with form fields
 * - Debounced validation calls (500ms)
 * - Support for field-level validation
 * 
 * Requirements: 5.1, 5.2
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import './ValidationPanel.css';

// Debounce delay for validation (500ms per requirements)
const VALIDATION_DEBOUNCE_MS = 500;

export interface FieldValidationResult {
  valid: boolean;
  type: 'error' | 'warning';
  message: string;
}

interface InlineValidationIndicatorProps {
  /** The field path to validate (e.g., "entities[0].attributes[0].ocsf_mapping.field") */
  fieldPath: string;
  /** The current field value to validate */
  value: string;
  /** Optional custom validation function */
  validate?: (value: string) => Promise<FieldValidationResult | null>;
  /** Whether to show the indicator even when valid */
  showWhenValid?: boolean;
}

/**
 * Default validation function that calls the backend API.
 */
async function defaultValidate(fieldPath: string, value: string): Promise<FieldValidationResult | null> {
  if (!value || value.trim() === '') {
    return null; // Don't validate empty values
  }
  
  try {
    const response = await fetch('/api/validate/field', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ path: fieldPath, value }),
    });
    
    if (!response.ok) {
      // If endpoint doesn't exist or fails, return null (no validation)
      return null;
    }
    
    const result = await response.json();
    
    if (result.valid) {
      return null;
    }
    
    return {
      valid: false,
      type: result.type || 'error',
      message: result.message || 'Invalid value',
    };
  } catch {
    // On network error, don't show validation error
    return null;
  }
}

export function InlineValidationIndicator({
  fieldPath,
  value,
  validate,
  showWhenValid = false,
}: InlineValidationIndicatorProps) {
  const [validationResult, setValidationResult] = useState<FieldValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const previousValueRef = useRef<string>('');

  const performValidation = useCallback(async () => {
    setIsValidating(true);
    
    try {
      const result = validate
        ? await validate(value)
        : await defaultValidate(fieldPath, value);
      
      setValidationResult(result);
    } catch (error) {
      console.error('Inline validation error:', error);
      setValidationResult(null);
    } finally {
      setIsValidating(false);
    }
  }, [fieldPath, value, validate]);

  /**
   * Debounced validation triggered by value changes.
   * Requirement 5.1: Validate within 500ms of field modification
   */
  useEffect(() => {
    // Skip if value hasn't changed
    if (value === previousValueRef.current) return;
    previousValueRef.current = value;
    
    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }
    
    // Clear validation result while waiting
    setValidationResult(null);
    
    // Set new debounced validation
    debounceTimerRef.current = setTimeout(() => {
      performValidation();
    }, VALIDATION_DEBOUNCE_MS);
    
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [value, performValidation]);

  // Don't render anything if no validation result and not showing when valid
  if (!validationResult && !showWhenValid && !isValidating) {
    return null;
  }

  // Show loading state
  if (isValidating) {
    return (
      <div className="inline-validation-indicator" style={{ opacity: 0.5 }}>
        <span className="inline-validation-icon">⋯</span>
        <span>Validating...</span>
      </div>
    );
  }

  // Show validation result
  if (validationResult) {
    return (
      <div className={`inline-validation-indicator ${validationResult.type}`}>
        <span className="inline-validation-icon">
          {validationResult.type === 'error' ? '✕' : '⚠'}
        </span>
        <span>{validationResult.message}</span>
      </div>
    );
  }

  // Show valid state if requested
  if (showWhenValid && value) {
    return (
      <div className="inline-validation-indicator" style={{ color: 'var(--accent-green)', background: 'rgba(126, 231, 135, 0.1)' }}>
        <span className="inline-validation-icon">✓</span>
        <span>Valid</span>
      </div>
    );
  }

  return null;
}

/**
 * Hook for managing field-level validation state.
 * Provides debounced validation with 500ms delay.
 */
export function useFieldValidation(
  fieldPath: string,
  value: string,
  validate?: (value: string) => Promise<FieldValidationResult | null>
) {
  const [validationResult, setValidationResult] = useState<FieldValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const previousValueRef = useRef<string>('');

  const performValidation = useCallback(async () => {
    setIsValidating(true);
    
    try {
      const result = validate
        ? await validate(value)
        : await defaultValidate(fieldPath, value);
      
      setValidationResult(result);
    } catch (error) {
      console.error('Field validation error:', error);
      setValidationResult(null);
    } finally {
      setIsValidating(false);
    }
  }, [fieldPath, value, validate]);

  useEffect(() => {
    // Skip if value hasn't changed
    if (value === previousValueRef.current) return;
    previousValueRef.current = value;
    
    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }
    
    // Clear validation result while waiting
    setValidationResult(null);
    
    // Set new debounced validation
    debounceTimerRef.current = setTimeout(() => {
      performValidation();
    }, VALIDATION_DEBOUNCE_MS);
    
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [value, performValidation]);

  return {
    validationResult,
    isValidating,
    isValid: !validationResult,
    hasError: validationResult?.type === 'error',
    hasWarning: validationResult?.type === 'warning',
    triggerValidation: performValidation,
  };
}

export default InlineValidationIndicator;
