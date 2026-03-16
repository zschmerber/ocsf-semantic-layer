//! Source lineage tracking for the OCSF Semantic Index.
//!
//! This module provides types for tracking which raw data sources contributed
//! to each OCSF table, enabling data provenance and debugging of data quality issues.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backend::IndexRecord;
use crate::types::{LineageId, RecordId};

// ============================================================================
// SourceLineageRecord
// ============================================================================

/// Source lineage record tracking which raw data sources contributed to an OCSF table.
///
/// A `SourceLineageRecord` represents a single source-to-target mapping, recording
/// which source system and table contributed data to a target OCSF table. This enables
/// data provenance tracking and debugging of data quality issues.
///
/// # Example
///
/// ```rust
/// use ocsf_index::source_lineage::SourceLineageRecord;
///
/// let lineage = SourceLineageRecord::new("splunk", "raw_network_logs", "network_activity")
///     .with_record_count(1_000_000)
///     .with_metadata("pipeline", "etl-v2")
///     .with_metadata("batch_id", "batch-2024-01-15-001");
///
/// assert_eq!(lineage.source_system, "splunk");
/// assert_eq!(lineage.source_table, "raw_network_logs");
/// assert_eq!(lineage.target_table, "network_activity");
/// assert_eq!(lineage.record_count, Some(1_000_000));
/// assert_eq!(lineage.metadata.get("pipeline"), Some(&"etl-v2".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceLineageRecord {
    /// Unique identifier for this lineage record (assigned by backend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LineageId>,

    /// The source system name (e.g., "splunk", "elastic", "kafka").
    pub source_system: String,

    /// The source table or topic name (e.g., "raw_network_logs", "security-events").
    pub source_table: String,

    /// The target OCSF table name that received the data.
    pub target_table: String,

    /// Timestamp when the data was ingested.
    pub ingestion_timestamp: DateTime<Utc>,

    /// Number of records ingested from this source (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u64>,

    /// Additional metadata key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl SourceLineageRecord {
    /// Creates a new SourceLineageRecord with the given source and target information.
    ///
    /// The record is created with:
    /// - `ingestion_timestamp`: current UTC time
    /// - `record_count`: None
    /// - `metadata`: empty HashMap
    ///
    /// # Arguments
    ///
    /// * `source_system` - The source system name (e.g., "splunk", "elastic")
    /// * `source_table` - The source table or topic name
    /// * `target_table` - The target OCSF table name
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    ///
    /// let lineage = SourceLineageRecord::new("splunk", "raw_network_logs", "network_activity");
    /// assert_eq!(lineage.source_system, "splunk");
    /// assert_eq!(lineage.source_table, "raw_network_logs");
    /// assert_eq!(lineage.target_table, "network_activity");
    /// assert!(lineage.record_count.is_none());
    /// assert!(lineage.metadata.is_empty());
    /// ```
    pub fn new(
        source_system: impl Into<String>,
        source_table: impl Into<String>,
        target_table: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            source_system: source_system.into(),
            source_table: source_table.into(),
            target_table: target_table.into(),
            ingestion_timestamp: Utc::now(),
            record_count: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the record count for this lineage record.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of records ingested from this source
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    ///
    /// let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
    ///     .with_record_count(1_000_000);
    /// assert_eq!(lineage.record_count, Some(1_000_000));
    /// ```
    pub fn with_record_count(mut self, count: u64) -> Self {
        self.record_count = Some(count);
        self
    }

    /// Adds a metadata key-value pair to this lineage record.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    ///
    /// let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
    ///     .with_metadata("pipeline", "etl-v2")
    ///     .with_metadata("batch_id", "batch-001");
    /// assert_eq!(lineage.metadata.get("pipeline"), Some(&"etl-v2".to_string()));
    /// assert_eq!(lineage.metadata.get("batch_id"), Some(&"batch-001".to_string()));
    /// ```
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets the ingestion timestamp for this lineage record.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The ingestion timestamp
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    /// use chrono::Utc;
    ///
    /// let timestamp = Utc::now();
    /// let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
    ///     .with_ingestion_timestamp(timestamp);
    /// assert_eq!(lineage.ingestion_timestamp, timestamp);
    /// ```
    pub fn with_ingestion_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.ingestion_timestamp = timestamp;
        self
    }

    /// Sets the lineage ID (typically called by the backend after persistence).
    ///
    /// # Arguments
    ///
    /// * `id` - The lineage ID
    pub fn with_id(mut self, id: LineageId) -> Self {
        self.id = Some(id);
        self
    }
}

impl IndexRecord for SourceLineageRecord {
    fn record_type() -> &'static str {
        "source_lineage"
    }

    fn id(&self) -> Option<RecordId> {
        self.id.map(|id| RecordId::new(id.0))
    }

    fn set_id(&mut self, id: RecordId) {
        self.id = Some(LineageId::new(id.0));
    }
}

// ============================================================================
// LineageEdge (for visualization)
// ============================================================================

/// A lineage edge for graph visualization.
///
/// This struct represents a source-to-target relationship in a format suitable
/// for rendering lineage graphs in the UI.
///
/// # Example
///
/// ```rust
/// use ocsf_index::source_lineage::{SourceLineageRecord, LineageEdge};
///
/// let record = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
///     .with_record_count(1000);
/// let edge: LineageEdge = record.into();
///
/// assert_eq!(edge.source, "splunk:raw_logs");
/// assert_eq!(edge.target, "network_activity");
/// assert_eq!(edge.record_count, Some(1000));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageEdge {
    /// The source identifier (format: "system:table").
    pub source: String,

    /// The target table name.
    pub target: String,

    /// Timestamp of the lineage relationship.
    pub timestamp: DateTime<Utc>,

    /// Number of records in this lineage relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u64>,
}

impl From<SourceLineageRecord> for LineageEdge {
    fn from(record: SourceLineageRecord) -> Self {
        Self {
            source: format!("{}:{}", record.source_system, record.source_table),
            target: record.target_table,
            timestamp: record.ingestion_timestamp,
            record_count: record.record_count,
        }
    }
}

impl From<&SourceLineageRecord> for LineageEdge {
    fn from(record: &SourceLineageRecord) -> Self {
        Self {
            source: format!("{}:{}", record.source_system, record.source_table),
            target: record.target_table.clone(),
            timestamp: record.ingestion_timestamp,
            record_count: record.record_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // SourceLineageRecord Tests
    // ========================================================================

    #[test]
    fn test_source_lineage_record_new() {
        let lineage = SourceLineageRecord::new("splunk", "raw_network_logs", "network_activity");

        assert!(lineage.id.is_none());
        assert_eq!(lineage.source_system, "splunk");
        assert_eq!(lineage.source_table, "raw_network_logs");
        assert_eq!(lineage.target_table, "network_activity");
        assert!(lineage.record_count.is_none());
        assert!(lineage.metadata.is_empty());
    }

    #[test]
    fn test_source_lineage_record_with_record_count() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1_000_000);

        assert_eq!(lineage.record_count, Some(1_000_000));
    }

    #[test]
    fn test_source_lineage_record_with_record_count_zero() {
        let lineage =
            SourceLineageRecord::new("splunk", "raw_logs", "network_activity").with_record_count(0);

        assert_eq!(lineage.record_count, Some(0));
    }

    #[test]
    fn test_source_lineage_record_with_metadata() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_metadata("pipeline", "etl-v2")
            .with_metadata("batch_id", "batch-001");

        assert_eq!(lineage.metadata.len(), 2);
        assert_eq!(
            lineage.metadata.get("pipeline"),
            Some(&"etl-v2".to_string())
        );
        assert_eq!(
            lineage.metadata.get("batch_id"),
            Some(&"batch-001".to_string())
        );
    }

    #[test]
    fn test_source_lineage_record_with_ingestion_timestamp() {
        let timestamp = Utc::now();
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_ingestion_timestamp(timestamp);

        assert_eq!(lineage.ingestion_timestamp, timestamp);
    }

    #[test]
    fn test_source_lineage_record_with_id() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_id(LineageId::new(42));

        assert_eq!(lineage.id, Some(LineageId::new(42)));
    }

    #[test]
    fn test_source_lineage_record_builder_chaining() {
        let timestamp = Utc::now();
        let lineage = SourceLineageRecord::new("kafka", "security-events", "authentication_events")
            .with_record_count(500_000)
            .with_metadata("topic", "security-events")
            .with_metadata("partition", "0")
            .with_ingestion_timestamp(timestamp)
            .with_id(LineageId::new(123));

        assert_eq!(lineage.id, Some(LineageId::new(123)));
        assert_eq!(lineage.source_system, "kafka");
        assert_eq!(lineage.source_table, "security-events");
        assert_eq!(lineage.target_table, "authentication_events");
        assert_eq!(lineage.record_count, Some(500_000));
        assert_eq!(lineage.metadata.len(), 2);
        assert_eq!(lineage.ingestion_timestamp, timestamp);
    }

    #[test]
    fn test_source_lineage_record_json_serialization() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000)
            .with_metadata("pipeline", "etl-v2");

        let json = serde_json::to_string(&lineage).unwrap();
        let deserialized: SourceLineageRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(lineage.source_system, deserialized.source_system);
        assert_eq!(lineage.source_table, deserialized.source_table);
        assert_eq!(lineage.target_table, deserialized.target_table);
        assert_eq!(lineage.record_count, deserialized.record_count);
        assert_eq!(lineage.metadata, deserialized.metadata);
    }

    #[test]
    fn test_source_lineage_record_json_empty_metadata_skipped() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");

        let json = serde_json::to_string(&lineage).unwrap();

        // Empty metadata should be skipped in serialization
        assert!(!json.contains("metadata"));
    }

    #[test]
    fn test_source_lineage_record_json_none_record_count_skipped() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");

        let json = serde_json::to_string(&lineage).unwrap();

        // None record_count should be skipped in serialization
        assert!(!json.contains("record_count"));
    }

    #[test]
    fn test_source_lineage_record_json_none_id_skipped() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");

        let json = serde_json::to_string(&lineage).unwrap();

        // None id should be skipped in serialization
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_source_lineage_record_json_deserialization_with_defaults() {
        // JSON with only required fields should deserialize with defaults
        let json = r#"{
            "source_system": "splunk",
            "source_table": "raw_logs",
            "target_table": "network_activity",
            "ingestion_timestamp": "2024-01-15T10:30:00Z"
        }"#;
        let lineage: SourceLineageRecord = serde_json::from_str(json).unwrap();

        assert_eq!(lineage.source_system, "splunk");
        assert_eq!(lineage.source_table, "raw_logs");
        assert_eq!(lineage.target_table, "network_activity");
        assert!(lineage.id.is_none());
        assert!(lineage.record_count.is_none());
        assert!(lineage.metadata.is_empty());
    }

    #[test]
    fn test_source_lineage_record_equality() {
        let timestamp = Utc::now();
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000)
            .with_ingestion_timestamp(timestamp);

        let lineage2 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000)
            .with_ingestion_timestamp(timestamp);

        let lineage3 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(2000)
            .with_ingestion_timestamp(timestamp);

        assert_eq!(lineage1, lineage2);
        assert_ne!(lineage1, lineage3);
    }

    #[test]
    fn test_source_lineage_record_clone() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000)
            .with_metadata("key", "value");

        let cloned = lineage.clone();
        assert_eq!(lineage, cloned);
    }

    // ========================================================================
    // IndexRecord Trait Tests
    // ========================================================================

    #[test]
    fn test_source_lineage_record_record_type() {
        assert_eq!(SourceLineageRecord::record_type(), "source_lineage");
    }

    #[test]
    fn test_source_lineage_record_id_none() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        assert!(lineage.id().is_none());
    }

    #[test]
    fn test_source_lineage_record_id_some() {
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_id(LineageId::new(42));
        assert_eq!(lineage.id(), Some(RecordId::new(42)));
    }

    #[test]
    fn test_source_lineage_record_set_id() {
        let mut lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        assert!(lineage.id.is_none());

        lineage.set_id(RecordId::new(99));
        assert_eq!(lineage.id, Some(LineageId::new(99)));
    }

    // ========================================================================
    // LineageEdge Tests
    // ========================================================================

    #[test]
    fn test_lineage_edge_from_source_lineage_record() {
        let record = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000);
        let edge: LineageEdge = record.clone().into();

        assert_eq!(edge.source, "splunk:raw_logs");
        assert_eq!(edge.target, "network_activity");
        assert_eq!(edge.timestamp, record.ingestion_timestamp);
        assert_eq!(edge.record_count, Some(1000));
    }

    #[test]
    fn test_lineage_edge_from_source_lineage_record_ref() {
        let record =
            SourceLineageRecord::new("kafka", "events", "auth_events").with_record_count(500);
        let edge: LineageEdge = (&record).into();

        assert_eq!(edge.source, "kafka:events");
        assert_eq!(edge.target, "auth_events");
        assert_eq!(edge.timestamp, record.ingestion_timestamp);
        assert_eq!(edge.record_count, Some(500));
    }

    #[test]
    fn test_lineage_edge_from_source_lineage_record_no_count() {
        let record = SourceLineageRecord::new("elastic", "logs", "security_events");
        let edge: LineageEdge = record.into();

        assert_eq!(edge.source, "elastic:logs");
        assert_eq!(edge.target, "security_events");
        assert!(edge.record_count.is_none());
    }

    #[test]
    fn test_lineage_edge_json_serialization() {
        let record = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000);
        let edge: LineageEdge = record.into();

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: LineageEdge = serde_json::from_str(&json).unwrap();

        assert_eq!(edge, deserialized);
    }

    #[test]
    fn test_lineage_edge_json_none_record_count_skipped() {
        let record = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        let edge: LineageEdge = record.into();

        let json = serde_json::to_string(&edge).unwrap();

        // None record_count should be skipped in serialization
        assert!(!json.contains("record_count"));
    }

    #[test]
    fn test_lineage_edge_equality() {
        let timestamp = Utc::now();
        let edge1 = LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp,
            record_count: Some(1000),
        };

        let edge2 = LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp,
            record_count: Some(1000),
        };

        let edge3 = LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp,
            record_count: Some(2000),
        };

        assert_eq!(edge1, edge2);
        assert_ne!(edge1, edge3);
    }

    #[test]
    fn test_lineage_edge_clone() {
        let edge = LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp: Utc::now(),
            record_count: Some(1000),
        };

        let cloned = edge.clone();
        assert_eq!(edge, cloned);
    }
}
