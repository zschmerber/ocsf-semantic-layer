/**
 * LogImport component for loading and parsing raw log samples.
 * 
 * Supports multiple log formats:
 * - JSON (objects and arrays)
 * - CSV (with header row)
 * - Syslog (BSD and ISO 8601 formats)
 * - Key-value pairs
 * 
 * Requirements: 14.1, 14.2, 14.3, 14.9
 */

import { useCallback, useState } from 'react';
import { useIndexStore } from '../../store/indexStore';
import { parseLogInput, detectLogFormat } from '../../utils/logParser';
import './LogImport.css';

export function LogImport() {
  const { 
    rawLogInput, setRawLogInput, 
    setParsedFields, setDetectedFormat 
  } = useIndexStore();
  const [parseError, setParseError] = useState<string | null>(null);
  
  const handleInputChange = useCallback((value: string) => {
    setRawLogInput(value);
    setParseError(null);
    
    if (!value.trim()) {
      setParsedFields([]);
      setDetectedFormat(null);
      return;
    }
    
    try {
      const format = detectLogFormat(value);
      setDetectedFormat(format);
      
      const fields = parseLogInput(value, format);
      setParsedFields(fields);
    } catch (err) {
      setParseError(err instanceof Error ? err.message : 'Failed to parse log');
      setParsedFields([]);
    }
  }, [setRawLogInput, setParsedFields, setDetectedFormat]);
  
  const handleFileUpload = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      handleInputChange(content);
    };
    reader.onerror = () => {
      setParseError('Failed to read file');
    };
    reader.readAsText(file);
  }, [handleInputChange]);
  
  const handleClear = useCallback(() => {
    setRawLogInput('');
    setParsedFields([]);
    setDetectedFormat(null);
    setParseError(null);
  }, [setRawLogInput, setParsedFields, setDetectedFormat]);
  
  return (
    <div className="log-import">
      <div className="log-import-header">
        <h3>Import Raw Log</h3>
        <div className="log-import-actions">
          <label className="file-upload-btn">
            <input 
              type="file" 
              accept=".json,.log,.txt,.csv"
              onChange={handleFileUpload}
            />
            Upload File
          </label>
          {rawLogInput && (
            <button 
              className="clear-btn"
              onClick={handleClear}
              type="button"
            >
              Clear
            </button>
          )}
        </div>
      </div>
      
      <textarea
        className="log-input"
        placeholder="Paste raw log data here (JSON, CSV, syslog, or key-value format)"
        value={rawLogInput}
        onChange={(e) => handleInputChange(e.target.value)}
        rows={10}
        spellCheck={false}
      />
      
      {parseError && (
        <div className="parse-error">
          <span className="error-icon">⚠</span>
          {parseError}
        </div>
      )}
      
      <ParsedFieldsPreview />
    </div>
  );
}

/**
 * Sub-component that displays the parsed fields in a table format.
 * Shows field path, type, and sample value for each extracted field.
 */
function ParsedFieldsPreview() {
  const { parsedFields, detectedFormat } = useIndexStore();
  
  if (parsedFields.length === 0) return null;
  
  return (
    <div className="parsed-fields-preview">
      <div className="preview-header">
        <div className="format-indicator">
          Detected format: <strong>{detectedFormat}</strong>
        </div>
        <div className="field-count">
          {parsedFields.length} field{parsedFields.length !== 1 ? 's' : ''} extracted
        </div>
      </div>
      <div className="fields-table-container">
        <table className="fields-table">
          <thead>
            <tr>
              <th>Field Path</th>
              <th>Type</th>
              <th>Sample Value</th>
            </tr>
          </thead>
          <tbody>
            {parsedFields.map((field, i) => (
              <tr key={`${field.path}-${i}`}>
                <td className="field-path">
                  <code>{field.path}</code>
                </td>
                <td className="field-type">
                  <span className={`type-badge type-${field.type}`}>
                    {field.type}
                  </span>
                </td>
                <td className="field-value">
                  {truncateValue(field.value)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

/**
 * Truncates a value string to a maximum length for display.
 * 
 * @param value - The value to truncate
 * @param maxLen - Maximum length before truncation (default: 50)
 * @returns Truncated string with ellipsis if needed
 */
function truncateValue(value: string, maxLen = 50): string {
  if (value.length <= maxLen) return value;
  return value.slice(0, maxLen) + '...';
}

export default LogImport;
