/**
 * NextStepPrompt component — slide-in banner that appears after step completion,
 * nudging the SME to the next workflow step.
 *
 * Shows a congratulatory message with a navigation button to the next tab.
 * Auto-dismisses after 10 seconds or on user click.
 * Shows a completion message when all 4 steps are complete, with an optional
 * mention of the Index tab for advanced enrichment.
 *
 * Requirements: 5.6, 5.7, 6.2, 6.4
 */

import { useEffect } from 'react';
import type { GuideStepId, TabId } from '../../types/guide';
import './NextStepPrompt.css';

export interface NextStepPromptProps {
  completedStepId: GuideStepId;
  nextStepId: GuideStepId | null;
  nextTabId: TabId | null;
  onNavigate: (tab: TabId) => void;
  onDismiss: () => void;
}

/** Prompt messages keyed by completed step ID. */
const PROMPT_MESSAGES: Record<GuideStepId, string> = {
  1: 'Event class detected! Next: define a semantic entity.',
  2: 'Entity created! Next: add metrics for analytics.',
  3: 'Metrics defined! Next: validate your model.',
  4: '🎉 All steps complete! Your semantic model is ready.',
};

/** Completion message shown when all 4 steps are done (Requirement 6.4). */
const COMPLETION_INDEX_HINT = 'For advanced enrichment, visit the Index tab to add data lineage, detection coverage (MITRE ATT&CK), and physical table registration.';

/** Navigation button labels keyed by next tab ID. */
const TAB_LABELS: Record<TabId, string> = {
  schema: 'Schema',
  entities: 'Entities',
  metrics: 'Metrics',
  validation: 'Validation',
  index: 'Index',
  architecture: 'Architecture',
  catalog: 'Catalog',
  etl: 'ETL',
};

export function NextStepPrompt({
  completedStepId,
  nextStepId,
  nextTabId,
  onNavigate,
  onDismiss,
}: NextStepPromptProps) {
  // Auto-dismiss after 10 seconds
  useEffect(() => {
    const timer = setTimeout(onDismiss, 10_000);
    return () => clearTimeout(timer);
  }, [onDismiss]);

  const message = PROMPT_MESSAGES[completedStepId] ?? '';
  const isAllComplete = nextStepId === null;

  return (
    <div className="next-step-prompt" role="status" aria-label="Step completion prompt">
      <span className="next-step-prompt__message">{message}</span>

      {isAllComplete && (
        <span className="next-step-prompt__index-hint">{COMPLETION_INDEX_HINT}</span>
      )}

      {isAllComplete && (
        <button
          className="next-step-prompt__navigate next-step-prompt__navigate--optional"
          onClick={() => onNavigate('index')}
        >
          → Index (optional)
        </button>
      )}

      {!isAllComplete && nextTabId && (
        <button
          className="next-step-prompt__navigate"
          onClick={() => onNavigate(nextTabId)}
        >
          → {TAB_LABELS[nextTabId] ?? nextTabId}
        </button>
      )}

      <button
        className="next-step-prompt__dismiss"
        onClick={onDismiss}
        title="Dismiss"
        aria-label="Dismiss step prompt"
      >
        ×
      </button>
    </div>
  );
}

export default NextStepPrompt;
