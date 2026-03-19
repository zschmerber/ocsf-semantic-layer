# Requirements Document

## Introduction

This document defines the requirements for the Analyst Data-First Workflow feature, which redesigns the OCSF Semantic Model Editor's guided workflow to be data-first instead of schema-first. The core change: analysts paste a transformed OCSF event (JSON) as step 1, and optionally provide the mapping artifact used to transform the raw log into OCSF — in any format or language. The system auto-detects the event class from `class_uid` (which is always present in a properly transformed OCSF event), extracts fields with types and sample values, flags observables, and seeds the entire downstream modeling workflow. When a mapping artifact is also provided, it is sent to the configured LLM (Anthropic or OpenAI, via the Rust backend proxy) alongside the Reference_Event for interpretation; the LLM extracts field-to-field mappings, transformation logic, and source system metadata from any format (Logstash configs, Cribl packs, Splunk props/transforms, Python scripts, dbt SQL, Sigma rules, etc.), and the structured result auto-populates field lineage. The existing manual schema-browsing flow is preserved as a fallback. The guided wizard steps are reduced from 6 to 4, with event class confirmation folded into the paste step (since `class_uid` makes it automatic) and the Index tab reframed as optional advanced functionality. OCSF event parsing, class_uid detection, and observable flagging happen client-side; mapping interpretation requires an LLM provider configured via the settings modal. The existing LogImport component, guide system (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt), GuideStore, EditorStore, IndexStore, and LLM configuration infrastructure (`setLLMConfig`/`getLLMConfig`, backend proxy, research feature pattern) provide the foundation for this feature.

## Glossary

- **Event_Paste_Panel**: The UI panel (promoted from or based on the existing LogImport component) that accepts pasted OCSF JSON events or file uploads, and an optional mapping definition, as the first step of the workflow
- **Mapping_Definition**: An optional mapping artifact in any format or language (Logstash configs, Cribl packs, Splunk props/transforms, Python scripts, dbt SQL models, custom YAML, Sigma rules, JSON mapping documents, or any other format) describing the field-to-field mappings and transformations used to convert a raw log into the OCSF event; when provided alongside a Reference_Event, it is sent to the LLM_Mapping_Interpreter for structured understanding and enables auto-populated field lineage
- **LLM_Mapping_Interpreter**: The LLM-powered logic that reads a mapping artifact in any format or language alongside the Reference_Event, sends both to the configured LLM provider (Anthropic or OpenAI, configured via the settings modal and `setLLMConfig`/`getLLMConfig` API), and receives a structured understanding of field-to-field mappings (raw_field → ocsf_field) with Mapping_Confidence per entry, transformation logic, source system/vendor metadata; the LLM call is proxied through the Rust backend (`ocsf-editor`) to avoid exposing API keys client-side
- **Reference_Event**: The parsed OCSF event stored in the ReferenceEventStore, carrying extracted fields, types, sample values, detected class_uid, category_uid, and observable flags through all subsequent workflow steps
- **ReferenceEventStore**: A new Zustand store that holds the Reference_Event data, confidence scores, detected metadata, Verification_Status per mapping entry, and computed Mapping_Coverage metrics, accessible to Entity, Metric, and Validation tabs
- **Class_UID_Detector**: The client-side logic that extracts `class_uid` and `category_uid` from a parsed OCSF JSON event and resolves them to an event class name using the loaded schema tree
- **Confidence_Indicator**: A visual badge (Definitive, High, Medium, Low) displayed next to auto-detected values (event class, field types, metric suggestions) to communicate detection certainty to the analyst; "Definitive" is used when the value is known from the event data (e.g., `class_uid` present)
- **Observable_Flagger**: The logic that identifies observable fields (IP addresses, hostnames, file hashes, URLs, email addresses) in the parsed event based on OCSF observable type conventions and value pattern heuristics
- **Suggested_Model**: An auto-generated draft entity with suggested name, attribute set, observable flags, and metric candidates that the analyst can accept, modify, or discard after event class confirmation
- **Schema_Type_Comparator**: The logic that compares observed field types from the pasted event against the OCSF schema-defined types and flags mismatches
- **Guide_Layer**: The collective set of overlay components (GuidedProgressBar, StepHint, EmptyStateGuide, NextStepPrompt, ContextualTooltip) from the guided-wizard-workflow spec
- **EditorStore**: The existing Zustand store (`useEditorStore`) holding the semantic model (entities, metrics, selected class)
- **IndexStore**: The existing Zustand store (`useIndexStore`) holding log import state, field mappings, and table registration
- **GuideStore**: The existing Zustand store (`useGuideStore`) managing guide visibility, step statuses, and persistence
- **Tab_UI**: The existing 8-tab editor interface (Schema, Entities, Metrics, Validation, Index, Architecture, Catalog, ETL)
- **Analyst**: The primary user — a security analyst or data engineer who has transformed OCSF events and wants to build a semantic model
- **Mapping_Confidence**: The confidence level assigned by the LLM_Mapping_Interpreter to each mapping entry (High, Medium, Low) based on how explicitly the mapping was defined in the artifact — "High" means explicitly defined, "Medium" means inferred from pattern, "Low" means LLM-generated guess
- **Verification_Status**: The cross-reference status of each LLM-interpreted mapping entry against the Reference_Event (Verified, Unverified, Conflict) — "Verified" means the ocsf_field matches a field in the Reference_Event, "Unverified" means the ocsf_field is not present, "Conflict" means the mapping contradicts observed data
- **Mapping_Coverage**: A set of computed metrics showing the completeness of the mapping relative to the Reference_Event fields, including percent_event_fields_mapped, percent_mapping_fields_unobserved, and percent_event_fields_unmapped
- **Dry_Run_Validation**: A simulation mode that executes the semantic model's metrics against the Reference_Event to show what each metric would produce, displaying computed values or null/missing indicators per metric

## Requirements

### Requirement 1: OCSF Event Paste and Parsing

**User Story:** As an Analyst, I want to paste a transformed OCSF JSON event into the editor as the first step, and optionally provide the mapping used to transform the raw log, so that the tool can auto-detect my event class, seed the modeling workflow from my actual data, and auto-populate field lineage when a mapping is available.

#### Acceptance Criteria

1. WHEN the Analyst navigates to the Schema tab with no Reference_Event loaded, THE Event_Paste_Panel SHALL render prominently at the top of the Schema tab with a primary textarea for pasting OCSF JSON, a file upload button, and a secondary collapsible textarea labeled "Mapping (optional — any format)" for pasting a mapping artifact in any format or language (Logstash configs, Cribl packs, Splunk props/transforms, Python scripts, dbt SQL, YAML, Sigma rules, etc.) which is interpreted via the LLM_Mapping_Interpreter
2. WHEN the Analyst pastes valid OCSF JSON into the Event_Paste_Panel, THE Event_Paste_Panel SHALL parse the JSON, extract all fields with their paths, inferred types, and sample values, and store the result as a Reference_Event in the ReferenceEventStore
3. WHEN the Analyst uploads a JSON file via the Event_Paste_Panel, THE Event_Paste_Panel SHALL read the file contents and process them identically to pasted input
4. IF the pasted text is not valid JSON, THEN THE Event_Paste_Panel SHALL display a descriptive parse error message and retain the pasted text for correction
5. WHEN a Reference_Event is successfully parsed, THE Event_Paste_Panel SHALL display a summary showing the number of extracted fields, the detected format, a preview table of field paths, types, and sample values, and (if a mapping was interpreted by the LLM_Mapping_Interpreter) the number of mapped field pairs and a summary of the source system identified by the LLM
6. THE Event_Paste_Panel SHALL display a clear label indicating that this step is optional and the Analyst can skip to manual schema browsing
7. WHEN the Analyst clicks the clear button on the Event_Paste_Panel, THE ReferenceEventStore SHALL remove the Reference_Event, any LLM-interpreted Mapping_Definition, and reset all derived state (detected class, confidence scores, observable flags, lineage data)

### Requirement 2: Event Class Auto-Detection

**User Story:** As an Analyst, I want the tool to auto-detect the OCSF event class from my pasted event, so that I do not have to manually browse the schema tree to find the right class.

#### Acceptance Criteria

1. WHEN a Reference_Event is parsed and the JSON contains a `class_uid` field, THE Class_UID_Detector SHALL resolve the `class_uid` value to the corresponding event class name using the loaded schema tree and store the result in the ReferenceEventStore as a definitively known event class (not a heuristic detection)
2. WHEN a Reference_Event is parsed and the JSON contains a `category_uid` field, THE Class_UID_Detector SHALL resolve the `category_uid` to the corresponding category name and store the result in the ReferenceEventStore
3. WHEN the Class_UID_Detector successfully resolves a `class_uid`, THE Confidence_Indicator SHALL display "Definitive" confidence next to the detected event class, indicating the class is known from the event data rather than inferred
4. WHEN the Class_UID_Detector resolves a `class_uid`, THE Guide_Layer SHALL present the detected event class as a visual confirmation step rather than a decision step, since a properly transformed OCSF event always contains `class_uid`
5. IF the `class_uid` field is missing from the parsed JSON, THEN THE Class_UID_Detector SHALL set the detected event class to null, THE Confidence_Indicator SHALL display "Low" confidence, and THE Event_Paste_Panel SHALL show a message instructing the Analyst to select the event class manually from the schema tree
6. IF the `class_uid` value does not match any event class in the loaded schema tree, THEN THE Class_UID_Detector SHALL set the detected event class to null and display a warning with the unrecognized `class_uid` value
7. WHEN the Class_UID_Detector resolves an event class, THE Schema tab SHALL auto-expand and highlight the detected event class node in the schema tree
8. WHEN the Class_UID_Detector resolves an event class, THE EditorStore SHALL set the selected class UID to the detected value, equivalent to the Analyst manually selecting the class in the schema tree

### Requirement 3: Observable Field Detection

**User Story:** As an Analyst, I want the tool to automatically identify observable fields (IPs, hostnames, hashes) in my pasted event, so that I can see which fields are relevant for threat detection without manual inspection.

#### Acceptance Criteria

1. WHEN a Reference_Event is parsed, THE Observable_Flagger SHALL scan all extracted fields and flag fields that match OCSF observable type conventions (IP addresses, hostnames, file hashes, URLs, email addresses, MAC addresses, process names)
2. WHEN the Observable_Flagger identifies an observable field, THE ReferenceEventStore SHALL store the observable type classification and the matched field path
3. THE Observable_Flagger SHALL use both field path heuristics (field names containing "ip", "host", "hash", "url", "email", "mac") and value pattern matching (IPv4/IPv6 regex, hostname patterns, hash length patterns) to classify observables
4. WHEN the Observable_Flagger classifies a field using value pattern matching only (no field name match), THE Confidence_Indicator for that observable SHALL display "Medium" confidence
5. WHEN the Observable_Flagger classifies a field using both field name and value pattern matching, THE Confidence_Indicator for that observable SHALL display "High" confidence

### Requirement 4: Schema vs Observed Type Comparison

**User Story:** As an Analyst, I want to see when a field's observed type in my event differs from the OCSF schema-defined type, so that I can identify data quality issues or transformation errors.

#### Acceptance Criteria

1. WHEN a Reference_Event is loaded and an event class is detected, THE Schema_Type_Comparator SHALL compare each extracted field's inferred type against the corresponding OCSF schema attribute type
2. WHEN a field's observed type differs from the schema-defined type, THE Schema_Type_Comparator SHALL flag the mismatch and store both the observed type and the schema type in the ReferenceEventStore
3. WHEN type mismatches are detected, THE Event_Paste_Panel summary SHALL display a count of mismatched fields with a visual warning indicator
4. WHEN the Analyst views the attribute list in the Entities tab, THE attribute row for a mismatched field SHALL display both the observed type and the schema type side by side with a mismatch icon

### Requirement 5: Guided Workflow Step Restructuring

**User Story:** As an Analyst, I want the guided workflow to reflect the new data-first flow with 4 core steps, so that the onboarding guidance matches the streamlined workflow where event class detection is automatic.

#### Acceptance Criteria

1. THE Guide_Layer SHALL define 4 core workflow steps: Step 1 "Paste Event (+ Optional Mapping)", Step 2 "Define Entity & Map Attributes", Step 3 "Add Metrics", Step 4 "Validate Model"
2. THE GuidedProgressBar SHALL render 4 step indicators mapped as follows: Step 1 to Schema tab, Step 2 to Entities tab, Step 3 to Metrics tab, Step 4 to Validation tab
3. THE STEP_PREREQUISITES chain SHALL be: Step 1 has no prerequisites; Step 2 requires Step 1 or manual class selection; Step 3 requires Step 2; Step 4 requires Steps 2 and 3
4. WHEN the Analyst skips Step 1 (no Reference_Event loaded) and manually selects an event class, THE Step_Derivation SHALL mark Step 1 as complete
5. WHEN a Reference_Event is loaded and the event class is auto-detected from `class_uid`, THE Step_Derivation SHALL mark Step 1 as complete, including the visual confirmation of the detected event class within the same step
6. THE StepHint for Step 1 SHALL display content explaining that pasting an OCSF event is optional but recommended, that an optional mapping can be provided for field lineage, and that the event class is auto-detected from `class_uid`
7. THE StepHint for Step 1 SHALL display the detected event class as a visual confirmation banner within the step, allowing the Analyst to override the class selection if needed

### Requirement 6: Index Tab Reframing

**User Story:** As an Analyst, I want the Index tab to be optional advanced functionality with auto-populated lineage when a mapping is provided, so that I can complete a semantic model without being required to register physical tables.

#### Acceptance Criteria

1. THE Guide_Layer SHALL exclude the Index tab from the core 4-step workflow
2. THE StepHint for the Index tab SHALL display content explaining that the Index tab provides optional enrichment for data lineage, field lineage, detection coverage (MITRE ATT&CK), and physical table registration
3. THE STEP_PREREQUISITES chain SHALL have no dependency on Index tab completion for any core workflow step
4. WHEN all 4 core steps are complete, THE NextStepPrompt for Step 4 SHALL display a completion message and optionally mention the Index tab as an advanced next step
5. WHEN a Mapping_Definition was provided in Step 1 and interpreted by the LLM_Mapping_Interpreter, THE Index tab lineage view SHALL be pre-seeded with the LLM-extracted field mappings (raw_field → ocsf_field with transformation and plain-language explanation) so the Analyst can review and edit lineage rather than building it from scratch
6. WHEN the Index tab lineage view is pre-seeded from an LLM-interpreted Mapping_Definition, THE Index tab SHALL display a banner indicating that lineage data was auto-populated from the LLM-interpreted mapping and can be edited

### Requirement 7: Reference Event Store

**User Story:** As an Analyst, I want the parsed event data to be accessible across all editor tabs, so that entity creation, metric definition, and validation can leverage my actual event data.

#### Acceptance Criteria

1. THE ReferenceEventStore SHALL store the following data: raw JSON input, parsed fields (path, type, sample value), detected class_uid, detected category_uid, resolved event class name, observable flags per field, type mismatch flags per field, confidence scores, and optionally the LLM-interpreted Mapping_Definition (raw_field, ocsf_field, transformation per entry, mapping_confidence per entry, verification_status per entry, source system metadata, and any LLM-provided context such as source log type identification and transformation explanations), and computed Mapping_Coverage metrics when both a Reference_Event and Mapping_Definition are available
2. WHEN the ReferenceEventStore state changes, THE EditorStore, Metrics tab, and Validation tab SHALL have access to the Reference_Event data via the ReferenceEventStore
3. WHEN the Analyst clears the Reference_Event, THE ReferenceEventStore SHALL reset all stored data including any LLM-interpreted Mapping_Definition and THE Event_Paste_Panel SHALL return to the empty input state
4. THE ReferenceEventStore SHALL persist the Reference_Event to localStorage so that the parsed event survives page reloads
5. IF localStorage read returns corrupted or incompatible Reference_Event data, THEN THE ReferenceEventStore SHALL fall back to empty state and log a console warning
6. THE ReferenceEventStore SHALL display a clear "This is one sample event" indicator wherever Reference_Event data is shown, to communicate that the data represents a single sample and not the full data population

### Requirement 8: Data-Seeded Entity Creation

**User Story:** As an Analyst, I want the entity creation form to be pre-populated with fields from my pasted event, so that I can build a semantic entity from my actual data instead of browsing the full schema attribute list.

#### Acceptance Criteria

1. WHEN a Reference_Event is loaded and the Analyst navigates to the Entities tab, THE EntityEditor SHALL display only the attributes present in the Reference_Event instead of the full schema attribute list
2. WHEN displaying Reference_Event-seeded attributes, THE EntityEditor SHALL show the sample value from the pasted event next to each attribute
3. WHEN displaying Reference_Event-seeded attributes, THE EntityEditor SHALL highlight attributes flagged as observables by the Observable_Flagger with a visual observable indicator
4. WHEN no Reference_Event is loaded, THE EntityEditor SHALL display the full schema attribute list as the current behavior
5. THE EntityEditor SHALL provide a toggle to switch between "Sample fields only" and "All schema fields" views when a Reference_Event is loaded
6. WHEN a type mismatch exists for an attribute, THE EntityEditor SHALL display both the observed type and the schema type with a mismatch warning icon

### Requirement 9: Suggested Model Generation

**User Story:** As an Analyst, I want the tool to auto-generate a suggested entity with attributes, observables, and metrics after event class confirmation, so that I have a starting point to accept, modify, or discard.

#### Acceptance Criteria

1. WHEN the event class is confirmed (Step 1 complete with class detected) and a Reference_Event is loaded, THE Suggested_Model generator SHALL produce a draft entity with a suggested name derived from the event class name, all populated attributes from the Reference_Event, observable flags, and candidate metrics for numeric fields
2. THE Suggested_Model SHALL present the draft entity in a review panel where the Analyst can accept all suggestions, modify individual attributes or metrics, or discard the entire suggestion
3. WHEN the Analyst accepts the Suggested_Model (fully or partially), THE EditorStore SHALL create the entity and metrics from the accepted suggestions
4. WHEN the Analyst discards the Suggested_Model, THE EntityEditor SHALL return to the empty entity creation state with Reference_Event-seeded attributes available
5. THE Suggested_Model SHALL organize suggested attributes into three tiers: "Core (High Confidence)" for attributes present in the Reference_Event that match the OCSF schema and are confirmed by the mapping if available, "Extended (Medium Confidence)" for attributes present in the Reference_Event but optional in the schema with weak or no mapping support, and "Potential (Low Confidence)" for attributes defined in the OCSF schema but not observed in the Reference_Event
6. THE Suggested_Model review panel SHALL group attributes by tier with clear visual separation: Core tier attributes SHALL be pre-selected by default, Extended tier attributes SHALL be shown but not pre-selected, and Potential tier attributes SHALL be collapsed and hidden by default with an expandable control to reveal them on demand
7. THE Suggested_Model review panel SHALL allow the Analyst to promote or demote individual attributes between tiers (e.g., move an Extended attribute to Core, or demote a Core attribute to Potential)
8. WHEN a Mapping_Definition is available in the ReferenceEventStore (interpreted by the LLM_Mapping_Interpreter), THE Suggested_Model SHALL include lineage information for each attribute, showing the source raw field name, the transformation applied to produce the OCSF field, and any additional context provided by the LLM (such as source log type and transformation explanations)
9. WHEN a Mapping_Definition is available, THE Suggested_Model review panel SHALL display a "Lineage" column next to each attribute showing the raw_field origin and transformation summary

### Requirement 10: Data-Seeded Metric Suggestions

**User Story:** As an Analyst, I want the tool to suggest metrics based on numeric fields in my pasted event, so that I can quickly define aggregation metrics without manually inspecting field types.

#### Acceptance Criteria

1. WHEN a Reference_Event is loaded and the Analyst navigates to the Metrics tab, THE MetricEditor SHALL display a "Suggested Metrics" section listing candidate metrics derived from numeric fields in the Reference_Event
2. WHEN suggesting metrics, THE MetricEditor SHALL identify numeric fields (integer, float) from the Reference_Event and propose count, sum, and avg aggregations with Confidence_Indicators
3. WHEN suggesting metrics, THE MetricEditor SHALL identify timestamp fields from the Reference_Event and propose time-granularity options (minute, hour, day)
4. WHEN suggesting metrics, THE MetricEditor SHALL propose cardinality (count_distinct) metrics for string fields with observable characteristics (e.g., unique source IPs, unique hostnames)
5. WHEN suggesting metrics, THE MetricEditor SHALL propose ratio metrics (success/failure ratios) when the Reference_Event contains status or result fields (e.g., `status_id`, `rcode_id`, `disposition_id`)
6. WHEN suggesting metrics, THE MetricEditor SHALL propose event-rate metrics (events per time window) for timestamp fields in the Reference_Event
7. WHEN suggesting metrics, THE MetricEditor SHALL propose top-N aggregation metrics for high-cardinality categorical string fields in the Reference_Event
8. WHEN displaying a suggested derived metric, THE MetricEditor SHALL include a plain-language description of what the metric measures (e.g., "Unique source IPs per hour" rather than "count_distinct on src_endpoint.ip")
9. WHEN the Analyst clicks "Add" on a suggested metric, THE EditorStore SHALL create the metric with the suggested configuration
10. WHEN no Reference_Event is loaded, THE MetricEditor SHALL display the current manual metric creation interface without suggestions

### Requirement 11: Data-Seeded Validation

**User Story:** As an Analyst, I want validation to cross-reference my semantic model against my actual event data, so that I can catch mismatches between the model and real data before deployment.

#### Acceptance Criteria

1. WHEN a Reference_Event is loaded and the Analyst runs validation, THE Validation tab SHALL cross-reference the semantic model against the Reference_Event field structure in addition to the OCSF schema
2. WHEN the semantic model references an attribute not present in the Reference_Event, THE Validation tab SHALL display a warning identifying the missing field and noting that the field exists in the schema but was absent from the sample event
3. WHEN a metric measure references a field that is not numeric in the Reference_Event, THE Validation tab SHALL display an error identifying the field and its observed type
4. WHEN no Reference_Event is loaded, THE Validation tab SHALL validate against the OCSF schema only, matching the current behavior
5. THE Validation tab SHALL clearly label which validation results are schema-based and which are sample-event-based
6. WHEN a Reference_Event is loaded and the Analyst clicks "Dry Run" or runs validation, THE Validation tab SHALL simulate each metric in the semantic model against the Reference_Event and display the computed value per metric (e.g., "dns_query_count → 1 (from 1 sample event)")
7. WHEN a dry-run simulation produces a null or missing result for a metric (the referenced field is not present in the Reference_Event), THE Validation tab SHALL flag that metric as a potential issue and display "null/missing" with the reason (e.g., "field 'response_size' not present in sample")
8. THE Validation tab SHALL display dry-run results in a separate section clearly labeled "Simulated against 1 sample event", distinct from schema-based and sample-event-based validation results

### Requirement 12: Dual Workflow Unification

**User Story:** As an Analyst, I want the "with sample event" path to be the default happy path and the "without sample event" path to be a seamless fallback, so that the workflow feels unified rather than branched.

#### Acceptance Criteria

1. THE Event_Paste_Panel SHALL present the "paste event" action as the default recommended first action on the Schema tab, with a secondary "Skip — browse schema manually" link
2. WHEN the Analyst skips the event paste step, THE Guide_Layer SHALL seamlessly transition to the manual workflow without displaying error states or missing-data warnings
3. WHEN the Analyst loads a Reference_Event after previously skipping step 1, THE ReferenceEventStore SHALL accept the late-loaded event and THE Guide_Layer SHALL update step statuses and seed downstream tabs accordingly
4. THE Guide_Layer SHALL use identical step labels and descriptions regardless of whether a Reference_Event is loaded, with conditional detail text noting the presence or absence of sample data

### Requirement 13: Single-Event MVP Messaging

**User Story:** As an Analyst, I want clear messaging that the pasted event is a single sample, so that I understand the tool is not inferring from a full dataset.

#### Acceptance Criteria

1. WHEN a Reference_Event is loaded, THE Event_Paste_Panel SHALL display a persistent informational banner stating "Working from a single sample event — field presence and types may vary across your full dataset"
2. WHEN Reference_Event data is displayed in the Entities tab, Metrics tab, or Validation tab, THE tab SHALL include a subtle "Based on 1 sample event" indicator near the Reference_Event-derived content
3. THE Event_Paste_Panel SHALL include a tooltip or help text explaining that multi-event support is planned for a future version

### Requirement 14: Error Handling and Graceful Degradation

**User Story:** As an Analyst, I want the data-first workflow to handle errors gracefully, so that parsing failures or missing fields do not block the modeling workflow.

#### Acceptance Criteria

1. IF the pasted JSON is valid JSON but does not resemble an OCSF event (no recognizable OCSF fields), THEN THE Event_Paste_Panel SHALL display a warning suggesting the Analyst verify the event format, and SHALL still extract fields and allow the workflow to proceed
2. IF the Class_UID_Detector encounters a `class_uid` value that is not a valid integer, THEN THE Class_UID_Detector SHALL display an error message and fall back to manual event class selection
3. IF the schema tree has not finished loading when a Reference_Event is parsed, THEN THE Class_UID_Detector SHALL queue the detection and execute the class resolution when the schema tree becomes available
4. IF the ReferenceEventStore encounters a localStorage write failure, THEN THE ReferenceEventStore SHALL operate in-memory without persistence and log a console warning
5. THE Event_Paste_Panel SHALL never throw an unhandled exception that prevents the Tab_UI from rendering
6. IF the parsed JSON contains nested objects or arrays, THEN THE Event_Paste_Panel SHALL flatten nested fields using dot-notation paths (e.g., `src_endpoint.ip`) consistent with the existing LogImport field extraction behavior


### Requirement 15: LLM-Powered Mapping Interpretation and Field Lineage

**User Story:** As an Analyst, I want to optionally paste the mapping artifact I used to transform my raw log into OCSF — in any format or language — alongside the event, so that the LLM can interpret the mapping, auto-populate field lineage, and provide rich context for the modeling workflow.

#### Acceptance Criteria

1. WHEN the Analyst pastes content into the "Mapping (optional — any format)" textarea of the Event_Paste_Panel and a Reference_Event is loaded, THE LLM_Mapping_Interpreter SHALL send both the Reference_Event JSON and the mapping artifact content to the configured LLM provider (Anthropic or OpenAI) via the Rust backend proxy (`ocsf-editor`), with a structured prompt requesting extraction of field-to-field mappings (raw_field → ocsf_field), transformation logic, source system/vendor identification, and any other relevant metadata
2. WHEN the LLM_Mapping_Interpreter receives a response from the LLM, THE LLM_Mapping_Interpreter SHALL parse the LLM response into a normalized internal structure: a list of entries each containing raw_field (string), ocsf_field (string), transformation (optional string describing the conversion logic), and mapping_confidence (High, Medium, or Low — where "High" means the mapping was explicitly defined in the artifact, "Medium" means inferred from naming conventions or transformation patterns, "Low" means LLM-generated guess based on field similarity), plus source system metadata (source log type, vendor) and optional plain-language explanations of transformation logic
3. WHEN the LLM_Mapping_Interpreter successfully interprets a mapping, THE ReferenceEventStore SHALL store the normalized mapping result and THE Event_Paste_Panel summary SHALL display the number of field mappings found, the identified source system/vendor, a preview table of raw_field → ocsf_field → transformation entries with Mapping_Confidence badges, and a breakdown of mapping confidence distribution (e.g., "8 High, 3 Medium, 1 Low")
4. WHEN the LLM_Mapping_Interpreter is processing a mapping artifact, THE Event_Paste_Panel SHALL display a loading indicator with a message such as "Interpreting mapping via LLM…" to communicate that the LLM call is in progress
5. IF no LLM provider is configured (no API key set via the settings modal), THEN THE Event_Paste_Panel SHALL render the mapping textarea in a disabled state with a message stating "Configure an LLM provider (Anthropic or OpenAI) in Settings to enable mapping interpretation" and a link or button to open the settings modal
6. IF the LLM call fails due to a network error, rate limit, authentication error, or timeout, THEN THE LLM_Mapping_Interpreter SHALL display a descriptive error message in the mapping textarea area, preserve the pasted mapping content for retry, and provide a "Retry" button; THE Reference_Event SHALL remain unaffected
7. WHEN both a Reference_Event and a Mapping_Definition are successfully interpreted, THE ReferenceEventStore SHALL cross-reference the LLM-extracted mapping entries against the extracted event fields and flag any ocsf_field in the mapping that does not appear in the Reference_Event
8. WHEN both a Reference_Event and a Mapping_Definition are provided, THE LLM_Mapping_Interpreter SHALL request the LLM to also identify the source log type, explain transformation logic in plain language for each mapping entry, and flag potential issues (e.g., lossy transformations, type coercions, unmapped fields)
9. THE LLM_Mapping_Interpreter SHALL accept mapping artifacts in any format or language, including but not limited to: Logstash pipeline configs, Cribl packs, Splunk props.conf/transforms.conf, Python transformation scripts, dbt SQL models, custom YAML mapping files, Sigma rules, JSON mapping documents, and simple key-value field lists
10. WHEN the Analyst clears the mapping textarea, THE ReferenceEventStore SHALL remove the LLM-interpreted Mapping_Definition and all derived lineage data while preserving the Reference_Event
11. WHILE no Reference_Event is loaded, THE Event_Paste_Panel SHALL allow the Analyst to paste mapping content but SHALL display a message stating "Paste an OCSF event first — the mapping will be interpreted together with the event" and SHALL defer the LLM call until a Reference_Event is available
12. THE LLM_Mapping_Interpreter SHALL send the LLM request through the existing Rust backend proxy (`ocsf-editor`) following the same pattern as the existing research feature, ensuring API keys are never exposed to the client side
13. WHEN displaying mapping entries in the Event_Paste_Panel summary, the Suggested_Model review panel, or the Index tab lineage view, THE UI SHALL display color-coded Mapping_Confidence badges per entry (e.g., green for High, yellow for Medium, red for Low)
14. WHEN mapping entries are displayed in any view, THE Analyst SHALL be able to filter and sort mapping entries by Mapping_Confidence level

### Requirement 16: Mapping Verification Layer

**User Story:** As an Analyst, I want each LLM-interpreted mapping entry to be cross-referenced against my pasted event, so that I can see which mappings are verified by the actual data and which are unverified or conflicting.

#### Acceptance Criteria

1. WHEN both a Reference_Event and a Mapping_Definition are available, THE ReferenceEventStore SHALL compute a Verification_Status for each mapping entry by cross-referencing the ocsf_field against the Reference_Event fields
2. WHEN the ocsf_field in a mapping entry matches a field present in the Reference_Event, THE Verification_Status for that entry SHALL be set to "Verified"
3. WHEN the ocsf_field in a mapping entry does not match any field present in the Reference_Event, THE Verification_Status for that entry SHALL be set to "Unverified"
4. WHEN the mapping entry contradicts observed data in the Reference_Event (e.g., the mapping declares a field as integer but the Reference_Event shows a string value), THE Verification_Status for that entry SHALL be set to "Conflict"
5. WHEN displaying mapping entries in the Event_Paste_Panel summary, the Suggested_Model review panel, or the Index tab lineage view, THE UI SHALL display color-coded Verification_Status badges per entry (e.g., green for Verified, gray for Unverified, red for Conflict)
6. WHEN mapping entries are displayed in any view, THE Analyst SHALL be able to filter and sort mapping entries by Verification_Status
7. WHEN a mapping entry has Verification_Status "Conflict", THE UI SHALL display a tooltip or inline detail explaining the nature of the conflict (e.g., "Mapping declares integer, event shows string")

### Requirement 17: Mapping Coverage Analysis

**User Story:** As an Analyst, I want to see coverage metrics showing how completely the mapping covers my event fields, so that I can identify gaps in the mapping before finalizing the semantic model.

#### Acceptance Criteria

1. WHEN both a Reference_Event and a Mapping_Definition are available, THE ReferenceEventStore SHALL compute the following Mapping_Coverage metrics: percent_event_fields_mapped (percentage of fields in the Reference_Event that have a corresponding mapping entry), percent_mapping_fields_unobserved (percentage of mapping entries that reference ocsf_fields not present in the Reference_Event), and percent_event_fields_unmapped (percentage of Reference_Event fields that have no mapping entry)
2. WHEN Mapping_Coverage metrics are computed, THE Event_Paste_Panel summary area SHALL display a coverage summary panel showing all three coverage percentages with visual indicators (progress bars or percentage badges) to make coverage gaps immediately visible
3. WHEN Mapping_Coverage metrics are computed, THE Index tab lineage view SHALL display the same coverage summary panel
4. WHEN percent_event_fields_unmapped is greater than zero, THE coverage summary panel SHALL list the unmapped field paths to help the Analyst identify specific gaps
5. WHEN percent_mapping_fields_unobserved is greater than zero, THE coverage summary panel SHALL list the unobserved mapping entries to help the Analyst identify mappings that may be incorrect or apply to fields not present in the sample event
