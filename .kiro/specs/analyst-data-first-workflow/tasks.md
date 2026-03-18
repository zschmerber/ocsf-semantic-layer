# Implementation Plan: Analyst Data-First Workflow

## Overview

Transform the OCSF Semantic Model Editor from a schema-first to a data-first workflow. The analyst pastes a transformed OCSF JSON event (and optionally a mapping artifact) as step 1. The system extracts `class_uid`, flags observables, infers types, and seeds entity creation, metric suggestions, and validation from actual data. The guided wizard is restructured from 6 steps to 4. TypeScript for frontend (React/Zustand), Rust for backend (ocsf-editor).

## Tasks

- [x] 1. Define core TypeScript types and interfaces
  - [x] 1.1 Create `editor-ui/src/types/referenceEvent.ts` with `ParsedField`, `ReferenceEvent`, `ObservableType`, `ObservableFlag`, `ClassDetectionResult`, `TypeMismatch`, `MappingEntry`, `InterpretedMapping`, `VerificationStatus`, `MappingCoverage`, `SuggestedAttribute`, `SuggestedMetric`, `SuggestedModel`, `AttributeTier`, `ReferenceEventSaveState` interfaces
    - All types from the design "Data Models" section
    - Export all types for use across modules
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 7.1, 9.1, 15.2, 16.1, 17.1_

- [x] 2. Implement pure utility modules
  - [x] 2.1 Create `editor-ui/src/utils/eventFieldExtractor.ts` — `extractFields(json: string): { fields: ParsedField[], error: string | null, warning: string | null }`
    - Parse JSON, recursively flatten nested objects to dot-notation paths
    - Infer types from JS values (string, integer, float, boolean, array, null)
    - Return structured error for invalid JSON, warning for valid JSON without OCSF fields
    - Never throw — always return result or error
    - _Requirements: 1.2, 1.4, 14.1, 14.5, 14.6_

  - [ ]* 2.2 Write property tests for eventFieldExtractor (fast-check)
    - **Property 1: JSON field extraction preserves structure**
    - **Property 3: Invalid JSON rejection**
    - **Property 25: Nested JSON flattened to dot-notation**
    - **Property 27: Robustness — no unhandled exceptions**
    - **Property 28: Valid JSON without OCSF fields warns but proceeds**
    - **Validates: Requirements 1.2, 1.4, 14.1, 14.5, 14.6**

  - [x] 2.3 Create `editor-ui/src/utils/classUidDetector.ts` — `detectClassUid(parsedEvent, schemaTree): ClassDetectionResult`
    - Extract `class_uid` and `category_uid` from parsed event
    - Resolve against schema tree to get class/category names
    - Return Definitive confidence when class_uid present and valid, Low when absent
    - Handle non-integer class_uid, unrecognized values — never throw
    - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 14.2_

  - [ ]* 2.4 Write property tests for classUidDetector (fast-check)
    - **Property 5: class_uid resolution correctness**
    - **Property 6: category_uid resolution correctness**
    - **Property 7: Detection confidence invariant**
    - **Property 8: Unrecognized class_uid produces null with warning**
    - **Property 26: Non-integer class_uid fallback**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.5, 2.6, 14.2**

  - [x] 2.5 Create `editor-ui/src/utils/observableFlagger.ts` — `flagObservables(fields: ParsedField[]): ObservableFlag[]`
    - Field name heuristics: check for "ip", "host", "hash", "url", "email", "mac", "process"
    - Value pattern matching: IPv4/IPv6 regex, hostname patterns, hash length patterns, URL/email/MAC patterns
    - Confidence: High when both name + value match, Medium when value only
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 2.6 Write property tests for observableFlagger (fast-check)
    - **Property 9: Observable detection correctness**
    - **Property 10: Observable confidence levels**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

  - [x] 2.7 Create `editor-ui/src/utils/schemaTypeComparator.ts` — `compareTypes(fields, classUid, schemaTree): TypeMismatch[]`
    - Compare each field's inferred type against schema-defined type for the detected class
    - Return mismatch entries with both observed and schema types
    - _Requirements: 4.1, 4.2_

  - [ ]* 2.8 Write property tests for schemaTypeComparator (fast-check)
    - **Property 11: Type comparison detects mismatches**
    - **Validates: Requirements 4.1, 4.2**

  - [x] 2.9 Create `editor-ui/src/utils/verificationComputer.ts` — `computeVerification(entries: MappingEntry[], fields: ParsedField[]): Map<string, VerificationStatus>`
    - Verified: ocsf_field matches a field path in the event
    - Unverified: ocsf_field not found
    - Conflict: ocsf_field found but type contradicts mapping
    - _Requirements: 16.1, 16.2, 16.3, 16.4_

  - [ ]* 2.10 Write property tests for verificationComputer (fast-check)
    - **Property 29: Verification status correctness**
    - **Validates: Requirements 16.1, 16.2, 16.3, 16.4**

  - [x] 2.11 Create `editor-ui/src/utils/coverageComputer.ts` — `computeCoverage(fields: ParsedField[], entries: MappingEntry[]): MappingCoverage`
    - Compute percent_event_fields_mapped, percent_mapping_fields_unobserved, percent_event_fields_unmapped
    - List unmapped field paths and unobserved mapping fields
    - _Requirements: 17.1, 17.4, 17.5_

  - [ ]* 2.12 Write property tests for coverageComputer (fast-check)
    - **Property 30: Mapping coverage metrics correctness**
    - **Validates: Requirements 17.1, 17.4, 17.5**

  - [x] 2.13 Create `editor-ui/src/utils/suggestedModelGenerator.ts` — `generateSuggestedModel(referenceEvent, schemaTree, mapping): SuggestedModel`
    - Generate entity name from event class name
    - Classify attributes into Core/Extended/Potential tiers
    - Suggest metrics: count/sum/avg for numeric fields, time-granularity for timestamps, count_distinct for observable strings, ratio for status fields, rate for timestamps, top-N for high-cardinality strings
    - Include plain-language descriptions for all metrics
    - Include lineage info when mapping available
    - _Requirements: 9.1, 9.5, 9.8, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8_

  - [ ]* 2.14 Write property tests for suggestedModelGenerator (fast-check)
    - **Property 16: Suggested model tier classification**
    - **Property 17: Numeric fields produce aggregation metric suggestions**
    - **Property 18: Timestamp fields produce time-based metric suggestions**
    - **Property 19: Observable string fields produce cardinality metrics**
    - **Property 20: Suggested metrics have plain-language descriptions**
    - **Validates: Requirements 9.1, 9.5, 10.2, 10.3, 10.4, 10.6, 10.8**

- [x] 3. Checkpoint — Ensure all utility tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement ReferenceEventStore
  - [x] 4.1 Create `editor-ui/src/store/referenceEventStore.ts` — Zustand store with `ReferenceEventState`
    - Store raw JSON, parsed fields, class detection, observable flags, type mismatches
    - Store mapping raw text, interpreted mapping, loading/error state
    - Store computed verification statuses and mapping coverage
    - `setReferenceEvent(json)`: parse JSON via eventFieldExtractor, run classUidDetector, observableFlagger, schemaTypeComparator; store all results
    - `clearReferenceEvent()`: reset all state including mapping
    - `setInterpretedMapping(mapping)`: store mapping, compute verification and coverage
    - `clearMapping()`: remove mapping/verification/coverage, preserve reference event
    - `setMappingLoading(loading)`, `setMappingError(error)`
    - `computeVerification()`, `computeCoverage()`: recompute derived state
    - localStorage persistence under key `ocsf-reference-event` — persist rawJson + mappingRawText + interpretedMapping, re-derive on load
    - Graceful fallback on corrupted localStorage
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 15.3, 15.10, 16.1_

  - [ ]* 4.2 Write property tests for ReferenceEventStore (fast-check)
    - **Property 4: Clear resets all derived state**
    - **Property 14: localStorage round-trip for Reference_Event**
    - **Property 15: Corrupted localStorage fallback**
    - **Property 31: Clearing mapping preserves Reference_Event**
    - **Property 32: Mapping without event defers LLM call**
    - **Validates: Requirements 1.7, 7.3, 7.4, 7.5, 15.10, 15.11**

- [x] 5. Update guide system for 4-step workflow
  - [x] 5.1 Update `editor-ui/src/types/guide.ts` — change `GuideStepId` from `1|2|3|4|5|6` to `1|2|3|4`, update `GUIDE_STEPS` to 4 steps (Paste Event, Define Entity, Add Metrics, Validate Model), update `STEP_TO_TAB` and `STEP_PREREQUISITES`, update `HINT_CONTENT_PENDING` and `HINT_CONTENT_COMPLETE` for new step labels
    - Step 1: Paste Event (+ Optional Mapping) → schema tab
    - Step 2: Define Entity & Map Attributes → entities tab
    - Step 3: Add Metrics → metrics tab
    - Step 4: Validate Model → validation tab
    - Prerequisites: 1→[], 2→[1], 3→[2], 4→[2,3]
    - No step depends on Index tab
    - _Requirements: 5.1, 5.2, 5.3, 5.6, 5.7, 6.1, 6.2, 6.3_

  - [x] 5.2 Update `editor-ui/src/store/guideLogic.ts` — update `deriveStepStatuses` to use ReferenceEventStore
    - Step 1 complete: referenceEvent loaded with class detected OR manual class selected in EditorStore
    - Step 2 complete: at least one entity with attributes
    - Step 3 complete: at least one metric defined
    - Step 4 complete: validation has been run
    - _Requirements: 5.4, 5.5, 12.2, 12.3_

  - [x] 5.3 Update `editor-ui/src/store/guideStore.ts` — change `DEFAULT_STEP_STATUSES` to 4 steps, update `refreshStepStatuses` to pass referenceEvent, update `TAB_TO_STEP` mapping
    - _Requirements: 5.1, 5.2_

  - [ ]* 5.4 Write property tests for guide step derivation (fast-check)
    - **Property 12: Step 1 completion derivation**
    - **Property 13: No core step depends on Index tab**
    - **Validates: Requirements 5.4, 5.5, 6.3**

  - [x] 5.5 Update `editor-ui/src/App.tsx` — update guide step iteration from 6 to 4, update `prevStepStatusesRef` loop, update `TAB_TO_STEP` in `refreshStepStatuses` call
    - _Requirements: 5.1, 5.2_

- [x] 6. Checkpoint — Ensure guide system and store tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement EventPastePanel component
  - [x] 7.1 Create `editor-ui/src/components/EventPastePanel/EventPastePanel.tsx`
    - Primary textarea for OCSF JSON paste, file upload button (.json)
    - Secondary collapsible textarea "Mapping (optional — any format)" — disabled with config message when no LLM configured
    - "Skip — browse schema manually" link
    - On paste/upload: call `referenceEventStore.setReferenceEvent(json)`
    - On mapping paste (when event loaded + LLM configured): call backend `/api/llm/interpret-mapping`
    - Summary panel: field count, detected class with Confidence_Indicator, observable count, type mismatch count
    - When mapping interpreted: show mapping count, source system, confidence distribution, preview table
    - "Working from a single sample event" persistent banner
    - Clear button to reset all state
    - Loading indicator during LLM call
    - Error display with retry for LLM failures
    - Mapping deferred message when no event loaded
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.3, 2.4, 2.5, 2.6, 4.3, 12.1, 13.1, 13.3, 14.1, 14.4, 14.5, 15.1, 15.3, 15.4, 15.5, 15.6, 15.7, 15.9, 15.11, 15.13, 16.5, 16.7, 17.2_

  - [x] 7.2 Create `editor-ui/src/components/EventPastePanel/index.ts` barrel export and register in `editor-ui/src/components/index.ts`
    - _Requirements: 1.1_

  - [x] 7.3 Wire EventPastePanel into Schema tab in `editor-ui/src/App.tsx`
    - Render EventPastePanel at top of Schema tab when no Reference_Event loaded (or always with summary when loaded)
    - On class detection: auto-set EditorStore selected class UID, auto-expand schema tree node
    - _Requirements: 1.1, 2.7, 2.8, 12.1_

- [x] 8. Implement backend mapping interpretation endpoint
  - [x] 8.1 Create `ocsf-editor/src/api/mapping.rs` with Rust types: `InterpretMappingRequest`, `MappingEntry`, `MappingConfidence`, `SourceSystem`, `InterpretMappingResponse`
    - Derive Serialize, Deserialize for all types
    - _Requirements: 15.1, 15.2_

  - [x] 8.2 Implement `POST /api/llm/interpret-mapping` handler in `ocsf-editor/src/api/mapping.rs`
    - Accept `InterpretMappingRequest` (event_json + mapping_text)
    - Build LLM prompt requesting field-to-field mappings, transformation logic, source system identification, issues
    - Send to configured LLM provider via existing `LlmService` proxy
    - Parse LLM response into `InterpretMappingResponse`
    - Return 400 if no LLM configured, 502 on LLM errors
    - _Requirements: 15.1, 15.2, 15.8, 15.9, 15.12_

  - [x] 8.3 Register the new route in `ocsf-editor/src/lib.rs` — add `.route("/llm/interpret-mapping", post(api::interpret_mapping))` to `create_api_router`
    - Also add `pub mod mapping;` to `ocsf-editor/src/api/mod.rs`
    - _Requirements: 15.12_

  - [ ]* 8.4 Write property tests for Rust serde round-trips (proptest)
    - **Property 33: LLM response normalization** — test `InterpretMappingResponse` serde round-trip
    - **Validates: Requirements 15.2**

- [x] 9. Checkpoint — Ensure backend compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Implement frontend API client for mapping endpoint
  - [x] 10.1 Add `interpretMapping` function to `editor-ui/src/api/client.ts`
    - `POST /llm/interpret-mapping` with `{ event_json, mapping_text }`
    - Return typed `InterpretMappingResponse`
    - Use existing `post` helper with appropriate timeout (60s for LLM calls)
    - _Requirements: 15.1, 15.12_

- [x] 11. Enhance EntityEditor with data-seeded attributes and suggested model
  - [x] 11.1 Update `editor-ui/src/components/EntityEditor/EntityEditor.tsx`
    - When Reference_Event loaded: show only attributes present in event (with sample values, observable indicators, type mismatch warnings)
    - Toggle between "Sample fields only" and "All schema fields"
    - When no Reference_Event: show full schema attribute list (current behavior)
    - "Based on 1 sample event" indicator
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 13.2_

  - [x] 11.2 Add SuggestedModelPanel to EntityEditor
    - Show after Step 1 complete with Reference_Event loaded
    - Display tiered attributes: Core (pre-selected), Extended (shown, not selected), Potential (collapsed)
    - Allow promote/demote between tiers
    - Show lineage column when mapping available (raw_field, transformation)
    - Accept all / modify / discard actions
    - On accept: create entity + metrics in EditorStore
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9_

- [x] 12. Enhance MetricEditor with suggested metrics
  - [x] 12.1 Update `editor-ui/src/components/MetricEditor/MetricEditor.tsx`
    - When Reference_Event loaded: show "Suggested Metrics" section with candidates from suggestedModelGenerator
    - Display plain-language descriptions, confidence indicators, aggregation type
    - "Add" button per suggestion → create metric in EditorStore
    - When no Reference_Event: show current manual interface
    - "Based on 1 sample event" indicator
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 10.9, 10.10, 13.2_

- [x] 13. Enhance ValidationPanel with sample-event validation and dry-run
  - [x] 13.1 Update `editor-ui/src/components/ValidationPanel/ValidationPanel.tsx` (or equivalent)
    - When Reference_Event loaded: cross-reference model against event fields
    - Warn on attributes not present in event, error on non-numeric metric fields
    - Separate sections: schema-based, sample-event-based, dry-run simulation
    - Dry-run: simulate each metric against Reference_Event, show computed value or null/missing with reason
    - When no Reference_Event: validate against schema only (current behavior)
    - "Simulated against 1 sample event" label
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 13.2_

  - [ ]* 13.2 Write property tests for validation logic (fast-check)
    - **Property 21: Validation cross-references model against Reference_Event**
    - **Property 22: Non-numeric metric field validation error**
    - **Property 23: Schema-only validation without Reference_Event**
    - **Property 24: Dry-run metric simulation**
    - **Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.6, 11.7**

- [x] 14. Checkpoint — Ensure all component tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 15. Index tab lineage pre-seeding and mapping display
  - [x] 15.1 Update Index tab lineage view to pre-seed from LLM-interpreted mapping when available
    - Display banner: "Lineage auto-populated from LLM-interpreted mapping — review and edit"
    - Show raw_field → ocsf_field with transformation and explanation
    - Display Mapping_Confidence and Verification_Status badges per entry
    - Filter/sort by confidence and verification status
    - Coverage summary panel with progress bars
    - _Requirements: 6.5, 6.6, 15.13, 15.14, 16.5, 16.6, 17.2, 17.3_

  - [ ]* 15.2 Write property tests for filter/sort operations (fast-check)
    - **Property 34: Filter mapping entries by status**
    - **Validates: Requirements 15.14, 16.6**

- [x] 16. Integration and wiring
  - [x] 16.1 Wire dual workflow: EventPastePanel as default on Schema tab, "Skip" link for manual flow
    - Seamless transition between paths — no error states when skipping
    - Late-loaded Reference_Event updates all downstream tabs
    - Identical step labels regardless of Reference_Event presence
    - _Requirements: 12.1, 12.2, 12.3, 12.4_

  - [x] 16.2 Wire guide overlay updates: StepHint content for all 4 steps, EmptyStateGuide for Schema tab, NextStepPrompt for Step 4 completion with optional Index mention
    - Step 1 hint: explain paste is optional, mapping for lineage, class_uid auto-detection
    - Step 1 hint: show detected class as confirmation banner with override option
    - Index tab hint: explain optional enrichment
    - Step 4 completion: mention Index as advanced next step
    - _Requirements: 5.6, 5.7, 6.2, 6.4_

  - [x] 16.3 Wire "Based on 1 sample event" indicator and single-event MVP messaging across all tabs
    - Persistent banner on EventPastePanel
    - Subtle indicator on Entities, Metrics, Validation tabs
    - Tooltip explaining multi-event support planned
    - _Requirements: 13.1, 13.2, 13.3_

  - [x] 16.4 Wire schema tree auto-expand on class detection — when classUidDetector resolves, auto-expand and highlight the detected class node in the schema tree, set EditorStore selected class UID
    - Queue detection if schema tree not loaded yet
    - _Requirements: 2.7, 2.8, 14.3_

  - [ ]* 16.5 Write property test for file upload equivalence (fast-check)
    - **Property 2: File upload and paste equivalence**
    - **Validates: Requirements 1.3**

- [x] 17. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (34 properties)
- Frontend: TypeScript with React/Zustand, fast-check for property tests
- Backend: Rust with axum, proptest for property tests
- The existing LogImport, guide overlay, EditorStore, IndexStore, and LLM infrastructure provide the foundation
