/**
 * Types and constants for the Guided Wizard Workflow overlay layer.
 * These define the 4-step data-first onboarding guide that teaches analysts
 * the correct workflow for building a semantic model using the existing tab UI.
 */

// Re-export TabId from App.tsx usage — keep in sync with the 8-tab definition there.
export type TabId = 'schema' | 'entities' | 'metrics' | 'validation' | 'index' | 'architecture' | 'catalog' | 'etl';

// ============================================
// Core Guide Types
// ============================================

/** The 4 workflow steps in the guided wizard. */
export type GuideStepId = 1 | 2 | 3 | 4;

/** Visual status of a guide step. */
export type StepStatus = 'pending' | 'active' | 'complete';

/** A single step in the guided workflow. */
export interface GuideStep {
  id: GuideStepId;
  label: string;
  targetTab: TabId;
  status: StepStatus;
  description: string;
}

/** Serializable guide state for localStorage persistence. */
export interface GuideSaveState {
  version: 1;
  guideVisible: boolean;
  dismissedHints: number[];
  shownPrompts: number[];
  lastValidationRun: string | null;
}

/** OCSF domain terms that can have contextual tooltips. */
export type OCSFTerm =
  | 'event_class'
  | 'category'
  | 'attribute'
  | 'observable'
  | 'dimension'
  | 'metric'
  | 'semantic_entity'
  | 'field_mapping'
  | 'source_lineage'
  | 'detection_coverage'
  | 'mitre_technique'
  | 'hot_path'
  | 'time_granularity';

// ============================================
// Step-to-Tab Mapping
// ============================================

/** Maps each guide step to its target tab. */
export const STEP_TO_TAB: Record<GuideStepId, TabId> = {
  1: 'schema',
  2: 'entities',
  3: 'metrics',
  4: 'validation',
};

/** Default step definitions (status will be derived at runtime). */
export const GUIDE_STEPS: Omit<GuideStep, 'status'>[] = [
  { id: 1, label: 'Paste Event (+ Optional Mapping)', targetTab: 'schema', description: 'Paste a transformed OCSF event to auto-detect the event class and seed the workflow. Optionally include a mapping artifact for field lineage.' },
  { id: 2, label: 'Define Entity & Map Attributes', targetTab: 'entities', description: 'Create a semantic entity from your event data. Accept, modify, or discard the suggested model.' },
  { id: 3, label: 'Add Metrics', targetTab: 'metrics', description: 'Define aggregation metrics. Review suggested metrics derived from your event fields.' },
  { id: 4, label: 'Validate Model', targetTab: 'validation', description: 'Validate your semantic model against the OCSF schema and your sample event data.' },
];

// ============================================
// Prerequisite Chain
// ============================================

/** Prerequisites that must be complete before each step can proceed. */
export const STEP_PREREQUISITES: Record<GuideStepId, GuideStepId[]> = {
  1: [],
  2: [1],
  3: [2],
  4: [2, 3],
};

// ============================================
// Hint Content
// ============================================

export interface HintContent {
  title: string;
  description: string;
}

/** Hint content shown when a step is pending (not yet completed). */
export const HINT_CONTENT_PENDING: Record<TabId, HintContent> = {
  schema: {
    title: 'Step 1: Paste Event (+ Optional Mapping)',
    description: 'Pasting an OCSF event is optional but recommended — the event class is auto-detected from class_uid. You can also provide an optional mapping artifact (any format) to auto-populate field lineage. Or skip and browse the schema manually.',
  },
  entities: {
    title: 'Step 2: Define Entity & Map Attributes',
    description: 'Create a semantic entity from your event data. Review the suggested model with tiered attributes, then accept, modify, or discard.',
  },
  metrics: {
    title: 'Step 3: Add Metrics',
    description: 'Define aggregation metrics for your entity. Review suggested metrics derived from your event fields, or create metrics manually.',
  },
  validation: {
    title: 'Step 4: Validate Model',
    description: 'Validate your semantic model against the OCSF schema and your sample event data. Run a dry-run simulation to preview metric results.',
  },
  index: {
    title: 'Index (Optional Enrichment)',
    description: 'The Index tab is optional. Use it to enrich your model with data lineage, field-level lineage, detection coverage (MITRE ATT&CK mapping), and physical table registration.',
  },
  architecture: {
    title: 'Architecture View',
    description: 'View the visual architecture of your semantic model. This is auto-generated from your entities and relationships.',
  },
  catalog: {
    title: 'Catalog',
    description: 'Browse and manage catalog entries for your semantic model artifacts.',
  },
  etl: {
    title: 'ETL Pipelines',
    description: 'Configure ETL pipelines to transform raw data into OCSF-normalized tables.',
  },
};

/** Hint content shown when a step is complete. */
export const HINT_CONTENT_COMPLETE: Record<TabId, HintContent> = {
  schema: {
    title: 'Step 1: Event Pasted ✓',
    description: 'Your event has been parsed and the event class auto-detected from class_uid. You can override the detected class in the schema tree, or paste a new event to re-seed the workflow.',
  },
  entities: {
    title: 'Step 2: Entity Defined ✓',
    description: 'Your entity and attributes are set up. You can continue refining them or move on to defining metrics.',
  },
  metrics: {
    title: 'Step 3: Metrics Defined ✓',
    description: 'Your metrics are configured. You can add more or proceed to validate your model.',
  },
  validation: {
    title: 'Step 4: Model Validated ✓',
    description: 'Validation has been run against the OCSF schema and your sample event data. Review any warnings or errors. Optionally, visit the Index tab for advanced lineage, detection coverage, and table registration.',
  },
  index: {
    title: 'Index (Optional)',
    description: 'Your table is registered and field lineage is mapped. Your semantic model is connected to actual data.',
  },
  architecture: {
    title: 'Architecture View',
    description: 'Your architecture diagram reflects the current state of your semantic model.',
  },
  catalog: {
    title: 'Catalog',
    description: 'Browse and manage catalog entries for your semantic model artifacts.',
  },
  etl: {
    title: 'ETL Pipelines',
    description: 'Configure ETL pipelines to transform raw data into OCSF-normalized tables.',
  },
};

// ============================================
// Tooltip Content
// ============================================

export interface TooltipContent {
  explanation: string;
  learnMoreUrl?: string;
}

/** Contextual tooltip explanations for OCSF domain terms. */
export const TOOLTIP_CONTENT: Record<OCSFTerm, TooltipContent> = {
  event_class: {
    explanation: 'An OCSF event class represents a specific type of security event (e.g., DNS Activity, Process Activity). Each class defines a set of attributes.',
    learnMoreUrl: 'https://schema.ocsf.io/categories',
  },
  category: {
    explanation: 'An OCSF category groups related event classes (e.g., Network Activity, System Activity).',
    learnMoreUrl: 'https://schema.ocsf.io/categories',
  },
  attribute: {
    explanation: 'An attribute is a named field within an OCSF event class, like src_endpoint.ip or process.name.',
    learnMoreUrl: 'https://schema.ocsf.io/objects',
  },
  observable: {
    explanation: 'An observable is a value that can be watched for threat detection — like an IP address, hostname, or file hash.',
    learnMoreUrl: 'https://schema.ocsf.io/objects/observable',
  },
  dimension: {
    explanation: 'A dimension is an attribute used for grouping and filtering in analytics queries (e.g., group by source IP).',
  },
  metric: {
    explanation: 'A metric is a numeric aggregation (count, sum, avg) computed over entity attributes for dashboards.',
  },
  semantic_entity: {
    explanation: 'A semantic entity maps business-friendly names to OCSF event class attributes, making queries more intuitive.',
  },
  field_mapping: {
    explanation: 'A field mapping connects a source data field to its corresponding OCSF attribute with an optional transformation.',
  },
  source_lineage: {
    explanation: 'Source lineage tracks where data comes from — which source system and table feed into an OCSF-normalized table.',
  },
  detection_coverage: {
    explanation: 'Detection coverage maps which MITRE ATT&CK techniques and tactics your data sources can detect.',
    learnMoreUrl: 'https://attack.mitre.org/',
  },
  mitre_technique: {
    explanation: 'A MITRE ATT&CK technique (e.g., T1071.004 DNS) describes a specific adversary behavior.',
    learnMoreUrl: 'https://attack.mitre.org/techniques/enterprise/',
  },
  hot_path: {
    explanation: 'A hot-path metric is pre-computed for real-time dashboards, trading storage for query speed.',
  },
  time_granularity: {
    explanation: 'Time granularity defines the time bucket size for metric aggregation (e.g., 1 minute, 1 hour, 1 day).',
  },
};

// ============================================
// localStorage Key
// ============================================

/** localStorage key for persisting guide preferences. */
export const GUIDE_STORAGE_KEY = 'ocsf-guide-preferences';
