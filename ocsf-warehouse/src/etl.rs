//! ETL pipeline generation for observable extraction.
//!
//! This module generates ETL logic for extracting observables from OCSF events
//! while preserving reverse-lookup references.
//!
//! # Lineage Capture Integration
//!
//! The ETL module integrates with `ocsf-index` to capture data lineage during ETL
//! execution. This enables tracking of source-to-target table relationships and
//! field-level transformations for data governance and debugging.
//!
//! ## Integration Pattern
//!
//! To capture lineage during ETL execution:
//!
//! 1. Create an `ETLConfig` with source and target table information
//! 2. Use `ETLConfig::to_lineage_capture()` to create a `LineageCapture` builder
//! 3. Add any additional field mappings or metadata
//! 4. Call `finalize()` with a `SemanticIndex` to persist the lineage
//!
//! ```rust,ignore
//! use ocsf_warehouse::etl::ETLConfig;
//! use ocsf_index::{SemanticIndex, IndexConfig};
//! use ocsf_index::backend::InMemoryBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set up the semantic index
//!     let backend = InMemoryBackend::new();
//!     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
//!
//!     // Configure ETL
//!     let config = ETLConfig {
//!         source_table: "raw_network_logs".to_string(),
//!         target_table: "ocsf_observables".to_string(),
//!         observable_types: vec![2, 5, 10], // IP, Email, Username
//!         batch_mode: false,
//!         batch_size: 10000,
//!     };
//!
//!     // Create lineage capture from ETL config
//!     let lineage_id = config
//!         .to_lineage_capture("splunk") // Specify source system
//!         .with_record_count(1_000_000)
//!         .with_metadata("pipeline", "observable-extraction-v1")
//!         .finalize(&index)
//!         .await?;
//!
//!     println!("Captured lineage with ID: {}", lineage_id);
//!     Ok(())
//! }
//! ```
//!
//! ## Field Mappings
//!
//! The `to_lineage_capture()` method automatically adds field mappings for the
//! standard observable extraction fields:
//!
//! - `observable_id` - Generated from event UID and attribute path
//! - `type_id` - Observable type identifier
//! - `type_name` - Human-readable type name
//! - `value` - Extracted observable value
//! - `event_uid` - Source event unique identifier (for reverse lookup)
//! - `event_class_uid` - Source event class (for reverse lookup)
//! - `event_time` - Event timestamp
//! - `attribute_path` - Path to the observable in the source event
//!
//! Additional field mappings can be added using the `LineageCapture` builder:
//!
//! ```rust,ignore
//! let capture = config
//!     .to_lineage_capture("kafka")
//!     .add_field_mapping("custom_field", "target_field")
//!     .add_field_mapping_with_transform("timestamp_str", "time", "TO_TIMESTAMP(timestamp_str)");
//! ```
//!
//! ## Metadata
//!
//! Use metadata to track additional context about the ETL execution:
//!
//! ```rust,ignore
//! let capture = config
//!     .to_lineage_capture("splunk")
//!     .with_metadata("batch_id", "batch-2024-01-15-001")
//!     .with_metadata("pipeline_version", "v2.1.0")
//!     .with_metadata("environment", "production");
//! ```

use ocsf_index::LineageCapture;
use ocsf_semantic::WarehouseDialect;
use serde::{Deserialize, Serialize};

/// ETL pipeline definition.
#[derive(Debug, Clone, Default)]
pub struct ETLDefinition {
    /// SQL statements for the ETL pipeline.
    pub statements: Vec<String>,
    /// Description of the pipeline.
    pub description: String,
}

impl ETLDefinition {
    /// Creates a new ETL definition.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a statement to the pipeline.
    pub fn add_statement(&mut self, statement: String) {
        self.statements.push(statement);
    }

    /// Returns all statements as a single SQL script.
    pub fn to_sql(&self) -> String {
        self.statements.join("\n\n")
    }
}

/// Configuration for ETL generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ETLConfig {
    /// Source events table name.
    pub source_table: String,
    /// Target observables table name.
    pub target_table: String,
    /// Observable type IDs to extract.
    pub observable_types: Vec<u32>,
    /// Whether to include batch processing logic.
    pub batch_mode: bool,
    /// Batch size for incremental processing.
    pub batch_size: u32,
}

impl Default for ETLConfig {
    fn default() -> Self {
        Self {
            source_table: "ocsf_events".to_string(),
            target_table: "ocsf_observables".to_string(),
            observable_types: vec![2, 5, 10, 22, 30], // IP, Email, Username, Hostname, Hash
            batch_mode: false,
            batch_size: 10000,
        }
    }
}

impl ETLConfig {
    /// Creates a `LineageCapture` builder from this ETL configuration.
    ///
    /// This method extracts source and target table information from the ETL config
    /// and creates a `LineageCapture` with standard observable extraction field mappings.
    ///
    /// # Arguments
    ///
    /// * `source_system` - The name of the source system (e.g., "splunk", "kafka", "elastic")
    ///
    /// # Returns
    ///
    /// A `LineageCapture` builder pre-populated with:
    /// - Source and target table information from the ETL config
    /// - Standard field mappings for observable extraction
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_warehouse::etl::ETLConfig;
    ///
    /// let config = ETLConfig {
    ///     source_table: "raw_network_logs".to_string(),
    ///     target_table: "ocsf_observables".to_string(),
    ///     observable_types: vec![2, 5],
    ///     batch_mode: false,
    ///     batch_size: 10000,
    /// };
    ///
    /// let capture = config.to_lineage_capture("splunk");
    ///
    /// assert_eq!(capture.source_system(), "splunk");
    /// assert_eq!(capture.source_table(), "raw_network_logs");
    /// assert_eq!(capture.target_table(), "ocsf_observables");
    ///
    /// // Standard field mappings are automatically added
    /// assert!(!capture.field_mappings().is_empty());
    /// ```
    ///
    /// # Integration with SemanticIndex
    ///
    /// After creating the `LineageCapture`, you can add additional metadata and
    /// finalize it to persist the lineage:
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// let backend = InMemoryBackend::new();
    /// let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    /// let lineage_id = config
    ///     .to_lineage_capture("splunk")
    ///     .with_record_count(1_000_000)
    ///     .with_metadata("batch_id", "batch-001")
    ///     .finalize(&index)
    ///     .await?;
    /// ```
    pub fn to_lineage_capture(&self, source_system: impl Into<String>) -> LineageCapture {
        let mut capture = LineageCapture::new(
            source_system,
            &self.source_table,
            &self.target_table,
        );

        // Add standard observable extraction field mappings
        // These represent the core fields extracted during observable ETL
        capture = capture
            .add_field_mapping_with_transform(
                "metadata_uid",
                "observable_id",
                "CONCAT(metadata_uid, '_', attribute_path)",
            )
            .add_field_mapping("type_id", "type_id")
            .add_field_mapping("type_name", "type_name")
            .add_field_mapping_with_transform(
                "raw_data",
                "value",
                "JSON_EXTRACT(raw_data, attribute_path)",
            )
            .add_field_mapping("metadata_uid", "event_uid")
            .add_field_mapping("class_uid", "event_class_uid")
            .add_field_mapping("time", "event_time")
            .add_field_mapping("attribute_path", "attribute_path");

        // Add metadata about the ETL configuration
        capture = capture
            .with_metadata("batch_mode", self.batch_mode.to_string())
            .with_metadata("batch_size", self.batch_size.to_string())
            .with_metadata(
                "observable_types",
                self.observable_types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );

        capture
    }
}

/// Observable extraction result for testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedObservable {
    /// Observable ID.
    pub observable_id: String,
    /// Observable type ID.
    pub type_id: u32,
    /// Observable type name.
    pub type_name: String,
    /// Observable value.
    pub value: Option<String>,
    /// Source event UID.
    pub event_uid: String,
    /// Source event class UID.
    pub event_class_uid: u32,
    /// Event timestamp.
    pub event_time: String,
    /// Attribute path in source event.
    pub attribute_path: String,
}

impl ExtractedObservable {
    /// Creates a new extracted observable.
    pub fn new(
        type_id: u32,
        type_name: impl Into<String>,
        value: Option<String>,
        event_uid: impl Into<String>,
        event_class_uid: u32,
        event_time: impl Into<String>,
        attribute_path: impl Into<String>,
    ) -> Self {
        let event_uid = event_uid.into();
        let attribute_path = attribute_path.into();
        
        // Generate observable_id from event_uid and path
        let observable_id = format!("{}_{}", event_uid, attribute_path.replace('.', "_"));

        Self {
            observable_id,
            type_id,
            type_name: type_name.into(),
            value,
            event_uid,
            event_class_uid,
            event_time: event_time.into(),
            attribute_path,
        }
    }

    /// Returns true if this observable has a valid reverse-lookup reference.
    pub fn has_valid_reverse_lookup(&self) -> bool {
        !self.event_uid.is_empty() && self.event_class_uid > 0
    }
}

/// Generates ETL pipelines for observable extraction.
pub struct ETLGenerator {
    dialect: WarehouseDialect,
    config: ETLConfig,
}

impl ETLGenerator {
    /// Creates a new ETL generator.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            config: ETLConfig::default(),
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: ETLConfig) -> Self {
        self.config = config;
        self
    }

    /// Generates the ETL pipeline.
    pub fn generate(&self) -> ETLDefinition {
        let mut etl = ETLDefinition::new();
        etl.description = "Observable extraction ETL pipeline".to_string();

        // Generate extraction SQL for each observable type
        let extraction_sql = self.generate_extraction_sql();
        etl.add_statement(extraction_sql);

        // Add batch processing wrapper if configured
        if self.config.batch_mode {
            let batch_sql = self.generate_batch_wrapper();
            etl.add_statement(batch_sql);
        }

        etl
    }

    /// Generates the main extraction SQL.
    fn generate_extraction_sql(&self) -> String {
        let mut sql = String::new();

        sql.push_str(&format!(
            "{} Observable extraction from {} to {}\n",
            self.dialect.comment_syntax(),
            self.config.source_table,
            self.config.target_table
        ));

        sql.push_str(&format!("INSERT INTO {}\n", self.config.target_table));
        sql.push_str("(\n");
        sql.push_str("    observable_id,\n");
        sql.push_str("    type_id,\n");
        sql.push_str("    type_name,\n");
        sql.push_str("    value,\n");
        sql.push_str("    event_uid,\n");
        sql.push_str("    event_class_uid,\n");
        sql.push_str("    event_time,\n");
        sql.push_str("    attribute_path,\n");
        sql.push_str("    ingestion_time\n");
        sql.push_str(")\n");

        // Generate UNION ALL for each observable type
        let mut unions = Vec::new();

        for (i, &type_id) in self.config.observable_types.iter().enumerate() {
            let type_name = self.get_observable_type_name(type_id);
            let paths = self.get_observable_paths(type_id);

            for path in paths {
                let select = self.generate_observable_select(type_id, &type_name, &path);
                if i == 0 && unions.is_empty() {
                    unions.push(select);
                } else {
                    unions.push(format!("UNION ALL\n{}", select));
                }
            }
        }

        sql.push_str(&unions.join("\n"));
        sql.push(';');

        sql
    }

    /// Generates a SELECT statement for extracting a specific observable.
    fn generate_observable_select(&self, type_id: u32, type_name: &str, path: &str) -> String {
        let id_expr = self.generate_observable_id_expr(path);
        let value_expr = self.dialect.json_extract("raw_data", path);

        format!(
            r#"SELECT
    {} AS observable_id,
    {} AS type_id,
    '{}' AS type_name,
    {} AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    '{}' AS attribute_path,
    {} AS ingestion_time
FROM {}
WHERE {} IS NOT NULL"#,
            id_expr,
            type_id,
            type_name,
            value_expr,
            path,
            self.dialect.current_timestamp_fn(),
            self.config.source_table,
            value_expr
        )
    }

    /// Generates the observable ID expression.
    fn generate_observable_id_expr(&self, path: &str) -> String {
        let path_suffix = path.replace('.', "_");
        match self.dialect {
            WarehouseDialect::BigQuery => {
                format!("CONCAT(metadata_uid, '_', '{}')", path_suffix)
            }
            WarehouseDialect::Snowflake | WarehouseDialect::Databricks | WarehouseDialect::Postgres => {
                format!("metadata_uid || '_' || '{}'", path_suffix)
            }
        }
    }

    /// Gets the type name for an observable type ID.
    fn get_observable_type_name(&self, type_id: u32) -> String {
        match type_id {
            2 => "IP Address".to_string(),
            5 => "Email Address".to_string(),
            10 => "User Name".to_string(),
            22 => "Hostname".to_string(),
            30 => "File Hash".to_string(),
            _ => format!("Type {}", type_id),
        }
    }

    /// Gets the common paths for an observable type.
    fn get_observable_paths(&self, type_id: u32) -> Vec<String> {
        match type_id {
            2 => vec![
                "src_endpoint.ip".to_string(),
                "dst_endpoint.ip".to_string(),
                "actor.user.ip".to_string(),
            ],
            5 => vec![
                "actor.user.email_addr".to_string(),
                "user.email_addr".to_string(),
            ],
            10 => vec![
                "actor.user.name".to_string(),
                "user.name".to_string(),
            ],
            22 => vec![
                "src_endpoint.hostname".to_string(),
                "dst_endpoint.hostname".to_string(),
            ],
            30 => vec![
                "file.hashes.md5".to_string(),
                "file.hashes.sha1".to_string(),
                "file.hashes.sha256".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Generates batch processing wrapper.
    fn generate_batch_wrapper(&self) -> String {
        format!(
            r#"{} Batch processing wrapper
{} Process in batches of {} records
{} Add your batch processing logic here based on warehouse capabilities"#,
            self.dialect.comment_syntax(),
            self.dialect.comment_syntax(),
            self.config.batch_size,
            self.dialect.comment_syntax()
        )
    }

    /// Extracts observables from a simulated event (for testing).
    pub fn extract_from_event(
        &self,
        event_uid: &str,
        event_class_uid: u32,
        event_time: &str,
        observables_data: &[(u32, &str, &str, Option<&str>)], // (type_id, type_name, path, value)
    ) -> Vec<ExtractedObservable> {
        observables_data
            .iter()
            .map(|(type_id, type_name, path, value)| {
                ExtractedObservable::new(
                    *type_id,
                    *type_name,
                    value.map(|v| v.to_string()),
                    event_uid,
                    event_class_uid,
                    event_time,
                    *path,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etl_generator_creates_pipeline() {
        let generator = ETLGenerator::new(WarehouseDialect::Snowflake);
        let etl = generator.generate();

        assert!(!etl.statements.is_empty());
        assert!(etl.to_sql().contains("INSERT INTO"));
        assert!(etl.to_sql().contains("observable_id"));
    }

    #[test]
    fn test_etl_contains_observable_types() {
        let config = ETLConfig {
            observable_types: vec![2, 5],
            ..Default::default()
        };
        let generator = ETLGenerator::new(WarehouseDialect::Postgres).with_config(config);
        let etl = generator.generate();

        let sql = etl.to_sql();
        assert!(sql.contains("type_id"));
        assert!(sql.contains("IP Address"));
        assert!(sql.contains("Email Address"));
    }

    #[test]
    fn test_etl_preserves_reverse_lookup() {
        let generator = ETLGenerator::new(WarehouseDialect::BigQuery);
        let etl = generator.generate();

        let sql = etl.to_sql();
        assert!(sql.contains("event_uid"));
        assert!(sql.contains("event_class_uid"));
        assert!(sql.contains("event_time"));
    }

    #[test]
    fn test_extracted_observable_has_valid_reverse_lookup() {
        let observable = ExtractedObservable::new(
            2,
            "IP Address",
            Some("192.168.1.1".to_string()),
            "event-123",
            3002,
            "2024-01-01T00:00:00Z",
            "src_endpoint.ip",
        );

        assert!(observable.has_valid_reverse_lookup());
        assert_eq!(observable.event_uid, "event-123");
        assert_eq!(observable.event_class_uid, 3002);
    }

    #[test]
    fn test_extract_from_event() {
        let generator = ETLGenerator::new(WarehouseDialect::Snowflake);
        
        let observables = generator.extract_from_event(
            "event-456",
            3002,
            "2024-01-01T12:00:00Z",
            &[
                (2, "IP Address", "src_endpoint.ip", Some("10.0.0.1")),
                (5, "Email Address", "actor.user.email_addr", Some("user@example.com")),
            ],
        );

        assert_eq!(observables.len(), 2);
        
        // Check first observable
        assert_eq!(observables[0].type_id, 2);
        assert_eq!(observables[0].value, Some("10.0.0.1".to_string()));
        assert_eq!(observables[0].event_uid, "event-456");
        assert!(observables[0].has_valid_reverse_lookup());

        // Check second observable
        assert_eq!(observables[1].type_id, 5);
        assert_eq!(observables[1].attribute_path, "actor.user.email_addr");
    }

    #[test]
    fn test_observable_id_generation() {
        let observable = ExtractedObservable::new(
            2,
            "IP Address",
            Some("192.168.1.1".to_string()),
            "event-789",
            3002,
            "2024-01-01T00:00:00Z",
            "src_endpoint.ip",
        );

        assert_eq!(observable.observable_id, "event-789_src_endpoint_ip");
    }

    // ========================================================================
    // Lineage Capture Integration Tests
    // ========================================================================

    #[test]
    fn test_etl_config_to_lineage_capture_basic() {
        let config = ETLConfig {
            source_table: "raw_network_logs".to_string(),
            target_table: "ocsf_observables".to_string(),
            observable_types: vec![2, 5],
            batch_mode: false,
            batch_size: 10000,
        };

        let capture = config.to_lineage_capture("splunk");

        assert_eq!(capture.source_system(), "splunk");
        assert_eq!(capture.source_table(), "raw_network_logs");
        assert_eq!(capture.target_table(), "ocsf_observables");
    }

    #[test]
    fn test_etl_config_to_lineage_capture_has_field_mappings() {
        let config = ETLConfig::default();
        let capture = config.to_lineage_capture("kafka");

        // Should have standard observable extraction field mappings
        let mappings = capture.field_mappings();
        assert!(!mappings.is_empty());

        // Check for expected field mappings
        let target_fields: Vec<&str> = mappings.iter().map(|(_, t, _)| t.as_str()).collect();
        assert!(target_fields.contains(&"observable_id"));
        assert!(target_fields.contains(&"type_id"));
        assert!(target_fields.contains(&"type_name"));
        assert!(target_fields.contains(&"value"));
        assert!(target_fields.contains(&"event_uid"));
        assert!(target_fields.contains(&"event_class_uid"));
        assert!(target_fields.contains(&"event_time"));
        assert!(target_fields.contains(&"attribute_path"));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_has_metadata() {
        let config = ETLConfig {
            source_table: "events".to_string(),
            target_table: "observables".to_string(),
            observable_types: vec![2, 5, 10],
            batch_mode: true,
            batch_size: 5000,
        };

        let capture = config.to_lineage_capture("elastic");
        let metadata = capture.metadata();

        // Should have ETL configuration metadata
        assert_eq!(metadata.get("batch_mode"), Some(&"true".to_string()));
        assert_eq!(metadata.get("batch_size"), Some(&"5000".to_string()));
        assert_eq!(metadata.get("observable_types"), Some(&"2,5,10".to_string()));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_with_string_source_system() {
        let config = ETLConfig::default();
        let capture = config.to_lineage_capture(String::from("custom-source"));

        assert_eq!(capture.source_system(), "custom-source");
    }

    #[test]
    fn test_etl_config_to_lineage_capture_can_add_more_mappings() {
        let config = ETLConfig::default();
        let capture = config
            .to_lineage_capture("splunk")
            .add_field_mapping("custom_field", "target_custom_field")
            .add_field_mapping_with_transform("timestamp_str", "time", "TO_TIMESTAMP(timestamp_str)");

        let mappings = capture.field_mappings();
        
        // Should have standard mappings plus the two we added
        let target_fields: Vec<&str> = mappings.iter().map(|(_, t, _)| t.as_str()).collect();
        assert!(target_fields.contains(&"target_custom_field"));
        assert!(target_fields.contains(&"time"));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_can_add_record_count() {
        let config = ETLConfig::default();
        let capture = config
            .to_lineage_capture("kafka")
            .with_record_count(1_000_000);

        assert_eq!(capture.record_count(), Some(1_000_000));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_can_add_more_metadata() {
        let config = ETLConfig::default();
        let capture = config
            .to_lineage_capture("splunk")
            .with_metadata("pipeline", "etl-v2")
            .with_metadata("batch_id", "batch-001");

        let metadata = capture.metadata();
        
        // Should have both ETL config metadata and custom metadata
        assert!(metadata.contains_key("batch_mode"));
        assert!(metadata.contains_key("batch_size"));
        assert!(metadata.contains_key("observable_types"));
        assert_eq!(metadata.get("pipeline"), Some(&"etl-v2".to_string()));
        assert_eq!(metadata.get("batch_id"), Some(&"batch-001".to_string()));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_observable_id_has_transform() {
        let config = ETLConfig::default();
        let capture = config.to_lineage_capture("splunk");

        let mappings = capture.field_mappings();
        
        // Find the observable_id mapping
        let observable_id_mapping = mappings
            .iter()
            .find(|(_, target, _)| target == "observable_id");
        
        assert!(observable_id_mapping.is_some());
        let (source, _, transform) = observable_id_mapping.unwrap();
        assert_eq!(source, "metadata_uid");
        assert!(transform.is_some());
        assert!(transform.as_ref().unwrap().contains("CONCAT"));
    }

    #[test]
    fn test_etl_config_to_lineage_capture_value_has_transform() {
        let config = ETLConfig::default();
        let capture = config.to_lineage_capture("kafka");

        let mappings = capture.field_mappings();
        
        // Find the value mapping
        let value_mapping = mappings
            .iter()
            .find(|(_, target, _)| target == "value");
        
        assert!(value_mapping.is_some());
        let (source, _, transform) = value_mapping.unwrap();
        assert_eq!(source, "raw_data");
        assert!(transform.is_some());
        assert!(transform.as_ref().unwrap().contains("JSON_EXTRACT"));
    }

    #[test]
    fn test_etl_config_default_to_lineage_capture() {
        let config = ETLConfig::default();
        let capture = config.to_lineage_capture("default-source");

        assert_eq!(capture.source_system(), "default-source");
        assert_eq!(capture.source_table(), "ocsf_events");
        assert_eq!(capture.target_table(), "ocsf_observables");
        
        // Check default observable types in metadata
        let metadata = capture.metadata();
        assert_eq!(metadata.get("observable_types"), Some(&"2,5,10,22,30".to_string()));
    }
}
