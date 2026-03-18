# OCSF Semantic Layer — Analyst Workflow

## Context

The OCSF Semantic Model Editor helps security analysts and data engineers build a **semantic abstraction layer** on top of OCSF-normalized security data. The semantic layer maps business-friendly names, metrics, and relationships onto the underlying OCSF event classes and attributes.

The typical analyst already has a **transformed OCSF event** (JSON) — they've already done the raw-to-OCSF normalization work. The editor's job is to help them build a semantic model on top of that data.

## The Problem with the Current Flow

The current guided workflow is:

1. Browse the OCSF schema tree → manually find and select an event class
2. Define a semantic entity
3. Map fields and attributes
4. Add metrics
5. Validate the model
6. Register in the Index tab (table registration, lineage, coverage)

This has two issues:

- **Step 1 is abstract.** The analyst has to browse a schema tree and know which event class their data maps to. But they already have the OCSF event — the `class_uid` is right there in the JSON.
- **Step 6 is confusing.** The Index tab asks the analyst to register physical tables and map field lineage. This is a data engineering concern, not an analyst concern. It feels disconnected from the semantic modeling workflow.

## Proposed Analyst Flow

### New Step 1: Paste Your OCSF Event (Optional, Recommended)

The analyst pastes a transformed OCSF event (JSON) into the editor. This is the same data they already have from their normalization pipeline.

**What the tool does with it:**

- **Auto-detects the event class** from `class_uid` (e.g., `4003` → DNS Activity) and `category_uid`
- **Extracts all fields** present in the event, with their types and sample values
- **Pre-selects the event class** in the Schema tab — no manual browsing needed
- **Seeds the attribute list** for entity creation — the analyst sees the actual fields from their data, not just the schema spec
- **Identifies observables** present in the event (IPs, hostnames, hashes, etc.)
- **Infers field types** (string, integer, timestamp, IP address) from actual values

**If the analyst skips this step**, the current manual flow still works — they browse the schema tree and pick an event class themselves.

### Step 2: Select / Confirm Event Class

If step 1 was completed, the event class is already selected and highlighted in the schema tree. The analyst can confirm it or change it.

If step 1 was skipped, this works exactly as it does today — browse the tree, pick a class.

### Step 3: Define Entity & Map Attributes

The analyst creates a semantic entity (e.g., "DNS Query") that maps business-friendly names to OCSF fields.

**With a pasted event (step 1 completed):**
- The attribute picker shows which fields actually exist in their data (not just the full schema spec)
- Sample values from the pasted event appear next to each attribute, giving concrete context
- The tool can suggest which attributes are most relevant based on what's populated in the event
- Observable fields (IPs, hostnames) are flagged automatically

**Without a pasted event:**
- Works as today — the full attribute list from the schema spec is available

### Step 4: Add Metrics

Define aggregation metrics (count, sum, avg) over entity attributes.

**With a pasted event:**
- The tool knows which fields are numeric (from actual values) and can suggest metric candidates
- Time fields are identified for time-granularity options

**Without a pasted event:**
- Works as today — manual metric definition

### Step 5: Validate Model

Run validation to check entity definitions, metric expressions, and attribute mappings.

**With a pasted event:**
- Validation can cross-reference the semantic model against the actual event structure
- Warns if the model references attributes not present in the sample event
- Confirms that metric measures point to real numeric fields

**Without a pasted event:**
- Validates against the OCSF schema spec only (current behavior)

### Index Tab: Optional Enrichment

The Index tab is reframed as **optional, advanced functionality** for analysts who want to:

- Track **source lineage** (where the data came from before OCSF normalization)
- Map **field-level lineage** (how each raw field became an OCSF field)
- Define **detection coverage** (which MITRE ATT&CK techniques this data supports)
- Register **physical tables** for warehouse integration

This is no longer part of the core guided workflow. It's available for data engineers and analysts who need governance/lineage tracking, but it's not a required step to build a semantic model.

## Summary: Before vs After

| Aspect | Current Flow | Proposed Flow |
|--------|-------------|---------------|
| Starting point | Browse schema tree manually | Paste your OCSF event (optional) |
| Event class selection | Manual tree navigation | Auto-detected from `class_uid` |
| Attribute context | Schema spec only | Actual field values from your data |
| Metric suggestions | None | Inferred from numeric fields |
| Validation depth | Schema spec only | Cross-referenced with actual event |
| Index tab | Required step 6 | Optional enrichment |
| Core steps | 6 | 5 (with step 1 as optional accelerator) |

## What This Means for the UI

- The **Schema tab** gets a "Paste OCSF Event" panel (or the existing LogImport component is promoted to step 1)
- The **guided progress bar** changes from 6 steps to 5 core steps, with step 1 being "Provide Sample Event (optional) → Select Event Class"
- The **Index tab** hint changes from "Register your physical table" to "Optionally enrich your model with lineage and detection coverage"
- The **prerequisite chain** simplifies — Index no longer blocks on validation
- A **reference event** concept is added to the editor store, carrying the parsed OCSF event data through all subsequent steps

## Existing Infrastructure

The codebase already has a `LogImport` component (currently inside the Index tab's Import sub-nav) that:
- Accepts pasted text or file upload
- Auto-detects format (JSON, CSV, syslog, key-value)
- Parses and extracts fields with types and sample values
- Stores results in `useIndexStore` (parsedFields, detectedFormat)

This component could be repurposed or cloned as the new step 1 entry point, with additional logic to:
- Extract `class_uid` / `category_uid` from the parsed JSON
- Auto-select the corresponding event class in the schema tree
- Store the parsed event as a "reference event" accessible to Entity, Metric, and Validation tabs
