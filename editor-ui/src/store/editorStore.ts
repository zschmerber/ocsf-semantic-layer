/**
 * Zustand store for the OCSF Semantic Model Editor.
 * 
 * Manages:
 * - Model state (entities, metrics, observable config)
 * - Schema state (loaded OCSF schema tree)
 * - UI state (selections, expanded nodes, search)
 * - Validation state (errors and warnings)
 * - Undo/redo history for model changes
 * 
 * Requirements: 2.6, 2.7, 3.7, 8.4
 */

import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type {
  SemanticModel,
  SemanticEntity,
  SemanticMetric,
  SemanticAttribute,
  Dataset,
  SchemaTree,
  ValidationError,
  ValidationWarning,
} from '../types';
import { createDefaultModel, createExampleModel } from '../types';

// ============================================
// History Configuration
// ============================================

/** Maximum number of history states to keep */
const MAX_HISTORY_SIZE = 50;

/** Use example model as initial state to help users understand the UI */
const INITIAL_MODEL = createExampleModel();

// ============================================
// State Interface
// ============================================

/**
 * Types of loading operations that can be tracked.
 * Requirement: 8.5 - Display loading indicators during API operations
 */
export type LoadingOperation = 
  | 'schema'      // Schema is being loaded
  | 'validation'  // Validation is running
  | 'llm'         // LLM research is in progress
  | 'export'      // Export operation is happening
  | 'import'      // Import operation is happening
  | 'generate';   // Generation operation is happening

export interface EditorState {
  // Model state
  model: SemanticModel;
  isDirty: boolean;

  // History state (Requirement: 8.4)
  history: SemanticModel[];
  historyIndex: number;
  canUndo: boolean;
  canRedo: boolean;

  // Schema state
  schema: SchemaTree | null;
  schemaLoading: boolean;
  schemaError: string | null;

  // Loading state (Requirement: 8.5)
  isLoading: boolean;
  loadingOperation: LoadingOperation | null;
  loadingMessage: string | null;

  // UI state
  selectedEntity: string | null;
  selectedMetric: string | null;
  expandedNodes: Set<string>;
  searchQuery: string;

  // Validation state
  validationErrors: ValidationError[];
  validationWarnings: ValidationWarning[];

  // Model actions
  setModel: (model: SemanticModel) => void;
  updateModelInfo: (updates: Partial<Pick<SemanticModel, 'name' | 'description' | 'ocsf_version'>>) => void;
  resetModel: () => void;

  // History actions (Requirement: 8.4)
  undo: () => void;
  redo: () => void;

  // Entity CRUD actions (Requirements: 2.6, 2.7)
  addEntity: (entity: SemanticEntity) => void;
  updateEntity: (name: string, updates: Partial<SemanticEntity>) => void;
  removeEntity: (name: string) => void;

  // Entity attribute actions
  addEntityAttribute: (entityName: string, attribute: SemanticAttribute) => void;
  updateEntityAttribute: (entityName: string, attributeName: string, updates: Partial<SemanticAttribute>) => void;
  removeEntityAttribute: (entityName: string, attributeName: string) => void;

  // Metric CRUD actions (Requirement: 3.7)
  addMetric: (metric: SemanticMetric) => void;
  updateMetric: (name: string, updates: Partial<SemanticMetric>) => void;
  removeMetric: (name: string) => void;

  // Dataset CRUD actions (Requirement: 8.6)
  addDataset: (dataset: Dataset) => void;
  updateDataset: (name: string, dataset: Dataset) => void;
  removeDataset: (name: string) => void;

  // Entity dataset ref action (Requirement: 8.4)
  updateEntityDatasetRef: (entityName: string, datasetRef: string | undefined) => void;

  // Schema actions
  setSchema: (schema: SchemaTree) => void;
  setSchemaLoading: (loading: boolean) => void;
  setSchemaError: (error: string | null) => void;

  // UI actions
  selectEntity: (name: string | null) => void;
  selectMetric: (name: string | null) => void;
  toggleNode: (nodeId: string) => void;
  expandNode: (nodeId: string) => void;
  collapseNode: (nodeId: string) => void;
  setSearchQuery: (query: string) => void;

  // Validation actions
  setValidationResults: (errors: ValidationError[], warnings: ValidationWarning[]) => void;
  clearValidation: () => void;

  // Loading state actions (Requirement: 8.5)
  startLoading: (operation: LoadingOperation, message?: string) => void;
  stopLoading: () => void;

  // Dirty state management
  markClean: () => void;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Deep clone a model for history storage.
 */
function cloneModel(model: SemanticModel): SemanticModel {
  return JSON.parse(JSON.stringify(model));
}

/**
 * Push a new state to history, managing the history size.
 */
function pushToHistory(
  history: SemanticModel[],
  historyIndex: number,
  newModel: SemanticModel
): { history: SemanticModel[]; historyIndex: number } {
  // Remove any future states if we're not at the end
  const newHistory = history.slice(0, historyIndex + 1);
  
  // Add the new state
  newHistory.push(cloneModel(newModel));
  
  // Trim history if it exceeds max size
  if (newHistory.length > MAX_HISTORY_SIZE) {
    newHistory.shift();
    return { history: newHistory, historyIndex: newHistory.length - 1 };
  }
  
  return { history: newHistory, historyIndex: newHistory.length - 1 };
}

// ============================================
// Store Implementation
// ============================================

export const useEditorStore = create<EditorState>()(
  subscribeWithSelector((set) => ({
    // Initial state - use example model to help users understand the UI
    model: INITIAL_MODEL,
    isDirty: false,

    // History state (Requirement: 8.4)
    history: [cloneModel(INITIAL_MODEL)],
    historyIndex: 0,
    canUndo: false,
    canRedo: false,

    schema: null,
    schemaLoading: false,
    schemaError: null,

    // Loading state (Requirement: 8.5)
    isLoading: false,
    loadingOperation: null,
    loadingMessage: null,

    selectedEntity: null,
    selectedMetric: null,
    expandedNodes: new Set<string>(),
    searchQuery: '',

    validationErrors: [],
    validationWarnings: [],

    // ========================================
    // Model Actions
    // ========================================

    setModel: (model) =>
      set({
        model,
        isDirty: false,
        selectedEntity: null,
        selectedMetric: null,
        validationErrors: [],
        validationWarnings: [],
        // Reset history when loading a new model
        history: [cloneModel(model)],
        historyIndex: 0,
        canUndo: false,
        canRedo: false,
      }),

    updateModelInfo: (updates) =>
      set((state) => {
        const newModel = { ...state.model, ...updates };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    resetModel: () =>
      set({
        model: createDefaultModel(),
        isDirty: false,
        selectedEntity: null,
        selectedMetric: null,
        validationErrors: [],
        validationWarnings: [],
        // Reset history
        history: [cloneModel(createDefaultModel())],
        historyIndex: 0,
        canUndo: false,
        canRedo: false,
      }),

    // ========================================
    // History Actions (Requirement: 8.4)
    // ========================================

    undo: () =>
      set((state) => {
        if (state.historyIndex <= 0) {
          return state; // Nothing to undo
        }
        const newIndex = state.historyIndex - 1;
        const previousModel = cloneModel(state.history[newIndex]);
        return {
          model: previousModel,
          historyIndex: newIndex,
          canUndo: newIndex > 0,
          canRedo: true,
          isDirty: true,
        };
      }),

    redo: () =>
      set((state) => {
        if (state.historyIndex >= state.history.length - 1) {
          return state; // Nothing to redo
        }
        const newIndex = state.historyIndex + 1;
        const nextModel = cloneModel(state.history[newIndex]);
        return {
          model: nextModel,
          historyIndex: newIndex,
          canUndo: true,
          canRedo: newIndex < state.history.length - 1,
          isDirty: true,
        };
      }),

    // ========================================
    // Entity CRUD Actions (Requirements: 2.6, 2.7)
    // ========================================

    addEntity: (entity) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: [...state.model.entities, entity],
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    updateEntity: (name, updates) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.map((e) =>
            e.name === name ? { ...e, ...updates } : e
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    removeEntity: (name) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.filter((e) => e.name !== name),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          selectedEntity: state.selectedEntity === name ? null : state.selectedEntity,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    // ========================================
    // Entity Attribute Actions
    // ========================================

    addEntityAttribute: (entityName, attribute) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.map((e) =>
            e.name === entityName
              ? { ...e, attributes: [...e.attributes, attribute] }
              : e
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    updateEntityAttribute: (entityName, attributeName, updates) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.map((e) =>
            e.name === entityName
              ? {
                  ...e,
                  attributes: e.attributes.map((a) =>
                    a.name === attributeName ? { ...a, ...updates } : a
                  ),
                }
              : e
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    removeEntityAttribute: (entityName, attributeName) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.map((e) =>
            e.name === entityName
              ? {
                  ...e,
                  attributes: e.attributes.filter((a) => a.name !== attributeName),
                }
              : e
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    // ========================================
    // Metric CRUD Actions (Requirement: 3.7)
    // ========================================

    addMetric: (metric) =>
      set((state) => {
        const newModel = {
          ...state.model,
          metrics: [...state.model.metrics, metric],
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    updateMetric: (name, updates) =>
      set((state) => {
        const newModel = {
          ...state.model,
          metrics: state.model.metrics.map((m) =>
            m.name === name ? { ...m, ...updates } : m
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    removeMetric: (name) =>
      set((state) => {
        const newModel = {
          ...state.model,
          metrics: state.model.metrics.filter((m) => m.name !== name),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          selectedMetric: state.selectedMetric === name ? null : state.selectedMetric,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    // ========================================
    // Dataset CRUD Actions (Requirement: 8.6)
    // ========================================

    addDataset: (dataset) =>
      set((state) => {
        const datasets = state.model.datasets ?? [];
        const newModel = {
          ...state.model,
          datasets: [...datasets, dataset],
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    updateDataset: (name, dataset) =>
      set((state) => {
        const datasets = state.model.datasets ?? [];
        const newModel = {
          ...state.model,
          datasets: datasets.map((d) => (d.name === name ? dataset : d)),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    removeDataset: (name) =>
      set((state) => {
        const datasets = state.model.datasets ?? [];
        const newModel = {
          ...state.model,
          datasets: datasets.filter((d) => d.name !== name),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    updateEntityDatasetRef: (entityName, datasetRef) =>
      set((state) => {
        const newModel = {
          ...state.model,
          entities: state.model.entities.map((e) =>
            e.name === entityName ? { ...e, dataset_ref: datasetRef } : e
          ),
        };
        const { history, historyIndex } = pushToHistory(state.history, state.historyIndex, newModel);
        return {
          model: newModel,
          isDirty: true,
          history,
          historyIndex,
          canUndo: historyIndex > 0,
          canRedo: false,
        };
      }),

    // ========================================
    // Schema Actions
    // ========================================

    setSchema: (schema) =>
      set({
        schema,
        schemaLoading: false,
        schemaError: null,
      }),

    setSchemaLoading: (loading) =>
      set({ schemaLoading: loading }),

    setSchemaError: (error) =>
      set({
        schemaError: error,
        schemaLoading: false,
      }),

    // ========================================
    // UI Actions
    // ========================================

    selectEntity: (name) =>
      set({ selectedEntity: name }),

    selectMetric: (name) =>
      set({ selectedMetric: name }),

    toggleNode: (nodeId) =>
      set((state) => {
        const newExpanded = new Set(state.expandedNodes);
        if (newExpanded.has(nodeId)) {
          newExpanded.delete(nodeId);
        } else {
          newExpanded.add(nodeId);
        }
        return { expandedNodes: newExpanded };
      }),

    expandNode: (nodeId) =>
      set((state) => {
        const newExpanded = new Set(state.expandedNodes);
        newExpanded.add(nodeId);
        return { expandedNodes: newExpanded };
      }),

    collapseNode: (nodeId) =>
      set((state) => {
        const newExpanded = new Set(state.expandedNodes);
        newExpanded.delete(nodeId);
        return { expandedNodes: newExpanded };
      }),

    setSearchQuery: (query) =>
      set({ searchQuery: query }),

    // ========================================
    // Validation Actions
    // ========================================

    setValidationResults: (errors, warnings) =>
      set({
        validationErrors: errors,
        validationWarnings: warnings,
      }),

    clearValidation: () =>
      set({
        validationErrors: [],
        validationWarnings: [],
      }),

    // ========================================
    // Loading State Actions (Requirement: 8.5)
    // ========================================

    startLoading: (operation, message) =>
      set({
        isLoading: true,
        loadingOperation: operation,
        loadingMessage: message || null,
      }),

    stopLoading: () =>
      set({
        isLoading: false,
        loadingOperation: null,
        loadingMessage: null,
      }),

    // ========================================
    // Dirty State Management
    // ========================================

    markClean: () =>
      set({ isDirty: false }),
  }))
);

// ============================================
// Selectors
// ============================================

/**
 * Get an entity by name from the current model.
 */
export const selectEntity = (state: EditorState, name: string): SemanticEntity | undefined =>
  state.model.entities.find((e) => e.name === name);

/**
 * Get a metric by name from the current model.
 */
export const selectMetric = (state: EditorState, name: string): SemanticMetric | undefined =>
  state.model.metrics.find((m) => m.name === name);

/**
 * Get the currently selected entity.
 */
export const selectCurrentEntity = (state: EditorState): SemanticEntity | undefined =>
  state.selectedEntity ? selectEntity(state, state.selectedEntity) : undefined;

/**
 * Get the currently selected metric.
 */
export const selectCurrentMetric = (state: EditorState): SemanticMetric | undefined =>
  state.selectedMetric ? selectMetric(state, state.selectedMetric) : undefined;

/**
 * Get all dimension attributes from all entities.
 */
export const selectAllDimensions = (state: EditorState): SemanticAttribute[] =>
  state.model.entities.flatMap((e) => e.attributes.filter((a) => a.is_dimension));

/**
 * Check if there are any validation errors.
 */
export const selectHasErrors = (state: EditorState): boolean =>
  state.validationErrors.length > 0;

/**
 * Check if there are any validation warnings.
 */
export const selectHasWarnings = (state: EditorState): boolean =>
  state.validationWarnings.length > 0;

/**
 * Get all datasets from the current model.
 */
export const selectDatasets = (state: EditorState): Dataset[] =>
  state.model.datasets ?? [];

/**
 * Get visible (non-hidden) attributes for an entity.
 */
export const selectVisibleAttributes = (state: EditorState, entityName: string): SemanticAttribute[] => {
  const entity = state.model.entities.find((e) => e.name === entityName);
  if (!entity) return [];
  return entity.attributes.filter((a) => !a.is_hidden);
};

/**
 * Group attributes by folder for an entity. Ungrouped attributes use empty string key.
 */
export const selectAttributesByFolder = (state: EditorState, entityName: string): Record<string, SemanticAttribute[]> => {
  const entity = state.model.entities.find((e) => e.name === entityName);
  if (!entity) return {};
  const grouped: Record<string, SemanticAttribute[]> = {};
  for (const attr of entity.attributes) {
    const folder = attr.folder ?? '';
    if (!grouped[folder]) {
      grouped[folder] = [];
    }
    grouped[folder].push(attr);
  }
  return grouped;
};
