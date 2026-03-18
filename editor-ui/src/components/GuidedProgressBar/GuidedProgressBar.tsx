/**
 * GuidedProgressBar component — persistent horizontal progress bar showing
 * the 6-step guided workflow for building a semantic model.
 *
 * Renders between the tab navigation and main content area.
 * Shows checkmarks for complete steps, highlighted circle for active,
 * and dots for pending steps. Includes a dismiss button (×).
 *
 * Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
 */

import type { GuideStep, TabId } from '../../types/guide';
import './GuidedProgressBar.css';

export interface GuidedProgressBarProps {
  steps: GuideStep[];
  currentTab: TabId;
  onStepClick: (step: GuideStep) => void;
  onDismiss: () => void;
}

export function GuidedProgressBar({
  steps,
  currentTab,
  onStepClick,
  onDismiss,
}: GuidedProgressBarProps) {
  return (
    <div className="guided-progress-bar" role="navigation" aria-label="Guided workflow progress">
      <div className="guided-progress-steps">
        {steps.map((step, index) => (
          <div key={step.id} className="guided-step-wrapper">
            {/* Connector line between steps */}
            {index > 0 && (
              <div
                className={`guided-step-connector ${
                  steps[index - 1].status === 'complete' ? 'complete' : ''
                }`}
              />
            )}

            <button
              className={`guided-step-indicator ${step.status}`}
              onClick={() => onStepClick(step)}
              title={`${step.label}: ${step.description}`}
              aria-label={`Step ${step.id}: ${step.label} — ${step.status}`}
              aria-current={step.targetTab === currentTab ? 'step' : undefined}
            >
              <span className="guided-step-circle">
                {step.status === 'complete' ? (
                  <span className="guided-step-check" aria-hidden="true">✓</span>
                ) : step.status === 'active' ? (
                  <span className="guided-step-number">{step.id}</span>
                ) : (
                  <span className="guided-step-dot" aria-hidden="true">·</span>
                )}
              </span>
              <span className="guided-step-label">{step.label}</span>
            </button>
          </div>
        ))}
      </div>

      <button
        className="guided-dismiss-btn"
        onClick={onDismiss}
        title="Dismiss guide"
        aria-label="Dismiss guided workflow"
      >
        ×
      </button>
    </div>
  );
}

export default GuidedProgressBar;
