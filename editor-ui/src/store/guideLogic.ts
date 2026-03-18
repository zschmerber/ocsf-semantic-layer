/**
 * Pure functions for deriving guide step statuses from model state.
 *
 * These functions are intentionally kept pure (no side effects, no store access)
 * so they can be easily tested with property-based tests.
 *
 * Requirements: 5.4, 5.5, 12.2, 12.3
 */

import type { SemanticModel } from '../types';
import type { GuideStepId, StepStatus, TabId } from '../types/guide';
import { STEP_TO_TAB, STEP_PREREQUISITES } from '../types/guide';
import type { ClassDetectionResult } from '../types/referenceEvent';

/**
 * Mapping from tab to the guide step it corresponds to.
 * Only tabs that are part of the 4-step workflow are included.
 */
export const TAB_TO_STEP: Partial<Record<TabId, GuideStepId>> = {
  schema: 1,
  entities: 2,
  metrics: 3,
  validation: 4,
};

/**
 * Derive step statuses from model state, reference event, and active tab.
 *
 * This is a pure function — calling it multiple times with the same inputs
 * produces identical results.
 *
 * Step 1 complete: referenceEvent loaded with class detected (classDetection.classUid !== null)
 *                  OR manual class selected in EditorStore (entity has source_event_classes)
 * Step 2 complete: at least one entity with attributes
 * Step 3 complete: at least one metric defined
 * Step 4 complete: validation has been run (lastValidationRun is not null)
 *
 * Requirements: 5.4, 5.5, 12.2, 12.3
 */
export function deriveStepStatuses(
  model: SemanticModel,
  classDetection: ClassDetectionResult | null,
  lastValidationRun: string | null,
  activeTab: TabId
): Record<GuideStepId, StepStatus> {
  const statuses: Record<GuideStepId, StepStatus> = {
    1: 'pending',
    2: 'pending',
    3: 'pending',
    4: 'pending',
  };

  try {
    // Step 1: Reference event loaded with class detected, OR manual class selected
    const hasClassFromEvent = classDetection?.classUid !== null && classDetection?.classUid !== undefined;
    const entities = model?.entities;
    const hasManualClass = Array.isArray(entities) && entities.some(
      (e) => Array.isArray(e?.source_event_classes) && e.source_event_classes.length > 0
    );
    if (hasClassFromEvent || hasManualClass) {
      statuses[1] = 'complete';
    }

    // Step 2: At least one entity defined with attributes
    if (Array.isArray(entities)) {
      const hasEntityWithAttributes = entities.some(
        (e) => Array.isArray(e?.attributes) && e.attributes.length > 0
      );
      if (hasEntityWithAttributes) statuses[2] = 'complete';
    } else {
      console.warn('[guideLogic] model.entities is not an array, defaulting Step 2 to pending');
    }

    // Step 3: At least one metric defined
    const metrics = model?.metrics;
    if (!Array.isArray(metrics)) {
      console.warn('[guideLogic] model.metrics is not an array, defaulting Step 3 to pending');
    } else if (metrics.length > 0) {
      statuses[3] = 'complete';
    }

    // Step 4: Validation has been run
    if (lastValidationRun !== null && lastValidationRun !== undefined) {
      statuses[4] = 'complete';
    }
  } catch (err) {
    console.warn('[guideLogic] Error deriving step statuses, defaulting to pending:', err);
    return { 1: 'pending', 2: 'pending', 3: 'pending', 4: 'pending' };
  }

  // Mark active step based on current tab
  const activeStep = TAB_TO_STEP[activeTab];
  if (activeStep !== undefined && statuses[activeStep] !== 'complete') {
    statuses[activeStep] = 'active';
  }

  return statuses;
}

/**
 * Check whether all prerequisites for a given step are met.
 *
 * Returns the first missing prerequisite step and its tab if any are incomplete.
 * Requirements: 4.1, 4.2
 */
export function checkPrerequisites(
  stepId: GuideStepId,
  stepStatuses: Record<GuideStepId, StepStatus>
): { met: boolean; missingStep: GuideStepId | null; missingTab: TabId | null } {
  const required = STEP_PREREQUISITES[stepId];
  if (!required) {
    return { met: true, missingStep: null, missingTab: null };
  }

  for (const req of required) {
    if (stepStatuses[req] !== 'complete') {
      return { met: false, missingStep: req, missingTab: STEP_TO_TAB[req] };
    }
  }

  return { met: true, missingStep: null, missingTab: null };
}

/**
 * Determine whether the next-step prompt should be shown for a completed step.
 *
 * Returns true at most once per step — after markPromptShown is called,
 * always returns false for that step (Requirement 6.1, 6.6).
 */
export function shouldShowNextPrompt(
  completedStepId: GuideStepId,
  guideVisible: boolean,
  shownPrompts: Set<GuideStepId>
): boolean {
  if (!guideVisible) return false;
  if (shownPrompts.has(completedStepId)) return false;
  return true;
}

/**
 * Get the next step and its target tab after completing a given step.
 *
 * Returns null for both fields when all steps are complete (step 4).
 */
export function getNextStep(completedStepId: GuideStepId): {
  nextStepId: GuideStepId | null;
  nextTab: TabId | null;
} {
  const nextMap: Record<GuideStepId, { nextStepId: GuideStepId | null; nextTab: TabId | null }> = {
    1: { nextStepId: 2, nextTab: 'entities' },
    2: { nextStepId: 3, nextTab: 'metrics' },
    3: { nextStepId: 4, nextTab: 'validation' },
    4: { nextStepId: null, nextTab: null },
  };

  return nextMap[completedStepId] ?? { nextStepId: null, nextTab: null };
}
