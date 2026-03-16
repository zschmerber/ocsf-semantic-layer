/**
 * SchemaSearch component for filtering the schema tree.
 * 
 * Provides a search input with debounced filtering and result count display.
 * 
 * Requirements: 1.5, 1.6
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import './SchemaBrowser.css';

interface SchemaSearchProps {
  value: string;
  onChange: (query: string) => void;
  resultCount?: number;
  debounceMs?: number;
}

/**
 * SchemaSearch provides a debounced search input for filtering the schema tree.
 * 
 * Features:
 * - Debounced input to avoid excessive filtering
 * - Clear button when search has content
 * - Result count display
 * - Keyboard shortcut support (Escape to clear)
 */
export function SchemaSearch({
  value,
  onChange,
  resultCount,
  debounceMs = 200,
}: SchemaSearchProps) {
  const [localValue, setLocalValue] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();
  
  // Sync local value with external value
  useEffect(() => {
    setLocalValue(value);
  }, [value]);
  
  // Debounced onChange
  const debouncedOnChange = useCallback((newValue: string) => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    
    debounceRef.current = setTimeout(() => {
      onChange(newValue);
    }, debounceMs);
  }, [onChange, debounceMs]);
  
  // Cleanup debounce on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);
  
  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setLocalValue(newValue);
    debouncedOnChange(newValue);
  }, [debouncedOnChange]);
  
  const handleClear = useCallback(() => {
    setLocalValue('');
    onChange('');
    inputRef.current?.focus();
  }, [onChange]);
  
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape' && localValue) {
      e.preventDefault();
      handleClear();
    }
  }, [localValue, handleClear]);

  return (
    <div className="schema-search">
      <div className="search-input-container">
        <span className="search-icon">🔍</span>
        <input
          ref={inputRef}
          type="text"
          className="search-input"
          placeholder="Search schema..."
          value={localValue}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          aria-label="Search schema"
        />
        {localValue && (
          <button
            className="search-clear-btn"
            onClick={handleClear}
            aria-label="Clear search"
          >
            ✕
          </button>
        )}
      </div>
      
      {resultCount !== undefined && localValue && (
        <div className="search-result-count">
          {resultCount === 0 ? 'No results' : `${resultCount} result${resultCount !== 1 ? 's' : ''}`}
        </div>
      )}
    </div>
  );
}

export default SchemaSearch;
