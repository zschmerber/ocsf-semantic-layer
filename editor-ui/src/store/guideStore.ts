/**
 * Zustand store for the Guided Wizard Workflow overlay layer.
 *
 * Manages guide visibility, step statuses, dismissed hints, shown prompts,
 * and localStorage persistence under the "ocsf-guide-preferences" key.
 *
 * Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 9.1, 9.2, 9.3, 9.4, 10.1, 10.2, 11.1
 */

import { create } from 'zustand';
import type { GuideStepId, StepStatus, TabId, GuideSaveState } from '../types/guide';
import { GUIDE_STORAGE_KEY } from '../types/guide';
import { deriveStepStatuses, TAB_TO_STEP } from './guideLogic';
import { useEditorStore } from './editorStore';
import type { ClassDetectionResult } from '../types/referenceEvent';

// ============================================
// State Interface
// ============================================

export interface GuideState {
  guideVisible: boolean;
  dismissedHints: Set<GuideStepId>;
  shownPrompts: Set<GuideStepId>;
  stepStatuses: Record<GuideStepId, StepStatus>;
  lastValidationRun: string | null;
  activeStepId: GuideStepId | null;

  // Actions
  setGuideVisible: (visible: boolean) => void;
  dismissHint: (stepId: GuideStepId) => void;
  markPromptShown: (stepId: GuideStepId) => void;
  refreshStepStatuses: (activeTab: TabId, classDetection: ClassDetectionResult | null) => void;
  markValidationRun: () => void;
  resetGuide: () => void;
}

// ============================================
// localStorage Helpers
// ============================================

/**
 * Check whether localStorage is available.
 * Returns false in private browsing, quota exceeded, or SSR contexts.
 */
function isLocalStorageAvailable(): boolean {
  try {
    const testKey = '__guide_storage_test__';
    localStorage.setItem(testKey, '1');
    localStorage.removeItem(testKey);
    return true;
  } catch {
    return false;
  }
}

const storageAvailable = isLocalStorageAvailable();

/**
 * Persist guide preferences to localStorage.
 * Silently no-ops if localStorage is unavailable.
 */
function persistState(state: {
  guideVisible: boolean;
  dismissedHints: Set<GuideStepId>;
  shownPrompts: Set<GuideStepId>;
  lastValidationRun: string | null;
}): void {
  if (!storageAvailable) return;
  try {
    const saveState: GuideSaveState = {
      version: 1,
      guideVisible: state.guideVisible,
      dismissedHints: Array.from(state.dismissedHints),
      shownPrompts: Array.from(state.shownPrompts),
      lastValidationRun: state.lastValidationRun,
    };
    localStorage.setItem(GUIDE_STORAGE_KEY, JSON.stringify(saveState));
  } catch {
    // Quota exceeded or other write error — silently ignore
  }
}

/**
 * Load guide preferences from localStorage.
 * Returns null if unavailable, missing, or corrupted.
 */
function loadState(): {
  guideVisible: boolean;
  dismissedHints: Set<GuideStepId>;
  shownPrompts: Set<GuideStepId>;
  lastValidationRun: string | null;
} | null {
  if (!storageAvailable) return null;
  try {
    const raw = localStorage.getItem(GUIDE_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as GuideSaveState;
    if (parsed.version !== 1) return null;
    if (typeof parsed.guideVisible !== 'boolean') return null;
    if (!Array.isArray(parsed.dismissedHints)) return null;
    if (!Array.isArray(parsed.shownPrompts)) return null;

    return {
      guideVisible: parsed.guideVisible,
      dismissedHints: new Set(parsed.dismissedHints as GuideStepId[]),
      shownPrompts: new Set(parsed.shownPrompts as GuideStepId[]),
      lastValidationRun: parsed.lastValidationRun ?? null,
    };
  } catch {
    return null;
  }
}

// ============================================
// Debounce Helper
// ============================================

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// ============================================
// Default State
// ============================================

const DEFAULT_STEP_STATUSES: Record<GuideStepId, StepStatus> = {
  1: 'pending',
  2: 'pending',
  3: 'pending',
  4: 'pending',
};

// ============================================
// Initialize from localStorage
// ============================================

const restored = loadState();

const initialGuideVisible = restored?.guideVisible ?? true;
const initialDismissedHints = restored?.dismissedHints ?? new Set<GuideStepId>();
const initialShownPrompts = restored?.shownPrompts ?? new Set<GuideStepId>();
const initialLastValidationRun = restored?.lastValidationRun ?? null;

// ============================================
// Store Implementation
// ============================================

export const useGuideStore = create<GuideState>()((set, get) => ({
  guideVisible: initialGuideVisible,
  dismissedHints: initialDismissedHints,
  shownPrompts: initialShownPrompts,
  stepStatuses: { ...DEFAULT_STEP_STATUSES },
  lastValidationRun: initialLastValidationRun,
  activeStepId: null,

  setGuideVisible: (visible: boolean) => {
    set({ guideVisible: visible });
    const state = get();
    persistState(state);
  },

  dismissHint: (stepId: GuideStepId) => {
    set((state) => {
      const newDismissed = new Set(state.dismissedHints);
      newDismissed.add(stepId);
      return { dismissedHints: newDismissed };
    });
    const state = get();
    persistState(state);
  },

  markPromptShown: (stepId: GuideStepId) => {
    set((state) => {
      const newShown = new Set(state.shownPrompts);
      newShown.add(stepId);
      return { shownPrompts: newShown };
    });
    const state = get();
    persistState(state);
  },

  refreshStepStatuses: (activeTab: TabId, classDetection: ClassDetectionResult | null) => {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      debounceTimer = null;
      const model = useEditorStore.getState().model;
      const { lastValidationRun } = get();
      const statuses = deriveStepStatuses(model, classDetection, lastValidationRun, activeTab);

      const activeStepId = TAB_TO_STEP[activeTab] ?? null;

      set({ stepStatuses: statuses, activeStepId });
    }, 100);
  },

  markValidationRun: () => {
    const timestamp = new Date().toISOString();
    set({ lastValidationRun: timestamp });
    const state = get();
    persistState(state);
  },

  resetGuide: () => {
    set({
      guideVisible: true,
      dismissedHints: new Set<GuideStepId>(),
      shownPrompts: new Set<GuideStepId>(),
      stepStatuses: { ...DEFAULT_STEP_STATUSES },
      lastValidationRun: null,
      activeStepId: null,
    });
    const state = get();
    persistState(state);
  },
}));
