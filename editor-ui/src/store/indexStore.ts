/**
 * Zustand store for the OCSF Index integration.
 * 
 * Manages:
 * - Log import state (raw input, parsed fields, detected format)
 * - Field mapping state (mappings, selections)
 * - Table registration state (pending table)
 * - UI state (selections, filters)
 * 
 * Requirements: 14.1, 14.4
 */

import { create } from 'zustand';
import type { 
  TableEntry, 
  FieldMapping, 
  ParsedLogField 
} from '../types';

// ============================================
// State Interface
// ============================================

export type LogFormat = 'json' | 'csv' | 'syslog' | 'kv' | null;

export interface IndexState {
  // Log Import State (Requirement 14.1)
  rawLogInput: string;
  parsedFields: ParsedLogField[];
  detectedFormat: LogFormat;
  
  // Mapping State (Requirement 14.4)
  fieldMappings: FieldMapping[];
  selectedSourceField: string | null;
  selectedTargetField: string | null;
  
  // Table Registration State
  pendingTable: Partial<TableEntry> | null;
  
  // UI State
  selectedTableId: number | null;
  selectedTechnique: string | null;
  lineageFilter: { source_system?: string; target_table?: string };
  activeView: 'lineage' | 'coverage' | 'tables' | 'statistics' | null;
  
  // Log Import Actions
  setRawLogInput: (input: string) => void;
  setParsedFields: (fields: ParsedLogField[]) => void;
  setDetectedFormat: (format: LogFormat) => void;
  clearLogImport: () => void;
  
  // Mapping Actions
  addFieldMapping: (mapping: FieldMapping) => void;
  removeFieldMapping: (sourceField: string) => void;
  updateFieldMapping: (sourceField: string, updates: Partial<FieldMapping>) => void;
  clearFieldMappings: () => void;
  setSelectedSourceField: (field: string | null) => void;
  setSelectedTargetField: (field: string | null) => void;
  
  // Table Registration Actions
  setPendingTable: (table: Partial<TableEntry> | null) => void;
  updatePendingTable: (updates: Partial<TableEntry>) => void;
  
  // UI Actions
  setSelectedTableId: (id: number | null) => void;
  setSelectedTechnique: (technique: string | null) => void;
  setLineageFilter: (filter: { source_system?: string; target_table?: string }) => void;
  setActiveView: (view: 'lineage' | 'coverage' | 'tables' | 'statistics' | null) => void;
  
  // Reset
  reset: () => void;
}

// ============================================
// Initial State
// ============================================

const initialState = {
  rawLogInput: '',
  parsedFields: [],
  detectedFormat: null as LogFormat,
  fieldMappings: [],
  selectedSourceField: null,
  selectedTargetField: null,
  pendingTable: null,
  selectedTableId: null,
  selectedTechnique: null,
  lineageFilter: {},
  activeView: null as 'lineage' | 'coverage' | 'tables' | 'statistics' | null,
};

// ============================================
// Store Implementation
// ============================================

export const useIndexStore = create<IndexState>((set) => ({
  ...initialState,
  
  // Log Import Actions
  setRawLogInput: (input) => set({ rawLogInput: input }),
  setParsedFields: (fields) => set({ parsedFields: fields }),
  setDetectedFormat: (format) => set({ detectedFormat: format }),
  clearLogImport: () => set({
    rawLogInput: '',
    parsedFields: [],
    detectedFormat: null,
  }),
  
  // Mapping Actions
  addFieldMapping: (mapping) => set((state) => ({
    fieldMappings: [
      ...state.fieldMappings.filter(m => m.source_field !== mapping.source_field),
      mapping,
    ],
  })),
  removeFieldMapping: (sourceField) => set((state) => ({
    fieldMappings: state.fieldMappings.filter(m => m.source_field !== sourceField),
  })),
  updateFieldMapping: (sourceField, updates) => set((state) => ({
    fieldMappings: state.fieldMappings.map(m =>
      m.source_field === sourceField ? { ...m, ...updates } : m
    ),
  })),
  clearFieldMappings: () => set({ fieldMappings: [] }),
  setSelectedSourceField: (field) => set({ selectedSourceField: field }),
  setSelectedTargetField: (field) => set({ selectedTargetField: field }),
  
  // Table Registration Actions
  setPendingTable: (table) => set({ pendingTable: table }),
  updatePendingTable: (updates) => set((state) => ({
    pendingTable: state.pendingTable ? { ...state.pendingTable, ...updates } : updates,
  })),
  
  // UI Actions
  setSelectedTableId: (id) => set({ selectedTableId: id }),
  setSelectedTechnique: (technique) => set({ selectedTechnique: technique }),
  setLineageFilter: (filter) => set({ lineageFilter: filter }),
  setActiveView: (view) => set({ activeView: view }),
  
  // Reset
  reset: () => set(initialState),
}));

// ============================================
// Selectors
// ============================================

/**
 * Get a field mapping by source field name.
 */
export const selectFieldMapping = (state: IndexState, sourceField: string): FieldMapping | undefined =>
  state.fieldMappings.find(m => m.source_field === sourceField);

/**
 * Check if a source field is already mapped.
 */
export const selectIsFieldMapped = (state: IndexState, sourceField: string): boolean =>
  state.fieldMappings.some(m => m.source_field === sourceField);

/**
 * Get all mapped target fields.
 */
export const selectMappedTargetFields = (state: IndexState): string[] =>
  state.fieldMappings.map(m => m.target_field);

/**
 * Get the count of field mappings.
 */
export const selectMappingCount = (state: IndexState): number =>
  state.fieldMappings.length;
