/**
 * TypeScript types for the OCSF Semantic Model Editor.
 * These types mirror the Rust backend types for API compatibility.
 */

// ============================================
// Semantic Model Types
// ============================================

/**
 * Threat relevance metadata for security-focused attributes.
 */
export interface ThreatRelevance {
  use_cases: string[];
  mitre_techniques: string[];
}

/**
 * Semantic type for entity attributes.
 */
export type SemanticType =
  | 'string'
  | 'integer'
  | 'float'
  | 'boolean'
  | 'timestamp'
  | 'json'
  | { array: SemanticType };

/**
 * A single level in a dimension drill-down hierarchy.
 */
export interface HierarchyLevel {
  name: string;
  attribute_ref: string;
}

/**
 * Classification of metric additivity behavior.
 */
export type MetricType = 'Additive' | 'SemiAdditive' | 'NonAdditive';

/**
 * A physical data source abstraction decoupled from semantic definitions.
 */
export interface Dataset {
  name: string;
  dialect: string;
  table: string;
  schema_name?: string;
  connection?: string;
}

/**
 * Mapping from semantic attribute to OCSF fields.
 */
export interface OCSFMapping {
  field?: string;
  expression?: string;
  join_condition?: string;
}

/**
 * A semantic attribute within an entity.
 */
export interface SemanticAttribute {
  name: string;
  caption: string;
  description: string;
  attr_type: SemanticType;
  ocsf_mapping: OCSFMapping;
  is_dimension: boolean;
  sample_values: string[];
  synonyms: string[];
  security_context?: string;
  value_pattern?: string;
  is_observable: boolean;
  threat_relevance?: ThreatRelevance;
  hierarchy?: HierarchyLevel[];
  is_hidden?: boolean;
  folder?: string;
}

/**
 * Cardinality of an entity relationship.
 */
export type Cardinality = 'one-to-one' | 'one-to-many' | 'many-to-one' | 'many-to-many';

/**
 * Type of relationship between entities.
 */
export type RelationshipType =
  | 'originated-from'
  | 'targeted-to'
  | 'performed-by'
  | 'contains'
  | 'referenced-in';

/**
 * A relationship between semantic entities.
 */
export interface EntityRelationship {
  name: string;
  target_entity: string;
  relationship_type?: RelationshipType;
  cardinality: Cardinality;
  join_condition: string;
  description: string;
  role_alias?: string;
}

/**
 * A semantic entity definition.
 */
export interface SemanticEntity {
  name: string;
  caption: string;
  description: string;
  source_event_classes: number[];
  attributes: SemanticAttribute[];
  relationships: EntityRelationship[];
  covers_observables: number[];
  dataset_ref?: string;
}

/**
 * Aggregation function for metrics.
 */
export type Aggregation = 'count' | 'sum' | 'avg' | 'min' | 'max' | 'count_distinct';

/**
 * Time granularity for time-based aggregations.
 */
export type TimeGranularity = 'minute' | 'hour' | 'day' | 'week' | 'month';

/**
 * A semantic metric definition.
 */
export interface SemanticMetric {
  name: string;
  caption: string;
  description: string;
  aggregation: Aggregation;
  measure: OCSFMapping;
  dimensions: string[];
  time_granularities: TimeGranularity[];
  is_hot_path: boolean;
  observable_type_id?: number;
  metric_type?: MetricType;
  formula?: string;
  non_additive_dimensions?: string[];
  is_hidden?: boolean;
  folder?: string;
}

/**
 * Configuration for observable extraction.
 */
export interface ObservableConfig {
  extract_to_table: boolean;
  table_name: string;
  include_types: number[];
}

/**
 * A semantic model definition.
 */
export interface SemanticModel {
  version: string;
  ocsf_version: string;
  name: string;
  description: string;
  entities: SemanticEntity[];
  metrics: SemanticMetric[];
  observable_config: ObservableConfig;
  datasets?: Dataset[];
}

// ============================================
// Schema Tree Types
// ============================================

export interface EnumValue {
  key: string;
  caption: string;
  description?: string;
}

export interface AttributeNode {
  name: string;
  caption: string;
  description: string;
  type: string;
  requirement: 'required' | 'recommended' | 'optional';
  is_array: boolean;
  enum_values?: EnumValue[];
  object_type?: string;
  children?: AttributeNode[];
}

export interface ClassNode {
  uid: number;
  name: string;
  caption: string;
  description: string;
  attributes: AttributeNode[];
}

export interface CategoryNode {
  uid: number;
  name: string;
  caption: string;
  description: string;
  classes: ClassNode[];
}

export interface ObjectNode {
  name: string;
  caption: string;
  description: string;
  attributes: AttributeNode[];
}

export interface SchemaTree {
  version: string;
  categories: CategoryNode[];
  objects: ObjectNode[];
}

// ============================================
// Validation Types
// ============================================

export interface ValidationError {
  path: string;
  message: string;
  code: string;
}

export interface ValidationWarning {
  path: string;
  message: string;
  code: string;
}

// ============================================
// API Types
// ============================================

export interface ValidateResponse {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface GeneratedFile {
  path: string;
  content: string;
}

export interface GenerateResponse {
  files: GeneratedFile[];
}

// ============================================
// LLM Research Types
// ============================================

/**
 * OCSF context for LLM research requests.
 */
export interface OCSFContext {
  class_name: string;
  class_caption: string;
  class_description: string;
  category_name: string;
  attribute_name?: string;
  attribute_type?: string;
  attribute_description?: string;
}

/**
 * Fields that can be researched by the LLM.
 */
export type ResearchField = 'description' | 'synonyms' | 'security_context' | 'sample_values';

/**
 * Target type for research requests.
 */
export type ResearchTargetType = 'entity' | 'attribute';

/**
 * Research target for the LLM panel.
 */
export type ResearchTarget =
  | { type: 'entity'; entity: SemanticEntity }
  | { type: 'attribute'; entity: SemanticEntity; attribute: SemanticAttribute };

/**
 * Request body for POST /api/llm/research endpoint.
 */
export interface ResearchRequest {
  target_type: ResearchTargetType;
  entity_name: string;
  attribute_name?: string;
  ocsf_context: OCSFContext;
  requested_fields: ResearchField[];
  is_observable?: boolean;
  additional_context?: string;
}

/**
 * A single target in a batch research request.
 */
export interface ResearchTargetRequest {
  target_type: ResearchTargetType;
  entity_name: string;
  attribute_name?: string;
  ocsf_context: OCSFContext;
  requested_fields: ResearchField[];
  is_observable?: boolean;
  additional_context?: string;
}

/**
 * Batch research request for multiple targets.
 */
export interface BatchResearchRequest {
  targets: ResearchTargetRequest[];
  shared_context?: string;
}

/**
 * LLM suggestion in API response.
 */
export interface LLMSuggestion {
  id: string;
  field: ResearchField;
  value: string | string[];
  confidence: number;
}

/**
 * Response for POST /api/llm/research endpoint.
 */
export interface ResearchResponse {
  suggestions: LLMSuggestion[];
  tokens_used: number;
}

// ============================================
// Factory Functions
// ============================================

/**
 * Creates a default empty semantic model.
 */
export function createDefaultModel(): SemanticModel {
  return {
    version: '1.0',
    ocsf_version: '',
    name: 'New Model',
    description: '',
    entities: [],
    metrics: [],
    observable_config: {
      extract_to_table: false,
      table_name: 'ocsf_observables',
      include_types: [],
    },
  };
}

/**
 * Creates an example semantic model for DNS analytics.
 * This demonstrates a complete semantic layer with entities, attributes, and metrics.
 */
export function createExampleModel(): SemanticModel {
  return {
    version: '1.0',
    ocsf_version: '1.6.0',
    name: 'dns-analytics-example',
    description: 'Example semantic layer for DNS activity analytics - demonstrates entities, attributes, metrics, and threat intelligence mappings',
    entities: [
      {
        name: 'dns_event',
        caption: 'DNS Event',
        description: 'DNS queries and responses with threat detection capabilities',
        source_event_classes: [4003],
        covers_observables: [1, 2],
        attributes: [
          {
            name: 'query_hostname',
            caption: 'Query Hostname',
            description: 'The fully qualified domain name being queried',
            attr_type: 'string',
            ocsf_mapping: { field: 'query.hostname' },
            is_dimension: true,
            is_observable: true,
            sample_values: ['example.com', 'mail.google.com', 'api.github.com'],
            synonyms: ['domain', 'fqdn', 'hostname', 'queried_domain'],
            security_context: 'Critical for detecting DGA domains, DNS tunneling, and C2 beaconing.',
            value_pattern: '^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$',
            threat_relevance: {
              use_cases: ['DGA detection', 'DNS tunneling', 'C2 beaconing'],
              mitre_techniques: ['T1071.004', 'T1568.002'],
            },
          },
          {
            name: 'query_type',
            caption: 'Query Type',
            description: 'DNS record type being queried (A, AAAA, CNAME, TXT, MX)',
            attr_type: 'string',
            ocsf_mapping: { field: 'query.type' },
            is_dimension: true,
            is_observable: false,
            sample_values: ['A', 'AAAA', 'CNAME', 'TXT', 'MX'],
            synonyms: ['record_type', 'dns_type', 'qtype'],
            security_context: 'TXT and NULL records are commonly abused for DNS tunneling.',
            threat_relevance: {
              use_cases: ['DNS tunneling detection', 'Data exfiltration'],
              mitre_techniques: ['T1048.003'],
            },
          },
          {
            name: 'source_ip',
            caption: 'Source IP',
            description: 'IP address of the client making the DNS query',
            attr_type: 'string',
            ocsf_mapping: { field: 'src_endpoint.ip' },
            is_dimension: true,
            is_observable: true,
            sample_values: ['10.0.1.100', '192.168.1.50', '172.16.0.25'],
            synonyms: ['client_ip', 'src_ip', 'source_address'],
            security_context: 'Track query patterns per source IP to detect compromised hosts.',
            threat_relevance: {
              use_cases: ['Compromised host detection', 'Lateral movement tracking'],
              mitre_techniques: ['T1071'],
            },
          },
          {
            name: 'response_code',
            caption: 'Response Code',
            description: 'DNS response code (NOERROR, NXDOMAIN, SERVFAIL, etc.)',
            attr_type: 'string',
            ocsf_mapping: { field: 'rcode' },
            is_dimension: true,
            is_observable: false,
            sample_values: ['NOERROR', 'NXDOMAIN', 'SERVFAIL', 'REFUSED'],
            synonyms: ['rcode', 'dns_response', 'response_status'],
            security_context: 'High NXDOMAIN rates indicate DGA activity.',
            threat_relevance: {
              use_cases: ['DGA detection', 'DNS infrastructure monitoring'],
              mitre_techniques: ['T1568.002'],
            },
          },
          {
            name: 'action',
            caption: 'Action',
            description: 'Action taken on the DNS query (Allowed, Blocked, etc.)',
            attr_type: 'string',
            ocsf_mapping: { field: 'action' },
            is_dimension: true,
            is_observable: false,
            sample_values: ['Allowed', 'Blocked', 'Dropped'],
            synonyms: ['disposition', 'verdict', 'dns_action'],
            security_context: 'Blocked queries indicate policy enforcement or threat prevention.',
          },
          {
            name: 'event_time',
            caption: 'Event Time',
            description: 'Timestamp when the DNS query occurred',
            attr_type: 'timestamp',
            ocsf_mapping: { field: 'time' },
            is_dimension: false,
            is_observable: false,
            sample_values: [],
            synonyms: ['timestamp', 'time', 'query_time'],
          },
        ],
        relationships: [
          {
            name: 'domain_threat_intel',
            target_entity: 'threat_intel_domains',
            relationship_type: 'referenced-in',
            cardinality: 'many-to-one',
            join_condition: 'dns_event.query_hostname = threat_intel_domains.domain',
            description: 'Match DNS queries against known malicious domains',
          },
        ],
      },
    ],
    metrics: [
      {
        name: 'dns_query_count',
        caption: 'DNS Query Count',
        description: 'Total number of DNS queries',
        aggregation: 'count',
        measure: { field: 'metadata.uid' },
        dimensions: ['query_hostname', 'source_ip', 'action'],
        time_granularities: ['hour', 'day'],
        is_hot_path: false,
      },
      {
        name: 'blocked_dns_rate',
        caption: 'Blocked DNS Rate',
        description: 'Percentage of blocked DNS queries',
        aggregation: 'avg',
        measure: { expression: "CASE WHEN action != 'Allowed' THEN 1.0 ELSE 0.0 END" },
        dimensions: ['source_ip'],
        time_granularities: ['hour', 'day'],
        is_hot_path: false,
      },
      {
        name: 'nxdomain_rate',
        caption: 'NXDOMAIN Rate',
        description: 'Percentage of DNS queries resulting in NXDOMAIN (potential DGA indicator)',
        aggregation: 'avg',
        measure: { expression: "CASE WHEN response_code = 'NXDOMAIN' THEN 1.0 ELSE 0.0 END" },
        dimensions: ['source_ip'],
        time_granularities: ['hour', 'day'],
        is_hot_path: true,
      },
    ],
    observable_config: {
      extract_to_table: true,
      table_name: 'dns_observables',
      include_types: [1, 2],
    },
  };
}

/**
 * Creates a default semantic entity.
 */
export function createDefaultEntity(name: string): SemanticEntity {
  return {
    name,
    caption: '',
    description: '',
    source_event_classes: [],
    attributes: [],
    relationships: [],
    covers_observables: [],
  };
}

/**
 * Creates a default semantic attribute.
 */
export function createDefaultAttribute(name: string): SemanticAttribute {
  return {
    name,
    caption: '',
    description: '',
    attr_type: 'string',
    ocsf_mapping: {},
    is_dimension: false,
    sample_values: [],
    synonyms: [],
    is_observable: false,
    is_hidden: false,
  };
}

/**
 * Creates a default semantic metric.
 */
export function createDefaultMetric(name: string): SemanticMetric {
  return {
    name,
    caption: '',
    description: '',
    aggregation: 'count',
    measure: {},
    dimensions: [],
    time_granularities: [],
    is_hot_path: false,
    metric_type: 'Additive' as MetricType,
  };
}


// ============================================
// LLM Configuration Types
// ============================================

/**
 * LLM provider type.
 */
export type LLMProvider = 'anthropic' | 'openai';

/**
 * Response for GET /api/llm/config endpoint.
 */
export interface LLMConfigResponse {
  configured: boolean;
  provider: LLMProvider | null;
}

/**
 * Request body for POST /api/llm/config endpoint.
 */
export interface LLMConfigRequest {
  provider: LLMProvider;
  api_key: string;
}

// ============================================
// Index Types (ocsf-index integration)
// ============================================

// Table Registry Types
export interface TableEntry {
  id: number;
  table_name: string;
  schema_name?: string;
  class_uid: number;
  ocsf_version: string;
  dialect: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
  metadata: Record<string, string>;
  detection_coverage?: DetectionCoverage;
}

export interface DetectionCoverage {
  mitre_techniques: string[];
  mitre_tactics: string[];
  data_sources: string[];
  detection_rules: string[];
  kill_chain_phases: string[];
  confidence_level?: 'low' | 'medium' | 'high';
  max_severity?: 'informational' | 'low' | 'medium' | 'high' | 'critical';
  tlp?: 'clear' | 'green' | 'amber' | 'amber_strict' | 'red';
}

// Lineage Types
export interface SourceLineageRecord {
  id?: number;
  source_system: string;
  source_table: string;
  target_table: string;
  ingestion_timestamp: string;
  record_count?: number;
  metadata: Record<string, string>;
}

export interface FieldLineageRecord {
  id?: number;
  source_lineage_id: number;
  source_field: string;
  target_field: string;
  transformation?: string;
  ocsf_version?: string;
}

export interface LineageGraph {
  nodes: LineageNode[];
  edges: LineageEdge[];
}

export interface LineageNode {
  id: string;
  node_type: 'source' | 'target';
  label: string;
  metadata: Record<string, string>;
}

export interface LineageEdge {
  source: string;
  target: string;
  timestamp: string;
  record_count?: number;
}

// Coverage Types
export interface DetectionCoverageSummary {
  tables_with_coverage: number;
  mitre_techniques: MitreTechniqueCoverage[];
  mitre_tactics: MitreTacticCoverage[];
  data_sources: DataSourceCoverage[];
  kill_chain_coverage: KillChainCoverage[];
}

export interface MitreTechniqueCoverage {
  technique_id: string;
  table_count: number;
  tables: string[];
}

export interface MitreTacticCoverage {
  tactic: string;
  technique_count: number;
  table_count: number;
}

export interface DataSourceCoverage {
  data_source: string;
  table_count: number;
  tables: string[];
}

export interface KillChainCoverage {
  phase: string;
  table_count: number;
}

export interface CoverageMatrix {
  tactics: TacticColumn[];
  techniques_by_tactic: Record<string, TechniqueCell[]>;
}

export interface TacticColumn {
  id: string;
  name: string;
  table_count: number;
}

export interface TechniqueCell {
  id: string;
  name: string;
  table_count: number;
  tables: string[];
  is_covered: boolean;
}

// Statistics Types
export interface TableStatistics {
  table_name: string;
  columns: ColumnStatistics[];
  total_rows: number;
  sample_rate: number;
  collected_at: string;
  is_stale: boolean;
}

export interface ColumnStatistics {
  column_name: string;
  distinct_count: number;
  null_count: number;
  total_count: number;
  min_value?: string;
  max_value?: string;
  null_rate: number;
  cardinality: number;
}

// Partition Types
export interface PartitionEntry {
  id: number;
  table_name: string;
  partition_key: string;
  start_time: string;
  end_time: string;
  row_count: number;
  size_bytes: number;
  is_empty: boolean;
  last_modified: string;
}

// Export/Import Types
export interface IndexModelExport {
  version: string;
  exported_at: string;
  tables: TableEntry[];
  source_lineage: SourceLineageRecord[];
  field_lineage: FieldLineageRecord[];
}

export interface CombinedModelExport {
  semantic_model: SemanticModel;
  index_model: IndexModelExport;
}

export interface ImportResult {
  success: boolean;
  tables_imported: number;
  source_lineage_imported: number;
  field_lineage_imported: number;
  errors?: string[];
}

// Log Import Types
export interface ParsedLogField {
  name: string;
  value: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array' | 'null';
  path: string;
}

export interface FieldMapping {
  source_field: string;
  target_field: string;
  transformation?: string;
}

// ============================================
// Catalog Types
// ============================================

export interface CatalogDetectionCoverage {
  mitre_techniques: string[];
  mitre_tactics: string[];
  data_sources: string[];
}

export interface CatalogLineageRecord {
  source_system: string;
  source_table: string;
  ingestion_timestamp: string;
}

export interface CatalogFieldLineage {
  source_field: string;
  target_ocsf_field: string;
  transformation?: string;
}

export interface CatalogEntry {
  id?: number | null;
  entity_name: string;
  caption: string;
  description: string;
  ocsf_version: string;
  source_event_classes: number[];
  attributes: unknown[];
  relationships: unknown[];
  covers_observables: number[];
  detection_coverage?: CatalogDetectionCoverage;
  source_lineage: CatalogLineageRecord[];
  field_lineage: CatalogFieldLineage[];
  catalog_version: number;
  updated_at: string;
}

export interface PluginInfo {
  engine: string;
  connected: boolean;
}

export interface PushResult {
  engine: string;
  entries_pushed: number;
}

export interface PullResult {
  engine: string;
  entries_pulled: number;
  entries_merged: number;
}

export interface SyncDiff {
  engine: string;
  added: CatalogEntry[];
  modified: CatalogEntry[];
  removed: CatalogEntry[];
}

export interface SyncResult {
  engine: string;
  push: PushResult;
  pull: PullResult;
  conflicts_resolved: number;
}
