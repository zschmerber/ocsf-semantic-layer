/**
 * StepHint component — contextual info banner displayed at the top of a tab's
 * content area, explaining what the SME should do on this tab and why.
 *
 * Shows different content for pending vs complete step status.
 * Supports per-step dismissal. Only renders when guideVisible is true
 * and the hint for this step has not been dismissed.
 *
 * Requirements: 5.1, 5.2, 5.3, 5.4
 */

import type { GuideStepId, TabId } from '../../types/guide';
import { HINT_CONTENT_PENDING, HINT_CONTENT_COMPLETE } from '../../types/guide';
import { useGuideStore } from '../../store/guideStore';
import './StepHint.css';

export interface StepHintProps {
  stepId: GuideStepId;
  tabId: TabId;
  isStepComplete: boolean;
  onDismissHint: (stepId: GuideStepId) => void;
}

export function StepHint({ stepId, tabId, isStepComplete, onDismissHint }: StepHintProps) {
  const guideVisible = useGuideStore((s) => s.guideVisible);
  const dismissedHints = useGuideStore((s) => s.dismissedHints);

  // Don't render if guide is hidden or this hint was dismissed
  if (!guideVisible || dismissedHints.has(stepId)) {
    return null;
  }

  const content = isStepComplete
    ? HINT_CONTENT_COMPLETE[tabId]
    : HINT_CONTENT_PENDING[tabId];

  if (!content) {
    return null;
  }

  const variant = isStepComplete ? 'complete' : 'pending';
  const icon = isStepComplete ? '✅' : 'ℹ️';

  return (
    <div
      className={`step-hint step-hint--${variant}`}
      role="status"
      aria-label={`Hint: ${content.title}`}
    >
      <span className="step-hint__icon" aria-hidden="true">{icon}</span>
      <div className="step-hint__content">
        <span className="step-hint__title">{content.title}</span>
        <span className="step-hint__description">{content.description}</span>
      </div>
      <button
        className="step-hint__dismiss"
        onClick={() => onDismissHint(stepId)}
        title="Dismiss hint"
        aria-label={`Dismiss hint for step ${stepId}`}
      >
        ×
      </button>
    </div>
  );
}

export default StepHint;
