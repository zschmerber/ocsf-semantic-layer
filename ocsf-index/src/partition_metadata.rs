//! Partition metadata tracking for the OCSF Semantic Index.
//!
//! This module provides types for tracking partition information including time bounds
//! and row counts, enabling query optimization through partition pruning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backend::IndexRecord;
use crate::types::{PartitionId, RecordId};

// ============================================================================
// PartitionEntry
// ============================================================================

/// Partition metadata entry for query optimization.
///
/// A `PartitionEntry` represents metadata about a single partition of a table,
/// including its time bounds, row count, and size. This information is used
/// for partition pruning during query execution.
///
/// # Example
///
/// ```rust
/// use ocsf_index::partition_metadata::PartitionEntry;
/// use chrono::{Utc, Duration};
///
/// let start = Utc::now();
/// let end = start + Duration::hours(1);
///
/// let partition = PartitionEntry::new("network_activity", "2024-01")
///     .with_time_range(start, end)
///     .with_row_count(1_000_000)
///     .with_size_bytes(500_000_000);
///
/// assert_eq!(partition.table_name, "network_activity");
/// assert_eq!(partition.partition_key, "2024-01");
/// assert_eq!(partition.row_count, 1_000_000);
/// assert_eq!(partition.size_bytes, 500_000_000);
/// assert!(!partition.is_empty);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionEntry {
    /// Unique identifier for this partition (assigned by backend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<PartitionId>,

    /// The table name this partition belongs to.
    pub table_name: String,

    /// The partition key (e.g., "2024-01", "2024-01-15").
    pub partition_key: String,

    /// Start time of the partition's time range.
    pub start_time: DateTime<Utc>,

    /// End time of the partition's time range.
    pub end_time: DateTime<Utc>,

    /// Number of rows in this partition.
    pub row_count: u64,

    /// Size of the partition in bytes.
    pub size_bytes: u64,

    /// Whether the partition is empty (has no data).
    pub is_empty: bool,

    /// Timestamp when the partition was last modified.
    pub last_modified: DateTime<Utc>,
}

impl PartitionEntry {
    /// Creates a new PartitionEntry with the given table name and partition key.
    ///
    /// The entry is created with:
    /// - `start_time` and `end_time`: current UTC time
    /// - `row_count`: 0
    /// - `size_bytes`: 0
    /// - `is_empty`: true
    /// - `last_modified`: current UTC time
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table this partition belongs to
    /// * `partition_key` - The partition key (e.g., "2024-01")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::partition_metadata::PartitionEntry;
    ///
    /// let partition = PartitionEntry::new("network_activity", "2024-01");
    /// assert_eq!(partition.table_name, "network_activity");
    /// assert_eq!(partition.partition_key, "2024-01");
    /// assert_eq!(partition.row_count, 0);
    /// assert!(partition.is_empty);
    /// ```
    pub fn new(table_name: impl Into<String>, partition_key: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            table_name: table_name.into(),
            partition_key: partition_key.into(),
            start_time: now,
            end_time: now,
            row_count: 0,
            size_bytes: 0,
            is_empty: true,
            last_modified: now,
        }
    }

    /// Sets the time range for this partition.
    ///
    /// # Arguments
    ///
    /// * `start` - The start time of the partition's time range
    /// * `end` - The end time of the partition's time range
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use chrono::{Utc, Duration};
    ///
    /// let start = Utc::now();
    /// let end = start + Duration::hours(1);
    ///
    /// let partition = PartitionEntry::new("network_activity", "2024-01")
    ///     .with_time_range(start, end);
    /// assert_eq!(partition.start_time, start);
    /// assert_eq!(partition.end_time, end);
    /// ```
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    /// Sets the row count for this partition.
    ///
    /// If the row count is 0, `is_empty` is set to true; otherwise, it's set to false.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of rows in this partition
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::partition_metadata::PartitionEntry;
    ///
    /// let partition = PartitionEntry::new("network_activity", "2024-01")
    ///     .with_row_count(1_000_000);
    /// assert_eq!(partition.row_count, 1_000_000);
    /// assert!(!partition.is_empty);
    ///
    /// let empty_partition = PartitionEntry::new("network_activity", "2024-02")
    ///     .with_row_count(0);
    /// assert_eq!(empty_partition.row_count, 0);
    /// assert!(empty_partition.is_empty);
    /// ```
    pub fn with_row_count(mut self, count: u64) -> Self {
        self.row_count = count;
        self.is_empty = count == 0;
        self
    }

    /// Sets the size in bytes for this partition.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the partition in bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::partition_metadata::PartitionEntry;
    ///
    /// let partition = PartitionEntry::new("network_activity", "2024-01")
    ///     .with_size_bytes(500_000_000);
    /// assert_eq!(partition.size_bytes, 500_000_000);
    /// ```
    pub fn with_size_bytes(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    /// Sets the partition ID (typically called by the backend after persistence).
    ///
    /// # Arguments
    ///
    /// * `id` - The partition ID
    pub fn with_id(mut self, id: PartitionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the last modified timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The last modified timestamp
    pub fn with_last_modified(mut self, timestamp: DateTime<Utc>) -> Self {
        self.last_modified = timestamp;
        self
    }

    /// Checks if this partition overlaps with the given time range.
    ///
    /// A partition overlaps with a time range if:
    /// - The partition's start time is before the range's end time, AND
    /// - The partition's end time is after the range's start time
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the time range to check
    /// * `end` - The end of the time range to check
    ///
    /// # Returns
    ///
    /// `true` if the partition overlaps with the given time range, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use chrono::{Utc, Duration};
    ///
    /// let base = Utc::now();
    /// let partition = PartitionEntry::new("network_activity", "2024-01")
    ///     .with_time_range(base, base + Duration::hours(2));
    ///
    /// // Query range fully within partition
    /// assert!(partition.overlaps(base + Duration::minutes(30), base + Duration::minutes(90)));
    ///
    /// // Query range overlaps start
    /// assert!(partition.overlaps(base - Duration::hours(1), base + Duration::minutes(30)));
    ///
    /// // Query range overlaps end
    /// assert!(partition.overlaps(base + Duration::minutes(90), base + Duration::hours(3)));
    ///
    /// // Query range fully contains partition
    /// assert!(partition.overlaps(base - Duration::hours(1), base + Duration::hours(3)));
    ///
    /// // Query range before partition - no overlap
    /// assert!(!partition.overlaps(base - Duration::hours(2), base - Duration::hours(1)));
    ///
    /// // Query range after partition - no overlap
    /// assert!(!partition.overlaps(base + Duration::hours(3), base + Duration::hours(4)));
    /// ```
    pub fn overlaps(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        self.start_time < end && self.end_time > start
    }
}

impl IndexRecord for PartitionEntry {
    fn record_type() -> &'static str {
        "partition"
    }

    fn id(&self) -> Option<RecordId> {
        self.id.map(|id| RecordId::new(id.0))
    }

    fn set_id(&mut self, id: RecordId) {
        self.id = Some(PartitionId::new(id.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // ========================================================================
    // PartitionEntry Construction Tests
    // ========================================================================

    #[test]
    fn test_partition_entry_new() {
        let partition = PartitionEntry::new("network_activity", "2024-01");

        assert!(partition.id.is_none());
        assert_eq!(partition.table_name, "network_activity");
        assert_eq!(partition.partition_key, "2024-01");
        assert_eq!(partition.row_count, 0);
        assert_eq!(partition.size_bytes, 0);
        assert!(partition.is_empty);
    }

    #[test]
    fn test_partition_entry_with_time_range() {
        let start = Utc::now();
        let end = start + Duration::hours(1);

        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_time_range(start, end);

        assert_eq!(partition.start_time, start);
        assert_eq!(partition.end_time, end);
    }

    #[test]
    fn test_partition_entry_with_row_count() {
        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_row_count(1_000_000);

        assert_eq!(partition.row_count, 1_000_000);
        assert!(!partition.is_empty);
    }

    #[test]
    fn test_partition_entry_with_row_count_zero() {
        let partition = PartitionEntry::new("network_activity", "2024-01").with_row_count(0);

        assert_eq!(partition.row_count, 0);
        assert!(partition.is_empty);
    }

    #[test]
    fn test_partition_entry_with_size_bytes() {
        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_size_bytes(500_000_000);

        assert_eq!(partition.size_bytes, 500_000_000);
    }

    #[test]
    fn test_partition_entry_with_id() {
        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_id(PartitionId::new(42));

        assert_eq!(partition.id, Some(PartitionId::new(42)));
    }

    #[test]
    fn test_partition_entry_with_last_modified() {
        let timestamp = Utc::now();
        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_last_modified(timestamp);

        assert_eq!(partition.last_modified, timestamp);
    }

    #[test]
    fn test_partition_entry_builder_chaining() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let modified = Utc::now();

        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1_000_000)
            .with_size_bytes(500_000_000)
            .with_last_modified(modified)
            .with_id(PartitionId::new(123));

        assert_eq!(partition.id, Some(PartitionId::new(123)));
        assert_eq!(partition.table_name, "network_activity");
        assert_eq!(partition.partition_key, "2024-01");
        assert_eq!(partition.start_time, start);
        assert_eq!(partition.end_time, end);
        assert_eq!(partition.row_count, 1_000_000);
        assert_eq!(partition.size_bytes, 500_000_000);
        assert!(!partition.is_empty);
        assert_eq!(partition.last_modified, modified);
    }

    // ========================================================================
    // Overlaps Tests
    // ========================================================================

    #[test]
    fn test_partition_overlaps_query_within_partition() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range fully within partition
        assert!(partition.overlaps(base + Duration::minutes(30), base + Duration::minutes(90)));
    }

    #[test]
    fn test_partition_overlaps_query_overlaps_start() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range overlaps partition start
        assert!(partition.overlaps(base - Duration::hours(1), base + Duration::minutes(30)));
    }

    #[test]
    fn test_partition_overlaps_query_overlaps_end() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range overlaps partition end
        assert!(partition.overlaps(base + Duration::minutes(90), base + Duration::hours(3)));
    }

    #[test]
    fn test_partition_overlaps_query_contains_partition() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range fully contains partition
        assert!(partition.overlaps(base - Duration::hours(1), base + Duration::hours(3)));
    }

    #[test]
    fn test_partition_overlaps_query_before_partition() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range before partition - no overlap
        assert!(!partition.overlaps(base - Duration::hours(2), base - Duration::hours(1)));
    }

    #[test]
    fn test_partition_overlaps_query_after_partition() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range after partition - no overlap
        assert!(!partition.overlaps(base + Duration::hours(3), base + Duration::hours(4)));
    }

    #[test]
    fn test_partition_overlaps_exact_boundary_start() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query ends exactly at partition start - no overlap (exclusive boundary)
        assert!(!partition.overlaps(base - Duration::hours(1), base));
    }

    #[test]
    fn test_partition_overlaps_exact_boundary_end() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query starts exactly at partition end - no overlap (exclusive boundary)
        assert!(!partition.overlaps(base + Duration::hours(2), base + Duration::hours(3)));
    }

    #[test]
    fn test_partition_overlaps_exact_match() {
        let base = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(base, base + Duration::hours(2));

        // Query range exactly matches partition
        assert!(partition.overlaps(base, base + Duration::hours(2)));
    }

    // ========================================================================
    // JSON Serialization Tests
    // ========================================================================

    #[test]
    fn test_partition_entry_json_serialization() {
        let start = Utc::now();
        let end = start + Duration::hours(1);

        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1_000_000)
            .with_size_bytes(500_000_000);

        let json = serde_json::to_string(&partition).unwrap();
        let deserialized: PartitionEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(partition.table_name, deserialized.table_name);
        assert_eq!(partition.partition_key, deserialized.partition_key);
        assert_eq!(partition.start_time, deserialized.start_time);
        assert_eq!(partition.end_time, deserialized.end_time);
        assert_eq!(partition.row_count, deserialized.row_count);
        assert_eq!(partition.size_bytes, deserialized.size_bytes);
        assert_eq!(partition.is_empty, deserialized.is_empty);
    }

    #[test]
    fn test_partition_entry_json_none_id_skipped() {
        let partition = PartitionEntry::new("network_activity", "2024-01");

        let json = serde_json::to_string(&partition).unwrap();

        // None id should be skipped in serialization
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_partition_entry_json_round_trip_with_id() {
        let start = Utc::now();
        let end = start + Duration::hours(1);

        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1_000_000)
            .with_size_bytes(500_000_000)
            .with_id(PartitionId::new(42));

        let json = serde_json::to_string(&partition).unwrap();
        let deserialized: PartitionEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(partition, deserialized);
    }

    // ========================================================================
    // IndexRecord Trait Tests
    // ========================================================================

    #[test]
    fn test_partition_entry_record_type() {
        assert_eq!(PartitionEntry::record_type(), "partition");
    }

    #[test]
    fn test_partition_entry_id_none() {
        let partition = PartitionEntry::new("network_activity", "2024-01");
        assert!(partition.id().is_none());
    }

    #[test]
    fn test_partition_entry_id_some() {
        let partition =
            PartitionEntry::new("network_activity", "2024-01").with_id(PartitionId::new(42));
        assert_eq!(partition.id(), Some(RecordId::new(42)));
    }

    #[test]
    fn test_partition_entry_set_id() {
        let mut partition = PartitionEntry::new("network_activity", "2024-01");
        assert!(partition.id.is_none());

        partition.set_id(RecordId::new(99));
        assert_eq!(partition.id, Some(PartitionId::new(99)));
    }

    // ========================================================================
    // Equality and Clone Tests
    // ========================================================================

    #[test]
    fn test_partition_entry_equality() {
        let start = Utc::now();
        let end = start + Duration::hours(1);
        let modified = Utc::now();

        let partition1 = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1000)
            .with_last_modified(modified);

        let partition2 = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(1000)
            .with_last_modified(modified);

        let partition3 = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(start, end)
            .with_row_count(2000)
            .with_last_modified(modified);

        assert_eq!(partition1, partition2);
        assert_ne!(partition1, partition3);
    }

    #[test]
    fn test_partition_entry_clone() {
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_row_count(1000)
            .with_size_bytes(500_000);

        let cloned = partition.clone();
        assert_eq!(partition, cloned);
    }
}
