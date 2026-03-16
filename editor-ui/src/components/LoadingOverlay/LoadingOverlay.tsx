/**
 * LoadingOverlay component for displaying loading indicators during API operations.
 * 
 * Shows a spinner and optional message, and can optionally disable interactions
 * while loading is in progress.
 * 
 * Requirement: 8.5 - Display loading indicators during API operations
 */

import { useEditorStore, type LoadingOperation } from '../../store';
import './LoadingOverlay.css';

// ============================================
// Helper Functions
// ============================================

/**
 * Get a user-friendly message for each loading operation type.
 */
function getDefaultMessage(operation: LoadingOperation | null): string {
  switch (operation) {
    case 'schema':
      return 'Loading OCSF schema...';
    case 'validation':
      return 'Validating model...';
    case 'llm':
      return 'Researching with LLM...';
    case 'export':
      return 'Exporting model...';
    case 'import':
      return 'Importing model...';
    case 'generate':
      return 'Generating artifacts...';
    default:
      return 'Loading...';
  }
}

/**
 * Get an icon for each loading operation type.
 */
function getOperationIcon(operation: LoadingOperation | null): string {
  switch (operation) {
    case 'schema':
      return '📋';
    case 'validation':
      return '✓';
    case 'llm':
      return '🔬';
    case 'export':
      return '📤';
    case 'import':
      return '📥';
    case 'generate':
      return '⚙️';
    default:
      return '⏳';
  }
}

// ============================================
// Component Props
// ============================================

export interface LoadingOverlayProps {
  /**
   * Whether to show the overlay as a full-screen blocking overlay.
   * When true, the overlay covers the entire screen and blocks interactions.
   * When false, it shows as a smaller inline indicator.
   */
  fullScreen?: boolean;
  
  /**
   * Custom message to display. If not provided, uses the default message
   * based on the current loading operation.
   */
  message?: string;
  
  /**
   * Whether to show the overlay. If not provided, uses the store's isLoading state.
   */
  show?: boolean;
  
  /**
   * The loading operation type. If not provided, uses the store's loadingOperation.
   */
  operation?: LoadingOperation | null;
}

// ============================================
// Component
// ============================================

export function LoadingOverlay({
  fullScreen = false,
  message,
  show,
  operation,
}: LoadingOverlayProps) {
  const storeIsLoading = useEditorStore((state) => state.isLoading);
  const storeOperation = useEditorStore((state) => state.loadingOperation);
  const storeMessage = useEditorStore((state) => state.loadingMessage);
  
  // Use props if provided, otherwise fall back to store state
  const isVisible = show ?? storeIsLoading;
  const currentOperation = operation ?? storeOperation;
  const displayMessage = message ?? storeMessage ?? getDefaultMessage(currentOperation);
  
  if (!isVisible) {
    return null;
  }
  
  const icon = getOperationIcon(currentOperation);
  
  if (fullScreen) {
    return (
      <div className="loading-overlay fullscreen" role="alert" aria-busy="true">
        <div className="loading-overlay-content">
          <div className="loading-spinner-large" />
          <div className="loading-info">
            <span className="loading-icon">{icon}</span>
            <span className="loading-message">{displayMessage}</span>
          </div>
        </div>
      </div>
    );
  }
  
  return (
    <div className="loading-overlay inline" role="alert" aria-busy="true">
      <div className="loading-spinner" />
      <span className="loading-message">{displayMessage}</span>
    </div>
  );
}

// ============================================
// Header Loading Indicator
// ============================================

/**
 * A compact loading indicator for the header bar.
 * Shows only when there's an active loading operation.
 */
export function HeaderLoadingIndicator() {
  const isLoading = useEditorStore((state) => state.isLoading);
  const operation = useEditorStore((state) => state.loadingOperation);
  const message = useEditorStore((state) => state.loadingMessage);
  
  if (!isLoading) {
    return null;
  }
  
  const displayMessage = message ?? getDefaultMessage(operation);
  const icon = getOperationIcon(operation);
  
  return (
    <div className="header-loading-indicator" role="status" aria-live="polite">
      <div className="spinner" />
      <span className="loading-icon">{icon}</span>
      <span className="loading-text">{displayMessage}</span>
    </div>
  );
}

export default LoadingOverlay;
