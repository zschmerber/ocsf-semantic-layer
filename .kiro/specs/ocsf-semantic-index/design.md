# Design Document: OCSF Semantic Index Layer

## Overview

The OCSF Semantic Index Layer (`ocsf-index` crate) provides a metadata management system that bridges the ETL pipeline and query engine. It tracks physical table locations, source-to-OCSF lineage, partition metadata, column statistics, and query result caching.

### Relationship to Semantic Model Metadata

The semantic index is distinct from the semantic model metadata in `ocsf-semantic`:

| Semantic Model (`ocsf-semantic`) | Semantic Index (`ocsf-index`) |
|----------------------------------|-------------------------------|
| What the data means | Where the data is |
| Entity definitions, attributes | Physical table locations |
| MITRE ATT&CK mappings | Source-to-target lineage |
| Security use cases | Field transformations |
| Query templates | Partition metadata |
| Threat relevance | Column statistics |

The semantic model defines the **schema and meaning** (including security context like MITRE techniques), while the semantic index tracks the **physical implementation** (where tables are, how data flows, what's cached).

The index integrates with the semantic model by:
1. Storing OCSF version per table (for version-aware queries)
2. Providing table locations for SQL generation
3. Enabling partition pruning based on semantic query filters

The design follows the existing project patterns:
- Trait-based abstractions for storage backends
- Builder pattern for complex struct construction
- Async operations with tokio
- Property-based testing with proptest
- Serialization with serde

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Semantic Index                                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │
│  │   Table     │ │   Source    │ │   Field     │ │  Partition  │       │
│  │  Registry   │ │  Lineage    │ │  Lineage    │ │  Metadata   │       │
│  │ + Detection │ │             │ │             │ │             │       │
│  │  Coverage   │ │             │ │             │ │             │       │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘       │
│         │               │               │               │               │
│  ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐       │
│  │   Table     │ │   Query     │ │  Lineage    │ │  Statistics │       │
│  │ Statistics  │ │   Cache     │ │  Capture    │ │  Collector  │       │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘       │
│         └───────────────┴───────────────┴───────────────┘               │
│                                 │                                        │
│                    ┌────────────┴────────────┐                          │
│                    │     Index Backend       │                          │
│                    │        (trait)          │                          │
│                    └────────────┬────────────┘                          │
│              ┌──────────────────┼──────────────────┐                    │
│              │                  │                  │                    │
│       ┌──────┴──────┐    ┌──────┴──────┐    ┌──────┴──────┐            │
│       │   SQLite    │    │ PostgreSQL  │    │  In-Memory  │            │
│       │   Backend   │    │   Backend   │    │   Backend   │            │
│       └─────────────┘    └─────────────┘    └─────────────┘            │
└─────────────────────────────────────────────────────────────────────────┘
         │                         │                         │
         ▼                         ▼                         ▼
┌─────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│  ocsf-warehouse │    │    ocsf-semantic    │    │     ocsf-editor     │
│ (ETL Integration│    │ (Query Integration) │    │  (API Integration)  │
│                 │    │                     │    │                     │
│ ETLGenerator ──►│    │◄── SqlGenerator     │    │◄── Lineage API      │
│ LineageCapture  │    │    QueryValidator   │    │    Coverage API     │
│                 │    │    QueryPlanHints   │    │    Statistics API   │
└─────────────────┘    └─────────────────────┘    └─────────────────────┘
                                │
                                ▼
                    ┌─────────────────────┐
                    │   Detection Gap     │
                    │     Analysis        │
                    │                     │
                    │ • MITRE Coverage    │
                    │ • Kill Chain Gaps   │
                    │ • Data Source Gaps  │
                    └─────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **Table Registry** | Physical table metadata, OCSF class mapping, detection coverage tracking |
| **Source Lineage** | Source-to-target table relationships, ingestion tracking |
| **Field Lineage** | Field-level transformations, version-aware mappings |
| **Partition Metadata** | Time bounds, row counts, partition pruning |
| **Table Statistics** | Cardinality, null rates, query optimization hints |
| **Query Cache** | Result caching, TTL management, LRU eviction |
| **Lineage Capture** | ETL integration, automatic lineage recording |
| **Detection Coverage** | MITRE ATT&CK mapping, kill chain coverage, gap analysis |
| **Index Backend** | Storage abstraction (SQLite, PostgreSQL, in-memory) |

### Data Flow

1. **ETL Time**: `ocsf-warehouse` → `LineageCapture` → `SemanticIndex` (records lineage, updates partitions)
2. **Query Time**: `ocsf-semantic` → `SemanticIndex` → `QueryPlanHints` (partition pruning, statistics)
3. **API Time**: `ocsf-editor` → `SemanticIndex` → JSON responses (lineage graphs, coverage reports)
4. **Analysis Time**: `SemanticIndex` → `DetectionCoverageSummary` (gap analysis, MITRE coverage)

## Components and Interfaces

### SemanticIndex (Main Entry Point)

The `SemanticIndex` struct is the primary interface for all index operations. It coordinates the various components and provides a unified API.

```rust
pub struct SemanticIndex<B: IndexBackend> {
    backend: B,
    table_registry: TableRegistry,
    source_lineage: SourceLineageStore,
    field_lineage: FieldLineageStore,
    partition_metadata: PartitionMetadataStore,
    table_statistics: TableStatisticsStore,
    query_cache: QueryCache,
    config: IndexConfig,
}

impl<B: IndexBackend> SemanticIndex<B> {
    // Construction
    pub async fn new(backend: B, config: IndexConfig) -> Result<Self>;
    pub async fn open(backend: B) -> Result<Self>;
    
    // Table Registry
    pub async fn register_table(&self, table: TableEntry) -> Result<TableId>;
    pub async fn deregister_table(&self, table_id: TableId) -> Result<()>;
    pub async fn get_tables_by_class(&self, class_uid: u32) -> Result<Vec<TableEntry>>;
    pub async fn get_table(&self, table_id: TableId) -> Result<Option<TableEntry>>;
    
    // Source Lineage
    pub async fn record_source_lineage(&self, lineage: SourceLineageRecord) -> Result<LineageId>;
    pub async fn get_source_lineage(&self, table_name: &str) -> Result<Vec<SourceLineageRecord>>;
    
    // Field Lineage
    pub async fn record_field_lineage(&self, lineage: FieldLineageRecord) -> Result<LineageId>;
    pub async fn get_field_lineage(&self, target_field: &str) -> Result<Vec<FieldLineageRecord>>;
    
    // Partition Metadata
    pub async fn update_partition(&self, partition: PartitionEntry) -> Result<()>;
    pub async fn get_partitions(&self, table_name: &str) -> Result<Vec<PartitionEntry>>;
    pub async fn get_partitions_in_range(&self, table_name: &str, start: DateTime, end: DateTime) -> Result<Vec<PartitionEntry>>;
    
    // Statistics
    pub async fn update_statistics(&self, stats: TableStatistics) -> Result<()>;
    pub async fn get_statistics(&self, table_name: &str) -> Result<Option<TableStatistics>>;
    pub async fn collect_statistics(&self, table_name: &str, sample_rate: f64) -> Result<TableStatistics>;
    
    // Query Cache
    pub async fn cache_query_result(&self, key: &QueryCacheKey, result: &QueryResult, ttl_secs: u64) -> Result<()>;
    pub async fn get_cached_result(&self, key: &QueryCacheKey) -> Result<Option<CachedResult>>;
    pub async fn invalidate_cache(&self, table_name: &str) -> Result<u64>;
    pub async fn get_cache_stats(&self) -> CacheStats;
    
    // ETL Integration
    pub fn begin_lineage_capture(&self, source_table: &str, target_table: &str) -> LineageCapture;
    
    // Query Integration
    pub async fn get_query_plan_hints(&self, query: &SemanticQuery) -> Result<QueryPlanHints>;
    
    // Detection Coverage Analysis
    pub async fn get_tables_by_mitre_technique(&self, technique: &str) -> Result<Vec<TableEntry>>;
    pub async fn get_tables_by_mitre_tactic(&self, tactic: &str) -> Result<Vec<TableEntry>>;
    pub async fn get_detection_coverage_summary(&self) -> Result<DetectionCoverageSummary>;
}
```

### IndexBackend Trait

The storage abstraction trait that all backends must implement:

```rust
#[async_trait]
pub trait IndexBackend: Send + Sync {
    // Generic CRUD operations
    async fn create<T: IndexRecord>(&self, record: &T) -> Result<RecordId>;
    async fn read<T: IndexRecord>(&self, id: RecordId) -> Result<Option<T>>;
    async fn update<T: IndexRecord>(&self, id: RecordId, record: &T) -> Result<()>;
    async fn delete(&self, id: RecordId) -> Result<()>;
    async fn list<T: IndexRecord>(&self, filter: &RecordFilter) -> Result<Vec<T>>;
    
    // Batch operations
    async fn batch_create<T: IndexRecord>(&self, records: &[T]) -> Result<Vec<RecordId>>;
    async fn batch_delete(&self, ids: &[RecordId]) -> Result<u64>;
    
    // Transaction support
    async fn begin_transaction(&self) -> Result<Transaction>;
    async fn commit(&self, tx: Transaction) -> Result<()>;
    async fn rollback(&self, tx: Transaction) -> Result<()>;
}

pub trait IndexRecord: Serialize + DeserializeOwned + Send + Sync {
    fn record_type() -> &'static str;
    fn id(&self) -> Option<RecordId>;
}
```

### TableRegistry

Manages physical table metadata:

```rust
pub struct TableRegistry {
    // Internal state managed by SemanticIndex
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableEntry {
    pub id: Option<TableId>,
    pub table_name: String,
    pub schema_name: Option<String>,
    pub class_uid: u32,
    pub ocsf_version: String,
    pub dialect: WarehouseDialect,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
    
    // Security detection coverage metadata
    pub detection_coverage: Option<DetectionCoverage>,
}

/// Security detection coverage metadata for a table.
/// Tracks which detection capabilities are supported by the data in this table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DetectionCoverage {
    /// MITRE ATT&CK technique IDs covered (e.g., "T1071.004", "T1110.003")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mitre_techniques: Vec<String>,
    
    /// MITRE ATT&CK tactics covered (e.g., "credential-access", "lateral-movement")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mitre_tactics: Vec<String>,
    
    /// Data sources available (e.g., "process_creation", "network_connection", "file_access")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_sources: Vec<String>,
    
    /// Detection rule IDs that use this table (Sigma, Elastic, Splunk rule references)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detection_rules: Vec<String>,
    
    /// Kill chain phases covered (e.g., "reconnaissance", "weaponization", "delivery")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<String>,
    
    /// Confidence level for detection (low, medium, high)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_level: Option<ConfidenceLevel>,
    
    /// Severity of threats detectable with this data (informational, low, medium, high, critical)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_severity: Option<Severity>,
    
    /// Traffic Light Protocol classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlp: Option<TLPLevel>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TLPLevel {
    Clear,
    Green,
    Amber,
    AmberStrict,
    Red,
}

impl TableEntry {
    pub fn new(table_name: impl Into<String>, class_uid: u32) -> Self;
    pub fn with_schema(self, schema: impl Into<String>) -> Self;
    pub fn with_ocsf_version(self, version: impl Into<String>) -> Self;
    pub fn with_dialect(self, dialect: WarehouseDialect) -> Self;
    pub fn with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    pub fn with_detection_coverage(self, coverage: DetectionCoverage) -> Self;
}

impl DetectionCoverage {
    pub fn new() -> Self;
    pub fn with_mitre_techniques(self, techniques: Vec<String>) -> Self;
    pub fn with_mitre_tactics(self, tactics: Vec<String>) -> Self;
    pub fn with_data_sources(self, sources: Vec<String>) -> Self;
    pub fn with_detection_rules(self, rules: Vec<String>) -> Self;
    pub fn with_kill_chain_phases(self, phases: Vec<String>) -> Self;
    pub fn with_confidence(self, level: ConfidenceLevel) -> Self;
    pub fn with_severity(self, severity: Severity) -> Self;
    pub fn with_tlp(self, tlp: TLPLevel) -> Self;
    
    /// Returns true if this coverage includes the given MITRE technique
    pub fn covers_technique(&self, technique: &str) -> bool;
    
    /// Returns true if this coverage includes the given tactic
    pub fn covers_tactic(&self, tactic: &str) -> bool;
}
```

### SourceLineageStore

Tracks source-to-target table relationships:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceLineageRecord {
    pub id: Option<LineageId>,
    pub source_system: String,
    pub source_table: String,
    pub target_table: String,
    pub ingestion_timestamp: DateTime<Utc>,
    pub record_count: Option<u64>,
    pub metadata: HashMap<String, String>,
}

impl SourceLineageRecord {
    pub fn new(source_system: impl Into<String>, source_table: impl Into<String>, target_table: impl Into<String>) -> Self;
    pub fn with_record_count(self, count: u64) -> Self;
    pub fn with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

// For visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub source: String,
    pub target: String,
    pub timestamp: DateTime<Utc>,
    pub record_count: Option<u64>,
}

impl From<SourceLineageRecord> for LineageEdge {
    fn from(record: SourceLineageRecord) -> Self;
}
```

### FieldLineageStore

Tracks field-level transformations:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldLineageRecord {
    pub id: Option<LineageId>,
    pub source_lineage_id: LineageId,
    pub source_field: String,
    pub target_field: String,
    pub transformation: Option<String>,
    pub ocsf_version: Option<String>,
}

impl FieldLineageRecord {
    pub fn new(source_lineage_id: LineageId, source_field: impl Into<String>, target_field: impl Into<String>) -> Self;
    pub fn with_transformation(self, expr: impl Into<String>) -> Self;
    pub fn with_ocsf_version(self, version: impl Into<String>) -> Self;
}

// For visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_path: String,
    pub target_path: String,
    pub transformation: Option<String>,
}

impl From<FieldLineageRecord> for FieldMapping {
    fn from(record: FieldLineageRecord) -> Self;
}
```

### PartitionMetadataStore

Tracks partition information for query optimization:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionEntry {
    pub id: Option<PartitionId>,
    pub table_name: String,
    pub partition_key: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub row_count: u64,
    pub size_bytes: u64,
    pub is_empty: bool,
    pub last_modified: DateTime<Utc>,
}

impl PartitionEntry {
    pub fn new(table_name: impl Into<String>, partition_key: impl Into<String>) -> Self;
    pub fn with_time_range(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;
    pub fn with_row_count(self, count: u64) -> Self;
    pub fn with_size_bytes(self, size: u64) -> Self;
    
    pub fn overlaps(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool;
}
```

### TableStatisticsStore

Tracks column statistics:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableStatistics {
    pub table_name: String,
    pub columns: Vec<ColumnStatistics>,
    pub total_rows: u64,
    pub sample_rate: f64,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnStatistics {
    pub column_name: String,
    pub distinct_count: u64,
    pub null_count: u64,
    pub total_count: u64,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
}

impl ColumnStatistics {
    pub fn null_rate(&self) -> f64;
    pub fn cardinality(&self) -> f64;
}

impl TableStatistics {
    pub fn new(table_name: impl Into<String>) -> Self;
    pub fn add_column(self, stats: ColumnStatistics) -> Self;
    pub fn with_sample_rate(self, rate: f64) -> Self;
    
    pub fn is_stale(&self, max_age_secs: u64) -> bool;
    pub fn get_column(&self, name: &str) -> Option<&ColumnStatistics>;
}
```

### QueryCache

Caches query results with TTL:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct QueryCacheKey {
    pub query_hash: String,
    pub table_names: Vec<String>,
}

impl QueryCacheKey {
    pub fn from_query(query: &SemanticQuery) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub key: QueryCacheKey,
    pub result: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub hit_count: u64,
}

impl CachedResult {
    pub fn is_expired(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub total_entries: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub total_size_bytes: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64;
}
```

### LineageCapture (ETL Integration)

Builder for capturing lineage during ETL:

```rust
pub struct LineageCapture {
    source_system: String,
    source_table: String,
    target_table: String,
    field_mappings: Vec<(String, String, Option<String>)>,
    record_count: Option<u64>,
    metadata: HashMap<String, String>,
}

impl LineageCapture {
    pub fn new(source_system: impl Into<String>, source_table: impl Into<String>, target_table: impl Into<String>) -> Self;
    
    pub fn add_field_mapping(self, source: impl Into<String>, target: impl Into<String>) -> Self;
    pub fn add_field_mapping_with_transform(self, source: impl Into<String>, target: impl Into<String>, transform: impl Into<String>) -> Self;
    pub fn with_record_count(self, count: u64) -> Self;
    pub fn with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    
    pub async fn finalize<B: IndexBackend>(self, index: &SemanticIndex<B>) -> Result<LineageId>;
}
```

### QueryPlanHints (Query Integration)

Hints for query optimization:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanHints {
    pub tables: Vec<TableHint>,
    pub partitions: Vec<PartitionHint>,
    pub statistics: Vec<TableStatistics>,
    pub cached_result: Option<CachedResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHint {
    pub table_name: String,
    pub schema_name: Option<String>,
    pub dialect: WarehouseDialect,
    pub ocsf_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionHint {
    pub table_name: String,
    pub partition_key: String,
    pub estimated_rows: u64,
}
```

## Data Models

### ID Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub u64);
```

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub cache_max_entries: usize,
    pub cache_default_ttl_secs: u64,
    pub statistics_sample_rate: f64,
    pub statistics_max_age_secs: u64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            cache_max_entries: 1000,
            cache_default_ttl_secs: 3600,
            statistics_sample_rate: 0.1,
            statistics_max_age_secs: 86400,
        }
    }
}
```

### Query Filters

```rust
#[derive(Debug, Clone, Default)]
pub struct RecordFilter {
    pub record_type: Option<String>,
    pub table_name: Option<String>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl RecordFilter {
    pub fn new() -> Self;
    pub fn with_table(self, table: impl Into<String>) -> Self;
    pub fn with_time_range(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;
    pub fn with_pagination(self, limit: usize, offset: usize) -> Self;
}
```

### Lineage Query Results

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageQueryResult {
    pub edges: Vec<LineageEdge>,
    pub total_count: u64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLineageQueryResult {
    pub mappings: Vec<FieldMapping>,
    pub total_count: u64,
    pub has_more: bool,
}
```

### Detection Coverage Summary

```rust
/// Summary of detection coverage across all indexed tables.
/// Useful for gap analysis and detection engineering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCoverageSummary {
    /// Total number of tables with detection coverage metadata
    pub tables_with_coverage: u64,
    
    /// All unique MITRE techniques covered across all tables
    pub mitre_techniques: Vec<MitreTechniqueCoverage>,
    
    /// All unique MITRE tactics covered across all tables
    pub mitre_tactics: Vec<MitreTacticCoverage>,
    
    /// All unique data sources available
    pub data_sources: Vec<DataSourceCoverage>,
    
    /// Coverage by kill chain phase
    pub kill_chain_coverage: Vec<KillChainCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitreTechniqueCoverage {
    pub technique_id: String,
    pub table_count: u64,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitreTacticCoverage {
    pub tactic: String,
    pub technique_count: u64,
    pub table_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceCoverage {
    pub data_source: String,
    pub table_count: u64,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillChainCoverage {
    pub phase: String,
    pub table_count: u64,
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following properties have been identified for property-based testing:

### Property 1: IndexRecord JSON Round-Trip

*For any* valid IndexRecord (TableEntry, SourceLineageRecord, FieldLineageRecord, PartitionEntry, TableStatistics, CachedResult), serializing to JSON and deserializing back SHALL produce an equivalent record.

**Validates: Requirements 1.6, 2.5, 3.6, 4.5, 5.5, 6.6**

### Property 2: Table Registration Class UID Validation

*For any* class_uid value, THE Table_Registry SHALL accept registration only when class_uid is a positive integer (> 0), and SHALL reject registration with an error when class_uid is zero or would overflow.

**Validates: Requirements 1.2**

### Property 3: Query By Class Returns All Tables

*For any* set of N tables registered with the same class_uid, querying by that class_uid SHALL return exactly N tables, and the returned set SHALL contain all originally registered tables.

**Validates: Requirements 1.3, 1.4**

### Property 4: Soft Delete Preserves Metadata

*For any* registered table, after deregistration, the table metadata SHALL still be retrievable with is_active set to false, and the original field values SHALL be preserved.

**Validates: Requirements 1.5**

### Property 5: Multiple Sources Per Target Table

*For any* target table with N source lineage records, querying lineage by target table SHALL return exactly N records, ordered by ingestion_timestamp ascending.

**Validates: Requirements 2.3, 2.4**

### Property 6: Field Lineage Multiple Mappings

*For any* target field with N field lineage records (from one-to-many or many-to-one mappings), querying field lineage by target field SHALL return all N source fields with their transformations.

**Validates: Requirements 3.3, 3.4, 3.5**

### Property 7: Partition Time Range Filtering

*For any* set of partitions and any time range [start, end], the partitions returned by get_partitions_in_range SHALL be exactly those partitions where partition.start_time < end AND partition.end_time > start (overlap condition).

**Validates: Requirements 4.3**

### Property 8: Empty Partition Handling

*For any* partition with row_count = 0, the partition SHALL have is_empty = true, and the partition metadata SHALL still be retrievable and contain valid time bounds.

**Validates: Requirements 4.4**

### Property 9: Statistics Freshness Tracking

*For any* TableStatistics, the collected_at timestamp SHALL be set to a time not in the future, and is_stale(max_age) SHALL return true if and only if (now - collected_at) > max_age.

**Validates: Requirements 5.3, 5.4**

### Property 10: Cache Key Normalization Idempotence

*For any* SemanticQuery, generating a QueryCacheKey twice SHALL produce identical keys, and two queries that differ only in whitespace or field ordering SHALL produce the same cache key.

**Validates: Requirements 6.4**

### Property 11: Cache LRU Eviction

*For any* cache with max_entries = N, after inserting N+M entries, the cache SHALL contain at most N entries, and the M least-recently-used entries SHALL have been evicted.

**Validates: Requirements 6.5**

### Property 12: Backend API Consistency

*For any* sequence of CRUD operations, executing the same operations on SQLite backend and in-memory backend SHALL produce equivalent results (same records returned, same errors raised).

**Validates: Requirements 7.4**

### Property 13: LineageCapture Persistence Round-Trip

*For any* LineageCapture with source_table, target_table, and field_mappings, after finalize(), querying source lineage and field lineage SHALL return records matching the captured data.

**Validates: Requirements 9.3**

### Property 14: Query Partition Pruning Correctness

*For any* SemanticQuery with time_range filter, the partitions returned by get_query_plan_hints SHALL all overlap with the query's time range, and no partition outside the range SHALL be included.

**Validates: Requirements 10.1**

### Property 15: Lineage Time Range Filtering

*For any* time range filter [start, end] on lineage queries, all returned LineageEdge records SHALL have timestamp >= start AND timestamp <= end.

**Validates: Requirements 12.3**

### Property 16: Lineage Pagination Correctness

*For any* lineage query with limit L and offset O, the returned results SHALL contain at most L records starting from the O-th record in the full result set, and has_more SHALL be true if and only if there are more records beyond offset + limit.

**Validates: Requirements 12.4**

## Error Handling

The crate uses `thiserror` for error definitions following the project conventions:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Table not found: {0}")]
    TableNotFound(String),
    
    #[error("Invalid class UID: {0}. Must be a positive integer.")]
    InvalidClassUid(u32),
    
    #[error("Lineage record not found: {0}")]
    LineageNotFound(LineageId),
    
    #[error("Partition not found: {table_name}/{partition_key}")]
    PartitionNotFound { table_name: String, partition_key: String },
    
    #[error("Statistics not found for table: {0}")]
    StatisticsNotFound(String),
    
    #[error("Cache key not found: {0}")]
    CacheKeyNotFound(String),
    
    #[error("Backend error: {0}")]
    BackendError(#[from] BackendError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid time range: start ({start}) must be before end ({end})")]
    InvalidTimeRange { start: String, end: String },
    
    #[error("Duplicate table registration: {0}")]
    DuplicateTable(String),
    
    #[error("Referential integrity violation: {0}")]
    ReferentialIntegrity(String),
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    
    #[error("PostgreSQL error: {0}")]
    PostgresError(String),
}

pub type IndexResult<T> = Result<T, IndexError>;
```

### Error Handling Patterns

1. **Validation Errors**: Return immediately on invalid input (e.g., invalid class_uid)
2. **Not Found Errors**: Return specific error types for missing records
3. **Backend Errors**: Wrap underlying database errors with context
4. **Referential Integrity**: Check foreign key relationships before operations

## Testing Strategy

### Unit Tests

Unit tests focus on specific examples and edge cases:

1. **TableEntry construction and validation**
   - Valid class_uid values (1, 100, u32::MAX)
   - Invalid class_uid (0)
   - Builder pattern chaining

2. **PartitionEntry overlap calculation**
   - Partition fully within range
   - Partition partially overlapping
   - Partition fully outside range
   - Edge cases: exact boundary matches

3. **CacheStats calculations**
   - Hit rate with various hit/miss counts
   - Division by zero handling (no requests)

4. **ColumnStatistics calculations**
   - Null rate calculation
   - Cardinality calculation
   - Edge cases: all nulls, no nulls

### Property-Based Tests

Property-based tests use `proptest` with minimum 100 iterations per property:

```rust
use proptest::prelude::*;

// Property 1: IndexRecord JSON Round-Trip
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: ocsf-semantic-index, Property 1: IndexRecord JSON Round-Trip
    #[test]
    fn table_entry_json_roundtrip(entry in arb_table_entry()) {
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TableEntry = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(entry, deserialized);
    }
}

// Property 7: Partition Time Range Filtering
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: ocsf-semantic-index, Property 7: Partition Time Range Filtering
    #[test]
    fn partition_range_filtering(
        partitions in prop::collection::vec(arb_partition_entry(), 0..20),
        range in arb_time_range()
    ) {
        let filtered = filter_partitions_by_range(&partitions, range.0, range.1);
        for partition in &filtered {
            prop_assert!(partition.overlaps(range.0, range.1));
        }
        for partition in &partitions {
            if partition.overlaps(range.0, range.1) {
                prop_assert!(filtered.contains(partition));
            }
        }
    }
}
```

### Test Generators

```rust
fn arb_table_entry() -> impl Strategy<Value = TableEntry> {
    (
        "[a-z][a-z0-9_]{2,20}",  // table_name
        1..u32::MAX,             // class_uid
        prop::option::of("[a-z][a-z0-9_]{2,10}"),  // schema_name
        arb_warehouse_dialect(),
        arb_datetime(),
    ).prop_map(|(name, class_uid, schema, dialect, created)| {
        let mut entry = TableEntry::new(name, class_uid)
            .with_dialect(dialect);
        if let Some(s) = schema {
            entry = entry.with_schema(s);
        }
        entry
    })
}

fn arb_partition_entry() -> impl Strategy<Value = PartitionEntry> {
    (
        "[a-z][a-z0-9_]{2,20}",  // table_name
        "[0-9]{4}-[0-9]{2}",     // partition_key (YYYY-MM format)
        arb_time_range(),
        0..1_000_000u64,         // row_count
        0..1_000_000_000u64,     // size_bytes
    ).prop_map(|(table, key, (start, end), rows, size)| {
        PartitionEntry::new(table, key)
            .with_time_range(start, end)
            .with_row_count(rows)
            .with_size_bytes(size)
    })
}

fn arb_time_range() -> impl Strategy<Value = (DateTime<Utc>, DateTime<Utc>)> {
    (0i64..1_000_000_000, 1i64..1_000_000)
        .prop_map(|(start_secs, duration_secs)| {
            let start = Utc.timestamp_opt(start_secs, 0).unwrap();
            let end = start + chrono::Duration::seconds(duration_secs);
            (start, end)
        })
}

fn arb_warehouse_dialect() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Postgres),
    ]
}
```

### Integration Tests

Integration tests verify component interactions:

1. **ETL Integration Flow**
   - Create LineageCapture → finalize → verify records persisted
   - Update partition metadata → verify statistics triggered

2. **Query Integration Flow**
   - Register tables → create partitions → query with time filter → verify pruning

3. **Cache Integration**
   - Store result → retrieve → verify hit
   - Exceed capacity → verify LRU eviction

4. **Backend Consistency**
   - Run same operations on SQLite and in-memory backends
   - Verify equivalent results

### Test Organization

```
ocsf-index/
├── src/
│   ├── lib.rs
│   ├── table_registry.rs
│   ├── source_lineage.rs
│   ├── field_lineage.rs
│   ├── partition_metadata.rs
│   ├── table_statistics.rs
│   ├── query_cache.rs
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── sqlite.rs
│   │   ├── postgres.rs
│   │   └── memory.rs
│   └── error.rs
├── tests/
│   ├── table_registry_test.rs
│   ├── lineage_test.rs
│   ├── partition_test.rs
│   ├── cache_test.rs
│   └── integration_test.rs
└── src/
    ├── table_registry_proptest.rs
    ├── lineage_proptest.rs
    ├── partition_proptest.rs
    └── cache_proptest.rs
```

