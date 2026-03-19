# Requirements Document

## Introduction

This document defines the requirements for the Guided Wizard Workflow feature, which adds an onboarding guidance layer to the existing OCSF Semantic Model Editor. The guidance layer consists of five overlay components (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt, ContextualTooltip) and a Zustand-based guide store that derives step completion from existing model state. The feature teaches SMEs the correct 6-step workflow for building a semantic model without replacing or modifying the existing tab UI. No backend changes are required.

## Glossary

- **Guide_Layer**: The collective set of overlay components (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt, ContextualTooltip) that provide onboarding guidance
- **GuidedProgressBar**: A persistent horizontal bar rendered between the tab navigation and main content area, showing 6 workflow steps with completion status
- **StepHint**: A contextual info banner displayed at the top of a tab's content area explaining what the SME should do on that tab
- **EmptyStateGuide**: A guided empty state panel shown on tabs with no content yet, providing calls-to-action or prerequisite navigation
- **NextStepPrompt**: A slide-in banner that appears after step completion, nudging the SME to the next workflow step
- **ContextualTooltip**: An inline popover triggered by a help icon (?) that explains OCSF-specific terms
- **GuideStore**: The Zustand store (`useGuideStore`) that manages guide visibility, step statuses, dismissed hints, and shown prompts
- **Step_Derivation**: The pure function `deriveStepStatuses` that computes step completion from EditorStore and IndexStore model state
- **EditorStore**: The existing Zustand store (`useEditorStore`) holding the semantic model (entities, metrics, selected class)
- **IndexStore**: The existing Zustand store (`useIndexStore`) holding registered tables and index state
- **SME**: Subject Matter Expert — the primary user of the guided workflow
- **Tab_UI**: The existing 8-tab editor interface (Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, ETL)

## Requirements

### Requirement 1: Guide Visibility Control

**User Story:** As an SME, I want to show or hide the onboarding guide, so that I can get guidance when I need it and dismiss it when I am comfortable with the workflow.

#### Acceptance Criteria

1. WHEN the editor loads with no prior guide preferences in localStorage, THE Guide_Layer SHALL default to visible
2. WHEN the SME clicks the dismiss button on the GuidedProgressBar, THE GuideStore SHALL set guideVisible to false and persist the preference to localStorage
3. WHEN guideVisible is false, THE Guide_Layer SHALL hide all overlay components (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt, ContextualTooltip)
4. WHEN the SME re-enables the guide via the header menu, THE GuideStore SHALL set guideVisible to true and persist the preference to localStorage
5. WHEN the editor loads with a previously saved guide preference, THE GuideStore SHALL restore the guideVisible value from localStorage
6. IF localStorage is unavailable, THEN THE GuideStore SHALL operate in-memory with guideVisible defaulting to true and preferences resetting on page reload

### Requirement 2: GuidedProgressBar Display

**User Story:** As an SME, I want to see a persistent progress bar showing the 6 workflow steps, so that I know where I am in the model-building process and what steps remain.

#### Acceptance Criteria

1. WHILE guideVisible is true, THE GuidedProgressBar SHALL render a horizontal bar between the tab navigation and the main content area showing 6 numbered step indicators
2. THE GuidedProgressBar SHALL map steps to tabs as follows: Step 1 to Schema, Step 2 to Entities, Step 3 to Entities, Step 4 to Metrics, Step 5 to Validation, Step 6 to Index
3. WHEN a step status is complete, THE GuidedProgressBar SHALL display a checkmark indicator for that step
4. WHEN a step status is pending, THE GuidedProgressBar SHALL display a dot indicator for that step
5. WHEN a step corresponds to the currently active tab and the step is not complete, THE GuidedProgressBar SHALL highlight that step as active
6. WHEN the SME clicks a step indicator, THE GuidedProgressBar SHALL navigate to that step's associated tab

### Requirement 3: Step Completion Derivation

**User Story:** As an SME, I want step completion to be automatically detected from my work, so that I do not need to manually mark steps as done.

#### Acceptance Criteria

1. THE Step_Derivation SHALL compute step statuses as a pure function of EditorStore state, IndexStore state, GuideStore state, and the active tab
2. WHEN the EditorStore model contains at least one entity with source_event_classes or a selected class UID, THE Step_Derivation SHALL mark Step 1 as complete
3. WHEN the EditorStore model contains at least one entity, THE Step_Derivation SHALL mark Step 2 as complete
4. WHEN at least one entity in the EditorStore model has one or more attributes, THE Step_Derivation SHALL mark Step 3 as complete
5. WHEN the EditorStore model contains at least one metric, THE Step_Derivation SHALL mark Step 4 as complete
6. WHEN the GuideStore lastValidationRun timestamp is not null, THE Step_Derivation SHALL mark Step 5 as complete
7. WHEN the IndexStore contains at least one registered table, THE Step_Derivation SHALL mark Step 6 as complete
8. THE Step_Derivation SHALL produce identical results when called multiple times with the same inputs

### Requirement 4: Prerequisite Enforcement

**User Story:** As an SME, I want to be informed when I visit a tab whose prerequisites are not yet met, so that I follow the correct workflow order.

#### Acceptance Criteria

1. THE EmptyStateGuide SHALL enforce the following prerequisite chain: Step 2 requires Step 1; Step 3 requires Steps 1 and 2; Step 4 requires Steps 1, 2, and 3; Step 5 requires Steps 1, 2, and 3; Step 6 requires Steps 1, 2, 3, and 5
2. WHEN an SME visits a tab whose prerequisites are not met, THE EmptyStateGuide SHALL display a message identifying the missing prerequisite step and provide a navigation link to the prerequisite tab
3. WHEN an SME visits a tab whose prerequisites are met and the tab content is empty, THE EmptyStateGuide SHALL display the primary call-to-action for that tab
4. WHEN guideVisible is false and a tab has no content, THE Tab_UI SHALL render the existing default empty state instead of the EmptyStateGuide

### Requirement 5: StepHint Contextual Banners

**User Story:** As an SME, I want contextual hints on each tab explaining what to do and why, so that I understand the purpose of each step in the workflow.

#### Acceptance Criteria

1. WHILE guideVisible is true, THE StepHint SHALL render an info banner at the top of each tab's content area with a title and description specific to that tab
2. WHEN a step is complete, THE StepHint SHALL display completion-specific content different from the pending-state content
3. WHEN the SME dismisses a StepHint for a specific step, THE GuideStore SHALL record the dismissal and THE StepHint for that step SHALL remain hidden for the session
4. THE StepHint SHALL provide content for all 8 tabs: Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, and ETL

### Requirement 6: NextStepPrompt Navigation

**User Story:** As an SME, I want to be nudged to the next step after completing work on a tab, so that I can follow the workflow without memorizing the step order.

#### Acceptance Criteria

1. WHEN a step transitions to complete and guideVisible is true and the prompt for that step has not been shown before, THE NextStepPrompt SHALL appear as a slide-in banner at the bottom of the content area
2. THE NextStepPrompt SHALL display a congratulatory message and a button to navigate to the next step's tab
3. WHEN the SME clicks the navigation button on the NextStepPrompt, THE Tab_UI SHALL switch to the next step's associated tab
4. WHEN 10 seconds elapse after the NextStepPrompt appears or the SME clicks to dismiss, THE NextStepPrompt SHALL hide
5. WHEN all 6 steps are complete, THE NextStepPrompt for Step 6 SHALL display a completion message with no navigation action
6. THE GuideStore SHALL track shown prompts so that each step's NextStepPrompt is displayed at most once per session

### Requirement 7: ContextualTooltip for OCSF Terms

**User Story:** As an SME, I want inline explanations of OCSF-specific terms, so that I can understand the domain concepts without leaving the editor.

#### Acceptance Criteria

1. WHILE guideVisible is true, THE ContextualTooltip SHALL render a help icon (?) next to wrapped OCSF terms in the UI
2. WHEN the SME hovers over or clicks the help icon, THE ContextualTooltip SHALL display a popover with a concise explanation of the OCSF term
3. THE ContextualTooltip SHALL include a "Learn more" link to OCSF documentation where applicable
4. IF the tooltip would render outside the viewport, THEN THE ContextualTooltip SHALL reposition to remain within viewport boundaries

### Requirement 8: Tab Structure Preservation

**User Story:** As a power user, I want the existing tab structure to remain unchanged, so that the guide does not interfere with my established workflow.

#### Acceptance Criteria

1. THE Tab_UI SHALL maintain the existing 8 tabs (Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, ETL) in their current order regardless of guide state
2. THE Guide_Layer SHALL operate as an overlay that does not modify, reorder, or replace any existing tab components
3. WHEN guideVisible is false, THE Tab_UI SHALL function identically to the pre-guide editor with zero overhead from guide components

### Requirement 9: Guide State Persistence

**User Story:** As an SME, I want my guide preferences to persist across page reloads, so that I do not have to re-dismiss hints or re-configure the guide each session.

#### Acceptance Criteria

1. WHEN the GuideStore state changes (guideVisible, dismissedHints, shownPrompts, lastValidationRun), THE GuideStore SHALL persist the state to localStorage under the key "ocsf-guide-preferences"
2. WHEN the editor loads, THE GuideStore SHALL read from localStorage and restore the persisted guide preferences
3. THE GuideStore SHALL serialize the persisted state using the GuideSaveState schema with version field set to 1
4. IF localStorage read returns corrupted or incompatible data, THEN THE GuideStore SHALL fall back to default state (guideVisible true, no dismissed hints, no shown prompts)

### Requirement 10: Model State Synchronization

**User Story:** As an SME, I want step statuses to update automatically when the model changes, so that the guide always reflects my current progress.

#### Acceptance Criteria

1. WHEN the EditorStore model state changes, THE GuideStore SHALL refresh step statuses within 100ms via debounced re-derivation
2. WHEN the IndexStore state changes, THE GuideStore SHALL refresh step statuses within 100ms via debounced re-derivation
3. WHEN an external model change occurs (import, undo, redo), THE GuideStore SHALL detect the change and update step statuses accordingly
4. THE Step_Derivation SHALL treat missing or unexpected model fields as "step not complete" and log a warning to the console

### Requirement 11: Error Handling and Graceful Degradation

**User Story:** As an SME, I want the guide to handle errors gracefully, so that editor functionality is never blocked by guide issues.

#### Acceptance Criteria

1. IF localStorage is unavailable (private browsing, quota exceeded), THEN THE GuideStore SHALL operate in-memory without persistence and default to guideVisible true
2. IF the EditorStore model has an unexpected shape (missing fields), THEN THE Step_Derivation SHALL default to "step not complete" for affected steps and log a console warning
3. IF a ContextualTooltip encounters a positioning error, THEN THE ContextualTooltip SHALL use viewport boundary detection to reposition automatically
4. THE Guide_Layer SHALL never throw an unhandled exception that prevents the Tab_UI from rendering
