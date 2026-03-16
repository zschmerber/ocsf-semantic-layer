/**
 * ImportModal component for importing index models.
 * 
 * Provides import functionality with:
 * - Import index model from JSON (Requirement 19.1)
 * - Validate imported model before applying (Requirement 19.3)
 * - Show import preview (Requirement 19.4)
 * - Merge or replace options (Requirement 19.5)
 * - Show import success/error feedback (Requirement 19.6)
 * 
 * Requirements: 19.1, 19.3, 19.4, 19.5, 19.6
 */

import { useCallback, useState, useMemo } from 'react';
import { importIndexModel } from '../../api/indexApi';
import { validateIndexModelExport, type IndexValidationResult } from '../../utils/indexValidation';
import type { IndexModelExport, ImportResult } from '../../types';
import './ExportImport.css';

// ============================================
// Types
// ============================================

export type ImportMode = 'merge' | 'replace';

export interface ImportModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback to close the modal */
  onClose: () => void;
  /** Callback when import succeeds */
  onSuccess?: (result: ImportResult) => void;
  /** Callback when import fails */
  onError?: (error: Error) => void;
}

interface ImportPreviewProps {
  model: IndexModelExport;
  validation: IndexValidationResult;
}

interface ImportModeSelectProps {
  mode: ImportMode;
  onChange: (mode: ImportMode) => void;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Parse JSON string to IndexModelExport.
 * 
 * Requirement 19.3: Validate imported model format
 */
function parseImportData(jsonString: string): IndexModelExport {
  try {
    const data = JSON.parse(jsonString);
    
    // Basic structure validation
    if (typeof data !== 'object' || data === null) {
      throw new Error('Invalid JSON: expected an object');
    }
    
    // Check required fields
    if (!data.version) {
      throw new Error('Missing required field: version');
    }
    
    if (!Array.isArray(data.tables)) {
      throw new Error('Missing or invalid field: tables (expected array)');
    }
    
    if (!Array.isArray(data.source_lineage)) {
      throw new Error('Missing or invalid field: source_lineage (expected array)');
    }
    
    if (!Array.isArray(data.field_lineage)) {
      throw new Error('Missing or invalid field: field_lineage (expected array)');
    }
    
    return data as IndexModelExport;
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new Error('Invalid JSON format: ' + error.message);
    }
    throw error;
  }
}

/**
 * Read file content as text.
 */
function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => resolve(e.target?.result as string);
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsText(file);
  });
}

// ============================================
// ImportPreview Sub-Component
// ============================================

/**
 * Preview of the import data with validation results.
 * 
 * Requirement 19.4: Show import preview
 */
function ImportPreview({ model, validation }: ImportPreviewProps) {
  return (
    <div className="import-preview">
      <h4>Import Preview</h4>
      
      {/* Model Summary */}
      <div className="preview-section">
        <h5>Model Summary</h5>
        <dl className="preview-stats">
          <div className="stat-item">
            <dt>Version</dt>
            <dd>{model.version}</dd>
          </div>
          <div className="stat-item">
            <dt>Exported At</dt>
            <dd>{model.exported_at ? new Date(model.exported_at).toLocaleString() : 'N/A'}</dd>
          </div>
          <div className="stat-item">
            <dt>Tables</dt>
            <dd>{model.tables.length}</dd>
          </div>
          <div className="stat-item">
            <dt>Source Lineage Records</dt>
            <dd>{model.source_lineage.length}</dd>
          </div>
          <div className="stat-item">
            <dt>Field Lineage Records</dt>
            <dd>{model.field_lineage.length}</dd>
          </div>
        </dl>
      </div>
      
      {/* Tables Preview */}
      {model.tables.length > 0 && (
        <div className="preview-section">
          <h5>Tables ({model.tables.length})</h5>
          <div className="preview-list">
            {model.tables.slice(0, 5).map((table, i) => (
              <div key={i} className="preview-item">
                <span className="item-icon">📋</span>
                <span className="item-name">{table.table_name}</span>
                <span className="item-meta">Class: {table.class_uid}</span>
              </div>
            ))}
            {model.tables.length > 5 && (
              <div className="preview-more">
                +{model.tables.length - 5} more tables
              </div>
            )}
          </div>
        </div>
      )}
      
      {/* Validation Results */}
      <div className="preview-section validation-section">
        <h5>Validation</h5>
        <div className={`validation-status ${validation.valid ? 'valid' : 'invalid'}`}>
          <span className="status-icon">{validation.valid ? '✓' : '⚠'}</span>
          <span className="status-text">
            {validation.valid ? 'Model is valid' : 'Model has validation issues'}
          </span>
        </div>
        
        {validation.errors.length > 0 && (
          <div className="validation-errors">
            <h6>Errors ({validation.errors.length})</h6>
            <ul className="error-list">
              {validation.errors.slice(0, 5).map((error, i) => (
                <li key={i} className="error-item">
                  <span className="error-path">{error.path}</span>
                  <span className="error-message">{error.message}</span>
                </li>
              ))}
              {validation.errors.length > 5 && (
                <li className="error-more">
                  +{validation.errors.length - 5} more errors
                </li>
              )}
            </ul>
          </div>
        )}
        
        {validation.warnings.length > 0 && (
          <div className="validation-warnings">
            <h6>Warnings ({validation.warnings.length})</h6>
            <ul className="warning-list">
              {validation.warnings.slice(0, 3).map((warning, i) => (
                <li key={i} className="warning-item">
                  <span className="warning-path">{warning.path}</span>
                  <span className="warning-message">{warning.message}</span>
                </li>
              ))}
              {validation.warnings.length > 3 && (
                <li className="warning-more">
                  +{validation.warnings.length - 3} more warnings
                </li>
              )}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================
// ImportModeSelect Sub-Component
// ============================================

/**
 * Radio buttons for selecting import mode.
 * 
 * Requirement 19.5: Merge or replace options
 */
function ImportModeSelect({ mode, onChange }: ImportModeSelectProps) {
  return (
    <div className="import-mode-select">
      <h5>Import Mode</h5>
      <div className="mode-options">
        <label className={`mode-option ${mode === 'merge' ? 'selected' : ''}`}>
          <input
            type="radio"
            name="importMode"
            value="merge"
            checked={mode === 'merge'}
            onChange={() => onChange('merge')}
          />
          <div className="mode-content">
            <span className="mode-title">Merge</span>
            <span className="mode-description">
              Add new tables and lineage records. Existing records with the same ID will be updated.
            </span>
          </div>
        </label>
        
        <label className={`mode-option ${mode === 'replace' ? 'selected' : ''}`}>
          <input
            type="radio"
            name="importMode"
            value="replace"
            checked={mode === 'replace'}
            onChange={() => onChange('replace')}
          />
          <div className="mode-content">
            <span className="mode-title">Replace</span>
            <span className="mode-description">
              Clear existing data and import fresh. Warning: This will delete all current tables and lineage.
            </span>
          </div>
        </label>
      </div>
    </div>
  );
}

// ============================================
// Main ImportModal Component
// ============================================

/**
 * ImportModal component for importing index models.
 * 
 * Features:
 * - Import index model from JSON (Requirement 19.1)
 * - Validate imported model before applying (Requirement 19.3)
 * - Show import preview (Requirement 19.4)
 * - Merge or replace options (Requirement 19.5)
 * - Show import success/error feedback (Requirement 19.6)
 */
export function ImportModal({
  isOpen,
  onClose,
  onSuccess,
  onError,
}: ImportModalProps) {
  // Input state
  const [inputMethod, setInputMethod] = useState<'file' | 'paste'>('file');
  const [pasteContent, setPasteContent] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  
  // Parsed data state
  const [parsedModel, setParsedModel] = useState<IndexModelExport | null>(null);
  const [parseError, setParseError] = useState<string | null>(null);
  
  // Import options
  const [importMode, setImportMode] = useState<ImportMode>('merge');
  
  // Import state
  const [isPending, setIsPending] = useState(false);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  
  /**
   * Validate the parsed model.
   * 
   * Requirement 19.3: Validate imported model before applying
   */
  const validation = useMemo<IndexValidationResult | null>(() => {
    if (!parsedModel) return null;
    return validateIndexModelExport(parsedModel);
  }, [parsedModel]);
  
  /**
   * Handle file selection.
   */
  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    
    setSelectedFile(file);
    setParseError(null);
    setParsedModel(null);
    
    try {
      const content = await readFileAsText(file);
      const model = parseImportData(content);
      setParsedModel(model);
    } catch (error) {
      setParseError(error instanceof Error ? error.message : 'Failed to parse file');
    }
  }, []);
  
  /**
   * Handle paste content change.
   */
  const handlePasteChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const content = e.target.value;
    setPasteContent(content);
    setParseError(null);
    setParsedModel(null);
    
    if (!content.trim()) return;
    
    try {
      const model = parseImportData(content);
      setParsedModel(model);
    } catch (error) {
      setParseError(error instanceof Error ? error.message : 'Failed to parse content');
    }
  }, []);
  
  /**
   * Handle import submission.
   * 
   * Requirements 19.1, 19.6: Import and show feedback
   */
  const handleImport = useCallback(async () => {
    if (!parsedModel || !validation?.valid) return;
    
    setIsPending(true);
    setImportError(null);
    setImportResult(null);
    
    try {
      // Add import mode to the model metadata
      const modelToImport = {
        ...parsedModel,
        // The backend will handle merge vs replace based on this
        metadata: {
          import_mode: importMode,
        },
      };
      
      const result = await importIndexModel(modelToImport);
      setImportResult(result);
      
      // Requirement 19.6: Show success feedback
      if (result.success) {
        onSuccess?.(result);
      } else {
        const errorMsg = result.errors?.join(', ') || 'Import failed';
        setImportError(errorMsg);
        onError?.(new Error(errorMsg));
      }
    } catch (error) {
      // Requirement 19.6: Show error feedback
      const errorMessage = error instanceof Error ? error.message : 'Import failed';
      setImportError(errorMessage);
      onError?.(error instanceof Error ? error : new Error(errorMessage));
    } finally {
      setIsPending(false);
    }
  }, [parsedModel, validation, importMode, onSuccess, onError]);
  
  /**
   * Reset modal state.
   */
  const handleReset = useCallback(() => {
    setPasteContent('');
    setSelectedFile(null);
    setParsedModel(null);
    setParseError(null);
    setImportResult(null);
    setImportError(null);
    setImportMode('merge');
  }, []);
  
  /**
   * Handle modal close.
   */
  const handleClose = useCallback(() => {
    handleReset();
    onClose();
  }, [handleReset, onClose]);
  
  if (!isOpen) return null;
  
  // Show success state
  if (importResult?.success) {
    return (
      <div className="modal-overlay" onClick={handleClose}>
        <div className="import-modal success-state" onClick={e => e.stopPropagation()}>
          <div className="modal-header">
            <h3>Import Successful</h3>
            <button 
              className="close-btn" 
              onClick={handleClose}
              aria-label="Close modal"
            >
              ×
            </button>
          </div>
          
          <div className="modal-content">
            <div className="success-message">
              <span className="success-icon">✓</span>
              <h4>Import Complete</h4>
              <p>Your index model has been imported successfully.</p>
            </div>
            
            <div className="import-summary">
              <dl className="summary-stats">
                <div className="stat-item">
                  <dt>Tables Imported</dt>
                  <dd>{importResult.tables_imported}</dd>
                </div>
                <div className="stat-item">
                  <dt>Source Lineage Records</dt>
                  <dd>{importResult.source_lineage_imported}</dd>
                </div>
                <div className="stat-item">
                  <dt>Field Lineage Records</dt>
                  <dd>{importResult.field_lineage_imported}</dd>
                </div>
              </dl>
            </div>
          </div>
          
          <div className="modal-actions">
            <button className="btn primary" onClick={handleClose}>
              Done
            </button>
          </div>
        </div>
      </div>
    );
  }
  
  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="import-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Import Index Model</h3>
          <button 
            className="close-btn" 
            onClick={handleClose}
            aria-label="Close modal"
          >
            ×
          </button>
        </div>
        
        <div className="modal-content">
          {/* Input Method Tabs */}
          <div className="input-method-tabs">
            <button
              className={`tab ${inputMethod === 'file' ? 'active' : ''}`}
              onClick={() => setInputMethod('file')}
            >
              <span className="tab-icon">📁</span>
              Upload File
            </button>
            <button
              className={`tab ${inputMethod === 'paste' ? 'active' : ''}`}
              onClick={() => setInputMethod('paste')}
            >
              <span className="tab-icon">📋</span>
              Paste JSON
            </button>
          </div>
          
          {/* File Upload */}
          {inputMethod === 'file' && (
            <div className="input-section file-input">
              <label className="file-drop-zone">
                <input
                  type="file"
                  accept=".json"
                  onChange={handleFileSelect}
                  className="file-input-hidden"
                />
                <div className="drop-zone-content">
                  <span className="drop-icon">📥</span>
                  <span className="drop-text">
                    {selectedFile 
                      ? selectedFile.name 
                      : 'Click to select or drag and drop a JSON file'}
                  </span>
                  <span className="drop-hint">Supports .json files</span>
                </div>
              </label>
            </div>
          )}
          
          {/* Paste Input */}
          {inputMethod === 'paste' && (
            <div className="input-section paste-input">
              <textarea
                className="paste-textarea"
                placeholder="Paste your JSON export here..."
                value={pasteContent}
                onChange={handlePasteChange}
                rows={8}
              />
            </div>
          )}
          
          {/* Parse Error */}
          {parseError && (
            <div className="parse-error">
              <span className="error-icon">⚠</span>
              <span className="error-text">{parseError}</span>
            </div>
          )}
          
          {/* Import Preview (Requirement 19.4) */}
          {parsedModel && validation && (
            <>
              <ImportPreview model={parsedModel} validation={validation} />
              
              {/* Import Mode (Requirement 19.5) */}
              <ImportModeSelect mode={importMode} onChange={setImportMode} />
            </>
          )}
          
          {/* Import Error (Requirement 19.6) */}
          {importError && (
            <div className="import-error">
              <span className="error-icon">⚠</span>
              <span className="error-text">{importError}</span>
            </div>
          )}
        </div>
        
        <div className="modal-actions">
          <button 
            className="btn secondary" 
            onClick={handleClose}
            disabled={isPending}
          >
            Cancel
          </button>
          <button
            className="btn primary"
            onClick={handleImport}
            disabled={!parsedModel || !validation?.valid || isPending}
          >
            {isPending ? 'Importing...' : 'Import'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ImportModal;
