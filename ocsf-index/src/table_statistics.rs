//! Table statistics tracking for the OCSF Semantic Index.
//!
//! This module provides types for tracking column statistics including cardinality
//! and null rates, enabling query optimization through statistics-based planning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backend::IndexRecord;
use crate::types::RecordId;

// ============================================================================
// ColumnStatistics
// ============================================================================

/// Statistics for a single column in a table.
///
/// `ColumnStatistics` tracks distinct count, null count, total count, and optional
/// min/max values for a column. This information is used for query optimization
/// and cost estimation.
///
/// # Example
///
/// ```rust
/// use ocsf_index::table_statistics::ColumnStatistics;
///
/// let stats = ColumnStatistics::new("src_ip")
///     .with_counts(1000, 50, 10000)
///     .with_min_max("10.0.0.1", "192.168.255.255");
///
/// assert_eq!(stats.column_name, "src_ip");
/// assert_eq!(stats.distinct_count, 1000);
/// assert_eq!(stats.null_count, 50);
/// assert_eq!(stats.total_count, 10000);
/// assert_eq!(stats.null_rate(), 0.005); // 50/10000
/// assert_eq!(stats.cardinality(), 0.1);  // 1000/10000
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnStatistics {
    /// The name of the column.
    pub column_name: String,

    /// Number of distinct values in the column.
    pub distinct_count: u64,

    /// Number of null values in the column.
    pub null_count: u64,

    /// Total number of values (rows) in the column.
    pub total_count: u64,

    /// Minimum value in the column (as string for generic representation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<String>,

    /// Maximum value in the column (as string for generic representation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<String>,
}

impl ColumnStatistics {
    /// Creates a new ColumnStatistics with the given column name.
    ///
    /// The statistics are initialized with:
    /// - `distinct_count`: 0
    /// - `null_count`: 0
    /// - `total_count`: 0
    /// - `min_value`: None
    /// - `max_value`: None
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::ColumnStatistics;
    ///
    /// let stats = ColumnStatistics::new("src_ip");
    /// assert_eq!(stats.column_name, "src_ip");
    /// assert_eq!(stats.distinct_count, 0);
    /// assert_eq!(stats.null_count, 0);
    /// assert_eq!(stats.total_count, 0);
    /// ```
    pub fn new(column_name: impl Into<String>) -> Self {
        Self {
            column_name: column_name.into(),
            distinct_count: 0,
            null_count: 0,
            total_count: 0,
            min_value: None,
            max_value: None,
        }
    }

    /// Sets the count statistics for this column.
    ///
    /// # Arguments
    ///
    /// * `distinct_count` - Number of distinct values
    /// * `null_count` - Number of null values
    /// * `total_count` - Total number of values (rows)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::ColumnStatistics;
    ///
    /// let stats = ColumnStatistics::new("src_ip")
    ///     .with_counts(1000, 50, 10000);
    /// assert_eq!(stats.distinct_count, 1000);
    /// assert_eq!(stats.null_count, 50);
    /// assert_eq!(stats.total_count, 10000);
    /// ```
    pub fn with_counts(mut self, distinct_count: u64, null_count: u64, total_count: u64) -> Self {
        self.distinct_count = distinct_count;
        self.null_count = null_count;
        self.total_count = total_count;
        self
    }

    /// Sets the min and max values for this column.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum value (as string)
    /// * `max` - The maximum value (as string)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::ColumnStatistics;
    ///
    /// let stats = ColumnStatistics::new("src_ip")
    ///     .with_min_max("10.0.0.1", "192.168.255.255");
    /// assert_eq!(stats.min_value, Some("10.0.0.1".to_string()));
    /// assert_eq!(stats.max_value, Some("192.168.255.255".to_string()));
    /// ```
    pub fn with_min_max(mut self, min: impl Into<String>, max: impl Into<String>) -> Self {
        self.min_value = Some(min.into());
        self.max_value = Some(max.into());
        self
    }

    /// Sets the distinct count for this column.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of distinct values
    pub fn with_distinct_count(mut self, count: u64) -> Self {
        self.distinct_count = count;
        self
    }

    /// Sets the null count for this column.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of null values
    pub fn with_null_count(mut self, count: u64) -> Self {
        self.null_count = count;
        self
    }

    /// Sets the total count for this column.
    ///
    /// # Arguments
    ///
    /// * `count` - Total number of values (rows)
    pub fn with_total_count(mut self, count: u64) -> Self {
        self.total_count = count;
        self
    }

    /// Calculates the null rate for this column.
    ///
    /// The null rate is the ratio of null values to total values.
    /// Returns 0.0 if total_count is 0 to avoid division by zero.
    ///
    /// # Returns
    ///
    /// The null rate as a value between 0.0 and 1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::ColumnStatistics;
    ///
    /// let stats = ColumnStatistics::new("src_ip")
    ///     .with_counts(1000, 50, 10000);
    /// assert_eq!(stats.null_rate(), 0.005); // 50/10000
    ///
    /// // Edge case: no data
    /// let empty_stats = ColumnStatistics::new("empty");
    /// assert_eq!(empty_stats.null_rate(), 0.0);
    /// ```
    pub fn null_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.null_count as f64 / self.total_count as f64
        }
    }

    /// Calculates the cardinality (selectivity) for this column.
    ///
    /// The cardinality is the ratio of distinct values to total values.
    /// A cardinality of 1.0 means all values are unique (like a primary key).
    /// A cardinality close to 0.0 means many duplicate values.
    /// Returns 0.0 if total_count is 0 to avoid division by zero.
    ///
    /// # Returns
    ///
    /// The cardinality as a value between 0.0 and 1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::ColumnStatistics;
    ///
    /// let stats = ColumnStatistics::new("src_ip")
    ///     .with_counts(1000, 50, 10000);
    /// assert_eq!(stats.cardinality(), 0.1); // 1000/10000
    ///
    /// // High cardinality (unique values)
    /// let unique_stats = ColumnStatistics::new("id")
    ///     .with_counts(10000, 0, 10000);
    /// assert_eq!(unique_stats.cardinality(), 1.0);
    ///
    /// // Edge case: no data
    /// let empty_stats = ColumnStatistics::new("empty");
    /// assert_eq!(empty_stats.cardinality(), 0.0);
    /// ```
    pub fn cardinality(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.distinct_count as f64 / self.total_count as f64
        }
    }
}

// ============================================================================
// TableStatistics
// ============================================================================

/// Unique identifier for table statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatisticsId(pub u64);

impl StatisticsId {
    /// Creates a new StatisticsId.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for StatisticsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StatisticsId({})", self.0)
    }
}

impl From<u64> for StatisticsId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<StatisticsId> for u64 {
    fn from(id: StatisticsId) -> Self {
        id.0
    }
}

/// Statistics for a table including column-level statistics.
///
/// `TableStatistics` tracks column statistics, total row count, sample rate,
/// and collection timestamp. This information is used for query optimization
/// and cost estimation.
///
/// # Example
///
/// ```rust
/// use ocsf_index::table_statistics::{TableStatistics, ColumnStatistics};
///
/// let stats = TableStatistics::new("network_activity")
///     .with_total_rows(1_000_000)
///     .with_sample_rate(0.1)
///     .add_column(
///         ColumnStatistics::new("src_ip")
///             .with_counts(50000, 100, 1_000_000)
///     )
///     .add_column(
///         ColumnStatistics::new("dst_port")
///             .with_counts(65535, 0, 1_000_000)
///     );
///
/// assert_eq!(stats.table_name, "network_activity");
/// assert_eq!(stats.total_rows, 1_000_000);
/// assert_eq!(stats.sample_rate, 0.1);
/// assert_eq!(stats.columns.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableStatistics {
    /// Unique identifier for these statistics (assigned by backend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<StatisticsId>,

    /// The name of the table these statistics are for.
    pub table_name: String,

    /// Statistics for each column in the table.
    pub columns: Vec<ColumnStatistics>,

    /// Total number of rows in the table.
    pub total_rows: u64,

    /// Sample rate used when collecting statistics (0.0 to 1.0).
    /// A value of 1.0 means all rows were sampled.
    pub sample_rate: f64,

    /// Timestamp when these statistics were collected.
    pub collected_at: DateTime<Utc>,
}

impl TableStatistics {
    /// Creates a new TableStatistics with the given table name.
    ///
    /// The statistics are initialized with:
    /// - `columns`: empty vector
    /// - `total_rows`: 0
    /// - `sample_rate`: 1.0 (full sample)
    /// - `collected_at`: current UTC time
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::TableStatistics;
    ///
    /// let stats = TableStatistics::new("network_activity");
    /// assert_eq!(stats.table_name, "network_activity");
    /// assert_eq!(stats.total_rows, 0);
    /// assert_eq!(stats.sample_rate, 1.0);
    /// assert!(stats.columns.is_empty());
    /// ```
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            id: None,
            table_name: table_name.into(),
            columns: Vec::new(),
            total_rows: 0,
            sample_rate: 1.0,
            collected_at: Utc::now(),
        }
    }

    /// Adds column statistics to this table.
    ///
    /// # Arguments
    ///
    /// * `stats` - The column statistics to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::{TableStatistics, ColumnStatistics};
    ///
    /// let stats = TableStatistics::new("network_activity")
    ///     .add_column(ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000))
    ///     .add_column(ColumnStatistics::new("dst_ip").with_counts(800, 100, 10000));
    ///
    /// assert_eq!(stats.columns.len(), 2);
    /// assert_eq!(stats.columns[0].column_name, "src_ip");
    /// assert_eq!(stats.columns[1].column_name, "dst_ip");
    /// ```
    pub fn add_column(mut self, stats: ColumnStatistics) -> Self {
        self.columns.push(stats);
        self
    }

    /// Sets the sample rate used when collecting statistics.
    ///
    /// The sample rate is clamped to the range [0.0, 1.0].
    ///
    /// # Arguments
    ///
    /// * `rate` - The sample rate (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::TableStatistics;
    ///
    /// let stats = TableStatistics::new("network_activity")
    ///     .with_sample_rate(0.1);
    /// assert_eq!(stats.sample_rate, 0.1);
    ///
    /// // Values are clamped
    /// let stats = TableStatistics::new("network_activity")
    ///     .with_sample_rate(1.5);
    /// assert_eq!(stats.sample_rate, 1.0);
    /// ```
    pub fn with_sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Sets the total row count for this table.
    ///
    /// # Arguments
    ///
    /// * `rows` - The total number of rows
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::TableStatistics;
    ///
    /// let stats = TableStatistics::new("network_activity")
    ///     .with_total_rows(1_000_000);
    /// assert_eq!(stats.total_rows, 1_000_000);
    /// ```
    pub fn with_total_rows(mut self, rows: u64) -> Self {
        self.total_rows = rows;
        self
    }

    /// Sets the collection timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The collection timestamp
    pub fn with_collected_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.collected_at = timestamp;
        self
    }

    /// Sets the statistics ID (typically called by the backend after persistence).
    ///
    /// # Arguments
    ///
    /// * `id` - The statistics ID
    pub fn with_id(mut self, id: StatisticsId) -> Self {
        self.id = Some(id);
        self
    }

    /// Checks if these statistics are stale based on the maximum age.
    ///
    /// Statistics are considered stale if the time since collection exceeds
    /// the specified maximum age in seconds.
    ///
    /// # Arguments
    ///
    /// * `max_age_secs` - Maximum age in seconds before statistics are considered stale
    ///
    /// # Returns
    ///
    /// `true` if the statistics are stale, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::TableStatistics;
    /// use chrono::{Utc, Duration};
    ///
    /// // Fresh statistics
    /// let stats = TableStatistics::new("network_activity");
    /// assert!(!stats.is_stale(3600)); // Not stale within 1 hour
    ///
    /// // Old statistics
    /// let old_stats = TableStatistics::new("network_activity")
    ///     .with_collected_at(Utc::now() - Duration::hours(2));
    /// assert!(old_stats.is_stale(3600)); // Stale after 1 hour
    /// ```
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.collected_at);
        age.num_seconds() > max_age_secs as i64
    }

    /// Gets column statistics by column name.
    ///
    /// # Arguments
    ///
    /// * `name` - The column name to look up
    ///
    /// # Returns
    ///
    /// A reference to the column statistics if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_statistics::{TableStatistics, ColumnStatistics};
    ///
    /// let stats = TableStatistics::new("network_activity")
    ///     .add_column(ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000))
    ///     .add_column(ColumnStatistics::new("dst_ip").with_counts(800, 100, 10000));
    ///
    /// let src_ip_stats = stats.get_column("src_ip");
    /// assert!(src_ip_stats.is_some());
    /// assert_eq!(src_ip_stats.unwrap().distinct_count, 1000);
    ///
    /// let unknown_stats = stats.get_column("unknown");
    /// assert!(unknown_stats.is_none());
    /// ```
    pub fn get_column(&self, name: &str) -> Option<&ColumnStatistics> {
        self.columns.iter().find(|c| c.column_name == name)
    }
}

impl IndexRecord for TableStatistics {
    fn record_type() -> &'static str {
        "table_statistics"
    }

    fn id(&self) -> Option<RecordId> {
        self.id.map(|id| RecordId::new(id.0))
    }

    fn set_id(&mut self, id: RecordId) {
        self.id = Some(StatisticsId::new(id.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // ========================================================================
    // ColumnStatistics Construction Tests
    // ========================================================================

    #[test]
    fn test_column_statistics_new() {
        let stats = ColumnStatistics::new("src_ip");

        assert_eq!(stats.column_name, "src_ip");
        assert_eq!(stats.distinct_count, 0);
        assert_eq!(stats.null_count, 0);
        assert_eq!(stats.total_count, 0);
        assert!(stats.min_value.is_none());
        assert!(stats.max_value.is_none());
    }

    #[test]
    fn test_column_statistics_with_counts() {
        let stats = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);

        assert_eq!(stats.distinct_count, 1000);
        assert_eq!(stats.null_count, 50);
        assert_eq!(stats.total_count, 10000);
    }

    #[test]
    fn test_column_statistics_with_min_max() {
        let stats = ColumnStatistics::new("src_ip").with_min_max("10.0.0.1", "192.168.255.255");

        assert_eq!(stats.min_value, Some("10.0.0.1".to_string()));
        assert_eq!(stats.max_value, Some("192.168.255.255".to_string()));
    }

    #[test]
    fn test_column_statistics_individual_setters() {
        let stats = ColumnStatistics::new("src_ip")
            .with_distinct_count(500)
            .with_null_count(25)
            .with_total_count(5000);

        assert_eq!(stats.distinct_count, 500);
        assert_eq!(stats.null_count, 25);
        assert_eq!(stats.total_count, 5000);
    }

    #[test]
    fn test_column_statistics_builder_chaining() {
        let stats = ColumnStatistics::new("src_ip")
            .with_counts(1000, 50, 10000)
            .with_min_max("10.0.0.1", "192.168.255.255");

        assert_eq!(stats.column_name, "src_ip");
        assert_eq!(stats.distinct_count, 1000);
        assert_eq!(stats.null_count, 50);
        assert_eq!(stats.total_count, 10000);
        assert_eq!(stats.min_value, Some("10.0.0.1".to_string()));
        assert_eq!(stats.max_value, Some("192.168.255.255".to_string()));
    }

    // ========================================================================
    // ColumnStatistics Calculation Tests
    // ========================================================================

    #[test]
    fn test_column_statistics_null_rate() {
        let stats = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);

        assert_eq!(stats.null_rate(), 0.005); // 50/10000
    }

    #[test]
    fn test_column_statistics_null_rate_zero_total() {
        let stats = ColumnStatistics::new("src_ip");
        assert_eq!(stats.null_rate(), 0.0);
    }

    #[test]
    fn test_column_statistics_null_rate_all_nulls() {
        let stats = ColumnStatistics::new("src_ip").with_counts(0, 1000, 1000);

        assert_eq!(stats.null_rate(), 1.0);
    }

    #[test]
    fn test_column_statistics_null_rate_no_nulls() {
        let stats = ColumnStatistics::new("src_ip").with_counts(1000, 0, 10000);

        assert_eq!(stats.null_rate(), 0.0);
    }

    #[test]
    fn test_column_statistics_cardinality() {
        let stats = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);

        assert_eq!(stats.cardinality(), 0.1); // 1000/10000
    }

    #[test]
    fn test_column_statistics_cardinality_zero_total() {
        let stats = ColumnStatistics::new("src_ip");
        assert_eq!(stats.cardinality(), 0.0);
    }

    #[test]
    fn test_column_statistics_cardinality_unique() {
        let stats = ColumnStatistics::new("id").with_counts(10000, 0, 10000);

        assert_eq!(stats.cardinality(), 1.0);
    }

    #[test]
    fn test_column_statistics_cardinality_low() {
        let stats = ColumnStatistics::new("status").with_counts(3, 0, 10000);

        assert_eq!(stats.cardinality(), 0.0003);
    }

    // ========================================================================
    // TableStatistics Construction Tests
    // ========================================================================

    #[test]
    fn test_table_statistics_new() {
        let stats = TableStatistics::new("network_activity");

        assert!(stats.id.is_none());
        assert_eq!(stats.table_name, "network_activity");
        assert!(stats.columns.is_empty());
        assert_eq!(stats.total_rows, 0);
        assert_eq!(stats.sample_rate, 1.0);
    }

    #[test]
    fn test_table_statistics_add_column() {
        let stats = TableStatistics::new("network_activity")
            .add_column(ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000))
            .add_column(ColumnStatistics::new("dst_ip").with_counts(800, 100, 10000));

        assert_eq!(stats.columns.len(), 2);
        assert_eq!(stats.columns[0].column_name, "src_ip");
        assert_eq!(stats.columns[1].column_name, "dst_ip");
    }

    #[test]
    fn test_table_statistics_with_sample_rate() {
        let stats = TableStatistics::new("network_activity").with_sample_rate(0.1);

        assert_eq!(stats.sample_rate, 0.1);
    }

    #[test]
    fn test_table_statistics_sample_rate_clamping() {
        let stats = TableStatistics::new("network_activity").with_sample_rate(1.5);
        assert_eq!(stats.sample_rate, 1.0);

        let stats = TableStatistics::new("network_activity").with_sample_rate(-0.5);
        assert_eq!(stats.sample_rate, 0.0);
    }

    #[test]
    fn test_table_statistics_with_total_rows() {
        let stats = TableStatistics::new("network_activity").with_total_rows(1_000_000);

        assert_eq!(stats.total_rows, 1_000_000);
    }

    #[test]
    fn test_table_statistics_with_collected_at() {
        let timestamp = Utc::now() - Duration::hours(1);
        let stats = TableStatistics::new("network_activity").with_collected_at(timestamp);

        assert_eq!(stats.collected_at, timestamp);
    }

    #[test]
    fn test_table_statistics_with_id() {
        let stats = TableStatistics::new("network_activity").with_id(StatisticsId::new(42));

        assert_eq!(stats.id, Some(StatisticsId::new(42)));
    }

    #[test]
    fn test_table_statistics_builder_chaining() {
        let timestamp = Utc::now();
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1)
            .with_collected_at(timestamp)
            .with_id(StatisticsId::new(123))
            .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 1_000_000))
            .add_column(ColumnStatistics::new("dst_port").with_counts(65535, 0, 1_000_000));

        assert_eq!(stats.id, Some(StatisticsId::new(123)));
        assert_eq!(stats.table_name, "network_activity");
        assert_eq!(stats.total_rows, 1_000_000);
        assert_eq!(stats.sample_rate, 0.1);
        assert_eq!(stats.collected_at, timestamp);
        assert_eq!(stats.columns.len(), 2);
    }

    // ========================================================================
    // TableStatistics Method Tests
    // ========================================================================

    #[test]
    fn test_table_statistics_is_stale_fresh() {
        let stats = TableStatistics::new("network_activity");
        assert!(!stats.is_stale(3600)); // Not stale within 1 hour
    }

    #[test]
    fn test_table_statistics_is_stale_old() {
        let old_stats = TableStatistics::new("network_activity")
            .with_collected_at(Utc::now() - Duration::hours(2));
        assert!(old_stats.is_stale(3600)); // Stale after 1 hour
    }

    #[test]
    fn test_table_statistics_is_stale_boundary() {
        // Exactly at the boundary
        let stats = TableStatistics::new("network_activity")
            .with_collected_at(Utc::now() - Duration::seconds(3600));
        // At exactly max_age, it should not be stale (> not >=)
        assert!(!stats.is_stale(3600));

        // Just past the boundary
        let stats = TableStatistics::new("network_activity")
            .with_collected_at(Utc::now() - Duration::seconds(3601));
        assert!(stats.is_stale(3600));
    }

    #[test]
    fn test_table_statistics_get_column_found() {
        let stats = TableStatistics::new("network_activity")
            .add_column(ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000))
            .add_column(ColumnStatistics::new("dst_ip").with_counts(800, 100, 10000));

        let src_ip_stats = stats.get_column("src_ip");
        assert!(src_ip_stats.is_some());
        assert_eq!(src_ip_stats.unwrap().distinct_count, 1000);

        let dst_ip_stats = stats.get_column("dst_ip");
        assert!(dst_ip_stats.is_some());
        assert_eq!(dst_ip_stats.unwrap().distinct_count, 800);
    }

    #[test]
    fn test_table_statistics_get_column_not_found() {
        let stats = TableStatistics::new("network_activity")
            .add_column(ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000));

        let unknown_stats = stats.get_column("unknown");
        assert!(unknown_stats.is_none());
    }

    #[test]
    fn test_table_statistics_get_column_empty() {
        let stats = TableStatistics::new("network_activity");
        assert!(stats.get_column("any").is_none());
    }

    // ========================================================================
    // JSON Serialization Tests
    // ========================================================================

    #[test]
    fn test_column_statistics_json_serialization() {
        let stats = ColumnStatistics::new("src_ip")
            .with_counts(1000, 50, 10000)
            .with_min_max("10.0.0.1", "192.168.255.255");

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: ColumnStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, deserialized);
    }

    #[test]
    fn test_column_statistics_json_none_values_skipped() {
        let stats = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);

        let json = serde_json::to_string(&stats).unwrap();

        // None values should be skipped in serialization
        assert!(!json.contains("\"min_value\""));
        assert!(!json.contains("\"max_value\""));
    }

    #[test]
    fn test_table_statistics_json_serialization() {
        let timestamp = Utc::now();
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1)
            .with_collected_at(timestamp)
            .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 1_000_000));

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: TableStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.table_name, deserialized.table_name);
        assert_eq!(stats.total_rows, deserialized.total_rows);
        assert_eq!(stats.sample_rate, deserialized.sample_rate);
        assert_eq!(stats.columns.len(), deserialized.columns.len());
        assert_eq!(stats.columns[0], deserialized.columns[0]);
    }

    #[test]
    fn test_table_statistics_json_none_id_skipped() {
        let stats = TableStatistics::new("network_activity");

        let json = serde_json::to_string(&stats).unwrap();

        // None id should be skipped in serialization
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_table_statistics_json_round_trip_with_id() {
        let timestamp = Utc::now();
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1)
            .with_collected_at(timestamp)
            .with_id(StatisticsId::new(42))
            .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 1_000_000));

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: TableStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, deserialized);
    }

    // ========================================================================
    // IndexRecord Trait Tests
    // ========================================================================

    #[test]
    fn test_table_statistics_record_type() {
        assert_eq!(TableStatistics::record_type(), "table_statistics");
    }

    #[test]
    fn test_table_statistics_id_none() {
        let stats = TableStatistics::new("network_activity");
        assert!(stats.id().is_none());
    }

    #[test]
    fn test_table_statistics_id_some() {
        let stats = TableStatistics::new("network_activity").with_id(StatisticsId::new(42));
        assert_eq!(stats.id(), Some(RecordId::new(42)));
    }

    #[test]
    fn test_table_statistics_set_id() {
        let mut stats = TableStatistics::new("network_activity");
        assert!(stats.id.is_none());

        stats.set_id(RecordId::new(99));
        assert_eq!(stats.id, Some(StatisticsId::new(99)));
    }

    // ========================================================================
    // StatisticsId Tests
    // ========================================================================

    #[test]
    fn test_statistics_id_display() {
        let id = StatisticsId::new(42);
        assert_eq!(format!("{}", id), "StatisticsId(42)");
    }

    #[test]
    fn test_statistics_id_from_u64() {
        let id: StatisticsId = 123u64.into();
        assert_eq!(id.0, 123);
    }

    #[test]
    fn test_statistics_id_into_u64() {
        let id = StatisticsId::new(456);
        let value: u64 = id.into();
        assert_eq!(value, 456);
    }

    // ========================================================================
    // Equality and Clone Tests
    // ========================================================================

    #[test]
    fn test_column_statistics_equality() {
        let stats1 = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);
        let stats2 = ColumnStatistics::new("src_ip").with_counts(1000, 50, 10000);
        let stats3 = ColumnStatistics::new("src_ip").with_counts(2000, 50, 10000);

        assert_eq!(stats1, stats2);
        assert_ne!(stats1, stats3);
    }

    #[test]
    fn test_column_statistics_clone() {
        let stats = ColumnStatistics::new("src_ip")
            .with_counts(1000, 50, 10000)
            .with_min_max("10.0.0.1", "192.168.255.255");

        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn test_table_statistics_equality() {
        let timestamp = Utc::now();
        let stats1 = TableStatistics::new("network_activity")
            .with_total_rows(1000)
            .with_collected_at(timestamp);
        let stats2 = TableStatistics::new("network_activity")
            .with_total_rows(1000)
            .with_collected_at(timestamp);
        let stats3 = TableStatistics::new("network_activity")
            .with_total_rows(2000)
            .with_collected_at(timestamp);

        assert_eq!(stats1, stats2);
        assert_ne!(stats1, stats3);
    }

    #[test]
    fn test_table_statistics_clone() {
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1)
            .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 1_000_000));

        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }
}
