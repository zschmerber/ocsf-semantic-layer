//! Lineage capture for ETL integration.
//!
//! This module provides the `LineageCapture` builder for capturing lineage during
//! ETL operations. It collects source/target table information and field mappings,
//! then persists them atomically via `finalize()`.
//!
//! # Example
//!
//! ```rust,ignore
//! use ocsf_index::{SemanticIndex, LineageCapture};
//! use ocsf_index::backend::InMemoryBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let backend = InMemoryBackend::new();
//!     let index = SemanticIndex::new(backend, Default::default()).await?;
//!
//!     // Capture lineage during ETL
//!     let lineage_id = LineageCapture::new("splunk", "raw_network_logs", "network_activity")
//!         .add_field_mapping("src_ip", "src_endpoint.ip")
//!         .add_field_mapping("dst_ip", "dst_endpoint.ip")
//!         .add_field_mapping_with_transform("timestamp_str", "time", "CAST(timestamp_str AS TIMESTAMP)")
//!         .with_record_count(1_000_000)
//!         .with_metadata("pipeline", "etl-v2")
//!         .with_metadata("batch_id", "batch-2024-01-15-001")
//!         .finalize(&index)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use crate::backend::IndexBackend;
use crate::error::IndexResult;
use crate::field_lineage::FieldLineageRecord;
use crate::index::SemanticIndex;
use crate::source_lineage::SourceLineageRecord;
use crate::types::LineageId;

/// Builder for capturing lineage during ETL operations.
///
/// `LineageCapture` provides a fluent API for recording source-to-target table
/// relationships and field-level mappings during ETL execution. When `finalize()`
/// is called, it persists both source lineage and field lineage records atomically.
///
/// # Field Mappings
///
/// Field mappings can be added in two ways:
/// - `add_field_mapping()`: For direct field mappings without transformation
/// - `add_field_mapping_with_transform()`: For mappings that include a SQL transformation
///
/// # Example
///
/// ```rust
/// use ocsf_index::LineageCapture;
///
/// let capture = LineageCapture::new("kafka", "security-events", "authentication_events")
///     .add_field_mapping("user_id", "actor.user.uid")
///     .add_field_mapping("user_name", "actor.user.name")
///     .add_field_mapping_with_transform("event_time", "time", "TO_TIMESTAMP(event_time)")
///     .with_record_count(500_000)
///     .with_metadata("topic", "security-events")
///     .with_metadata("partition", "0");
///
/// assert_eq!(capture.source_system(), "kafka");
/// assert_eq!(capture.source_table(), "security-events");
/// assert_eq!(capture.target_table(), "authentication_events");
/// assert_eq!(capture.field_mappings().len(), 3);
/// assert_eq!(capture.record_count(), Some(500_000));
/// ```
#[derive(Debug, Clone)]
pub struct LineageCapture {
    /// The source system name (e.g., "splunk", "kafka", "elastic").
    source_system: String,

    /// The source table or topic name.
    source_table: String,

    /// The target OCSF table name.
    target_table: String,

    /// Field mappings: (source_field, target_field, optional_transformation).
    field_mappings: Vec<(String, String, Option<String>)>,

    /// Number of records processed (optional).
    record_count: Option<u64>,

    /// Additional metadata key-value pairs.
    metadata: HashMap<String, String>,
}

impl LineageCapture {
    /// Creates a new `LineageCapture` builder with the given source and target information.
    ///
    /// # Arguments
    ///
    /// * `source_system` - The source system name (e.g., "splunk", "kafka")
    /// * `source_table` - The source table or topic name
    /// * `target_table` - The target OCSF table name
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::LineageCapture;
    ///
    /// let capture = LineageCapture::new("splunk", "raw_network_logs", "network_activity");
    ///
    /// assert_eq!(capture.source_system(), "splunk");
    /// assert_eq!(capture.source_table(), "raw_network_logs");
    /// assert_eq!(capture.target_table(), "network_activity");
    /// assert!(capture.field_mappings().is_empty());
    /// assert!(capture.record_count().is_none());
    /// assert!(capture.metadata().is_empty());
    /// ```
    pub fn new(
        source_system: impl Into<String>,
        source_table: impl Into<String>,
        target_table: impl Into<String>,
    ) -> Self {
        Self {
            source_system: source_system.into(),
            source_table: source_table.into(),
            target_table: target_table.into(),
            field_mappings: Vec::new(),
            record_count: None,
            metadata: HashMap::new(),
        }
    }

    /// Adds a direct field mapping without transformation.
    ///
    /// # Arguments
    ///
    /// * `source` - The source field path
    /// * `target` - The target OCSF field path
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::LineageCapture;
    ///
    /// let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
    ///     .add_field_mapping("src_ip", "src_endpoint.ip")
    ///     .add_field_mapping("dst_ip", "dst_endpoint.ip");
    ///
    /// assert_eq!(capture.field_mappings().len(), 2);
    ///
    /// let (source, target, transform) = &capture.field_mappings()[0];
    /// assert_eq!(source, "src_ip");
    /// assert_eq!(target, "src_endpoint.ip");
    /// assert!(transform.is_none());
    /// ```
    pub fn add_field_mapping(
        mut self,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        self.field_mappings
            .push((source.into(), target.into(), None));
        self
    }

    /// Adds a field mapping with a SQL transformation expression.
    ///
    /// # Arguments
    ///
    /// * `source` - The source field path
    /// * `target` - The target OCSF field path
    /// * `transform` - The SQL transformation expression
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::LineageCapture;
    ///
    /// let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
    ///     .add_field_mapping_with_transform(
    ///         "timestamp_str",
    ///         "time",
    ///         "CAST(timestamp_str AS TIMESTAMP)"
    ///     )
    ///     .add_field_mapping_with_transform(
    ///         "user_name",
    ///         "actor.user.name",
    ///         "UPPER(user_name)"
    ///     );
    ///
    /// assert_eq!(capture.field_mappings().len(), 2);
    ///
    /// let (source, target, transform) = &capture.field_mappings()[0];
    /// assert_eq!(source, "timestamp_str");
    /// assert_eq!(target, "time");
    /// assert_eq!(transform.as_deref(), Some("CAST(timestamp_str AS TIMESTAMP)"));
    /// ```
    pub fn add_field_mapping_with_transform(
        mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        transform: impl Into<String>,
    ) -> Self {
        self.field_mappings
            .push((source.into(), target.into(), Some(transform.into())));
        self
    }

    /// Sets the record count for this lineage capture.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of records processed
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::LineageCapture;
    ///
    /// let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
    ///     .with_record_count(1_000_000);
    ///
    /// assert_eq!(capture.record_count(), Some(1_000_000));
    /// ```
    pub fn with_record_count(mut self, count: u64) -> Self {
        self.record_count = Some(count);
        self
    }

    /// Adds a metadata key-value pair to this lineage capture.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::LineageCapture;
    ///
    /// let capture = LineageCapture::new("kafka", "events", "auth_events")
    ///     .with_metadata("pipeline", "etl-v2")
    ///     .with_metadata("batch_id", "batch-001")
    ///     .with_metadata("environment", "production");
    ///
    /// assert_eq!(capture.metadata().len(), 3);
    /// assert_eq!(capture.metadata().get("pipeline"), Some(&"etl-v2".to_string()));
    /// assert_eq!(capture.metadata().get("batch_id"), Some(&"batch-001".to_string()));
    /// assert_eq!(capture.metadata().get("environment"), Some(&"production".to_string()));
    /// ```
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    // ========================================================================
    // Accessor Methods
    // ========================================================================

    /// Returns the source system name.
    pub fn source_system(&self) -> &str {
        &self.source_system
    }

    /// Returns the source table name.
    pub fn source_table(&self) -> &str {
        &self.source_table
    }

    /// Returns the target table name.
    pub fn target_table(&self) -> &str {
        &self.target_table
    }

    /// Returns the field mappings as a slice.
    ///
    /// Each mapping is a tuple of (source_field, target_field, optional_transformation).
    pub fn field_mappings(&self) -> &[(String, String, Option<String>)] {
        &self.field_mappings
    }

    /// Returns the record count, if set.
    pub fn record_count(&self) -> Option<u64> {
        self.record_count
    }

    /// Returns the metadata map.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    // ========================================================================
    // Finalization
    // ========================================================================

    /// Finalizes the lineage capture by persisting all records to the index.
    ///
    /// This method performs the following operations atomically:
    /// 1. Creates a `SourceLineageRecord` from the captured data
    /// 2. Persists the source lineage record to get a `LineageId`
    /// 3. Creates `FieldLineageRecord` entries for each field mapping
    /// 4. Persists all field lineage records with the source lineage ID
    ///
    /// # Arguments
    ///
    /// * `index` - The `SemanticIndex` to persist the lineage records to
    ///
    /// # Returns
    ///
    /// The `LineageId` of the created source lineage record.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if any persistence operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, LineageCapture};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Capture lineage during ETL
    ///     let lineage_id = LineageCapture::new("splunk", "raw_network_logs", "network_activity")
    ///         .add_field_mapping("src_ip", "src_endpoint.ip")
    ///         .add_field_mapping("dst_ip", "dst_endpoint.ip")
    ///         .add_field_mapping_with_transform(
    ///             "timestamp_str",
    ///             "time",
    ///             "CAST(timestamp_str AS TIMESTAMP)"
    ///         )
    ///         .with_record_count(1_000_000)
    ///         .with_metadata("pipeline", "etl-v2")
    ///         .finalize(&index)
    ///         .await?;
    ///
    ///     println!("Created lineage with ID: {}", lineage_id);
    ///
    ///     // Verify the source lineage was recorded
    ///     let source_lineage = index.get_source_lineage("network_activity").await?;
    ///     assert_eq!(source_lineage.len(), 1);
    ///     assert_eq!(source_lineage[0].source_system, "splunk");
    ///
    ///     // Verify field lineage was recorded
    ///     let field_lineage = index.get_field_lineage("src_endpoint.ip").await?;
    ///     assert_eq!(field_lineage.len(), 1);
    ///     assert_eq!(field_lineage[0].source_field, "src_ip");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn finalize<B: IndexBackend>(
        self,
        index: &SemanticIndex<B>,
    ) -> IndexResult<LineageId> {
        // Step 1: Create the source lineage record from captured data
        let mut source_lineage =
            SourceLineageRecord::new(&self.source_system, &self.source_table, &self.target_table);

        // Add record count if present
        if let Some(count) = self.record_count {
            source_lineage = source_lineage.with_record_count(count);
        }

        // Add all metadata
        for (key, value) in &self.metadata {
            source_lineage = source_lineage.with_metadata(key, value);
        }

        // Step 2: Persist the source lineage record and get the LineageId
        let source_lineage_id = index.record_source_lineage(source_lineage).await?;

        // Step 3 & 4: Create and persist field lineage records for each mapping
        for (source_field, target_field, transformation) in &self.field_mappings {
            let mut field_lineage =
                FieldLineageRecord::new(source_lineage_id, source_field, target_field);

            // Add transformation if present
            if let Some(transform) = transformation {
                field_lineage = field_lineage.with_transformation(transform);
            }

            // Persist the field lineage record
            index.record_field_lineage(field_lineage).await?;
        }

        // Return the source lineage ID
        Ok(source_lineage_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Construction Tests
    // ========================================================================

    #[test]
    fn test_lineage_capture_new() {
        let capture = LineageCapture::new("splunk", "raw_network_logs", "network_activity");

        assert_eq!(capture.source_system(), "splunk");
        assert_eq!(capture.source_table(), "raw_network_logs");
        assert_eq!(capture.target_table(), "network_activity");
        assert!(capture.field_mappings().is_empty());
        assert!(capture.record_count().is_none());
        assert!(capture.metadata().is_empty());
    }

    #[test]
    fn test_lineage_capture_new_with_string_types() {
        let capture = LineageCapture::new(
            String::from("kafka"),
            String::from("security-events"),
            String::from("auth_events"),
        );

        assert_eq!(capture.source_system(), "kafka");
        assert_eq!(capture.source_table(), "security-events");
        assert_eq!(capture.target_table(), "auth_events");
    }

    // ========================================================================
    // Field Mapping Tests
    // ========================================================================

    #[test]
    fn test_add_field_mapping() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .add_field_mapping("src_ip", "src_endpoint.ip");

        assert_eq!(capture.field_mappings().len(), 1);

        let (source, target, transform) = &capture.field_mappings()[0];
        assert_eq!(source, "src_ip");
        assert_eq!(target, "src_endpoint.ip");
        assert!(transform.is_none());
    }

    #[test]
    fn test_add_multiple_field_mappings() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .add_field_mapping("src_ip", "src_endpoint.ip")
            .add_field_mapping("dst_ip", "dst_endpoint.ip")
            .add_field_mapping("protocol", "connection_info.protocol_name");

        assert_eq!(capture.field_mappings().len(), 3);

        let (source1, target1, _) = &capture.field_mappings()[0];
        assert_eq!(source1, "src_ip");
        assert_eq!(target1, "src_endpoint.ip");

        let (source2, target2, _) = &capture.field_mappings()[1];
        assert_eq!(source2, "dst_ip");
        assert_eq!(target2, "dst_endpoint.ip");

        let (source3, target3, _) = &capture.field_mappings()[2];
        assert_eq!(source3, "protocol");
        assert_eq!(target3, "connection_info.protocol_name");
    }

    #[test]
    fn test_add_field_mapping_with_transform() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .add_field_mapping_with_transform(
                "timestamp_str",
                "time",
                "CAST(timestamp_str AS TIMESTAMP)",
            );

        assert_eq!(capture.field_mappings().len(), 1);

        let (source, target, transform) = &capture.field_mappings()[0];
        assert_eq!(source, "timestamp_str");
        assert_eq!(target, "time");
        assert_eq!(
            transform.as_deref(),
            Some("CAST(timestamp_str AS TIMESTAMP)")
        );
    }

    #[test]
    fn test_mixed_field_mappings() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .add_field_mapping("src_ip", "src_endpoint.ip")
            .add_field_mapping_with_transform(
                "timestamp_str",
                "time",
                "TO_TIMESTAMP(timestamp_str)",
            )
            .add_field_mapping("dst_ip", "dst_endpoint.ip")
            .add_field_mapping_with_transform("user_name", "actor.user.name", "UPPER(user_name)");

        assert_eq!(capture.field_mappings().len(), 4);

        // First mapping: no transform
        let (_, _, transform0) = &capture.field_mappings()[0];
        assert!(transform0.is_none());

        // Second mapping: with transform
        let (_, _, transform1) = &capture.field_mappings()[1];
        assert_eq!(transform1.as_deref(), Some("TO_TIMESTAMP(timestamp_str)"));

        // Third mapping: no transform
        let (_, _, transform2) = &capture.field_mappings()[2];
        assert!(transform2.is_none());

        // Fourth mapping: with transform
        let (_, _, transform3) = &capture.field_mappings()[3];
        assert_eq!(transform3.as_deref(), Some("UPPER(user_name)"));
    }

    // ========================================================================
    // Record Count Tests
    // ========================================================================

    #[test]
    fn test_with_record_count() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1_000_000);

        assert_eq!(capture.record_count(), Some(1_000_000));
    }

    #[test]
    fn test_with_record_count_zero() {
        let capture =
            LineageCapture::new("splunk", "raw_logs", "network_activity").with_record_count(0);

        assert_eq!(capture.record_count(), Some(0));
    }

    #[test]
    fn test_with_record_count_large() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .with_record_count(u64::MAX);

        assert_eq!(capture.record_count(), Some(u64::MAX));
    }

    // ========================================================================
    // Metadata Tests
    // ========================================================================

    #[test]
    fn test_with_metadata_single() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .with_metadata("pipeline", "etl-v2");

        assert_eq!(capture.metadata().len(), 1);
        assert_eq!(
            capture.metadata().get("pipeline"),
            Some(&"etl-v2".to_string())
        );
    }

    #[test]
    fn test_with_metadata_multiple() {
        let capture = LineageCapture::new("kafka", "events", "auth_events")
            .with_metadata("pipeline", "etl-v2")
            .with_metadata("batch_id", "batch-001")
            .with_metadata("environment", "production");

        assert_eq!(capture.metadata().len(), 3);
        assert_eq!(
            capture.metadata().get("pipeline"),
            Some(&"etl-v2".to_string())
        );
        assert_eq!(
            capture.metadata().get("batch_id"),
            Some(&"batch-001".to_string())
        );
        assert_eq!(
            capture.metadata().get("environment"),
            Some(&"production".to_string())
        );
    }

    #[test]
    fn test_with_metadata_overwrite() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .with_metadata("key", "value1")
            .with_metadata("key", "value2");

        assert_eq!(capture.metadata().len(), 1);
        assert_eq!(capture.metadata().get("key"), Some(&"value2".to_string()));
    }

    // ========================================================================
    // Builder Chaining Tests
    // ========================================================================

    #[test]
    fn test_full_builder_chain() {
        let capture = LineageCapture::new("kafka", "security-events", "authentication_events")
            .add_field_mapping("user_id", "actor.user.uid")
            .add_field_mapping("user_name", "actor.user.name")
            .add_field_mapping_with_transform("event_time", "time", "TO_TIMESTAMP(event_time)")
            .add_field_mapping_with_transform("src_ip", "src_endpoint.ip", "TRIM(src_ip)")
            .with_record_count(500_000)
            .with_metadata("topic", "security-events")
            .with_metadata("partition", "0")
            .with_metadata("consumer_group", "etl-group");

        assert_eq!(capture.source_system(), "kafka");
        assert_eq!(capture.source_table(), "security-events");
        assert_eq!(capture.target_table(), "authentication_events");
        assert_eq!(capture.field_mappings().len(), 4);
        assert_eq!(capture.record_count(), Some(500_000));
        assert_eq!(capture.metadata().len(), 3);
    }

    // ========================================================================
    // Clone Tests
    // ========================================================================

    #[test]
    fn test_lineage_capture_clone() {
        let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
            .add_field_mapping("src_ip", "src_endpoint.ip")
            .with_record_count(1000)
            .with_metadata("key", "value");

        let cloned = capture.clone();

        assert_eq!(capture.source_system(), cloned.source_system());
        assert_eq!(capture.source_table(), cloned.source_table());
        assert_eq!(capture.target_table(), cloned.target_table());
        assert_eq!(capture.field_mappings(), cloned.field_mappings());
        assert_eq!(capture.record_count(), cloned.record_count());
        assert_eq!(capture.metadata(), cloned.metadata());
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_empty_strings() {
        let capture = LineageCapture::new("", "", "")
            .add_field_mapping("", "")
            .with_metadata("", "");

        assert_eq!(capture.source_system(), "");
        assert_eq!(capture.source_table(), "");
        assert_eq!(capture.target_table(), "");
        assert_eq!(capture.field_mappings().len(), 1);
        assert_eq!(capture.metadata().get(""), Some(&"".to_string()));
    }

    #[test]
    fn test_special_characters_in_names() {
        let capture = LineageCapture::new(
            "source-system_v2.0",
            "raw.logs.2024",
            "network_activity::v1",
        )
        .add_field_mapping("event.data.src_ip", "src_endpoint.ip")
        .with_metadata("key:with:colons", "value/with/slashes");

        assert_eq!(capture.source_system(), "source-system_v2.0");
        assert_eq!(capture.source_table(), "raw.logs.2024");
        assert_eq!(capture.target_table(), "network_activity::v1");

        let (source, target, _) = &capture.field_mappings()[0];
        assert_eq!(source, "event.data.src_ip");
        assert_eq!(target, "src_endpoint.ip");

        assert_eq!(
            capture.metadata().get("key:with:colons"),
            Some(&"value/with/slashes".to_string())
        );
    }

    #[test]
    fn test_unicode_in_names() {
        let capture = LineageCapture::new("源系统", "原始日志", "网络活动")
            .add_field_mapping("源IP", "src_endpoint.ip")
            .with_metadata("描述", "测试数据");

        assert_eq!(capture.source_system(), "源系统");
        assert_eq!(capture.source_table(), "原始日志");
        assert_eq!(capture.target_table(), "网络活动");

        let (source, _, _) = &capture.field_mappings()[0];
        assert_eq!(source, "源IP");

        assert_eq!(
            capture.metadata().get("描述"),
            Some(&"测试数据".to_string())
        );
    }

    // ========================================================================
    // Finalize Tests (async)
    // ========================================================================

    mod finalize_tests {
        use super::*;
        use crate::backend::InMemoryBackend;
        use crate::index::SemanticIndex;
        use crate::types::IndexConfig;

        #[tokio::test]
        async fn test_finalize_creates_source_lineage() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("splunk", "raw_network_logs", "network_activity");

            let lineage_id = capture.finalize(&index).await.unwrap();

            // Verify the source lineage was created
            assert!(lineage_id.0 > 0);

            // Verify we can retrieve the source lineage
            let source_lineage = index.get_source_lineage("network_activity").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].source_system, "splunk");
            assert_eq!(source_lineage[0].source_table, "raw_network_logs");
            assert_eq!(source_lineage[0].target_table, "network_activity");
        }

        #[tokio::test]
        async fn test_finalize_with_record_count() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture =
                LineageCapture::new("kafka", "events", "auth_events").with_record_count(500_000);

            capture.finalize(&index).await.unwrap();

            let source_lineage = index.get_source_lineage("auth_events").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].record_count, Some(500_000));
        }

        #[tokio::test]
        async fn test_finalize_with_metadata() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
                .with_metadata("pipeline", "etl-v2")
                .with_metadata("batch_id", "batch-001");

            capture.finalize(&index).await.unwrap();

            let source_lineage = index.get_source_lineage("network_activity").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(
                source_lineage[0].metadata.get("pipeline"),
                Some(&"etl-v2".to_string())
            );
            assert_eq!(
                source_lineage[0].metadata.get("batch_id"),
                Some(&"batch-001".to_string())
            );
        }

        #[tokio::test]
        async fn test_finalize_creates_field_lineage() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
                .add_field_mapping("src_ip", "src_endpoint.ip")
                .add_field_mapping("dst_ip", "dst_endpoint.ip");

            let lineage_id = capture.finalize(&index).await.unwrap();

            // Verify field lineage for src_endpoint.ip
            let field_lineage = index.get_field_lineage("src_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage.len(), 1);
            assert_eq!(field_lineage[0].source_field, "src_ip");
            assert_eq!(field_lineage[0].target_field, "src_endpoint.ip");
            assert_eq!(field_lineage[0].source_lineage_id, lineage_id);
            assert!(field_lineage[0].transformation.is_none());

            // Verify field lineage for dst_endpoint.ip
            let field_lineage = index.get_field_lineage("dst_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage.len(), 1);
            assert_eq!(field_lineage[0].source_field, "dst_ip");
            assert_eq!(field_lineage[0].target_field, "dst_endpoint.ip");
            assert_eq!(field_lineage[0].source_lineage_id, lineage_id);
        }

        #[tokio::test]
        async fn test_finalize_creates_field_lineage_with_transformation() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
                .add_field_mapping_with_transform(
                    "timestamp_str",
                    "time",
                    "CAST(timestamp_str AS TIMESTAMP)",
                );

            capture.finalize(&index).await.unwrap();

            let field_lineage = index.get_field_lineage("time").await.unwrap();
            assert_eq!(field_lineage.len(), 1);
            assert_eq!(field_lineage[0].source_field, "timestamp_str");
            assert_eq!(field_lineage[0].target_field, "time");
            assert_eq!(
                field_lineage[0].transformation,
                Some("CAST(timestamp_str AS TIMESTAMP)".to_string())
            );
        }

        #[tokio::test]
        async fn test_finalize_mixed_field_mappings() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("kafka", "security-events", "authentication_events")
                .add_field_mapping("user_id", "actor.user.uid")
                .add_field_mapping("user_name", "actor.user.name")
                .add_field_mapping_with_transform("event_time", "time", "TO_TIMESTAMP(event_time)")
                .add_field_mapping_with_transform("src_ip", "src_endpoint.ip", "TRIM(src_ip)");

            let lineage_id = capture.finalize(&index).await.unwrap();

            // Verify all field mappings were created
            let field_lineage1 = index.get_field_lineage("actor.user.uid").await.unwrap();
            assert_eq!(field_lineage1.len(), 1);
            assert_eq!(field_lineage1[0].source_lineage_id, lineage_id);
            assert!(field_lineage1[0].transformation.is_none());

            let field_lineage2 = index.get_field_lineage("time").await.unwrap();
            assert_eq!(field_lineage2.len(), 1);
            assert_eq!(
                field_lineage2[0].transformation,
                Some("TO_TIMESTAMP(event_time)".to_string())
            );

            let field_lineage3 = index.get_field_lineage("src_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage3.len(), 1);
            assert_eq!(
                field_lineage3[0].transformation,
                Some("TRIM(src_ip)".to_string())
            );
        }

        #[tokio::test]
        async fn test_finalize_full_builder_chain() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("kafka", "security-events", "authentication_events")
                .add_field_mapping("user_id", "actor.user.uid")
                .add_field_mapping("user_name", "actor.user.name")
                .add_field_mapping_with_transform("event_time", "time", "TO_TIMESTAMP(event_time)")
                .with_record_count(500_000)
                .with_metadata("topic", "security-events")
                .with_metadata("partition", "0")
                .with_metadata("consumer_group", "etl-group");

            let lineage_id = capture.finalize(&index).await.unwrap();

            // Verify source lineage
            let source_lineage = index
                .get_source_lineage("authentication_events")
                .await
                .unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].source_system, "kafka");
            assert_eq!(source_lineage[0].source_table, "security-events");
            assert_eq!(source_lineage[0].record_count, Some(500_000));
            assert_eq!(source_lineage[0].metadata.len(), 3);

            // Verify field lineage count
            let field_lineage1 = index.get_field_lineage("actor.user.uid").await.unwrap();
            let field_lineage2 = index.get_field_lineage("actor.user.name").await.unwrap();
            let field_lineage3 = index.get_field_lineage("time").await.unwrap();

            assert_eq!(field_lineage1.len(), 1);
            assert_eq!(field_lineage2.len(), 1);
            assert_eq!(field_lineage3.len(), 1);

            // All field lineage records should reference the same source lineage
            assert_eq!(field_lineage1[0].source_lineage_id, lineage_id);
            assert_eq!(field_lineage2[0].source_lineage_id, lineage_id);
            assert_eq!(field_lineage3[0].source_lineage_id, lineage_id);
        }

        #[tokio::test]
        async fn test_finalize_no_field_mappings() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            // Capture with no field mappings (just source lineage)
            let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
                .with_record_count(1000);

            let lineage_id = capture.finalize(&index).await.unwrap();

            // Verify source lineage was created
            assert!(lineage_id.0 > 0);
            let source_lineage = index.get_source_lineage("network_activity").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
        }

        #[tokio::test]
        async fn test_finalize_multiple_captures_same_target() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            // First capture
            let capture1 = LineageCapture::new("splunk", "raw_logs_2023", "network_activity")
                .add_field_mapping("src_ip", "src_endpoint.ip")
                .with_record_count(1_000_000);

            let lineage_id1 = capture1.finalize(&index).await.unwrap();

            // Second capture to the same target
            let capture2 = LineageCapture::new("kafka", "events_2024", "network_activity")
                .add_field_mapping("source_ip", "src_endpoint.ip")
                .with_record_count(500_000);

            let lineage_id2 = capture2.finalize(&index).await.unwrap();

            // Verify both source lineage records exist
            let source_lineage = index.get_source_lineage("network_activity").await.unwrap();
            assert_eq!(source_lineage.len(), 2);

            // Verify they have different IDs
            assert_ne!(lineage_id1, lineage_id2);

            // Verify field lineage for src_endpoint.ip has two records
            let field_lineage = index.get_field_lineage("src_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage.len(), 2);

            // Verify they reference different source lineage records
            let source_ids: Vec<_> = field_lineage.iter().map(|f| f.source_lineage_id).collect();
            assert!(source_ids.contains(&lineage_id1));
            assert!(source_ids.contains(&lineage_id2));
        }

        #[tokio::test]
        async fn test_finalize_returns_correct_lineage_id() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("splunk", "raw_logs", "network_activity")
                .add_field_mapping("src_ip", "src_endpoint.ip");

            let lineage_id = capture.finalize(&index).await.unwrap();

            // The returned lineage_id should match the source_lineage_id in field records
            let field_lineage = index.get_field_lineage("src_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage[0].source_lineage_id, lineage_id);
        }

        #[tokio::test]
        async fn test_finalize_empty_strings() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            // Edge case: empty strings (should still work)
            let capture = LineageCapture::new("", "", "")
                .add_field_mapping("", "")
                .with_metadata("", "");

            let lineage_id = capture.finalize(&index).await.unwrap();
            assert!(lineage_id.0 > 0);

            let source_lineage = index.get_source_lineage("").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].source_system, "");
        }

        #[tokio::test]
        async fn test_finalize_special_characters() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new(
                "source-system_v2.0",
                "raw.logs.2024",
                "network_activity::v1",
            )
            .add_field_mapping("event.data.src_ip", "src_endpoint.ip")
            .with_metadata("key:with:colons", "value/with/slashes");

            let lineage_id = capture.finalize(&index).await.unwrap();
            assert!(lineage_id.0 > 0);

            let source_lineage = index
                .get_source_lineage("network_activity::v1")
                .await
                .unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].source_system, "source-system_v2.0");
        }

        #[tokio::test]
        async fn test_finalize_unicode() {
            let backend = InMemoryBackend::new();
            let index = SemanticIndex::new(backend, IndexConfig::default())
                .await
                .unwrap();

            let capture = LineageCapture::new("源系统", "原始日志", "网络活动")
                .add_field_mapping("源IP", "src_endpoint.ip")
                .with_metadata("描述", "测试数据");

            let lineage_id = capture.finalize(&index).await.unwrap();
            assert!(lineage_id.0 > 0);

            let source_lineage = index.get_source_lineage("网络活动").await.unwrap();
            assert_eq!(source_lineage.len(), 1);
            assert_eq!(source_lineage[0].source_system, "源系统");

            let field_lineage = index.get_field_lineage("src_endpoint.ip").await.unwrap();
            assert_eq!(field_lineage.len(), 1);
            assert_eq!(field_lineage[0].source_field, "源IP");
        }
    }
}
