//! Index API endpoint handlers for the OCSF Semantic Model Editor.
//!
//! This module contains the HTTP handlers for all index-related API endpoints,
//! including table registry, lineage tracking, detection coverage, statistics,
//! and partition metadata.
//!
//! # Endpoints
//!
//! - `/api/index/tables` - Table registry CRUD operations
//! - `/api/index/lineage/source` - Source lineage tracking
//! - `/api/index/lineage/field` - Field lineage tracking
//! - `/api/index/lineage/graph` - Lineage graph for visualization
//! - `/api/index/coverage/*` - Detection coverage queries
//! - `/api/index/statistics/*` - Table statistics
//! - `/api/index/partitions/*` - Partition metadata
//! - `/api/index/export` - Export index model
//! - `/api/index/import` - Import index model

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::EditorApiError;
use crate::AppState;

// Import IndexBackend trait for backend operations
use ocsf_index::backend::IndexBackend;

// Re-export ocsf-index types for convenience
pub use ocsf_index::detection_coverage::{
    DataSourceCoverage, DetectionCoverageSummary, KillChainCoverage, MitreTacticCoverage,
    MitreTechniqueCoverage,
};
pub use ocsf_index::field_lineage::{FieldLineageRecord, FieldMapping};
pub use ocsf_index::partition_metadata::PartitionEntry;
pub use ocsf_index::source_lineage::{LineageEdge, SourceLineageRecord};
pub use ocsf_index::table_registry::{DetectionCoverage, TableEntry};
pub use ocsf_index::table_statistics::{ColumnStatistics, TableStatistics};
pub use ocsf_index::types::{LineageId, PartitionId, TableId};

// ============================================================================
// Table Registry Types (Requirement 2)
// ============================================================================

/// Request body for POST /api/index/tables endpoint.
///
/// Used to register a new table in the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterTableRequest {
    /// The physical table name in the warehouse.
    pub table_name: String,
    /// Optional schema name (e.g., "ocsf", "security").
    pub schema_name: Option<String>,
    /// OCSF event class UID that this table stores.
    pub class_uid: u32,
    /// OCSF schema version (e.g., "1.3.0", "1.4.0").
    pub ocsf_version: String,
    /// Warehouse dialect for SQL generation.
    pub dialect: String,
    /// Additional metadata key-value pairs.
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    /// Security detection coverage metadata.
    pub detection_coverage: Option<DetectionCoverageRequest>,
}

/// Detection coverage metadata in API requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCoverageRequest {
    /// MITRE ATT&CK technique IDs covered (e.g., "T1071.004", "T1110.003").
    #[serde(default)]
    pub mitre_techniques: Vec<String>,
    /// MITRE ATT&CK tactics covered (e.g., "credential-access", "lateral-movement").
    #[serde(default)]
    pub mitre_tactics: Vec<String>,
    /// Data sources available (e.g., "process_creation", "network_connection").
    #[serde(default)]
    pub data_sources: Vec<String>,
    /// Detection rule IDs that use this table.
    #[serde(default)]
    pub detection_rules: Vec<String>,
    /// Kill chain phases covered.
    #[serde(default)]
    pub kill_chain_phases: Vec<String>,
    /// Confidence level for detection (low, medium, high).
    pub confidence_level: Option<String>,
    /// Maximum severity of threats detectable.
    pub max_severity: Option<String>,
}

/// Response for table operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResponse {
    /// Unique identifier for this table entry.
    pub id: u64,
    /// The physical table name in the warehouse.
    pub table_name: String,
    /// Optional schema name.
    pub schema_name: Option<String>,
    /// OCSF event class UID.
    pub class_uid: u32,
    /// OCSF schema version.
    pub ocsf_version: String,
    /// Warehouse dialect.
    pub dialect: String,
    /// Timestamp when the table was registered.
    pub created_at: String,
    /// Timestamp when the table was last updated.
    pub updated_at: String,
    /// Whether the table is active.
    pub is_active: bool,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Detection coverage metadata.
    pub detection_coverage: Option<DetectionCoverageResponse>,
}

/// Detection coverage metadata in API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCoverageResponse {
    /// MITRE ATT&CK technique IDs covered.
    pub mitre_techniques: Vec<String>,
    /// MITRE ATT&CK tactics covered.
    pub mitre_tactics: Vec<String>,
    /// Data sources available.
    pub data_sources: Vec<String>,
    /// Detection rule IDs.
    pub detection_rules: Vec<String>,
    /// Kill chain phases covered.
    pub kill_chain_phases: Vec<String>,
    /// Confidence level.
    pub confidence_level: Option<String>,
    /// Maximum severity.
    pub max_severity: Option<String>,
    /// TLP classification.
    pub tlp: Option<String>,
}

// ============================================================================
// Lineage Types (Requirements 3, 4)
// ============================================================================

/// Request body for POST /api/index/lineage/source endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSourceLineageRequest {
    /// The source system name (e.g., "splunk", "elastic", "kafka").
    pub source_system: String,
    /// The source table or topic name.
    pub source_table: String,
    /// The target OCSF table name.
    pub target_table: String,
    /// Number of records ingested from this source.
    pub record_count: Option<u64>,
    /// Additional metadata key-value pairs.
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Response for lineage graph visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraphResponse {
    /// Nodes in the lineage graph.
    pub nodes: Vec<LineageNode>,
    /// Edges connecting nodes.
    pub edges: Vec<LineageEdgeResponse>,
}

/// A node in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    /// Unique identifier for this node.
    pub id: String,
    /// Type of node: "source" or "target".
    pub node_type: String,
    /// Display label for the node.
    pub label: String,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// An edge in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdgeResponse {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Timestamp of the lineage relationship.
    pub timestamp: String,
    /// Number of records in this lineage relationship.
    pub record_count: Option<u64>,
}

/// Request body for POST /api/index/lineage/field endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordFieldLineageRequest {
    /// The ID of the source lineage record this field mapping belongs to.
    pub source_lineage_id: u64,
    /// The source field path.
    pub source_field: String,
    /// The target OCSF field path.
    pub target_field: String,
    /// Optional SQL transformation expression.
    pub transformation: Option<String>,
    /// Optional OCSF schema version.
    pub ocsf_version: Option<String>,
}

/// Response for field lineage queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLineageResponse {
    /// Unique identifier for this field lineage record.
    pub id: u64,
    /// The ID of the source lineage record.
    pub source_lineage_id: u64,
    /// The source field path.
    pub source_field: String,
    /// The target OCSF field path.
    pub target_field: String,
    /// Optional transformation expression.
    pub transformation: Option<String>,
    /// Optional OCSF schema version.
    pub ocsf_version: Option<String>,
}

// ============================================================================
// Coverage Types (Requirement 5)
// ============================================================================

/// Response for MITRE ATT&CK coverage matrix visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMatrixResponse {
    /// Tactics in kill chain order.
    pub tactics: Vec<TacticColumn>,
    /// Techniques grouped by tactic.
    pub techniques_by_tactic: HashMap<String, Vec<TechniqueCell>>,
}

/// A tactic column in the coverage matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticColumn {
    /// Tactic ID (e.g., "credential-access").
    pub id: String,
    /// Display name for the tactic.
    pub name: String,
    /// Number of tables covering this tactic.
    pub table_count: u64,
}

/// A technique cell in the coverage matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechniqueCell {
    /// Technique ID (e.g., "T1071.004").
    pub id: String,
    /// Display name for the technique.
    pub name: String,
    /// Number of tables covering this technique.
    pub table_count: u64,
    /// Names of tables covering this technique.
    pub tables: Vec<String>,
    /// Whether this technique is covered (table_count > 0).
    pub is_covered: bool,
}

/// Summary response for detection coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCoverageSummaryResponse {
    /// Total number of tables with detection coverage metadata.
    pub tables_with_coverage: u64,
    /// All unique MITRE ATT&CK techniques covered.
    pub mitre_techniques: Vec<MitreTechniqueCoverage>,
    /// All unique MITRE ATT&CK tactics covered.
    pub mitre_tactics: Vec<MitreTacticCoverage>,
    /// All unique data sources available.
    pub data_sources: Vec<DataSourceCoverage>,
    /// Coverage by kill chain phase.
    pub kill_chain_coverage: Vec<KillChainCoverage>,
}

// ============================================================================
// Statistics Types (Requirement 6)
// ============================================================================

/// Response for table statistics queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStatisticsResponse {
    /// The name of the table.
    pub table_name: String,
    /// Statistics for each column.
    pub columns: Vec<ColumnStatisticsResponse>,
    /// Total number of rows in the table.
    pub total_rows: u64,
    /// Sample rate used when collecting statistics.
    pub sample_rate: f64,
    /// Timestamp when statistics were collected.
    pub collected_at: String,
    /// Whether the statistics are stale.
    pub is_stale: bool,
}

/// Statistics for a single column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatisticsResponse {
    /// The name of the column.
    pub column_name: String,
    /// Number of distinct values.
    pub distinct_count: u64,
    /// Number of null values.
    pub null_count: u64,
    /// Total number of values (rows).
    pub total_count: u64,
    /// Minimum value (as string).
    pub min_value: Option<String>,
    /// Maximum value (as string).
    pub max_value: Option<String>,
    /// Null rate (null_count / total_count).
    pub null_rate: f64,
    /// Cardinality (distinct_count / total_count).
    pub cardinality: f64,
}

/// Request body for POST /api/index/statistics/{table_name}/collect endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectStatisticsRequest {
    /// Sample rate for statistics collection (0.0 to 1.0).
    pub sample_rate: Option<f64>,
}

// ============================================================================
// Partition Types (Requirement 7)
// ============================================================================

/// Response for partition metadata queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionEntryResponse {
    /// Unique identifier for this partition.
    pub id: u64,
    /// The table name this partition belongs to.
    pub table_name: String,
    /// The partition key (e.g., "2024-01").
    pub partition_key: String,
    /// Start time of the partition's time range.
    pub start_time: String,
    /// End time of the partition's time range.
    pub end_time: String,
    /// Number of rows in this partition.
    pub row_count: u64,
    /// Size of the partition in bytes.
    pub size_bytes: u64,
    /// Whether the partition is empty.
    pub is_empty: bool,
    /// Timestamp when the partition was last modified.
    pub last_modified: String,
}

/// Request body for POST /api/index/partitions endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePartitionRequest {
    /// The table name this partition belongs to.
    pub table_name: String,
    /// The partition key (e.g., "2024-01").
    pub partition_key: String,
    /// Start time of the partition's time range (ISO 8601).
    pub start_time: String,
    /// End time of the partition's time range (ISO 8601).
    pub end_time: String,
    /// Number of rows in this partition.
    pub row_count: u64,
    /// Size of the partition in bytes.
    pub size_bytes: u64,
}

// ============================================================================
// Export/Import Types (Requirements 18, 19)
// ============================================================================

/// Export format for the index model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexModelExport {
    /// Export format version.
    pub version: String,
    /// Timestamp when the export was created.
    pub exported_at: String,
    /// All registered tables.
    pub tables: Vec<TableResponse>,
    /// All source lineage records.
    pub source_lineage: Vec<SourceLineageResponse>,
    /// All field lineage records.
    pub field_lineage: Vec<FieldLineageResponse>,
}

/// Source lineage record in export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLineageResponse {
    /// Unique identifier.
    pub id: u64,
    /// The source system name.
    pub source_system: String,
    /// The source table or topic name.
    pub source_table: String,
    /// The target OCSF table name.
    pub target_table: String,
    /// Timestamp when the data was ingested.
    pub ingestion_timestamp: String,
    /// Number of records ingested.
    pub record_count: Option<u64>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Combined export format for semantic and index models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedModelExport {
    /// The semantic model.
    pub semantic_model: serde_json::Value,
    /// The index model.
    pub index_model: IndexModelExport,
}

/// Result of an import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Whether the import was fully successful (no errors).
    pub success: bool,
    /// Number of tables successfully imported.
    pub tables_imported: u64,
    /// Number of source lineage records successfully imported.
    pub source_lineage_imported: u64,
    /// Number of field lineage records successfully imported.
    pub field_lineage_imported: u64,
    /// List of errors encountered during import (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Query parameters for GET /api/index/tables endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ListTablesParams {
    /// Filter by OCSF class UID.
    pub class_uid: Option<u32>,
    /// Filter by active status.
    pub is_active: Option<bool>,
}

/// Query parameters for GET /api/index/lineage/source endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ListSourceLineageParams {
    /// Filter by target table name.
    pub target_table: Option<String>,
    /// Filter by source system.
    pub source_system: Option<String>,
    /// Maximum number of records to return.
    pub limit: Option<u32>,
    /// Number of records to skip.
    pub offset: Option<u32>,
}

/// Query parameters for GET /api/index/lineage/graph endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct LineageGraphParams {
    /// Filter by target table name.
    pub target_table: Option<String>,
}

/// Query parameters for GET /api/index/lineage/field endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFieldLineageParams {
    /// Filter by target field path.
    pub target_field: Option<String>,
    /// Filter by source lineage ID.
    pub source_lineage_id: Option<u64>,
}

/// Query parameters for GET /api/index/partitions/{table_name} endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPartitionsParams {
    /// Filter by start time (ISO 8601).
    pub start_time: Option<String>,
    /// Filter by end time (ISO 8601).
    pub end_time: Option<String>,
}

/// Query parameters for GET /api/index/export endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ExportParams {
    /// If true, export combined semantic and index models.
    pub combined: Option<bool>,
}

// ============================================================================
// Router
// ============================================================================

/// Create the index API router with all endpoints.
///
/// This router provides endpoints for:
/// - Table registry management (Requirement 2)
/// - Source lineage tracking (Requirement 3)
/// - Field lineage tracking (Requirement 4)
/// - Detection coverage queries (Requirement 5)
/// - Table statistics (Requirement 6)
/// - Partition metadata (Requirement 7)
/// - Export/Import (Requirements 18, 19)
pub fn create_index_router() -> Router<AppState> {
    Router::new()
        // Table Registry endpoints (Requirement 2)
        .route("/tables", get(list_tables).post(register_table))
        .route(
            "/tables/:id",
            get(get_table).put(update_table).delete(deactivate_table),
        )
        // Source Lineage endpoints (Requirement 3)
        .route(
            "/lineage/source",
            get(list_source_lineage).post(record_source_lineage),
        )
        .route("/lineage/graph", get(get_lineage_graph))
        // Field Lineage endpoints (Requirement 4)
        .route(
            "/lineage/field",
            get(list_field_lineage).post(record_field_lineage),
        )
        // Detection Coverage endpoints (Requirement 5)
        .route("/coverage/summary", get(get_coverage_summary))
        .route("/coverage/techniques/:technique_id", get(get_tables_by_technique))
        .route("/coverage/tactics/:tactic", get(get_tables_by_tactic))
        .route("/coverage/matrix", get(get_coverage_matrix))
        // Statistics endpoints (Requirement 6)
        .route("/statistics/:table_name", get(get_statistics))
        .route("/statistics/:table_name/collect", post(collect_statistics))
        // Partition endpoints (Requirement 7)
        .route("/partitions/:table_name", get(list_partitions))
        .route("/partitions", post(update_partition))
        // Export/Import endpoints (Requirements 18, 19)
        .route("/export", get(export_index_model))
        .route("/import", post(import_index_model))
}

// ============================================================================
// Handler Stubs (to be implemented in subsequent tasks)
// ============================================================================

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse a dialect string into a WarehouseDialect.
fn parse_dialect(dialect: &str) -> Result<ocsf_index::types::WarehouseDialect, EditorApiError> {
    match dialect.to_lowercase().as_str() {
        "snowflake" => Ok(ocsf_index::types::WarehouseDialect::Snowflake),
        "databricks" => Ok(ocsf_index::types::WarehouseDialect::Databricks),
        "bigquery" => Ok(ocsf_index::types::WarehouseDialect::BigQuery),
        "postgres" => Ok(ocsf_index::types::WarehouseDialect::Postgres),
        _ => Err(EditorApiError::InvalidRequest(format!(
            "Invalid dialect: {}. Valid values: snowflake, databricks, bigquery, postgres",
            dialect
        ))),
    }
}

/// Convert a WarehouseDialect to a string.
fn dialect_to_string(dialect: ocsf_index::types::WarehouseDialect) -> String {
    match dialect {
        ocsf_index::types::WarehouseDialect::Snowflake => "snowflake".to_string(),
        ocsf_index::types::WarehouseDialect::Databricks => "databricks".to_string(),
        ocsf_index::types::WarehouseDialect::BigQuery => "bigquery".to_string(),
        ocsf_index::types::WarehouseDialect::Postgres => "postgres".to_string(),
    }
}

/// Convert a TableEntry to a TableResponse.
fn table_entry_to_response(entry: TableEntry) -> TableResponse {
    TableResponse {
        id: entry.id.map(|id| id.0).unwrap_or(0),
        table_name: entry.table_name,
        schema_name: entry.schema_name,
        class_uid: entry.class_uid,
        ocsf_version: entry.ocsf_version,
        dialect: dialect_to_string(entry.dialect),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        is_active: entry.is_active,
        metadata: entry.metadata,
        detection_coverage: entry.detection_coverage.map(detection_coverage_to_response),
    }
}

/// Convert a DetectionCoverage to a DetectionCoverageResponse.
fn detection_coverage_to_response(coverage: DetectionCoverage) -> DetectionCoverageResponse {
    DetectionCoverageResponse {
        mitre_techniques: coverage.mitre_techniques,
        mitre_tactics: coverage.mitre_tactics,
        data_sources: coverage.data_sources,
        detection_rules: coverage.detection_rules,
        kill_chain_phases: coverage.kill_chain_phases,
        confidence_level: coverage.confidence_level.map(|l| format!("{}", l)),
        max_severity: coverage.max_severity.map(|s| format!("{}", s)),
        tlp: coverage.tlp.map(|t| format!("{}", t)),
    }
}

/// Convert a DetectionCoverageRequest to a DetectionCoverage.
fn detection_coverage_from_request(request: DetectionCoverageRequest) -> DetectionCoverage {
    let mut coverage = DetectionCoverage::new()
        .with_mitre_techniques(request.mitre_techniques)
        .with_mitre_tactics(request.mitre_tactics)
        .with_data_sources(request.data_sources)
        .with_detection_rules(request.detection_rules)
        .with_kill_chain_phases(request.kill_chain_phases);

    if let Some(level) = request.confidence_level {
        if let Some(parsed) = parse_confidence_level(&level) {
            coverage = coverage.with_confidence(parsed);
        }
    }

    if let Some(severity) = request.max_severity {
        if let Some(parsed) = parse_severity(&severity) {
            coverage = coverage.with_severity(parsed);
        }
    }

    coverage
}

/// Parse a confidence level string.
fn parse_confidence_level(s: &str) -> Option<ocsf_index::types::ConfidenceLevel> {
    match s.to_lowercase().as_str() {
        "low" => Some(ocsf_index::types::ConfidenceLevel::Low),
        "medium" => Some(ocsf_index::types::ConfidenceLevel::Medium),
        "high" => Some(ocsf_index::types::ConfidenceLevel::High),
        _ => None,
    }
}

/// Parse a severity string.
fn parse_severity(s: &str) -> Option<ocsf_index::types::Severity> {
    match s.to_lowercase().as_str() {
        "informational" => Some(ocsf_index::types::Severity::Informational),
        "low" => Some(ocsf_index::types::Severity::Low),
        "medium" => Some(ocsf_index::types::Severity::Medium),
        "high" => Some(ocsf_index::types::Severity::High),
        "critical" => Some(ocsf_index::types::Severity::Critical),
        _ => None,
    }
}

/// Convert a SourceLineageRecord to a SourceLineageResponse.
fn source_lineage_to_response(record: SourceLineageRecord) -> SourceLineageResponse {
    SourceLineageResponse {
        id: record.id.map(|id| id.0).unwrap_or(0),
        source_system: record.source_system,
        source_table: record.source_table,
        target_table: record.target_table,
        ingestion_timestamp: record.ingestion_timestamp.to_rfc3339(),
        record_count: record.record_count,
        metadata: record.metadata,
    }
}

/// Convert a FieldLineageRecord to a FieldLineageResponse.
fn field_lineage_to_response(record: FieldLineageRecord) -> FieldLineageResponse {
    FieldLineageResponse {
        id: record.id.map(|id| id.0).unwrap_or(0),
        source_lineage_id: record.source_lineage_id.0,
        source_field: record.source_field,
        target_field: record.target_field,
        transformation: record.transformation,
        ocsf_version: record.ocsf_version,
    }
}

/// Convert a PartitionEntry to a PartitionEntryResponse.
fn partition_entry_to_response(entry: PartitionEntry) -> PartitionEntryResponse {
    PartitionEntryResponse {
        id: entry.id.map(|id| id.0).unwrap_or(0),
        table_name: entry.table_name,
        partition_key: entry.partition_key,
        start_time: entry.start_time.to_rfc3339(),
        end_time: entry.end_time.to_rfc3339(),
        row_count: entry.row_count,
        size_bytes: entry.size_bytes,
        is_empty: entry.is_empty,
        last_modified: entry.last_modified.to_rfc3339(),
    }
}

// ============================================================================
// Table Registry Handlers (Requirement 2)
// ============================================================================

/// POST /api/index/tables - Register a new table.
///
/// Requirement 2.1: Register table and return assigned TableId.
/// Requirement 2.7: Return 400 if class_uid is 0.
async fn register_table(
    State(state): State<AppState>,
    Json(request): Json<RegisterTableRequest>,
) -> Result<Json<TableResponse>, EditorApiError> {
    // Validate class_uid > 0 (Requirement 2.7)
    if request.class_uid == 0 {
        return Err(EditorApiError::InvalidRequest(
            "class_uid must be greater than 0".to_string(),
        ));
    }

    // Parse the dialect
    let dialect = parse_dialect(&request.dialect)?;

    // Build the TableEntry
    let mut table = TableEntry::new(&request.table_name, request.class_uid)
        .with_ocsf_version(&request.ocsf_version)
        .with_dialect(dialect);

    if let Some(schema) = &request.schema_name {
        table = table.with_schema(schema);
    }

    // Add metadata
    if let Some(metadata) = &request.metadata {
        for (key, value) in metadata {
            table = table.with_metadata(key, value);
        }
    }

    // Add detection coverage
    if let Some(coverage_req) = &request.detection_coverage {
        let coverage = detection_coverage_from_request(coverage_req.clone());
        table = table.with_detection_coverage(coverage);
    }

    // Register the table
    let table_id = state.semantic_index.register_table(table.clone()).await?;

    // Retrieve the registered table to get the full response with ID
    let registered = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound("Table not found after registration".to_string()))?;

    Ok(Json(table_entry_to_response(registered)))
}

/// GET /api/index/tables - List all registered tables.
///
/// Requirement 2.2: Return list of all active registered tables.
/// Requirement 2.4: Support filtering by class_uid.
async fn list_tables(
    State(state): State<AppState>,
    Query(params): Query<ListTablesParams>,
) -> Result<Json<Vec<TableResponse>>, EditorApiError> {
    // Get all tables, filtering by active status if specified
    let active_only = params.is_active.unwrap_or(true);
    let tables = state.semantic_index.list_tables(active_only).await?;

    // Filter by class_uid if provided
    let filtered: Vec<TableEntry> = if let Some(class_uid) = params.class_uid {
        tables
            .into_iter()
            .filter(|t| t.class_uid == class_uid)
            .collect()
    } else {
        tables
    };

    // Convert to response format
    let responses: Vec<TableResponse> = filtered
        .into_iter()
        .map(table_entry_to_response)
        .collect();

    Ok(Json(responses))
}

/// GET /api/index/tables/:id - Get a specific table by ID.
///
/// Requirement 2.3: Return TableEntry for the given ID.
async fn get_table(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<TableResponse>, EditorApiError> {
    let table_id = TableId::new(id);

    let table = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound(format!("Table with ID {} not found", id)))?;

    Ok(Json(table_entry_to_response(table)))
}

/// PUT /api/index/tables/:id - Update a table.
///
/// Requirement 2.6: Update table metadata.
/// Requirement 2.8: Return table entries with detection_coverage metadata when present.
async fn update_table(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(request): Json<RegisterTableRequest>,
) -> Result<Json<TableResponse>, EditorApiError> {
    let table_id = TableId::new(id);

    // Get the existing table
    let existing = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound(format!("Table with ID {} not found", id)))?;

    // Parse the dialect
    let dialect = parse_dialect(&request.dialect)?;

    // Build the updated TableEntry, preserving the ID and created_at
    let mut updated = TableEntry::new(&request.table_name, request.class_uid)
        .with_ocsf_version(&request.ocsf_version)
        .with_dialect(dialect)
        .with_id(table_id);

    // Preserve created_at from existing entry
    updated.created_at = existing.created_at;
    updated.is_active = existing.is_active;

    if let Some(schema) = &request.schema_name {
        updated = updated.with_schema(schema);
    }

    // Add metadata
    if let Some(metadata) = &request.metadata {
        for (key, value) in metadata {
            updated = updated.with_metadata(key, value);
        }
    }

    // Add detection coverage
    if let Some(coverage_req) = &request.detection_coverage {
        let coverage = detection_coverage_from_request(coverage_req.clone());
        updated = updated.with_detection_coverage(coverage);
    }

    // Update the table
    state.semantic_index.update_table(table_id, updated).await?;

    // Retrieve the updated table
    let result = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound(format!("Table with ID {} not found", id)))?;

    Ok(Json(table_entry_to_response(result)))
}

/// DELETE /api/index/tables/:id - Deactivate a table (soft delete).
///
/// Requirement 2.5: Soft-delete the table (set is_active to false).
async fn deactivate_table(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<TableResponse>, EditorApiError> {
    let table_id = TableId::new(id);

    // Verify the table exists
    let _existing = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound(format!("Table with ID {} not found", id)))?;

    // Deactivate the table
    state.semantic_index.deregister_table(table_id).await?;

    // Retrieve the deactivated table
    let result = state
        .semantic_index
        .get_table(table_id)
        .await?
        .ok_or_else(|| EditorApiError::NotFound(format!("Table with ID {} not found", id)))?;

    Ok(Json(table_entry_to_response(result)))
}

// ============================================================================
// Lineage Handlers (Requirements 3, 4) - Stubs
// ============================================================================

/// POST /api/index/lineage/source - Record source lineage.
///
/// Requirement 3.1: Record lineage and return assigned LineageId.
async fn record_source_lineage(
    State(state): State<AppState>,
    Json(request): Json<RecordSourceLineageRequest>,
) -> Result<Json<SourceLineageResponse>, EditorApiError> {
    // Build the SourceLineageRecord from the request
    let mut lineage = SourceLineageRecord::new(
        &request.source_system,
        &request.source_table,
        &request.target_table,
    );

    // Add optional record count
    if let Some(count) = request.record_count {
        lineage = lineage.with_record_count(count);
    }

    // Add optional metadata
    if let Some(metadata) = &request.metadata {
        for (key, value) in metadata {
            lineage = lineage.with_metadata(key, value);
        }
    }

    // Record the lineage in the semantic index
    let lineage_id = state.semantic_index.record_source_lineage(lineage.clone()).await?;

    // Build the response with the assigned ID
    Ok(Json(SourceLineageResponse {
        id: lineage_id.0,
        source_system: request.source_system,
        source_table: request.source_table,
        target_table: request.target_table,
        ingestion_timestamp: lineage.ingestion_timestamp.to_rfc3339(),
        record_count: request.record_count,
        metadata: request.metadata.unwrap_or_default(),
    }))
}

/// GET /api/index/lineage/source - List source lineage records.
///
/// Requirement 3.2, 3.3: Return lineage records with optional filters.
/// Requirement 3.4: Return records ordered by ingestion_timestamp ascending.
/// Requirement 3.6: Support pagination via limit and offset query parameters.
async fn list_source_lineage(
    State(state): State<AppState>,
    Query(params): Query<ListSourceLineageParams>,
) -> Result<Json<Vec<SourceLineageResponse>>, EditorApiError> {
    // Get all source lineage records from the backend
    // We need to use the backend directly since SemanticIndex.get_source_lineage
    // only filters by target_table
    let filter = ocsf_index::backend::RecordFilter::new();
    let all_records: Vec<SourceLineageRecord> = state
        .semantic_index
        .backend()
        .list(&filter)
        .await
        .map_err(ocsf_index::error::IndexError::from)?;

    // Filter by target_table if provided (Requirement 3.2)
    let filtered_by_target: Vec<SourceLineageRecord> = if let Some(target) = &params.target_table {
        all_records
            .into_iter()
            .filter(|r| r.target_table == *target)
            .collect()
    } else {
        all_records
    };

    // Filter by source_system if provided (Requirement 3.3)
    let filtered_by_source: Vec<SourceLineageRecord> = if let Some(source) = &params.source_system {
        filtered_by_target
            .into_iter()
            .filter(|r| r.source_system == *source)
            .collect()
    } else {
        filtered_by_target
    };

    // Sort by ingestion_timestamp ascending (Requirement 3.4)
    let mut sorted_records = filtered_by_source;
    sorted_records.sort_by(|a, b| a.ingestion_timestamp.cmp(&b.ingestion_timestamp));

    // Apply pagination (Requirement 3.6)
    let offset = params.offset.unwrap_or(0) as usize;
    let limit = params.limit.unwrap_or(100) as usize;

    let paginated_records: Vec<SourceLineageRecord> = sorted_records
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    // Convert to response format
    let responses: Vec<SourceLineageResponse> = paginated_records
        .into_iter()
        .map(source_lineage_to_response)
        .collect();

    Ok(Json(responses))
}

/// GET /api/index/lineage/graph - Get lineage as graph data.
///
/// Requirement 3.5: Return lineage data formatted for graph visualization.
async fn get_lineage_graph(
    State(state): State<AppState>,
    Query(params): Query<LineageGraphParams>,
) -> Result<Json<LineageGraphResponse>, EditorApiError> {
    use std::collections::HashSet;

    // Get all source lineage records from the backend
    let filter = ocsf_index::backend::RecordFilter::new();
    let all_records: Vec<SourceLineageRecord> = state
        .semantic_index
        .backend()
        .list(&filter)
        .await
        .map_err(ocsf_index::error::IndexError::from)?;

    // Filter by target_table if provided
    let lineage_records: Vec<SourceLineageRecord> = if let Some(target) = &params.target_table {
        all_records
            .into_iter()
            .filter(|r| r.target_table == *target)
            .collect()
    } else {
        all_records
    };

    // Build nodes and edges for graph visualization
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_sources: HashSet<String> = HashSet::new();
    let mut seen_targets: HashSet<String> = HashSet::new();

    for record in lineage_records {
        // Create source node ID as "system:table"
        let source_id = format!("{}:{}", record.source_system, record.source_table);

        // Add source node if not already seen
        if !seen_sources.contains(&source_id) {
            nodes.push(LineageNode {
                id: source_id.clone(),
                node_type: "source".to_string(),
                label: record.source_table.clone(),
                metadata: [("system".to_string(), record.source_system.clone())].into(),
            });
            seen_sources.insert(source_id.clone());
        }

        // Add target node if not already seen
        if !seen_targets.contains(&record.target_table) {
            nodes.push(LineageNode {
                id: record.target_table.clone(),
                node_type: "target".to_string(),
                label: record.target_table.clone(),
                metadata: HashMap::new(),
            });
            seen_targets.insert(record.target_table.clone());
        }

        // Add edge from source to target
        edges.push(LineageEdgeResponse {
            source: source_id,
            target: record.target_table,
            timestamp: record.ingestion_timestamp.to_rfc3339(),
            record_count: record.record_count,
        });
    }

    Ok(Json(LineageGraphResponse { nodes, edges }))
}

/// POST /api/index/lineage/field - Record field lineage.
///
/// Requirement 4.1: Record field lineage and return assigned LineageId.
async fn record_field_lineage(
    State(state): State<AppState>,
    Json(request): Json<RecordFieldLineageRequest>,
) -> Result<Json<FieldLineageResponse>, EditorApiError> {
    // Build the FieldLineageRecord from the request
    let mut lineage = FieldLineageRecord::new(
        LineageId::new(request.source_lineage_id),
        &request.source_field,
        &request.target_field,
    );

    // Add optional transformation expression
    if let Some(transformation) = &request.transformation {
        lineage = lineage.with_transformation(transformation);
    }

    // Add optional OCSF version
    if let Some(ocsf_version) = &request.ocsf_version {
        lineage = lineage.with_ocsf_version(ocsf_version);
    }

    // Record the field lineage in the semantic index
    let lineage_id = state.semantic_index.record_field_lineage(lineage).await?;

    // Build the response with the assigned ID
    Ok(Json(FieldLineageResponse {
        id: lineage_id.0,
        source_lineage_id: request.source_lineage_id,
        source_field: request.source_field,
        target_field: request.target_field,
        transformation: request.transformation,
        ocsf_version: request.ocsf_version,
    }))
}

/// GET /api/index/lineage/field - List field lineage records.
///
/// Requirement 4.2: Return field lineage records filtered by target_field.
/// Requirement 4.3: Return field lineage records filtered by source_lineage_id.
/// Requirement 4.4: Return transformation expressions when present.
async fn list_field_lineage(
    State(state): State<AppState>,
    Query(params): Query<ListFieldLineageParams>,
) -> Result<Json<Vec<FieldLineageResponse>>, EditorApiError> {
    // Get all field lineage records from the backend
    let filter = ocsf_index::backend::RecordFilter::new();
    let all_records: Vec<FieldLineageRecord> = state
        .semantic_index
        .backend()
        .list(&filter)
        .await
        .map_err(ocsf_index::error::IndexError::from)?;

    // Filter by target_field if provided (Requirement 4.2)
    let filtered_by_target: Vec<FieldLineageRecord> = if let Some(target) = &params.target_field {
        all_records
            .into_iter()
            .filter(|r| r.target_field == *target)
            .collect()
    } else {
        all_records
    };

    // Filter by source_lineage_id if provided (Requirement 4.3)
    let filtered_by_source: Vec<FieldLineageRecord> = if let Some(source_id) = params.source_lineage_id {
        filtered_by_target
            .into_iter()
            .filter(|r| r.source_lineage_id.0 == source_id)
            .collect()
    } else {
        filtered_by_target
    };

    // Convert to response format, including transformation expressions (Requirement 4.4)
    let responses: Vec<FieldLineageResponse> = filtered_by_source
        .into_iter()
        .map(field_lineage_to_response)
        .collect();

    Ok(Json(responses))
}

/// GET /api/index/coverage/summary - Get detection coverage summary.
///
/// Requirement 5.1: Return aggregated coverage across all tables.
/// Requirement 5.4: Include mitre_techniques, mitre_tactics, data_sources, and kill_chain_coverage.
async fn get_coverage_summary(
    State(state): State<AppState>,
) -> Result<Json<DetectionCoverageSummaryResponse>, EditorApiError> {
    // Get the detection coverage summary from the semantic index
    let summary = state.semantic_index.get_detection_coverage_summary().await?;

    // Convert to response format
    Ok(Json(DetectionCoverageSummaryResponse {
        tables_with_coverage: summary.tables_with_coverage,
        mitre_techniques: summary.mitre_techniques,
        mitre_tactics: summary.mitre_tactics,
        data_sources: summary.data_sources,
        kill_chain_coverage: summary.kill_chain_coverage,
    }))
}

/// GET /api/index/coverage/techniques/:technique_id - Get tables by technique.
///
/// Requirement 5.2: Return all tables covering the given MITRE technique.
/// Requirement 5.6: Include table_count and tables list for each technique.
async fn get_tables_by_technique(
    State(state): State<AppState>,
    Path(technique_id): Path<String>,
) -> Result<Json<Vec<TableResponse>>, EditorApiError> {
    // Get all active tables with detection coverage
    let all_tables = state.semantic_index.list_tables(true).await?;

    // Filter tables that have the given technique_id in their mitre_techniques
    let matching_tables: Vec<TableResponse> = all_tables
        .into_iter()
        .filter(|table| {
            if let Some(coverage) = &table.detection_coverage {
                coverage.mitre_techniques.contains(&technique_id)
            } else {
                false
            }
        })
        .map(table_entry_to_response)
        .collect();

    Ok(Json(matching_tables))
}

/// GET /api/index/coverage/tactics/:tactic - Get tables by tactic.
///
/// Requirement 5.3: Return all tables covering the given MITRE tactic.
/// Requirement 5.6: Include table_count and tables list for each tactic.
async fn get_tables_by_tactic(
    State(state): State<AppState>,
    Path(tactic): Path<String>,
) -> Result<Json<Vec<TableResponse>>, EditorApiError> {
    // Get all active tables with detection coverage
    let all_tables = state.semantic_index.list_tables(true).await?;

    // Filter tables that have the given tactic in their mitre_tactics
    let matching_tables: Vec<TableResponse> = all_tables
        .into_iter()
        .filter(|table| {
            if let Some(coverage) = &table.detection_coverage {
                coverage.mitre_tactics.contains(&tactic)
            } else {
                false
            }
        })
        .map(table_entry_to_response)
        .collect();

    Ok(Json(matching_tables))
}

/// GET /api/index/coverage/matrix - Get MITRE ATT&CK coverage matrix.
///
/// Requirement 5.5: Return coverage data formatted for matrix visualization.
/// Requirement 5.6: Include table_count and tables list for each technique and tactic.
async fn get_coverage_matrix(
    State(state): State<AppState>,
) -> Result<Json<CoverageMatrixResponse>, EditorApiError> {
    // Get the detection coverage summary from the semantic index
    let summary = state.semantic_index.get_detection_coverage_summary().await?;

    // Build tactics list from the summary
    let tactics: Vec<TacticColumn> = summary
        .mitre_tactics
        .iter()
        .map(|t| TacticColumn {
            id: t.tactic.clone(),
            name: format_tactic_name(&t.tactic),
            table_count: t.table_count,
        })
        .collect();

    // Build techniques_by_tactic map
    // We need to associate each technique with its tactic(s)
    let mut techniques_by_tactic: HashMap<String, Vec<TechniqueCell>> = HashMap::new();

    // Initialize empty vectors for all tactics
    for tactic in &summary.mitre_tactics {
        techniques_by_tactic.insert(tactic.tactic.clone(), Vec::new());
    }

    // For each technique, determine which tactic(s) it belongs to
    // We use the technique ID prefix to infer the tactic, or we can look at
    // which tables have both the technique and tactic
    for technique in &summary.mitre_techniques {
        let tactic = get_tactic_for_technique(&technique.technique_id);
        let cell = TechniqueCell {
            id: technique.technique_id.clone(),
            name: format_technique_name(&technique.technique_id),
            table_count: technique.table_count,
            tables: technique.tables.clone(),
            is_covered: technique.table_count > 0,
        };
        techniques_by_tactic
            .entry(tactic)
            .or_default()
            .push(cell);
    }

    Ok(Json(CoverageMatrixResponse {
        tactics,
        techniques_by_tactic,
    }))
}

/// Format a tactic ID into a human-readable name.
///
/// Converts kebab-case tactic IDs like "credential-access" to
/// title case names like "Credential Access".
fn format_tactic_name(tactic_id: &str) -> String {
    tactic_id
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a technique ID into a human-readable name.
///
/// For now, this just returns the technique ID as the name.
/// In a full implementation, this would look up the technique name
/// from a MITRE ATT&CK database.
fn format_technique_name(technique_id: &str) -> String {
    // Return the technique ID as the name for now
    // A full implementation would look up the actual technique name
    technique_id.to_string()
}

/// Get the tactic for a given technique ID.
///
/// This uses a simplified mapping based on common MITRE ATT&CK technique prefixes.
/// In a full implementation, this would look up the tactic from a MITRE ATT&CK database.
fn get_tactic_for_technique(technique_id: &str) -> String {
    // MITRE ATT&CK technique IDs follow patterns like T1xxx or T1xxx.xxx
    // We use a simplified mapping based on common technique ranges
    // This is a heuristic - a full implementation would use the actual MITRE data
    
    // Extract the base technique number (e.g., "1071" from "T1071.004")
    let base_num = technique_id
        .trim_start_matches('T')
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // Simplified mapping based on MITRE ATT&CK technique ranges
    // These ranges are approximate and based on common patterns
    match base_num {
        1..=99 => "initial-access".to_string(),
        100..=199 => "execution".to_string(),
        200..=299 => "persistence".to_string(),
        300..=399 => "privilege-escalation".to_string(),
        400..=499 => "defense-evasion".to_string(),
        500..=599 => "credential-access".to_string(),
        600..=699 => "discovery".to_string(),
        700..=799 => "lateral-movement".to_string(),
        800..=899 => "collection".to_string(),
        1000..=1099 => "command-and-control".to_string(),
        1100..=1199 => "exfiltration".to_string(),
        1200..=1299 => "impact".to_string(),
        1500..=1599 => "reconnaissance".to_string(),
        1600..=1699 => "resource-development".to_string(),
        _ => "unknown".to_string(),
    }
}

/// GET /api/index/statistics/:table_name - Get table statistics.
///
/// Requirement 6.1: Return TableStatistics for the given table.
/// Requirement 6.3: Include column-level statistics with distinct_count, null_count, min_value, max_value.
/// Requirement 6.4: Include collected_at timestamp and is_stale indicator.
/// Requirement 6.5: Return 404 if statistics don't exist.
async fn get_statistics(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
) -> Result<Json<TableStatisticsResponse>, EditorApiError> {
    // Get statistics from the semantic index
    let stats = state
        .semantic_index
        .get_statistics(&table_name)
        .await?;

    // Return 404 if statistics don't exist (Requirement 6.5)
    let stats = stats.ok_or_else(|| {
        EditorApiError::NotFound(format!("Statistics not found for table: {}", table_name))
    })?;

    // Check if statistics are stale based on config (Requirement 6.4)
    let is_stale = stats.is_stale(state.semantic_index.config().statistics_max_age_secs);

    // Convert to response format with column-level statistics (Requirement 6.3)
    let columns: Vec<ColumnStatisticsResponse> = stats
        .columns
        .iter()
        .map(|col| ColumnStatisticsResponse {
            column_name: col.column_name.clone(),
            distinct_count: col.distinct_count,
            null_count: col.null_count,
            total_count: col.total_count,
            min_value: col.min_value.clone(),
            max_value: col.max_value.clone(),
            null_rate: col.null_rate(),
            cardinality: col.cardinality(),
        })
        .collect();

    Ok(Json(TableStatisticsResponse {
        table_name: stats.table_name,
        columns,
        total_rows: stats.total_rows,
        sample_rate: stats.sample_rate,
        collected_at: stats.collected_at.to_rfc3339(),
        is_stale,
    }))
}

/// POST /api/index/statistics/:table_name/collect - Collect table statistics.
///
/// Requirement 6.2: Trigger statistics collection for the given table.
/// Accepts optional sample_rate parameter.
async fn collect_statistics(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
    Json(request): Json<CollectStatisticsRequest>,
) -> Result<Json<TableStatisticsResponse>, EditorApiError> {
    // Use provided sample_rate or default from config
    let sample_rate = request
        .sample_rate
        .unwrap_or(state.semantic_index.config().statistics_sample_rate);

    // Validate sample_rate is in valid range
    if !(0.0..=1.0).contains(&sample_rate) {
        return Err(EditorApiError::InvalidRequest(
            "sample_rate must be between 0.0 and 1.0".to_string(),
        ));
    }

    // Call collect_statistics on the semantic index
    // Note: The collect_statistics method is currently a stub that returns an error
    // indicating it's not yet implemented. The error will be converted to an
    // appropriate HTTP response via the IndexError -> EditorApiError conversion.
    let stats = state
        .semantic_index
        .collect_statistics(&table_name, sample_rate)
        .await?;

    // Check if statistics are stale based on config
    let is_stale = stats.is_stale(state.semantic_index.config().statistics_max_age_secs);

    // Convert to response format
    let columns: Vec<ColumnStatisticsResponse> = stats
        .columns
        .iter()
        .map(|col| ColumnStatisticsResponse {
            column_name: col.column_name.clone(),
            distinct_count: col.distinct_count,
            null_count: col.null_count,
            total_count: col.total_count,
            min_value: col.min_value.clone(),
            max_value: col.max_value.clone(),
            null_rate: col.null_rate(),
            cardinality: col.cardinality(),
        })
        .collect();

    Ok(Json(TableStatisticsResponse {
        table_name: stats.table_name,
        columns,
        total_rows: stats.total_rows,
        sample_rate: stats.sample_rate,
        collected_at: stats.collected_at.to_rfc3339(),
        is_stale,
    }))
}

/// GET /api/index/partitions/:table_name - List partitions for a table.
///
/// Requirement 7.1: Return all PartitionEntry records for the table.
/// Requirement 7.2: Support start_time and end_time query parameters to filter
///                  partitions overlapping that time range.
/// Requirement 7.3: Include partition_key, start_time, end_time, row_count, size_bytes.
async fn list_partitions(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
    Query(params): Query<ListPartitionsParams>,
) -> Result<Json<Vec<PartitionEntryResponse>>, EditorApiError> {
    // Get partitions based on whether time range filter is provided
    let partitions = match (&params.start_time, &params.end_time) {
        (Some(start_str), Some(end_str)) => {
            // Parse ISO 8601 timestamps
            let start = chrono::DateTime::parse_from_rfc3339(start_str)
                .map_err(|e| {
                    EditorApiError::InvalidRequest(format!(
                        "Invalid start_time format (expected ISO 8601): {}",
                        e
                    ))
                })?
                .with_timezone(&chrono::Utc);

            let end = chrono::DateTime::parse_from_rfc3339(end_str)
                .map_err(|e| {
                    EditorApiError::InvalidRequest(format!(
                        "Invalid end_time format (expected ISO 8601): {}",
                        e
                    ))
                })?
                .with_timezone(&chrono::Utc);

            // Get partitions overlapping the time range (Requirement 7.2)
            state
                .semantic_index
                .get_partitions_in_range(&table_name, start, end)
                .await?
        }
        (Some(_), None) | (None, Some(_)) => {
            // Both start_time and end_time must be provided together
            return Err(EditorApiError::InvalidRequest(
                "Both start_time and end_time must be provided for time range filtering"
                    .to_string(),
            ));
        }
        (None, None) => {
            // Get all partitions for the table (Requirement 7.1)
            state.semantic_index.get_partitions(&table_name).await?
        }
    };

    // Convert to response format (Requirement 7.3)
    let responses: Vec<PartitionEntryResponse> = partitions
        .into_iter()
        .map(partition_entry_to_response)
        .collect();

    Ok(Json(responses))
}

/// POST /api/index/partitions - Update or create partition metadata.
///
/// Requirement 7.4: Update or create partition metadata.
/// Parses start_time and end_time as ISO 8601 timestamps.
async fn update_partition(
    State(state): State<AppState>,
    Json(request): Json<UpdatePartitionRequest>,
) -> Result<Json<PartitionEntryResponse>, EditorApiError> {
    // Parse ISO 8601 timestamps
    let start_time = chrono::DateTime::parse_from_rfc3339(&request.start_time)
        .map_err(|e| {
            EditorApiError::InvalidRequest(format!(
                "Invalid start_time format (expected ISO 8601): {}",
                e
            ))
        })?
        .with_timezone(&chrono::Utc);

    let end_time = chrono::DateTime::parse_from_rfc3339(&request.end_time)
        .map_err(|e| {
            EditorApiError::InvalidRequest(format!(
                "Invalid end_time format (expected ISO 8601): {}",
                e
            ))
        })?
        .with_timezone(&chrono::Utc);

    // Validate time range
    if end_time <= start_time {
        return Err(EditorApiError::InvalidRequest(
            "end_time must be after start_time".to_string(),
        ));
    }

    // Build the PartitionEntry
    let partition = PartitionEntry::new(&request.table_name, &request.partition_key)
        .with_time_range(start_time, end_time)
        .with_row_count(request.row_count)
        .with_size_bytes(request.size_bytes);

    // Update or create the partition (Requirement 7.4)
    state.semantic_index.update_partition(partition).await?;

    // Retrieve the updated partition to return the response
    // We need to find the partition we just created/updated
    let partitions = state
        .semantic_index
        .get_partitions(&request.table_name)
        .await?;

    let updated_partition = partitions
        .into_iter()
        .find(|p| p.partition_key == request.partition_key)
        .ok_or_else(|| {
            EditorApiError::NotFound("Partition not found after update".to_string())
        })?;

    Ok(Json(partition_entry_to_response(updated_partition)))
}

/// GET /api/index/export - Export the index model.
///
/// Requirement 18.1, 18.3: Export index model as JSON.
/// Requirement 18.4: Support combined=true query param for semantic+index bundle.
async fn export_index_model(
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> Result<Json<serde_json::Value>, EditorApiError> {
    use chrono::Utc;

    // Get all tables from the semantic index (including inactive for complete export)
    let tables = state.semantic_index.list_tables(false).await?;
    let table_responses: Vec<TableResponse> = tables
        .into_iter()
        .map(table_entry_to_response)
        .collect();

    // Get all source lineage records from the backend
    let source_filter = ocsf_index::backend::RecordFilter::new();
    let source_records: Vec<SourceLineageRecord> = state
        .semantic_index
        .backend()
        .list(&source_filter)
        .await
        .map_err(ocsf_index::error::IndexError::from)?;
    let source_lineage_responses: Vec<SourceLineageResponse> = source_records
        .into_iter()
        .map(source_lineage_to_response)
        .collect();

    // Get all field lineage records from the backend
    let field_filter = ocsf_index::backend::RecordFilter::new();
    let field_records: Vec<FieldLineageRecord> = state
        .semantic_index
        .backend()
        .list(&field_filter)
        .await
        .map_err(ocsf_index::error::IndexError::from)?;
    let field_lineage_responses: Vec<FieldLineageResponse> = field_records
        .into_iter()
        .map(field_lineage_to_response)
        .collect();

    // Build the index model export
    let index_model = IndexModelExport {
        version: "1.0.0".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        tables: table_responses,
        source_lineage: source_lineage_responses,
        field_lineage: field_lineage_responses,
    };

    // Check if combined export is requested (Requirement 18.4)
    if params.combined.unwrap_or(false) {
        // For combined export, we include a placeholder for the semantic model
        // The actual semantic model would be fetched from the semantic model store
        // For now, we return an empty object as a placeholder
        let combined = CombinedModelExport {
            semantic_model: serde_json::json!({}),
            index_model,
        };
        Ok(Json(serde_json::to_value(combined).map_err(|e| {
            EditorApiError::InvalidRequest(format!("Failed to serialize combined export: {}", e))
        })?))
    } else {
        Ok(Json(serde_json::to_value(index_model).map_err(|e| {
            EditorApiError::InvalidRequest(format!("Failed to serialize index export: {}", e))
        })?))
    }
}

/// POST /api/index/import - Import an index model.
///
/// Requirement 19.1, 19.2: Import index model from JSON.
/// Registers all tables, records all source lineage, and records all field lineage.
async fn import_index_model(
    State(state): State<AppState>,
    Json(model): Json<IndexModelExport>,
) -> Result<Json<ImportResult>, EditorApiError> {
    let mut tables_imported = 0u64;
    let mut source_lineage_imported = 0u64;
    let mut field_lineage_imported = 0u64;
    let mut errors: Vec<String> = Vec::new();

    // Import all tables (Requirement 19.2)
    for table_response in &model.tables {
        // Convert TableResponse back to TableEntry for registration
        let dialect = parse_dialect(&table_response.dialect)?;
        
        let mut table = TableEntry::new(&table_response.table_name, table_response.class_uid)
            .with_ocsf_version(&table_response.ocsf_version)
            .with_dialect(dialect);

        if let Some(schema) = &table_response.schema_name {
            table = table.with_schema(schema);
        }

        // Add metadata
        for (key, value) in &table_response.metadata {
            table = table.with_metadata(key, value);
        }

        // Add detection coverage if present
        if let Some(coverage_resp) = &table_response.detection_coverage {
            let mut coverage = DetectionCoverage::new()
                .with_mitre_techniques(coverage_resp.mitre_techniques.clone())
                .with_mitre_tactics(coverage_resp.mitre_tactics.clone())
                .with_data_sources(coverage_resp.data_sources.clone())
                .with_detection_rules(coverage_resp.detection_rules.clone())
                .with_kill_chain_phases(coverage_resp.kill_chain_phases.clone());

            if let Some(level) = &coverage_resp.confidence_level {
                if let Some(parsed) = parse_confidence_level(level) {
                    coverage = coverage.with_confidence(parsed);
                }
            }

            if let Some(severity) = &coverage_resp.max_severity {
                if let Some(parsed) = parse_severity(severity) {
                    coverage = coverage.with_severity(parsed);
                }
            }

            table = table.with_detection_coverage(coverage);
        }

        // Set active status
        if !table_response.is_active {
            table.deactivate();
        }

        // Register the table
        match state.semantic_index.register_table(table).await {
            Ok(_) => tables_imported += 1,
            Err(e) => errors.push(format!(
                "Failed to import table '{}': {}",
                table_response.table_name, e
            )),
        }
    }

    // Import all source lineage records (Requirement 19.2)
    for lineage_resp in &model.source_lineage {
        let mut lineage = SourceLineageRecord::new(
            &lineage_resp.source_system,
            &lineage_resp.source_table,
            &lineage_resp.target_table,
        );

        // Add record count if present
        if let Some(count) = lineage_resp.record_count {
            lineage = lineage.with_record_count(count);
        }

        // Add metadata
        for (key, value) in &lineage_resp.metadata {
            lineage = lineage.with_metadata(key, value);
        }

        // Record the source lineage
        match state.semantic_index.record_source_lineage(lineage).await {
            Ok(_) => source_lineage_imported += 1,
            Err(e) => errors.push(format!(
                "Failed to import source lineage '{}:{}' -> '{}': {}",
                lineage_resp.source_system,
                lineage_resp.source_table,
                lineage_resp.target_table,
                e
            )),
        }
    }

    // Import all field lineage records (Requirement 19.2)
    for lineage_resp in &model.field_lineage {
        let mut lineage = FieldLineageRecord::new(
            LineageId::new(lineage_resp.source_lineage_id),
            &lineage_resp.source_field,
            &lineage_resp.target_field,
        );

        // Add transformation if present
        if let Some(transformation) = &lineage_resp.transformation {
            lineage = lineage.with_transformation(transformation);
        }

        // Add OCSF version if present
        if let Some(ocsf_version) = &lineage_resp.ocsf_version {
            lineage = lineage.with_ocsf_version(ocsf_version);
        }

        // Record the field lineage
        match state.semantic_index.record_field_lineage(lineage).await {
            Ok(_) => field_lineage_imported += 1,
            Err(e) => errors.push(format!(
                "Failed to import field lineage '{}' -> '{}': {}",
                lineage_resp.source_field, lineage_resp.target_field, e
            )),
        }
    }

    Ok(Json(ImportResult {
        success: errors.is_empty(),
        tables_imported,
        source_lineage_imported,
        field_lineage_imported,
        errors: if errors.is_empty() { None } else { Some(errors) },
    }))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_table_request_serialization() {
        let request = RegisterTableRequest {
            table_name: "network_activity".to_string(),
            schema_name: Some("ocsf".to_string()),
            class_uid: 4001,
            ocsf_version: "1.3.0".to_string(),
            dialect: "snowflake".to_string(),
            metadata: Some([("owner".to_string(), "security-team".to_string())].into()),
            detection_coverage: Some(DetectionCoverageRequest {
                mitre_techniques: vec!["T1071.004".to_string()],
                mitre_tactics: vec!["command-and-control".to_string()],
                data_sources: vec!["network_connection".to_string()],
                detection_rules: vec![],
                kill_chain_phases: vec!["delivery".to_string()],
                confidence_level: Some("high".to_string()),
                max_severity: Some("critical".to_string()),
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RegisterTableRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.table_name, deserialized.table_name);
        assert_eq!(request.class_uid, deserialized.class_uid);
        assert!(deserialized.detection_coverage.is_some());
    }

    #[test]
    fn test_table_response_serialization() {
        let response = TableResponse {
            id: 1,
            table_name: "network_activity".to_string(),
            schema_name: Some("ocsf".to_string()),
            class_uid: 4001,
            ocsf_version: "1.3.0".to_string(),
            dialect: "snowflake".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            updated_at: "2024-01-15T10:30:00Z".to_string(),
            is_active: true,
            metadata: HashMap::new(),
            detection_coverage: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TableResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.table_name, deserialized.table_name);
    }

    #[test]
    fn test_lineage_graph_response_serialization() {
        let response = LineageGraphResponse {
            nodes: vec![
                LineageNode {
                    id: "splunk:raw_logs".to_string(),
                    node_type: "source".to_string(),
                    label: "raw_logs".to_string(),
                    metadata: [("system".to_string(), "splunk".to_string())].into(),
                },
                LineageNode {
                    id: "network_activity".to_string(),
                    node_type: "target".to_string(),
                    label: "network_activity".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            edges: vec![LineageEdgeResponse {
                source: "splunk:raw_logs".to_string(),
                target: "network_activity".to_string(),
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                record_count: Some(1000),
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LineageGraphResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.nodes.len(), deserialized.nodes.len());
        assert_eq!(response.edges.len(), deserialized.edges.len());
    }

    #[test]
    fn test_coverage_matrix_response_serialization() {
        let response = CoverageMatrixResponse {
            tactics: vec![TacticColumn {
                id: "credential-access".to_string(),
                name: "Credential Access".to_string(),
                table_count: 3,
            }],
            techniques_by_tactic: [(
                "credential-access".to_string(),
                vec![TechniqueCell {
                    id: "T1110.003".to_string(),
                    name: "Password Spraying".to_string(),
                    table_count: 2,
                    tables: vec!["auth_logs".to_string(), "ad_events".to_string()],
                    is_covered: true,
                }],
            )]
            .into(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CoverageMatrixResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.tactics.len(), deserialized.tactics.len());
        assert!(deserialized.techniques_by_tactic.contains_key("credential-access"));
    }

    #[test]
    fn test_statistics_response_serialization() {
        let response = TableStatisticsResponse {
            table_name: "network_activity".to_string(),
            columns: vec![ColumnStatisticsResponse {
                column_name: "src_ip".to_string(),
                distinct_count: 1000,
                null_count: 50,
                total_count: 10000,
                min_value: Some("10.0.0.1".to_string()),
                max_value: Some("192.168.255.255".to_string()),
                null_rate: 0.005,
                cardinality: 0.1,
            }],
            total_rows: 10000,
            sample_rate: 0.1,
            collected_at: "2024-01-15T10:30:00Z".to_string(),
            is_stale: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TableStatisticsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.table_name, deserialized.table_name);
        assert_eq!(response.columns.len(), deserialized.columns.len());
    }

    #[test]
    fn test_index_model_export_serialization() {
        let export = IndexModelExport {
            version: "1.0.0".to_string(),
            exported_at: "2024-01-15T10:30:00Z".to_string(),
            tables: vec![],
            source_lineage: vec![],
            field_lineage: vec![],
        };

        let json = serde_json::to_string(&export).unwrap();
        let deserialized: IndexModelExport = serde_json::from_str(&json).unwrap();

        assert_eq!(export.version, deserialized.version);
    }

    #[test]
    fn test_record_field_lineage_request_serialization() {
        let request = RecordFieldLineageRequest {
            source_lineage_id: 1,
            source_field: "src_ip".to_string(),
            target_field: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
            ocsf_version: Some("1.3.0".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RecordFieldLineageRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.source_lineage_id, deserialized.source_lineage_id);
        assert_eq!(request.source_field, deserialized.source_field);
        assert_eq!(request.target_field, deserialized.target_field);
        assert_eq!(request.transformation, deserialized.transformation);
        assert_eq!(request.ocsf_version, deserialized.ocsf_version);
    }

    #[test]
    fn test_record_field_lineage_request_minimal() {
        // Test with only required fields
        let json = r#"{
            "source_lineage_id": 42,
            "source_field": "timestamp_str",
            "target_field": "time"
        }"#;

        let request: RecordFieldLineageRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.source_lineage_id, 42);
        assert_eq!(request.source_field, "timestamp_str");
        assert_eq!(request.target_field, "time");
        assert!(request.transformation.is_none());
        assert!(request.ocsf_version.is_none());
    }

    #[test]
    fn test_field_lineage_response_serialization() {
        let response = FieldLineageResponse {
            id: 1,
            source_lineage_id: 10,
            source_field: "src_ip".to_string(),
            target_field: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
            ocsf_version: Some("1.3.0".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: FieldLineageResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.source_lineage_id, deserialized.source_lineage_id);
        assert_eq!(response.source_field, deserialized.source_field);
        assert_eq!(response.target_field, deserialized.target_field);
        assert_eq!(response.transformation, deserialized.transformation);
        assert_eq!(response.ocsf_version, deserialized.ocsf_version);
    }

    #[test]
    fn test_field_lineage_response_without_optional_fields() {
        let response = FieldLineageResponse {
            id: 1,
            source_lineage_id: 10,
            source_field: "src_ip".to_string(),
            target_field: "src_endpoint.ip".to_string(),
            transformation: None,
            ocsf_version: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: FieldLineageResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.id, deserialized.id);
        assert!(deserialized.transformation.is_none());
        assert!(deserialized.ocsf_version.is_none());
    }

    #[test]
    fn test_list_field_lineage_params_serialization() {
        // Test with target_field filter
        let params = ListFieldLineageParams {
            target_field: Some("src_endpoint.ip".to_string()),
            source_lineage_id: None,
        };

        assert_eq!(params.target_field, Some("src_endpoint.ip".to_string()));
        assert!(params.source_lineage_id.is_none());

        // Test with source_lineage_id filter
        let params2 = ListFieldLineageParams {
            target_field: None,
            source_lineage_id: Some(42),
        };

        assert!(params2.target_field.is_none());
        assert_eq!(params2.source_lineage_id, Some(42));

        // Test with both filters
        let params3 = ListFieldLineageParams {
            target_field: Some("time".to_string()),
            source_lineage_id: Some(10),
        };

        assert_eq!(params3.target_field, Some("time".to_string()));
        assert_eq!(params3.source_lineage_id, Some(10));
    }

    #[test]
    fn test_field_lineage_to_response_helper() {
        let record = FieldLineageRecord::new(
            LineageId::new(5),
            "event.user.name",
            "actor.user.name"
        )
        .with_transformation("UPPER(event.user.name)")
        .with_ocsf_version("1.4.0")
        .with_id(LineageId::new(100));

        let response = field_lineage_to_response(record);

        assert_eq!(response.id, 100);
        assert_eq!(response.source_lineage_id, 5);
        assert_eq!(response.source_field, "event.user.name");
        assert_eq!(response.target_field, "actor.user.name");
        assert_eq!(response.transformation, Some("UPPER(event.user.name)".to_string()));
        assert_eq!(response.ocsf_version, Some("1.4.0".to_string()));
    }

    #[test]
    fn test_field_lineage_to_response_without_id() {
        // Test when the record has no ID assigned yet
        let record = FieldLineageRecord::new(
            LineageId::new(1),
            "src_ip",
            "src_endpoint.ip"
        );

        let response = field_lineage_to_response(record);

        // ID should default to 0 when not set
        assert_eq!(response.id, 0);
        assert_eq!(response.source_lineage_id, 1);
        assert_eq!(response.source_field, "src_ip");
        assert_eq!(response.target_field, "src_endpoint.ip");
        assert!(response.transformation.is_none());
        assert!(response.ocsf_version.is_none());
    }

    // ========================================================================
    // Coverage Handler Helper Function Tests
    // ========================================================================

    #[test]
    fn test_format_tactic_name_single_word() {
        assert_eq!(format_tactic_name("execution"), "Execution");
        assert_eq!(format_tactic_name("discovery"), "Discovery");
        assert_eq!(format_tactic_name("impact"), "Impact");
    }

    #[test]
    fn test_format_tactic_name_multi_word() {
        assert_eq!(format_tactic_name("credential-access"), "Credential Access");
        assert_eq!(format_tactic_name("lateral-movement"), "Lateral Movement");
        assert_eq!(format_tactic_name("command-and-control"), "Command And Control");
        assert_eq!(format_tactic_name("initial-access"), "Initial Access");
        assert_eq!(format_tactic_name("defense-evasion"), "Defense Evasion");
        assert_eq!(format_tactic_name("privilege-escalation"), "Privilege Escalation");
    }

    #[test]
    fn test_format_tactic_name_empty() {
        assert_eq!(format_tactic_name(""), "");
    }

    #[test]
    fn test_format_technique_name() {
        // Currently just returns the technique ID
        assert_eq!(format_technique_name("T1071.004"), "T1071.004");
        assert_eq!(format_technique_name("T1110.003"), "T1110.003");
    }

    #[test]
    fn test_get_tactic_for_technique_command_and_control() {
        // T1071 is in the 1000-1099 range (command-and-control)
        assert_eq!(get_tactic_for_technique("T1071"), "command-and-control");
        assert_eq!(get_tactic_for_technique("T1071.004"), "command-and-control");
    }

    #[test]
    fn test_get_tactic_for_technique_credential_access() {
        // T1110 is in the 500-599 range (credential-access)
        // Note: This is a simplified mapping - actual MITRE mapping may differ
        assert_eq!(get_tactic_for_technique("T110"), "execution");
        assert_eq!(get_tactic_for_technique("T500"), "credential-access");
        assert_eq!(get_tactic_for_technique("T550.001"), "credential-access");
    }

    #[test]
    fn test_get_tactic_for_technique_various_ranges() {
        assert_eq!(get_tactic_for_technique("T50"), "initial-access");
        assert_eq!(get_tactic_for_technique("T150"), "execution");
        assert_eq!(get_tactic_for_technique("T250"), "persistence");
        assert_eq!(get_tactic_for_technique("T350"), "privilege-escalation");
        assert_eq!(get_tactic_for_technique("T450"), "defense-evasion");
        assert_eq!(get_tactic_for_technique("T650"), "discovery");
        assert_eq!(get_tactic_for_technique("T750"), "lateral-movement");
        assert_eq!(get_tactic_for_technique("T850"), "collection");
        assert_eq!(get_tactic_for_technique("T1150"), "exfiltration");
        assert_eq!(get_tactic_for_technique("T1250"), "impact");
        assert_eq!(get_tactic_for_technique("T1550"), "reconnaissance");
        assert_eq!(get_tactic_for_technique("T1650"), "resource-development");
    }

    #[test]
    fn test_get_tactic_for_technique_unknown() {
        // Invalid technique IDs should return "unknown"
        assert_eq!(get_tactic_for_technique("invalid"), "unknown");
        assert_eq!(get_tactic_for_technique("T9999"), "unknown");
    }

    #[test]
    fn test_detection_coverage_summary_response_serialization() {
        let response = DetectionCoverageSummaryResponse {
            tables_with_coverage: 5,
            mitre_techniques: vec![MitreTechniqueCoverage {
                technique_id: "T1071.004".to_string(),
                table_count: 2,
                tables: vec!["network_activity".to_string(), "dns_logs".to_string()],
            }],
            mitre_tactics: vec![MitreTacticCoverage {
                tactic: "command-and-control".to_string(),
                technique_count: 3,
                table_count: 2,
            }],
            data_sources: vec![DataSourceCoverage {
                data_source: "network_connection".to_string(),
                table_count: 3,
                tables: vec![
                    "network_activity".to_string(),
                    "firewall_logs".to_string(),
                    "proxy_logs".to_string(),
                ],
            }],
            kill_chain_coverage: vec![KillChainCoverage {
                phase: "delivery".to_string(),
                table_count: 2,
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: DetectionCoverageSummaryResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.tables_with_coverage, deserialized.tables_with_coverage);
        assert_eq!(response.mitre_techniques.len(), deserialized.mitre_techniques.len());
        assert_eq!(response.mitre_tactics.len(), deserialized.mitre_tactics.len());
        assert_eq!(response.data_sources.len(), deserialized.data_sources.len());
        assert_eq!(response.kill_chain_coverage.len(), deserialized.kill_chain_coverage.len());
    }

    #[test]
    fn test_tactic_column_serialization() {
        let column = TacticColumn {
            id: "credential-access".to_string(),
            name: "Credential Access".to_string(),
            table_count: 5,
        };

        let json = serde_json::to_string(&column).unwrap();
        let deserialized: TacticColumn = serde_json::from_str(&json).unwrap();

        assert_eq!(column.id, deserialized.id);
        assert_eq!(column.name, deserialized.name);
        assert_eq!(column.table_count, deserialized.table_count);
    }

    #[test]
    fn test_technique_cell_serialization() {
        let cell = TechniqueCell {
            id: "T1110.003".to_string(),
            name: "Password Spraying".to_string(),
            table_count: 2,
            tables: vec!["auth_logs".to_string(), "ad_events".to_string()],
            is_covered: true,
        };

        let json = serde_json::to_string(&cell).unwrap();
        let deserialized: TechniqueCell = serde_json::from_str(&json).unwrap();

        assert_eq!(cell.id, deserialized.id);
        assert_eq!(cell.name, deserialized.name);
        assert_eq!(cell.table_count, deserialized.table_count);
        assert_eq!(cell.tables, deserialized.tables);
        assert_eq!(cell.is_covered, deserialized.is_covered);
    }

    #[test]
    fn test_technique_cell_not_covered() {
        let cell = TechniqueCell {
            id: "T1234".to_string(),
            name: "Some Technique".to_string(),
            table_count: 0,
            tables: vec![],
            is_covered: false,
        };

        assert!(!cell.is_covered);
        assert_eq!(cell.table_count, 0);
        assert!(cell.tables.is_empty());
    }

    // ========================================================================
    // Statistics Handler Tests
    // ========================================================================

    #[test]
    fn test_collect_statistics_request_serialization() {
        let request = CollectStatisticsRequest {
            sample_rate: Some(0.1),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CollectStatisticsRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.sample_rate, deserialized.sample_rate);
    }

    #[test]
    fn test_collect_statistics_request_without_sample_rate() {
        let json = r#"{}"#;
        let request: CollectStatisticsRequest = serde_json::from_str(json).unwrap();

        assert!(request.sample_rate.is_none());
    }

    #[test]
    fn test_column_statistics_response_serialization() {
        let response = ColumnStatisticsResponse {
            column_name: "src_ip".to_string(),
            distinct_count: 1000,
            null_count: 50,
            total_count: 10000,
            min_value: Some("10.0.0.1".to_string()),
            max_value: Some("192.168.255.255".to_string()),
            null_rate: 0.005,
            cardinality: 0.1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ColumnStatisticsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.column_name, deserialized.column_name);
        assert_eq!(response.distinct_count, deserialized.distinct_count);
        assert_eq!(response.null_count, deserialized.null_count);
        assert_eq!(response.total_count, deserialized.total_count);
        assert_eq!(response.min_value, deserialized.min_value);
        assert_eq!(response.max_value, deserialized.max_value);
        assert!((response.null_rate - deserialized.null_rate).abs() < f64::EPSILON);
        assert!((response.cardinality - deserialized.cardinality).abs() < f64::EPSILON);
    }

    #[test]
    fn test_column_statistics_response_without_min_max() {
        let response = ColumnStatisticsResponse {
            column_name: "status".to_string(),
            distinct_count: 3,
            null_count: 0,
            total_count: 10000,
            min_value: None,
            max_value: None,
            null_rate: 0.0,
            cardinality: 0.0003,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ColumnStatisticsResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.min_value.is_none());
        assert!(deserialized.max_value.is_none());
    }

    #[test]
    fn test_table_statistics_response_with_multiple_columns() {
        let response = TableStatisticsResponse {
            table_name: "network_activity".to_string(),
            columns: vec![
                ColumnStatisticsResponse {
                    column_name: "src_ip".to_string(),
                    distinct_count: 1000,
                    null_count: 50,
                    total_count: 10000,
                    min_value: Some("10.0.0.1".to_string()),
                    max_value: Some("192.168.255.255".to_string()),
                    null_rate: 0.005,
                    cardinality: 0.1,
                },
                ColumnStatisticsResponse {
                    column_name: "dst_port".to_string(),
                    distinct_count: 65535,
                    null_count: 0,
                    total_count: 10000,
                    min_value: Some("1".to_string()),
                    max_value: Some("65535".to_string()),
                    null_rate: 0.0,
                    cardinality: 6.5535,
                },
            ],
            total_rows: 10000,
            sample_rate: 0.1,
            collected_at: "2024-01-15T10:30:00Z".to_string(),
            is_stale: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TableStatisticsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.table_name, deserialized.table_name);
        assert_eq!(response.columns.len(), 2);
        assert_eq!(deserialized.columns.len(), 2);
        assert_eq!(deserialized.columns[0].column_name, "src_ip");
        assert_eq!(deserialized.columns[1].column_name, "dst_port");
    }

    #[test]
    fn test_table_statistics_response_stale_indicator() {
        let stale_response = TableStatisticsResponse {
            table_name: "old_table".to_string(),
            columns: vec![],
            total_rows: 1000,
            sample_rate: 1.0,
            collected_at: "2024-01-01T00:00:00Z".to_string(),
            is_stale: true,
        };

        assert!(stale_response.is_stale);

        let fresh_response = TableStatisticsResponse {
            table_name: "new_table".to_string(),
            columns: vec![],
            total_rows: 1000,
            sample_rate: 1.0,
            collected_at: "2024-01-15T10:30:00Z".to_string(),
            is_stale: false,
        };

        assert!(!fresh_response.is_stale);
    }

    #[test]
    fn test_table_statistics_response_empty_columns() {
        let response = TableStatisticsResponse {
            table_name: "empty_table".to_string(),
            columns: vec![],
            total_rows: 0,
            sample_rate: 1.0,
            collected_at: "2024-01-15T10:30:00Z".to_string(),
            is_stale: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TableStatisticsResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.columns.is_empty());
        assert_eq!(deserialized.total_rows, 0);
    }

    // ========================================================================
    // Partition Handler Tests (Requirement 7)
    // ========================================================================

    #[test]
    fn test_partition_entry_response_serialization() {
        let response = PartitionEntryResponse {
            id: 1,
            table_name: "network_activity".to_string(),
            partition_key: "2024-01".to_string(),
            start_time: "2024-01-01T00:00:00Z".to_string(),
            end_time: "2024-02-01T00:00:00Z".to_string(),
            row_count: 1_000_000,
            size_bytes: 500_000_000,
            is_empty: false,
            last_modified: "2024-01-15T10:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PartitionEntryResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.table_name, deserialized.table_name);
        assert_eq!(response.partition_key, deserialized.partition_key);
        assert_eq!(response.start_time, deserialized.start_time);
        assert_eq!(response.end_time, deserialized.end_time);
        assert_eq!(response.row_count, deserialized.row_count);
        assert_eq!(response.size_bytes, deserialized.size_bytes);
        assert_eq!(response.is_empty, deserialized.is_empty);
        assert_eq!(response.last_modified, deserialized.last_modified);
    }

    #[test]
    fn test_partition_entry_response_empty_partition() {
        let response = PartitionEntryResponse {
            id: 2,
            table_name: "network_activity".to_string(),
            partition_key: "2024-02".to_string(),
            start_time: "2024-02-01T00:00:00Z".to_string(),
            end_time: "2024-03-01T00:00:00Z".to_string(),
            row_count: 0,
            size_bytes: 0,
            is_empty: true,
            last_modified: "2024-02-01T00:00:00Z".to_string(),
        };

        assert!(response.is_empty);
        assert_eq!(response.row_count, 0);
        assert_eq!(response.size_bytes, 0);
    }

    #[test]
    fn test_update_partition_request_serialization() {
        let request = UpdatePartitionRequest {
            table_name: "network_activity".to_string(),
            partition_key: "2024-01".to_string(),
            start_time: "2024-01-01T00:00:00Z".to_string(),
            end_time: "2024-02-01T00:00:00Z".to_string(),
            row_count: 1_000_000,
            size_bytes: 500_000_000,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdatePartitionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.table_name, deserialized.table_name);
        assert_eq!(request.partition_key, deserialized.partition_key);
        assert_eq!(request.start_time, deserialized.start_time);
        assert_eq!(request.end_time, deserialized.end_time);
        assert_eq!(request.row_count, deserialized.row_count);
        assert_eq!(request.size_bytes, deserialized.size_bytes);
    }

    #[test]
    fn test_list_partitions_params_serialization() {
        // Test with time range filter
        let params = ListPartitionsParams {
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
            end_time: Some("2024-02-01T00:00:00Z".to_string()),
        };

        assert_eq!(params.start_time, Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(params.end_time, Some("2024-02-01T00:00:00Z".to_string()));

        // Test without time range filter
        let params_no_filter = ListPartitionsParams {
            start_time: None,
            end_time: None,
        };

        assert!(params_no_filter.start_time.is_none());
        assert!(params_no_filter.end_time.is_none());
    }

    #[test]
    fn test_partition_entry_to_response_helper() {
        use chrono::{Duration, Utc};
        use ocsf_index::types::PartitionId;

        let start = Utc::now();
        let end = start + Duration::hours(1);
        let modified = Utc::now();

        let entry = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1_000_000)
            .with_size_bytes(500_000_000)
            .with_last_modified(modified)
            .with_id(PartitionId::new(42));

        let response = partition_entry_to_response(entry);

        assert_eq!(response.id, 42);
        assert_eq!(response.table_name, "network_activity");
        assert_eq!(response.partition_key, "2024-01");
        assert_eq!(response.row_count, 1_000_000);
        assert_eq!(response.size_bytes, 500_000_000);
        assert!(!response.is_empty);
        // Verify timestamps are in RFC3339 format
        assert!(response.start_time.contains("T"));
        assert!(response.end_time.contains("T"));
        assert!(response.last_modified.contains("T"));
    }

    #[test]
    fn test_partition_entry_to_response_without_id() {
        use chrono::{Duration, Utc};

        let start = Utc::now();
        let end = start + Duration::hours(1);

        let entry = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(0);

        let response = partition_entry_to_response(entry);

        // ID should default to 0 when not set
        assert_eq!(response.id, 0);
        assert_eq!(response.table_name, "network_activity");
        assert_eq!(response.partition_key, "2024-01");
        assert_eq!(response.row_count, 0);
        assert!(response.is_empty);
    }

    #[test]
    fn test_partition_entry_to_response_preserves_all_fields() {
        use chrono::{Duration, Utc};
        use ocsf_index::types::PartitionId;

        let start = Utc::now();
        let end = start + Duration::days(30);
        let modified = start + Duration::days(15);

        let entry = PartitionEntry::new("process_activity", "2024-Q1")
            .with_time_range(start, end)
            .with_row_count(5_000_000)
            .with_size_bytes(2_500_000_000)
            .with_last_modified(modified)
            .with_id(PartitionId::new(100));

        let response = partition_entry_to_response(entry);

        assert_eq!(response.id, 100);
        assert_eq!(response.table_name, "process_activity");
        assert_eq!(response.partition_key, "2024-Q1");
        assert_eq!(response.row_count, 5_000_000);
        assert_eq!(response.size_bytes, 2_500_000_000);
        assert!(!response.is_empty);
    }

    // ========================================================================
    // Export/Import Handler Tests (Requirements 18, 19)
    // ========================================================================

    #[test]
    fn test_export_params_default() {
        let params = ExportParams { combined: None };
        assert!(params.combined.is_none());
    }

    #[test]
    fn test_export_params_combined_true() {
        let params = ExportParams { combined: Some(true) };
        assert_eq!(params.combined, Some(true));
    }

    #[test]
    fn test_export_params_combined_false() {
        let params = ExportParams { combined: Some(false) };
        assert_eq!(params.combined, Some(false));
    }

    #[test]
    fn test_import_result_success() {
        let result = ImportResult {
            success: true,
            tables_imported: 5,
            source_lineage_imported: 10,
            field_lineage_imported: 50,
            errors: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.success);
        assert_eq!(deserialized.tables_imported, 5);
        assert_eq!(deserialized.source_lineage_imported, 10);
        assert_eq!(deserialized.field_lineage_imported, 50);
        assert!(deserialized.errors.is_none());
    }

    #[test]
    fn test_import_result_with_errors() {
        let result = ImportResult {
            success: false,
            tables_imported: 3,
            source_lineage_imported: 5,
            field_lineage_imported: 20,
            errors: Some(vec![
                "Failed to import table 'test_table': duplicate".to_string(),
                "Failed to import lineage: invalid reference".to_string(),
            ]),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.success);
        assert_eq!(deserialized.tables_imported, 3);
        assert!(deserialized.errors.is_some());
        assert_eq!(deserialized.errors.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_import_result_errors_skipped_when_none() {
        let result = ImportResult {
            success: true,
            tables_imported: 1,
            source_lineage_imported: 1,
            field_lineage_imported: 1,
            errors: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        // Verify that "errors" field is not present in JSON when None
        assert!(!json.contains("errors"));
    }

    #[test]
    fn test_index_model_export_full() {
        let export = IndexModelExport {
            version: "1.0.0".to_string(),
            exported_at: "2024-01-15T10:30:00Z".to_string(),
            tables: vec![TableResponse {
                id: 1,
                table_name: "network_activity".to_string(),
                schema_name: Some("ocsf".to_string()),
                class_uid: 4001,
                ocsf_version: "1.3.0".to_string(),
                dialect: "snowflake".to_string(),
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T10:30:00Z".to_string(),
                is_active: true,
                metadata: HashMap::new(),
                detection_coverage: None,
            }],
            source_lineage: vec![SourceLineageResponse {
                id: 1,
                source_system: "splunk".to_string(),
                source_table: "raw_logs".to_string(),
                target_table: "network_activity".to_string(),
                ingestion_timestamp: "2024-01-15T10:30:00Z".to_string(),
                record_count: Some(1000),
                metadata: HashMap::new(),
            }],
            field_lineage: vec![FieldLineageResponse {
                id: 1,
                source_lineage_id: 1,
                source_field: "src_ip".to_string(),
                target_field: "src_endpoint.ip".to_string(),
                transformation: None,
                ocsf_version: Some("1.3.0".to_string()),
            }],
        };

        let json = serde_json::to_string(&export).unwrap();
        let deserialized: IndexModelExport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.tables.len(), 1);
        assert_eq!(deserialized.source_lineage.len(), 1);
        assert_eq!(deserialized.field_lineage.len(), 1);
        assert_eq!(deserialized.tables[0].table_name, "network_activity");
        assert_eq!(deserialized.source_lineage[0].source_system, "splunk");
        assert_eq!(deserialized.field_lineage[0].source_field, "src_ip");
    }

    #[test]
    fn test_index_model_export_empty() {
        let export = IndexModelExport {
            version: "1.0.0".to_string(),
            exported_at: "2024-01-15T10:30:00Z".to_string(),
            tables: vec![],
            source_lineage: vec![],
            field_lineage: vec![],
        };

        let json = serde_json::to_string(&export).unwrap();
        let deserialized: IndexModelExport = serde_json::from_str(&json).unwrap();

        assert!(deserialized.tables.is_empty());
        assert!(deserialized.source_lineage.is_empty());
        assert!(deserialized.field_lineage.is_empty());
    }

    #[test]
    fn test_combined_model_export_serialization() {
        let combined = CombinedModelExport {
            semantic_model: serde_json::json!({
                "entities": [],
                "metrics": []
            }),
            index_model: IndexModelExport {
                version: "1.0.0".to_string(),
                exported_at: "2024-01-15T10:30:00Z".to_string(),
                tables: vec![],
                source_lineage: vec![],
                field_lineage: vec![],
            },
        };

        let json = serde_json::to_string(&combined).unwrap();
        let deserialized: CombinedModelExport = serde_json::from_str(&json).unwrap();

        assert!(deserialized.semantic_model.is_object());
        assert_eq!(deserialized.index_model.version, "1.0.0");
    }
}
