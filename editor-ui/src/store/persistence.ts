/**
 * LocalStorage persistence for the OCSF Semantic Model Editor.
 * 
 * Implements:
 * - Auto-save model state every 30 seconds
 * - Restore state on editor load
 * - Handle storage quota errors
 * 
 * Requirements: 6.5, 6.6
 */

import type { SemanticModel } from '../types';
import { useEditorStore } from './editorStore';

// ============================================
// Constants
// ============================================

const STORAGE_KEY = 'ocsf-semantic-editor-model';
const AUTO_SAVE_INTERVAL_MS = 30_000; // 30 seconds

// ============================================
// Storage Operations
// ============================================

/**
 * Saves the model to localStorage.
 * Returns true if successful, false if storage quota exceeded.
 */
export function saveModelToStorage(model: SemanticModel): boolean {
  try {
    const serialized = JSON.stringify(model);
    localStorage.setItem(STORAGE_KEY, serialized);
    return true;
  } catch (error) {
    // Handle QuotaExceededError
    if (error instanceof DOMException && error.name === 'QuotaExceededError') {
      console.warn('LocalStorage quota exceeded. Unable to auto-save model.');
      return false;
    }
    // Re-throw unexpected errors
    console.error('Failed to save model to localStorage:', error);
    return false;
  }
}

/**
 * Loads the model from localStorage.
 * Returns the saved model or null if not found/invalid.
 */
export function loadModelFromStorage(): SemanticModel | null {
  try {
    const serialized = localStorage.getItem(STORAGE_KEY);
    if (!serialized) {
      return null;
    }

    const parsed = JSON.parse(serialized);
    
    // Basic validation - ensure it has required fields
    if (!isValidModel(parsed)) {
      console.warn('Invalid model structure in localStorage. Ignoring saved state.');
      return null;
    }

    return parsed as SemanticModel;
  } catch (error) {
    console.error('Failed to load model from localStorage:', error);
    return null;
  }
}

/**
 * Clears the saved model from localStorage.
 */
export function clearStoredModel(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.error('Failed to clear model from localStorage:', error);
  }
}

/**
 * Basic validation to check if an object looks like a SemanticModel.
 */
function isValidModel(obj: unknown): obj is SemanticModel {
  if (typeof obj !== 'object' || obj === null) {
    return false;
  }

  const model = obj as Record<string, unknown>;

  // Check required fields exist
  return (
    typeof model.version === 'string' &&
    typeof model.name === 'string' &&
    Array.isArray(model.entities) &&
    Array.isArray(model.metrics) &&
    typeof model.observable_config === 'object'
  );
}

// ============================================
// Auto-Save Manager
// ============================================

let autoSaveIntervalId: ReturnType<typeof setInterval> | null = null;
let lastSavedModel: string | null = null;

/**
 * Starts the auto-save interval.
 * Saves the model every 30 seconds if it has changed.
 */
export function startAutoSave(): void {
  if (autoSaveIntervalId !== null) {
    // Already running
    return;
  }

  autoSaveIntervalId = setInterval(() => {
    const state = useEditorStore.getState();
    const currentModelJson = JSON.stringify(state.model);

    // Only save if the model has changed since last save
    if (currentModelJson !== lastSavedModel) {
      const success = saveModelToStorage(state.model);
      if (success) {
        lastSavedModel = currentModelJson;
        console.debug('Auto-saved model to localStorage');
      }
    }
  }, AUTO_SAVE_INTERVAL_MS);

  console.debug('Auto-save started');
}

/**
 * Stops the auto-save interval.
 */
export function stopAutoSave(): void {
  if (autoSaveIntervalId !== null) {
    clearInterval(autoSaveIntervalId);
    autoSaveIntervalId = null;
    console.debug('Auto-save stopped');
  }
}

/**
 * Checks if a model has meaningful content (entities or metrics).
 * Used to decide whether to restore from localStorage or use the example model.
 */
function hasContent(model: SemanticModel): boolean {
  return model.entities.length > 0 || model.metrics.length > 0;
}

/**
 * Restores the model from localStorage if available and has content.
 * Returns true if a model was restored, false otherwise.
 * 
 * Note: Only restores if the saved model has entities or metrics.
 * This ensures new users see the example model instead of an empty one.
 */
export function restoreFromStorage(): boolean {
  const savedModel = loadModelFromStorage();
  
  if (savedModel && hasContent(savedModel)) {
    useEditorStore.getState().setModel(savedModel);
    lastSavedModel = JSON.stringify(savedModel);
    console.debug('Restored model from localStorage');
    return true;
  }

  // If saved model is empty, clear it so we don't keep checking
  if (savedModel && !hasContent(savedModel)) {
    clearStoredModel();
    console.debug('Cleared empty model from localStorage');
  }

  return false;
}

/**
 * Initializes persistence: restores saved state and starts auto-save.
 * Call this when the editor loads.
 */
export function initializePersistence(): { restored: boolean } {
  const restored = restoreFromStorage();
  startAutoSave();
  return { restored };
}

/**
 * Cleans up persistence: stops auto-save.
 * Call this when the editor unmounts.
 */
export function cleanupPersistence(): void {
  stopAutoSave();
}

/**
 * Forces an immediate save to localStorage.
 * Useful for explicit save actions.
 */
export function saveNow(): boolean {
  const state = useEditorStore.getState();
  const success = saveModelToStorage(state.model);
  if (success) {
    lastSavedModel = JSON.stringify(state.model);
  }
  return success;
}
