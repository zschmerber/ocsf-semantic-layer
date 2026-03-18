/**
 * Zustand store for the Reference Event in the Analyst Data-First Workflow.
 *
 * Holds parsed event data, class detection, observable flags, type mismatches,
 * LLM mapping interpretation, verification statuses, and coverage metrics.
 * Persists rawJson + mapping to localStorage under "ocsf-reference-event".
 *
 * Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 15.3, 15.10, 16.1
 */

import { create } from 'zustand';
import type {
  ParsedField,
  ObservableFlag,
  ClassDetectionResult,
  TypeMismatch,
  InterpretedMapping,
  VerificationStatus,
  MappingCoverage,
  ReferenceEventSaveState,
} from '../types/referenceEvent';
import type { SchemaTree } from '../types';
import { extractFields } from '../utils/eventFieldExtractor';
import { detectClassUid } from '../utils/classUidDetector';
import { flagObservables } from '../utils/observableFlagger';
import { compareTypes } from '../utils/schemaTypeComparator';
import { computeVerification } from '../utils/verificationComputer';
import { computeCoverage } from '../utils/coverageComputer';

// ============================================
// Constants
// ============================================

const STORAGE_KEY = 'ocsf-reference-event';

// ============================================
// State Interface
// ============================================

export interface ReferenceEventState {
  // Core event data
  rawJson: string | null;
  parsedFields: ParsedField[];
  classDetection: ClassDetectionResult | null;
  observableFlags: ObservableFlag[];
  typeMismatches: TypeMismatch[];

  // LLM mapping interpretation
  mappingRawText: string | null;
  interpretedMapping: InterpretedMapping | null;
  mappingLoading: boolean;
  mappingError: string | null;

  // Verification & coverage (computed from event + mapping)
  verificationStatuses: Map<string, VerificationStatus>;
  mappingCoverage: MappingCoverage | null;

  // Actions
  setReferenceEvent: (json: string, schemaTree: SchemaTree) => void;
  clearReferenceEvent: () => void;
  setInterpretedMapping: (mapping: InterpretedMapping) => void;
  clearMapping: () => void;
  setMappingLoading: (loading: boolean) => void;
  setMappingError: (error: string | null) => void;
  setMappingRawText: (text: string | null) => void;
  rehydrate: (schemaTree: SchemaTree) => void;
}

// ============================================
// localStorage Helpers
// ============================================

function isLocalStorageAvailable(): boolean {
  try {
    const testKey = '__ref_event_storage_test__';
    localStorage.setItem(testKey, '1');
    localStorage.removeItem(testKey);
    return true;
  } catch {
    return false;
  }
}

const storageAvailable = isLocalStorageAvailable();

function persistState(state: {
  rawJson: string | null;
  mappingRawText: string | null;
  interpretedMapping: InterpretedMapping | null;
}): void {
  if (!storageAvailable || !state.rawJson) return;
  try {
    const saveState: ReferenceEventSaveState = {
      version: 1,
      rawJson: state.rawJson,
      mappingRawText: state.mappingRawText,
      interpretedMapping: state.interpretedMapping,
      parsedAt: new Date().toISOString(),
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(saveState));
  } catch {
    console.warn('Failed to persist reference event to localStorage');
  }
}

function loadState(): ReferenceEventSaveState | null {
  if (!storageAvailable) return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as ReferenceEventSaveState;
    if (parsed.version !== 1) {
      console.warn('Incompatible reference event localStorage version, falling back to empty state');
      return null;
    }
    if (typeof parsed.rawJson !== 'string') {
      console.warn('Corrupted reference event localStorage data, falling back to empty state');
      return null;
    }
    return parsed;
  } catch {
    console.warn('Failed to read reference event from localStorage, falling back to empty state');
    return null;
  }
}

function clearPersistedState(): void {
  if (!storageAvailable) return;
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Silently ignore
  }
}

// ============================================
// Initial State from localStorage
// ============================================

const restored = loadState();

// ============================================
// Empty State
// ============================================

const EMPTY_STATE = {
  rawJson: null as string | null,
  parsedFields: [] as ParsedField[],
  classDetection: null as ClassDetectionResult | null,
  observableFlags: [] as ObservableFlag[],
  typeMismatches: [] as TypeMismatch[],
  mappingRawText: null as string | null,
  interpretedMapping: null as InterpretedMapping | null,
  mappingLoading: false,
  mappingError: null as string | null,
  verificationStatuses: new Map<string, VerificationStatus>(),
  mappingCoverage: null as MappingCoverage | null,
};

// ============================================
// Store Implementation
// ============================================

export const useReferenceEventStore = create<ReferenceEventState>()((set, get) => ({
  // Initialize with rawJson from localStorage (derived state computed on rehydrate)
  rawJson: restored?.rawJson ?? null,
  parsedFields: [],
  classDetection: null,
  observableFlags: [],
  typeMismatches: [],
  mappingRawText: restored?.mappingRawText ?? null,
  interpretedMapping: restored?.interpretedMapping ?? null,
  mappingLoading: false,
  mappingError: null,
  verificationStatuses: new Map<string, VerificationStatus>(),
  mappingCoverage: null,

  setReferenceEvent: (json: string, schemaTree: SchemaTree) => {
    const { fields, error } = extractFields(json);
    if (error) {
      // Don't update state on parse error — caller should handle
      return;
    }

    // Build a plain object for class detection
    let parsedObj: Record<string, unknown> = {};
    try {
      parsedObj = JSON.parse(json) as Record<string, unknown>;
    } catch {
      return;
    }

    const classDetection = detectClassUid(parsedObj, schemaTree);
    const observableFlags = flagObservables(fields);
    const typeMismatches =
      classDetection.classUid !== null
        ? compareTypes(fields, classDetection.classUid, schemaTree)
        : [];

    // Check if there's an existing interpreted mapping to recompute verification/coverage
    const { interpretedMapping } = get();
    let verificationStatuses = new Map<string, VerificationStatus>();
    let mappingCoverage: MappingCoverage | null = null;

    if (interpretedMapping) {
      verificationStatuses = computeVerification(interpretedMapping.entries, fields);
      mappingCoverage = computeCoverage(fields, interpretedMapping.entries);
    }

    set({
      rawJson: json,
      parsedFields: fields,
      classDetection,
      observableFlags,
      typeMismatches,
      verificationStatuses,
      mappingCoverage,
    });

    persistState({
      rawJson: json,
      mappingRawText: get().mappingRawText,
      interpretedMapping: get().interpretedMapping,
    });
  },

  clearReferenceEvent: () => {
    set({ ...EMPTY_STATE, verificationStatuses: new Map() });
    clearPersistedState();
  },

  setInterpretedMapping: (mapping: InterpretedMapping) => {
    const { parsedFields } = get();
    const verificationStatuses = computeVerification(mapping.entries, parsedFields);
    const mappingCoverage = computeCoverage(parsedFields, mapping.entries);

    set({
      interpretedMapping: mapping,
      verificationStatuses,
      mappingCoverage,
      mappingLoading: false,
      mappingError: null,
    });

    persistState({
      rawJson: get().rawJson,
      mappingRawText: get().mappingRawText,
      interpretedMapping: mapping,
    });
  },

  clearMapping: () => {
    set({
      mappingRawText: null,
      interpretedMapping: null,
      mappingLoading: false,
      mappingError: null,
      verificationStatuses: new Map<string, VerificationStatus>(),
      mappingCoverage: null,
    });

    // Update persisted state — keep rawJson, remove mapping
    const { rawJson } = get();
    if (rawJson) {
      persistState({ rawJson, mappingRawText: null, interpretedMapping: null });
    }
  },

  setMappingLoading: (loading: boolean) => {
    set({ mappingLoading: loading });
  },

  setMappingError: (error: string | null) => {
    set({ mappingError: error });
  },

  setMappingRawText: (text: string | null) => {
    set({ mappingRawText: text });
  },

  rehydrate: (schemaTree: SchemaTree) => {
    const { rawJson, interpretedMapping } = get();
    if (!rawJson) return;

    const { fields, error } = extractFields(rawJson);
    if (error) {
      // Stored JSON is no longer parseable — reset
      console.warn('Stored reference event JSON is corrupted, resetting');
      set({ ...EMPTY_STATE, verificationStatuses: new Map() });
      clearPersistedState();
      return;
    }

    let parsedObj: Record<string, unknown> = {};
    try {
      parsedObj = JSON.parse(rawJson) as Record<string, unknown>;
    } catch {
      console.warn('Stored reference event JSON is corrupted, resetting');
      set({ ...EMPTY_STATE, verificationStatuses: new Map() });
      clearPersistedState();
      return;
    }

    const classDetection = detectClassUid(parsedObj, schemaTree);
    const observableFlags = flagObservables(fields);
    const typeMismatches =
      classDetection.classUid !== null
        ? compareTypes(fields, classDetection.classUid, schemaTree)
        : [];

    let verificationStatuses = new Map<string, VerificationStatus>();
    let mappingCoverage: MappingCoverage | null = null;

    if (interpretedMapping) {
      verificationStatuses = computeVerification(interpretedMapping.entries, fields);
      mappingCoverage = computeCoverage(fields, interpretedMapping.entries);
    }

    set({
      parsedFields: fields,
      classDetection,
      observableFlags,
      typeMismatches,
      verificationStatuses,
      mappingCoverage,
    });
  },
}));
