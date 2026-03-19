# Implementation Plan: Guided Wizard Workflow

## Overview

Add an onboarding guidance layer to the existing OCSF Semantic Model Editor tabs. The implementation creates a Zustand-based `useGuideStore` that derives step completion from existing `useEditorStore` and `useIndexStore` state, plus 5 overlay components (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt, ContextualTooltip). No backend changes. All guidance components are conditionally rendered based on `guideVisible` and do not modify existing tab components.

## Tasks

- [x] 1. Create guide types, constants, and Zustand store
  - [x] 1.1 Define guide types and constants in `editor-ui/src/types/guide.ts`
    - Define `GuideStepId`, `StepStatus`, `GuideStep`, `TabId` mapping types
    - Define `OCSFTerm` union type for tooltip terms
    - Define `GuideSaveState` interface for localStorage serialization (version field = 1)
    - Define step-to-tab mapping constant and prerequisite chain constant
    - Define hint content map and tooltip content map as typed constants
    - _Requirements: 2.2, 4.1, 5.4, 7.2_

  - [x] 1.2 Implement `deriveStepStatuses` pure function in `editor-ui/src/store/guideLogic.ts`
    - Compute step statuses from EditorStore model (entities, attributes, metrics, selectedClassUid), IndexStore state (registered tables), GuideStore state (lastValidationRun), and active tab
    - Handle missing/unexpected fields by defaulting to 'pending' and logging a console warning
    - Implement `checkPrerequisites` function with the defined prerequisite chain
    - Implement `shouldShowNextPrompt` and `getNextStep` helper functions
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 4.1, 4.2, 6.1, 10.4, 11.2_

  - [ ]* 1.3 Write property tests for `deriveStepStatuses` (fast-check)
    - **Property 2: Step derivation purity** — calling `deriveStepStatuses` twice with same inputs produces identical results
    - **Property 3: Step completion correctness** — each step marked complete iff its condition is met
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8**

  - [ ]* 1.4 Write property tests for `checkPrerequisites` (fast-check)
    - **Property 4: Prerequisite enforcement correctness** — returns met=false with correct missing step when any prerequisite incomplete, met=true when all complete
    - **Validates: Requirements 4.1, 4.2, 4.3**

  - [ ]* 1.5 Write property test for prompt uniqueness (fast-check)
    - **Property 7: Prompt uniqueness** — `shouldShowNextPrompt` returns true at most once per step; after `markPromptShown`, always returns false
    - **Validates: Requirements 6.1, 6.6**

  - [x] 1.6 Implement `useGuideStore` Zustand store in `editor-ui/src/store/guideStore.ts`
    - State: `guideVisible`, `dismissedHints` (Set), `shownPrompts` (Set), `stepStatuses`, `lastValidationRun`, `activeStepId`
    - Actions: `setGuideVisible`, `dismissHint`, `markPromptShown`, `refreshStepStatuses`, `markValidationRun`, `resetGuide`
    - Persist to localStorage under key `ocsf-guide-preferences` using `GuideSaveState` schema
    - Restore from localStorage on initialization; fall back to defaults on corrupted/missing data
    - Handle localStorage unavailability gracefully (in-memory only, guideVisible defaults to true)
    - Debounce `refreshStepStatuses` by 100ms
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 9.1, 9.2, 9.3, 9.4, 10.1, 10.2, 11.1_

  - [ ]* 1.7 Write property test for guide state persistence round-trip (fast-check)
    - **Property 1: Guide state persistence round-trip** — serialize to localStorage and deserialize back produces equivalent state
    - **Validates: Requirements 1.2, 1.4, 1.5, 9.1, 9.2, 9.3**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement GuidedProgressBar component
  - [x] 3.1 Create `editor-ui/src/components/GuidedProgressBar/GuidedProgressBar.tsx`
    - Render a compact horizontal bar with 6 numbered step indicators
    - Show checkmark for complete steps, dot for pending, highlighted for active
    - Clicking a step calls `onStepClick` to navigate to the step's target tab
    - Include a dismiss button (×) that calls `onDismiss`
    - Render between tab navigation and main content area
    - Use CSS transitions for step status changes
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [x] 3.2 Create `editor-ui/src/components/GuidedProgressBar/GuidedProgressBar.css`
    - Style the progress bar, step indicators, checkmarks, active highlight, dismiss button
    - Ensure compact layout that doesn't take excessive vertical space
    - _Requirements: 2.1_

  - [ ]* 3.3 Write unit tests for GuidedProgressBar
    - Test rendering with various step status combinations (all pending, mixed, all complete)
    - Test step click navigation callback
    - Test dismiss button callback
    - **Property 5: Step indicator rendering correctness** — correct visual indicator per status
    - **Validates: Requirements 2.3, 2.4, 2.5, 2.6**

- [x] 4. Implement StepHint component
  - [x] 4.1 Create `editor-ui/src/components/StepHint/StepHint.tsx`
    - Render a colored info banner with icon, title, and description at the top of tab content
    - Show different content for pending vs complete step status (from hint content map)
    - Support per-step dismissal via `onDismissHint` callback
    - Only render when `guideVisible` is true and the hint for this step is not dismissed
    - Provide content for all 8 tabs (Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, ETL)
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

  - [x] 4.2 Create `editor-ui/src/components/StepHint/StepHint.css`
    - Style the info banner with appropriate colors, icon placement, dismiss button
    - _Requirements: 5.1_

  - [ ]* 4.3 Write unit tests for StepHint
    - Test rendering correct content per tab
    - Test pending vs complete content switching
    - Test dismissal hides the hint
    - **Property 6: StepHint content and dismissal correctness**
    - **Validates: Requirements 5.1, 5.2, 5.3**

- [x] 5. Implement EmptyStateGuide component
  - [x] 5.1 Create `editor-ui/src/components/EmptyStateGuide/EmptyStateGuide.tsx`
    - Show guided empty state with icon and call-to-action when tab content is empty and guide is active
    - If prerequisites not met, display message identifying missing step with navigation link to prerequisite tab
    - If prerequisites met, show primary action for the tab
    - Fall back to existing empty state when guide is dismissed
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [x] 5.2 Create `editor-ui/src/components/EmptyStateGuide/EmptyStateGuide.css`
    - Style the empty state panel with icon, message, and call-to-action button
    - _Requirements: 4.3_

  - [ ]* 5.3 Write unit tests for EmptyStateGuide
    - Test prerequisite-not-met message and navigation link
    - Test prerequisite-met call-to-action
    - Test fallback to default empty state when guide is off
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4**

- [x] 6. Implement NextStepPrompt component
  - [x] 6.1 Create `editor-ui/src/components/NextStepPrompt/NextStepPrompt.tsx`
    - Render a slide-in banner at the bottom of the content area
    - Show congratulatory message and navigation button to next step's tab
    - Auto-dismiss after 10 seconds or on user click
    - Show completion message with no navigation when all 6 steps are complete
    - Only show once per step completion (check `shownPrompts` in GuideStore)
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

  - [x] 6.2 Create `editor-ui/src/components/NextStepPrompt/NextStepPrompt.css`
    - Style the slide-in animation, banner layout, navigation button, dismiss button
    - _Requirements: 6.1_

  - [ ]* 6.3 Write unit tests for NextStepPrompt
    - Test rendering correct message per completed step
    - Test auto-dismiss after 10 seconds (use fake timers)
    - Test navigation button callback
    - Test completion message when all steps done
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

- [x] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Implement ContextualTooltip component
  - [x] 8.1 Create `editor-ui/src/components/ContextualTooltip/ContextualTooltip.tsx`
    - Render a (?) help icon next to wrapped children
    - On hover/click, show a popover with the OCSF term explanation from the tooltip content map
    - Include "Learn more" link to OCSF docs where applicable
    - Use viewport boundary detection to reposition if tooltip would overflow
    - Only render help icon when `guideVisible` is true
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 11.3_

  - [x] 8.2 Create `editor-ui/src/components/ContextualTooltip/ContextualTooltip.css`
    - Style the help icon, popover, "Learn more" link, and positioning logic
    - _Requirements: 7.1_

  - [ ]* 8.3 Write unit tests for ContextualTooltip
    - Test tooltip renders correct content per OCSF term
    - Test visibility toggled by guideVisible
    - Test hover/click triggers popover
    - **Property 9: Tooltip content correctness** — correct explanation per OCSFTerm
    - **Validates: Requirements 7.1, 7.2, 7.3**

- [x] 9. Integrate guide components into App.tsx and existing tab components
  - [x] 9.1 Wire GuidedProgressBar into `editor-ui/src/App.tsx`
    - Import `useGuideStore` and `GuidedProgressBar`
    - Add `useEffect` to refresh step statuses when `editorStore.model` or `indexStore` state changes
    - Render `GuidedProgressBar` between tab navigation and main content, conditionally on `guideVisible`
    - Wire `onStepClick` to `setActiveTab` and `onDismiss` to `setGuideVisible(false)`
    - Existing tab structure and order must remain unchanged
    - _Requirements: 2.1, 8.1, 8.2, 8.3, 10.1, 10.2, 10.3_

  - [x] 9.2 Add StepHint and EmptyStateGuide to existing tab components
    - Add `StepHint` at the top of each tab's content area (Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, ETL)
    - Add `EmptyStateGuide` to tabs that can have empty content (Schema, Entities, Metrics, Index)
    - Read `guideVisible`, `stepStatuses`, `dismissedHints` from `useGuideStore`
    - Ensure existing components are not modified or replaced — guide components are additive only
    - _Requirements: 4.2, 4.3, 4.4, 5.1, 8.1, 8.2_

  - [x] 9.3 Add NextStepPrompt trigger logic to App.tsx or tab components
    - Detect step completion transitions and render `NextStepPrompt` when appropriate
    - Call `markPromptShown` after displaying each prompt
    - Wire navigation button to `setActiveTab`
    - _Requirements: 6.1, 6.3, 6.6_

  - [x] 9.4 Add "Show Guide" toggle to the header menu area
    - Add a button/toggle in the header to re-enable the guide when it has been dismissed
    - Wire to `useGuideStore.setGuideVisible(true)`
    - _Requirements: 1.4_

  - [x] 9.5 Wire `markValidationRun` into the Validation tab
    - Call `useGuideStore.getState().markValidationRun()` when validation is executed in the ValidationPanel
    - _Requirements: 3.6_

  - [ ]* 9.6 Write integration tests for guide overlay behavior
    - Test guide visible by default on fresh load
    - Test dismiss hides all guide components
    - Test re-enable shows guide with current step statuses
    - Test step completion updates progress bar
    - **Property 8: Guide visibility independence** — model operations produce identical results regardless of guideVisible
    - **Validates: Requirements 1.1, 1.3, 8.1, 8.2, 8.3**

- [x] 10. Add ContextualTooltip annotations to existing OCSF terms in tab components
  - Wrap key OCSF terms (event_class, observable, dimension, semantic_entity, metric, field_mapping, etc.) with `ContextualTooltip` in Schema, Entities, Metrics, and Index tab components
  - Only add tooltips to the most prominent occurrences — avoid over-annotating
  - _Requirements: 7.1, 7.2_

- [x] 11. Handle model sync and graceful degradation
  - [x] 11.1 Ensure `refreshStepStatuses` is called on import, undo, and redo operations
    - Subscribe to EditorStore and IndexStore changes in `useGuideStore` or via `useEffect` in App.tsx
    - Verify debounced refresh triggers within 100ms of model changes
    - _Requirements: 10.1, 10.2, 10.3_

  - [x] 11.2 Add error boundaries around guide components
    - Wrap guide overlay components so that a guide error never prevents Tab_UI from rendering
    - Log errors to console for debugging
    - _Requirements: 11.4_

  - [ ]* 11.3 Write property test for graceful degradation (fast-check)
    - **Property 10: Graceful degradation on malformed state** — `deriveStepStatuses` defaults affected steps to pending without throwing for any malformed EditorStore model
    - **Validates: Requirements 10.4, 11.2, 11.4**

  - [ ]* 11.4 Write property test for model sync re-derivation (fast-check)
    - **Property 11: Model sync triggers re-derivation** — after any EditorStore/IndexStore change, stepStatuses reflects updated model state
    - **Validates: Requirements 10.1, 10.2, 10.3**

- [x] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use fast-check and validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- All guide components are additive overlays — no existing components are modified or replaced
- The implementation language is TypeScript (React + Zustand + Vite)
