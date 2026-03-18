/**
 * EmptyStateGuide component — guided empty state panel shown on tabs with no
 * content yet, providing calls-to-action or prerequisite navigation.
 *
 * When prerequisites are not met, displays a warning with a link to the
 * prerequisite tab. When prerequisites are met, shows the primary action
 * for the tab. Falls back to existing empty state when guide is dismissed.
 *
 * Requirements: 4.1, 4.2, 4.3, 4.4
 */

import type { TabId, GuideStepId } from '../../types/guide';
import { useGuideStore } from '../../store/guideStore';
import './EmptyStateGuide.css';

export interface EmptyStateGuideProps {
  tabId: TabId;
  stepId: GuideStepId | null;
  prerequisitesMet: boolean;
  onNavigateToPrerequisite: (tab: TabId) => void;
}

/** Tab-specific empty state content when prerequisites are met. */
const READY_CONTENT: Partial<Record<TabId, { icon: string; heading: string; description: string; action?: string }>> = {
  schema: {
    icon: '📋',
    heading: 'Start by pasting an event',
    description: 'Paste a transformed OCSF JSON event to auto-detect the event class and seed the workflow. Or skip and browse the schema tree manually.',
  },
  entities: {
    icon: '✅',
    heading: 'Great!',
    description: 'Now click \u2018+ New Entity\u2019 to create a semantic entity.',
    action: '+ New Entity',
  },
  metrics: {
    icon: '📊',
    heading: 'Ready to add metrics!',
    description: 'Click \u2018+ New Metric\u2019 to define aggregations over your entity attributes.',
    action: '+ New Metric',
  },
  validation: {
    icon: '🔍',
    heading: 'Validate your model',
    description: 'Run validation to check your model for errors before registering.',
    action: 'Run Validation',
  },
  index: {
    icon: '📋',
    heading: 'Register your table',
    description: 'Register your physical table and map field lineage to connect your semantic model to data.',
    action: 'Register Table',
  },
};

/** Tab-specific prerequisite-not-met content. */
const PREREQUISITE_CONTENT: Partial<Record<TabId, { message: string; prerequisiteTab: TabId; prerequisiteLabel: string }>> = {
  entities: {
    message: 'First, paste an OCSF event or select an event class in the Schema tab.',
    prerequisiteTab: 'schema',
    prerequisiteLabel: 'Go to Schema',
  },
  metrics: {
    message: 'Define at least one entity with attributes first.',
    prerequisiteTab: 'entities',
    prerequisiteLabel: 'Go to Entities',
  },
  validation: {
    message: 'Define at least one entity with attributes first.',
    prerequisiteTab: 'entities',
    prerequisiteLabel: 'Go to Entities',
  },
  index: {
    message: 'Validate your model first.',
    prerequisiteTab: 'validation',
    prerequisiteLabel: 'Go to Validation',
  },
};

export function EmptyStateGuide({
  tabId,
  stepId,
  prerequisitesMet,
  onNavigateToPrerequisite,
}: EmptyStateGuideProps) {
  const guideVisible = useGuideStore((s) => s.guideVisible);

  // Requirement 4.4: Fall back to existing empty state when guide is dismissed
  if (!guideVisible) {
    return null;
  }

  // Tabs not in the 4-step flow — no guided empty state
  if (stepId === null) {
    return null;
  }

  // Requirement 4.2: Prerequisites not met — show warning with navigation link
  if (!prerequisitesMet) {
    const prereqContent = PREREQUISITE_CONTENT[tabId];
    const message = prereqContent?.message ?? 'Complete the previous steps first.';
    const navTab = prereqContent?.prerequisiteTab ?? 'schema';
    const navLabel = prereqContent?.prerequisiteLabel ?? 'Go to previous step';

    return (
      <div
        className="empty-state-guide empty-state-guide--warning"
        role="status"
        aria-label="Prerequisites not met"
      >
        <span className="empty-state-guide__icon" aria-hidden="true">⚠️</span>
        <div className="empty-state-guide__content">
          <p className="empty-state-guide__message">{message}</p>
          <button
            className="empty-state-guide__nav-button"
            onClick={() => onNavigateToPrerequisite(navTab)}
            aria-label={navLabel}
          >
            {navLabel} →
          </button>
        </div>
      </div>
    );
  }

  // Requirement 4.3: Prerequisites met — show primary call-to-action
  const readyContent = READY_CONTENT[tabId];
  if (!readyContent) {
    return null;
  }

  return (
    <div
      className="empty-state-guide empty-state-guide--ready"
      role="status"
      aria-label={readyContent.heading}
    >
      <span className="empty-state-guide__icon" aria-hidden="true">{readyContent.icon}</span>
      <div className="empty-state-guide__content">
        <h3 className="empty-state-guide__heading">{readyContent.heading}</h3>
        <p className="empty-state-guide__description">{readyContent.description}</p>
      </div>
    </div>
  );
}

export default EmptyStateGuide;
