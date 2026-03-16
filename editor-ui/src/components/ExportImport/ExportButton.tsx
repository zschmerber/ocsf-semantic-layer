/**
 * ExportButton component for exporting index models.
 * 
 * Provides export functionality with:
 * - Export index model as JSON (Requirement 18.1)
 * - Export combined semantic + index model (Requirement 18.2)
 * - Download as file (Requirement 18.4)
 * - Copy to clipboard (Requirement 18.5)
 * - Include export timestamp (Requirement 18.6)
 * - Include version information (Requirement 18.7)
 * 
 * Requirements: 18.1, 18.2, 18.4, 18.5, 18.6, 18.7
 */

import { useCallback, useState } from 'react';
import { exportIndexModel } from '../../api/indexApi';
import type { IndexModelExport } from '../../types';
import './ExportImport.css';

// ============================================
// Types
// ============================================

export type ExportFormat = 'index' | 'combined';

export interface ExportButtonProps {
  /** Export format: 'index' for index model only, 'combined' for semantic + index */
  format?: ExportFormat;
  /** Custom button label */
  label?: string;
  /** Additional CSS class */
  className?: string;
  /** Callback when export succeeds */
  onSuccess?: (data: IndexModelExport) => void;
  /** Callback when export fails */
  onError?: (error: Error) => void;
  /** Whether to show dropdown with download/copy options */
  showDropdown?: boolean;
  /** Disabled state */
  disabled?: boolean;
}

interface ExportDropdownProps {
  isOpen: boolean;
  onDownload: () => void;
  onCopyToClipboard: () => void;
  onClose: () => void;
  isPending: boolean;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Generate a filename for the export based on format and timestamp.
 */
function generateFilename(format: ExportFormat): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const prefix = format === 'combined' ? 'ocsf-combined-model' : 'ocsf-index-model';
  return `${prefix}-${timestamp}.json`;
}

/**
 * Download data as a JSON file.
 * 
 * Requirement 18.4: Download as file
 */
function downloadAsFile(data: IndexModelExport, filename: string): void {
  const jsonString = JSON.stringify(data, null, 2);
  const blob = new Blob([jsonString], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Copy data to clipboard.
 * 
 * Requirement 18.5: Copy to clipboard
 */
async function copyToClipboard(data: IndexModelExport): Promise<void> {
  const jsonString = JSON.stringify(data, null, 2);
  await navigator.clipboard.writeText(jsonString);
}

// ============================================
// ExportDropdown Sub-Component
// ============================================

function ExportDropdown({ 
  isOpen, 
  onDownload, 
  onCopyToClipboard, 
  onClose,
  isPending 
}: ExportDropdownProps) {
  if (!isOpen) return null;
  
  return (
    <>
      <div className="export-dropdown-backdrop" onClick={onClose} />
      <div className="export-dropdown">
        <button 
          className="dropdown-item"
          onClick={onDownload}
          disabled={isPending}
        >
          <span className="dropdown-icon">📥</span>
          <span className="dropdown-label">Download as JSON</span>
        </button>
        <button 
          className="dropdown-item"
          onClick={onCopyToClipboard}
          disabled={isPending}
        >
          <span className="dropdown-icon">📋</span>
          <span className="dropdown-label">Copy to Clipboard</span>
        </button>
      </div>
    </>
  );
}

// ============================================
// Main ExportButton Component
// ============================================

/**
 * ExportButton component for exporting index models.
 * 
 * Features:
 * - Export index model as JSON (Requirement 18.1)
 * - Export combined semantic + index model (Requirement 18.2)
 * - Download as file (Requirement 18.4)
 * - Copy to clipboard (Requirement 18.5)
 * - Include export timestamp (Requirement 18.6)
 * - Include version information (Requirement 18.7)
 */
export function ExportButton({
  format = 'index',
  label,
  className = '',
  onSuccess,
  onError,
  showDropdown = true,
  disabled = false,
}: ExportButtonProps) {
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isPending, setIsPending] = useState(false);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  
  // Clear feedback after a delay
  const showFeedback = useCallback((type: 'success' | 'error', message: string) => {
    setFeedback({ type, message });
    setTimeout(() => setFeedback(null), 3000);
  }, []);
  
  /**
   * Fetch export data from API.
   * 
   * Requirements 18.6, 18.7: Include timestamp and version
   */
  const fetchExportData = useCallback(async (): Promise<IndexModelExport> => {
    const combined = format === 'combined';
    const data = await exportIndexModel(combined);
    return data;
  }, [format]);
  
  /**
   * Handle download action.
   * 
   * Requirement 18.4: Download as file
   */
  const handleDownload = useCallback(async () => {
    setIsPending(true);
    setIsDropdownOpen(false);
    
    try {
      const data = await fetchExportData();
      const filename = generateFilename(format);
      downloadAsFile(data, filename);
      showFeedback('success', `Downloaded ${filename}`);
      onSuccess?.(data);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Export failed';
      showFeedback('error', errorMessage);
      onError?.(error instanceof Error ? error : new Error(errorMessage));
    } finally {
      setIsPending(false);
    }
  }, [fetchExportData, format, showFeedback, onSuccess, onError]);
  
  /**
   * Handle copy to clipboard action.
   * 
   * Requirement 18.5: Copy to clipboard
   */
  const handleCopyToClipboard = useCallback(async () => {
    setIsPending(true);
    setIsDropdownOpen(false);
    
    try {
      const data = await fetchExportData();
      await copyToClipboard(data);
      showFeedback('success', 'Copied to clipboard');
      onSuccess?.(data);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Copy failed';
      showFeedback('error', errorMessage);
      onError?.(error instanceof Error ? error : new Error(errorMessage));
    } finally {
      setIsPending(false);
    }
  }, [fetchExportData, showFeedback, onSuccess, onError]);
  
  /**
   * Handle button click - either show dropdown or download directly.
   */
  const handleClick = useCallback(() => {
    if (showDropdown) {
      setIsDropdownOpen(!isDropdownOpen);
    } else {
      handleDownload();
    }
  }, [showDropdown, isDropdownOpen, handleDownload]);
  
  const buttonLabel = label || (format === 'combined' ? 'Export Combined' : 'Export Index');
  
  return (
    <div className={`export-button-container ${className}`}>
      <button
        className={`btn export-btn ${isPending ? 'loading' : ''}`}
        onClick={handleClick}
        disabled={disabled || isPending}
        aria-expanded={isDropdownOpen}
        aria-haspopup={showDropdown ? 'menu' : undefined}
      >
        <span className="export-icon">📤</span>
        <span className="export-label">{isPending ? 'Exporting...' : buttonLabel}</span>
        {showDropdown && (
          <span className={`dropdown-arrow ${isDropdownOpen ? 'open' : ''}`}>▼</span>
        )}
      </button>
      
      {showDropdown && (
        <ExportDropdown
          isOpen={isDropdownOpen}
          onDownload={handleDownload}
          onCopyToClipboard={handleCopyToClipboard}
          onClose={() => setIsDropdownOpen(false)}
          isPending={isPending}
        />
      )}
      
      {feedback && (
        <div className={`export-feedback ${feedback.type}`}>
          <span className="feedback-icon">
            {feedback.type === 'success' ? '✓' : '⚠'}
          </span>
          <span className="feedback-text">{feedback.message}</span>
        </div>
      )}
    </div>
  );
}

export default ExportButton;
