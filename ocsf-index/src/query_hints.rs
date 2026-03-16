//! Query plan hints for the OCSF Semantic Index.
//!
//! This module provides types for query optimization hints that are returned
//! by the semantic index to help the query engine make better planning decisions.
//! These hints include table locations, partition information, statistics,
//! and cached results.

use serde::{Deserialize, Serialize};

use crate::query_cache::CachedResult;
use crate::table_statistics::TableStatistics;
use crate::types::WarehouseDialect;

// ============================================================================
// QueryPlanHints
// ============================================================================

/// Hints for query optimization based on index metadata.
///
/// `QueryPlanHints` provides information to the query engine for optimizing
/// query execution. This includes:
/// - Table locations and dialects for SQL generation
/// - Partition information for partition pruning
/// - Statistics for cost estimation
/// - Cached results to avoid re-execution
///
/// # Example
///
/// ```rust
/// use ocsf_index::query_hints::{QueryPlanHints, TableHint, PartitionHint};
/// use ocsf_index::table_statistics::TableStatistics;
/// use ocsf_semantic::WarehouseDialect;
///
/// let hints = QueryPlanHints::new()
///     .add_table(TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0"))
///     .add_partition(PartitionHint::new("network_activity", "2024-01", 1_000_000))
///     .add_statistics(TableStatistics::new("network_activity").with_total_rows(10_000_000));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanHints {
    /// Table hints with location and dialect information.
    pub tables: Vec<TableHint>,
    /// Partition hints for partition pruning.
    pub partitions: Vec<PartitionHint>,
    /// Table statistics for cost estimation.
    pub statistics: Vec<TableStatistics>,
    /// Cached result if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_result: Option<CachedResult>,
}

impl QueryPlanHints {
    /// Creates a new empty QueryPlanHints.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::QueryPlanHints;
    ///
    /// let hints = QueryPlanHints::new();
    /// assert!(hints.tables.is_empty());
    /// assert!(hints.partitions.is_empty());
    /// assert!(hints.statistics.is_empty());
    /// assert!(hints.cached_result.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            partitions: Vec::new(),
            statistics: Vec::new(),
            cached_result: None,
        }
    }

    /// Adds a table hint.
    ///
    /// # Arguments
    ///
    /// * `hint` - The table hint to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::{QueryPlanHints, TableHint};
    /// use ocsf_semantic::WarehouseDialect;
    ///
    /// let hints = QueryPlanHints::new()
    ///     .add_table(TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0"));
    /// assert_eq!(hints.tables.len(), 1);
    /// ```
    pub fn add_table(mut self, hint: TableHint) -> Self {
        self.tables.push(hint);
        self
    }

    /// Adds a partition hint.
    ///
    /// # Arguments
    ///
    /// * `hint` - The partition hint to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::{QueryPlanHints, PartitionHint};
    ///
    /// let hints = QueryPlanHints::new()
    ///     .add_partition(PartitionHint::new("network_activity", "2024-01", 1_000_000));
    /// assert_eq!(hints.partitions.len(), 1);
    /// ```
    pub fn add_partition(mut self, hint: PartitionHint) -> Self {
        self.partitions.push(hint);
        self
    }

    /// Adds table statistics.
    ///
    /// # Arguments
    ///
    /// * `stats` - The table statistics to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::QueryPlanHints;
    /// use ocsf_index::table_statistics::TableStatistics;
    ///
    /// let hints = QueryPlanHints::new()
    ///     .add_statistics(TableStatistics::new("network_activity").with_total_rows(10_000_000));
    /// assert_eq!(hints.statistics.len(), 1);
    /// ```
    pub fn add_statistics(mut self, stats: TableStatistics) -> Self {
        self.statistics.push(stats);
        self
    }

    /// Sets the cached result.
    ///
    /// # Arguments
    ///
    /// * `result` - The cached result to set
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::QueryPlanHints;
    /// use ocsf_index::query_cache::{QueryCacheKey, CachedResult};
    ///
    /// let key = QueryCacheKey::new("abc123", vec!["network_activity".to_string()]);
    /// let cached = CachedResult::new(key, serde_json::json!({"data": "test"}), 3600);
    ///
    /// let hints = QueryPlanHints::new()
    ///     .with_cached_result(cached);
    /// assert!(hints.cached_result.is_some());
    /// ```
    pub fn with_cached_result(mut self, result: CachedResult) -> Self {
        self.cached_result = Some(result);
        self
    }

    /// Returns true if a cached result is available and not expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::QueryPlanHints;
    ///
    /// let hints = QueryPlanHints::new();
    /// assert!(!hints.has_valid_cache());
    /// ```
    pub fn has_valid_cache(&self) -> bool {
        self.cached_result
            .as_ref()
            .map(|r| !r.is_expired())
            .unwrap_or(false)
    }

    /// Returns the total estimated rows across all partitions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::{QueryPlanHints, PartitionHint};
    ///
    /// let hints = QueryPlanHints::new()
    ///     .add_partition(PartitionHint::new("table", "2024-01", 1_000_000))
    ///     .add_partition(PartitionHint::new("table", "2024-02", 500_000));
    /// assert_eq!(hints.total_estimated_rows(), 1_500_000);
    /// ```
    pub fn total_estimated_rows(&self) -> u64 {
        self.partitions.iter().map(|p| p.estimated_rows).sum()
    }
}

impl Default for QueryPlanHints {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TableHint
// ============================================================================

/// Hint about a table's location and dialect for SQL generation.
///
/// `TableHint` provides information about where a table is located and
/// what SQL dialect should be used when generating queries against it.
///
/// # Example
///
/// ```rust
/// use ocsf_index::query_hints::TableHint;
/// use ocsf_semantic::WarehouseDialect;
///
/// let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0")
///     .with_schema("ocsf");
///
/// assert_eq!(hint.table_name, "network_activity");
/// assert_eq!(hint.schema_name, Some("ocsf".to_string()));
/// assert_eq!(hint.dialect, WarehouseDialect::Snowflake);
/// assert_eq!(hint.ocsf_version, "1.3.0");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableHint {
    /// The name of the table.
    pub table_name: String,
    /// The schema name (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,
    /// The warehouse dialect for SQL generation.
    pub dialect: WarehouseDialect,
    /// The OCSF schema version for this table.
    pub ocsf_version: String,
}

impl TableHint {
    /// Creates a new TableHint with the given table name, dialect, and OCSF version.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table
    /// * `dialect` - The warehouse dialect for SQL generation
    /// * `ocsf_version` - The OCSF schema version
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::TableHint;
    /// use ocsf_semantic::WarehouseDialect;
    ///
    /// let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0");
    /// assert_eq!(hint.table_name, "network_activity");
    /// assert!(hint.schema_name.is_none());
    /// ```
    pub fn new(
        table_name: impl Into<String>,
        dialect: WarehouseDialect,
        ocsf_version: impl Into<String>,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            schema_name: None,
            dialect,
            ocsf_version: ocsf_version.into(),
        }
    }

    /// Sets the schema name for this table.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema name
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::TableHint;
    /// use ocsf_semantic::WarehouseDialect;
    ///
    /// let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0")
    ///     .with_schema("ocsf");
    /// assert_eq!(hint.schema_name, Some("ocsf".to_string()));
    /// ```
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema_name = Some(schema.into());
        self
    }

    /// Returns the fully qualified table name (schema.table or just table).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::TableHint;
    /// use ocsf_semantic::WarehouseDialect;
    ///
    /// let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0");
    /// assert_eq!(hint.qualified_name(), "network_activity");
    ///
    /// let hint = hint.with_schema("ocsf");
    /// assert_eq!(hint.qualified_name(), "ocsf.network_activity");
    /// ```
    pub fn qualified_name(&self) -> String {
        match &self.schema_name {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => self.table_name.clone(),
        }
    }
}

// ============================================================================
// PartitionHint
// ============================================================================

/// Hint about a partition for partition pruning.
///
/// `PartitionHint` provides information about a partition that is relevant
/// to a query, including the estimated row count for cost estimation.
///
/// # Example
///
/// ```rust
/// use ocsf_index::query_hints::PartitionHint;
///
/// let hint = PartitionHint::new("network_activity", "2024-01", 1_000_000);
///
/// assert_eq!(hint.table_name, "network_activity");
/// assert_eq!(hint.partition_key, "2024-01");
/// assert_eq!(hint.estimated_rows, 1_000_000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionHint {
    /// The name of the table this partition belongs to.
    pub table_name: String,
    /// The partition key (e.g., "2024-01" for monthly partitions).
    pub partition_key: String,
    /// Estimated number of rows in this partition.
    pub estimated_rows: u64,
}

impl PartitionHint {
    /// Creates a new PartitionHint.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table
    /// * `partition_key` - The partition key
    /// * `estimated_rows` - Estimated number of rows
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::query_hints::PartitionHint;
    ///
    /// let hint = PartitionHint::new("network_activity", "2024-01", 1_000_000);
    /// assert_eq!(hint.table_name, "network_activity");
    /// assert_eq!(hint.partition_key, "2024-01");
    /// assert_eq!(hint.estimated_rows, 1_000_000);
    /// ```
    pub fn new(
        table_name: impl Into<String>,
        partition_key: impl Into<String>,
        estimated_rows: u64,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            partition_key: partition_key.into(),
            estimated_rows,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_cache::QueryCacheKey;
    use crate::table_statistics::ColumnStatistics;

    // ========================================================================
    // QueryPlanHints Tests
    // ========================================================================

    #[test]
    fn test_query_plan_hints_new() {
        let hints = QueryPlanHints::new();

        assert!(hints.tables.is_empty());
        assert!(hints.partitions.is_empty());
        assert!(hints.statistics.is_empty());
        assert!(hints.cached_result.is_none());
    }

    #[test]
    fn test_query_plan_hints_default() {
        let hints = QueryPlanHints::default();

        assert!(hints.tables.is_empty());
        assert!(hints.partitions.is_empty());
        assert!(hints.statistics.is_empty());
        assert!(hints.cached_result.is_none());
    }

    #[test]
    fn test_query_plan_hints_add_table() {
        let hints = QueryPlanHints::new()
            .add_table(TableHint::new(
                "table1",
                WarehouseDialect::Snowflake,
                "1.3.0",
            ))
            .add_table(TableHint::new(
                "table2",
                WarehouseDialect::Databricks,
                "1.4.0",
            ));

        assert_eq!(hints.tables.len(), 2);
        assert_eq!(hints.tables[0].table_name, "table1");
        assert_eq!(hints.tables[1].table_name, "table2");
    }

    #[test]
    fn test_query_plan_hints_add_partition() {
        let hints = QueryPlanHints::new()
            .add_partition(PartitionHint::new("table", "2024-01", 1_000_000))
            .add_partition(PartitionHint::new("table", "2024-02", 500_000));

        assert_eq!(hints.partitions.len(), 2);
        assert_eq!(hints.partitions[0].partition_key, "2024-01");
        assert_eq!(hints.partitions[1].partition_key, "2024-02");
    }

    #[test]
    fn test_query_plan_hints_add_statistics() {
        let hints = QueryPlanHints::new()
            .add_statistics(
                TableStatistics::new("table1")
                    .with_total_rows(1_000_000)
                    .add_column(ColumnStatistics::new("col1").with_counts(100, 10, 1_000_000)),
            )
            .add_statistics(TableStatistics::new("table2").with_total_rows(500_000));

        assert_eq!(hints.statistics.len(), 2);
        assert_eq!(hints.statistics[0].table_name, "table1");
        assert_eq!(hints.statistics[1].table_name, "table2");
    }

    #[test]
    fn test_query_plan_hints_with_cached_result() {
        let key = QueryCacheKey::new("abc123", vec!["table".to_string()]);
        let cached = CachedResult::new(key, serde_json::json!({"data": "test"}), 3600);

        let hints = QueryPlanHints::new().with_cached_result(cached);

        assert!(hints.cached_result.is_some());
        assert_eq!(
            hints.cached_result.as_ref().unwrap().key.query_hash,
            "abc123"
        );
    }

    #[test]
    fn test_query_plan_hints_has_valid_cache_none() {
        let hints = QueryPlanHints::new();
        assert!(!hints.has_valid_cache());
    }

    #[test]
    fn test_query_plan_hints_has_valid_cache_expired() {
        let key = QueryCacheKey::new("abc123", vec!["table".to_string()]);
        // TTL of 0 means immediately expired
        let cached = CachedResult::new(key, serde_json::json!({"data": "test"}), 0);

        let hints = QueryPlanHints::new().with_cached_result(cached);

        assert!(!hints.has_valid_cache());
    }

    #[test]
    fn test_query_plan_hints_has_valid_cache_valid() {
        let key = QueryCacheKey::new("abc123", vec!["table".to_string()]);
        let cached = CachedResult::new(key, serde_json::json!({"data": "test"}), 3600);

        let hints = QueryPlanHints::new().with_cached_result(cached);

        assert!(hints.has_valid_cache());
    }

    #[test]
    fn test_query_plan_hints_total_estimated_rows() {
        let hints = QueryPlanHints::new()
            .add_partition(PartitionHint::new("table", "2024-01", 1_000_000))
            .add_partition(PartitionHint::new("table", "2024-02", 500_000))
            .add_partition(PartitionHint::new("table", "2024-03", 750_000));

        assert_eq!(hints.total_estimated_rows(), 2_250_000);
    }

    #[test]
    fn test_query_plan_hints_total_estimated_rows_empty() {
        let hints = QueryPlanHints::new();
        assert_eq!(hints.total_estimated_rows(), 0);
    }

    #[test]
    fn test_query_plan_hints_builder_chaining() {
        let key = QueryCacheKey::new("abc123", vec!["network_activity".to_string()]);
        let cached = CachedResult::new(key, serde_json::json!({"rows": 100}), 3600);

        let hints = QueryPlanHints::new()
            .add_table(TableHint::new(
                "network_activity",
                WarehouseDialect::Snowflake,
                "1.3.0",
            ))
            .add_partition(PartitionHint::new("network_activity", "2024-01", 1_000_000))
            .add_statistics(TableStatistics::new("network_activity").with_total_rows(10_000_000))
            .with_cached_result(cached);

        assert_eq!(hints.tables.len(), 1);
        assert_eq!(hints.partitions.len(), 1);
        assert_eq!(hints.statistics.len(), 1);
        assert!(hints.cached_result.is_some());
    }

    // ========================================================================
    // TableHint Tests
    // ========================================================================

    #[test]
    fn test_table_hint_new() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0");

        assert_eq!(hint.table_name, "network_activity");
        assert!(hint.schema_name.is_none());
        assert_eq!(hint.dialect, WarehouseDialect::Snowflake);
        assert_eq!(hint.ocsf_version, "1.3.0");
    }

    #[test]
    fn test_table_hint_with_schema() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0")
            .with_schema("ocsf");

        assert_eq!(hint.schema_name, Some("ocsf".to_string()));
    }

    #[test]
    fn test_table_hint_qualified_name_no_schema() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0");

        assert_eq!(hint.qualified_name(), "network_activity");
    }

    #[test]
    fn test_table_hint_qualified_name_with_schema() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0")
            .with_schema("ocsf");

        assert_eq!(hint.qualified_name(), "ocsf.network_activity");
    }

    #[test]
    fn test_table_hint_different_dialects() {
        let snowflake = TableHint::new("table", WarehouseDialect::Snowflake, "1.3.0");
        let databricks = TableHint::new("table", WarehouseDialect::Databricks, "1.3.0");
        let bigquery = TableHint::new("table", WarehouseDialect::BigQuery, "1.3.0");
        let postgres = TableHint::new("table", WarehouseDialect::Postgres, "1.3.0");

        assert_eq!(snowflake.dialect, WarehouseDialect::Snowflake);
        assert_eq!(databricks.dialect, WarehouseDialect::Databricks);
        assert_eq!(bigquery.dialect, WarehouseDialect::BigQuery);
        assert_eq!(postgres.dialect, WarehouseDialect::Postgres);
    }

    // ========================================================================
    // PartitionHint Tests
    // ========================================================================

    #[test]
    fn test_partition_hint_new() {
        let hint = PartitionHint::new("network_activity", "2024-01", 1_000_000);

        assert_eq!(hint.table_name, "network_activity");
        assert_eq!(hint.partition_key, "2024-01");
        assert_eq!(hint.estimated_rows, 1_000_000);
    }

    #[test]
    fn test_partition_hint_zero_rows() {
        let hint = PartitionHint::new("network_activity", "2024-01", 0);

        assert_eq!(hint.estimated_rows, 0);
    }

    #[test]
    fn test_partition_hint_large_rows() {
        let hint = PartitionHint::new("network_activity", "2024-01", u64::MAX);

        assert_eq!(hint.estimated_rows, u64::MAX);
    }

    // ========================================================================
    // JSON Serialization Tests
    // ========================================================================

    #[test]
    fn test_query_plan_hints_json_serialization() {
        let hints = QueryPlanHints::new()
            .add_table(TableHint::new(
                "network_activity",
                WarehouseDialect::Snowflake,
                "1.3.0",
            ))
            .add_partition(PartitionHint::new("network_activity", "2024-01", 1_000_000));

        let json = serde_json::to_string(&hints).unwrap();
        let deserialized: QueryPlanHints = serde_json::from_str(&json).unwrap();

        assert_eq!(hints.tables.len(), deserialized.tables.len());
        assert_eq!(hints.partitions.len(), deserialized.partitions.len());
        assert_eq!(hints.tables[0], deserialized.tables[0]);
        assert_eq!(hints.partitions[0], deserialized.partitions[0]);
    }

    #[test]
    fn test_query_plan_hints_json_none_cached_result_skipped() {
        let hints = QueryPlanHints::new();

        let json = serde_json::to_string(&hints).unwrap();

        // None cached_result should be skipped in serialization
        assert!(!json.contains("\"cached_result\""));
    }

    #[test]
    fn test_table_hint_json_serialization() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0")
            .with_schema("ocsf");

        let json = serde_json::to_string(&hint).unwrap();
        let deserialized: TableHint = serde_json::from_str(&json).unwrap();

        assert_eq!(hint, deserialized);
    }

    #[test]
    fn test_table_hint_json_none_schema_skipped() {
        let hint = TableHint::new("network_activity", WarehouseDialect::Snowflake, "1.3.0");

        let json = serde_json::to_string(&hint).unwrap();

        // None schema_name should be skipped in serialization
        assert!(!json.contains("\"schema_name\""));
    }

    #[test]
    fn test_partition_hint_json_serialization() {
        let hint = PartitionHint::new("network_activity", "2024-01", 1_000_000);

        let json = serde_json::to_string(&hint).unwrap();
        let deserialized: PartitionHint = serde_json::from_str(&json).unwrap();

        assert_eq!(hint, deserialized);
    }
}
