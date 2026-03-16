//! Table registry types for the OCSF Semantic Index.
//!
//! This module provides types for tracking physical table metadata and
//! security detection coverage information.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backend::IndexRecord;
use crate::types::{ConfidenceLevel, RecordId, Severity, TLPLevel, TableId, WarehouseDialect};

// ============================================================================
// Detection Coverage
// ============================================================================

/// Security detection coverage metadata for a table.
/// Tracks which detection capabilities are supported by the data in this table.
///
/// This struct captures MITRE ATT&CK mappings, data sources, detection rules,
/// and other security-relevant metadata that helps with detection gap analysis.
///
/// # Example
///
/// ```rust
/// use ocsf_index::table_registry::DetectionCoverage;
/// use ocsf_index::types::{ConfidenceLevel, Severity, TLPLevel};
///
/// let coverage = DetectionCoverage::new()
///     .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()])
///     .with_mitre_tactics(vec!["credential-access".to_string()])
///     .with_data_sources(vec!["process_creation".to_string(), "network_connection".to_string()])
///     .with_kill_chain_phases(vec!["delivery".to_string(), "exploitation".to_string()])
///     .with_confidence(ConfidenceLevel::High)
///     .with_severity(Severity::Critical)
///     .with_tlp(TLPLevel::Amber);
///
/// assert!(coverage.covers_technique("T1071.004"));
/// assert!(coverage.covers_tactic("credential-access"));
/// assert!(!coverage.covers_technique("T9999"));
/// ```
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

impl DetectionCoverage {
    /// Creates a new empty DetectionCoverage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the MITRE ATT&CK technique IDs covered by this table.
    ///
    /// # Arguments
    ///
    /// * `techniques` - A vector of MITRE technique IDs (e.g., "T1071.004", "T1110.003")
    pub fn with_mitre_techniques(mut self, techniques: Vec<String>) -> Self {
        self.mitre_techniques = techniques;
        self
    }

    /// Sets the MITRE ATT&CK tactics covered by this table.
    ///
    /// # Arguments
    ///
    /// * `tactics` - A vector of MITRE tactic names (e.g., "credential-access", "lateral-movement")
    pub fn with_mitre_tactics(mut self, tactics: Vec<String>) -> Self {
        self.mitre_tactics = tactics;
        self
    }

    /// Sets the data sources available in this table.
    ///
    /// # Arguments
    ///
    /// * `sources` - A vector of data source names (e.g., "process_creation", "network_connection")
    pub fn with_data_sources(mut self, sources: Vec<String>) -> Self {
        self.data_sources = sources;
        self
    }

    /// Sets the detection rule IDs that use this table.
    ///
    /// # Arguments
    ///
    /// * `rules` - A vector of detection rule IDs (Sigma, Elastic, Splunk rule references)
    pub fn with_detection_rules(mut self, rules: Vec<String>) -> Self {
        self.detection_rules = rules;
        self
    }

    /// Sets the kill chain phases covered by this table.
    ///
    /// # Arguments
    ///
    /// * `phases` - A vector of kill chain phase names (e.g., "reconnaissance", "delivery")
    pub fn with_kill_chain_phases(mut self, phases: Vec<String>) -> Self {
        self.kill_chain_phases = phases;
        self
    }

    /// Sets the confidence level for detections using this table.
    ///
    /// # Arguments
    ///
    /// * `level` - The confidence level (Low, Medium, High)
    pub fn with_confidence(mut self, level: ConfidenceLevel) -> Self {
        self.confidence_level = Some(level);
        self
    }

    /// Sets the maximum severity of threats detectable with this table's data.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level (Informational, Low, Medium, High, Critical)
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.max_severity = Some(severity);
        self
    }

    /// Sets the Traffic Light Protocol (TLP) classification for this table.
    ///
    /// # Arguments
    ///
    /// * `tlp` - The TLP level (Clear, Green, Amber, AmberStrict, Red)
    pub fn with_tlp(mut self, tlp: TLPLevel) -> Self {
        self.tlp = Some(tlp);
        self
    }

    /// Returns true if this coverage includes the given MITRE technique.
    ///
    /// The comparison is case-sensitive and exact match.
    ///
    /// # Arguments
    ///
    /// * `technique` - The MITRE technique ID to check (e.g., "T1071.004")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::DetectionCoverage;
    ///
    /// let coverage = DetectionCoverage::new()
    ///     .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()]);
    ///
    /// assert!(coverage.covers_technique("T1071.004"));
    /// assert!(coverage.covers_technique("T1110.003"));
    /// assert!(!coverage.covers_technique("T9999"));
    /// assert!(!coverage.covers_technique("t1071.004")); // Case-sensitive
    /// ```
    pub fn covers_technique(&self, technique: &str) -> bool {
        self.mitre_techniques.iter().any(|t| t == technique)
    }

    /// Returns true if this coverage includes the given MITRE tactic.
    ///
    /// The comparison is case-sensitive and exact match.
    ///
    /// # Arguments
    ///
    /// * `tactic` - The MITRE tactic name to check (e.g., "credential-access")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::DetectionCoverage;
    ///
    /// let coverage = DetectionCoverage::new()
    ///     .with_mitre_tactics(vec!["credential-access".to_string(), "lateral-movement".to_string()]);
    ///
    /// assert!(coverage.covers_tactic("credential-access"));
    /// assert!(coverage.covers_tactic("lateral-movement"));
    /// assert!(!coverage.covers_tactic("unknown-tactic"));
    /// ```
    pub fn covers_tactic(&self, tactic: &str) -> bool {
        self.mitre_tactics.iter().any(|t| t == tactic)
    }
}

// ============================================================================
// TableEntry
// ============================================================================

/// Physical table metadata for the OCSF Semantic Index.
///
/// A `TableEntry` represents a registered physical table that stores OCSF event data.
/// It tracks the table's location, OCSF class mapping, warehouse dialect, and optional
/// security detection coverage metadata.
///
/// # Example
///
/// ```rust
/// use ocsf_index::table_registry::{TableEntry, DetectionCoverage};
/// use ocsf_index::types::{WarehouseDialect, ConfidenceLevel};
///
/// let table = TableEntry::new("network_activity", 4001)
///     .with_schema("ocsf")
///     .with_ocsf_version("1.3.0")
///     .with_dialect(WarehouseDialect::Snowflake)
///     .with_metadata("owner", "security-team")
///     .with_detection_coverage(
///         DetectionCoverage::new()
///             .with_mitre_techniques(vec!["T1071.004".to_string()])
///             .with_confidence(ConfidenceLevel::High)
///     );
///
/// assert_eq!(table.table_name, "network_activity");
/// assert_eq!(table.class_uid, 4001);
/// assert_eq!(table.schema_name, Some("ocsf".to_string()));
/// assert!(table.is_active);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableEntry {
    /// Unique identifier for this table entry (assigned by backend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<TableId>,

    /// The physical table name in the warehouse.
    pub table_name: String,

    /// Optional schema name (e.g., "ocsf", "security").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,

    /// OCSF event class UID that this table stores.
    pub class_uid: u32,

    /// OCSF schema version (e.g., "1.3.0", "1.4.0").
    pub ocsf_version: String,

    /// Warehouse dialect for SQL generation.
    pub dialect: WarehouseDialect,

    /// Timestamp when the table was registered.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the table was last updated.
    pub updated_at: DateTime<Utc>,

    /// Whether the table is active (false = soft deleted).
    pub is_active: bool,

    /// Additional metadata key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// Security detection coverage metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detection_coverage: Option<DetectionCoverage>,
}

impl TableEntry {
    /// Creates a new TableEntry with the given table name and OCSF class UID.
    ///
    /// The entry is created with default values:
    /// - `ocsf_version`: "1.3.0"
    /// - `dialect`: Snowflake
    /// - `is_active`: true
    /// - `created_at` and `updated_at`: current UTC time
    ///
    /// # Arguments
    ///
    /// * `table_name` - The physical table name in the warehouse
    /// * `class_uid` - The OCSF event class UID (must be > 0)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    ///
    /// let table = TableEntry::new("network_activity", 4001);
    /// assert_eq!(table.table_name, "network_activity");
    /// assert_eq!(table.class_uid, 4001);
    /// assert!(table.is_active);
    /// ```
    pub fn new(table_name: impl Into<String>, class_uid: u32) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            table_name: table_name.into(),
            schema_name: None,
            class_uid,
            ocsf_version: "1.3.0".to_string(),
            dialect: WarehouseDialect::Snowflake,
            created_at: now,
            updated_at: now,
            is_active: true,
            metadata: HashMap::new(),
            detection_coverage: None,
        }
    }

    /// Sets the schema name for this table.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema name (e.g., "ocsf", "security")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_schema("ocsf");
    /// assert_eq!(table.schema_name, Some("ocsf".to_string()));
    /// ```
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema_name = Some(schema.into());
        self
    }

    /// Sets the OCSF schema version for this table.
    ///
    /// # Arguments
    ///
    /// * `version` - The OCSF version string (e.g., "1.3.0", "1.4.0")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_ocsf_version("1.4.0");
    /// assert_eq!(table.ocsf_version, "1.4.0");
    /// ```
    pub fn with_ocsf_version(mut self, version: impl Into<String>) -> Self {
        self.ocsf_version = version.into();
        self
    }

    /// Sets the warehouse dialect for SQL generation.
    ///
    /// # Arguments
    ///
    /// * `dialect` - The warehouse dialect (Snowflake, Databricks, BigQuery, etc.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    /// use ocsf_index::types::WarehouseDialect;
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_dialect(WarehouseDialect::BigQuery);
    /// assert_eq!(table.dialect, WarehouseDialect::BigQuery);
    /// ```
    pub fn with_dialect(mut self, dialect: WarehouseDialect) -> Self {
        self.dialect = dialect;
        self
    }

    /// Adds a metadata key-value pair to this table.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_metadata("owner", "security-team")
    ///     .with_metadata("environment", "production");
    /// assert_eq!(table.metadata.get("owner"), Some(&"security-team".to_string()));
    /// assert_eq!(table.metadata.get("environment"), Some(&"production".to_string()));
    /// ```
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets the detection coverage metadata for this table.
    ///
    /// # Arguments
    ///
    /// * `coverage` - The detection coverage metadata
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::{TableEntry, DetectionCoverage};
    /// use ocsf_index::types::ConfidenceLevel;
    ///
    /// let coverage = DetectionCoverage::new()
    ///     .with_mitre_techniques(vec!["T1071.004".to_string()])
    ///     .with_confidence(ConfidenceLevel::High);
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_detection_coverage(coverage);
    ///
    /// assert!(table.detection_coverage.is_some());
    /// ```
    pub fn with_detection_coverage(mut self, coverage: DetectionCoverage) -> Self {
        self.detection_coverage = Some(coverage);
        self
    }

    /// Sets the table ID (typically called by the backend after persistence).
    ///
    /// # Arguments
    ///
    /// * `id` - The table ID
    pub fn with_id(mut self, id: TableId) -> Self {
        self.id = Some(id);
        self
    }

    /// Marks the table as inactive (soft delete).
    ///
    /// This sets `is_active` to false and updates the `updated_at` timestamp.
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Marks the table as active.
    ///
    /// This sets `is_active` to true and updates the `updated_at` timestamp.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Returns the fully qualified table name (schema.table_name if schema is set).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::table_registry::TableEntry;
    ///
    /// let table = TableEntry::new("network_activity", 4001)
    ///     .with_schema("ocsf");
    /// assert_eq!(table.qualified_name(), "ocsf.network_activity");
    ///
    /// let table_no_schema = TableEntry::new("network_activity", 4001);
    /// assert_eq!(table_no_schema.qualified_name(), "network_activity");
    /// ```
    pub fn qualified_name(&self) -> String {
        match &self.schema_name {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => self.table_name.clone(),
        }
    }
}

impl IndexRecord for TableEntry {
    fn record_type() -> &'static str {
        "table_entry"
    }

    fn id(&self) -> Option<RecordId> {
        self.id.map(|id| RecordId::new(id.0))
    }

    fn set_id(&mut self, id: RecordId) {
        self.id = Some(TableId::new(id.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_coverage_new() {
        let coverage = DetectionCoverage::new();
        assert!(coverage.mitre_techniques.is_empty());
        assert!(coverage.mitre_tactics.is_empty());
        assert!(coverage.data_sources.is_empty());
        assert!(coverage.detection_rules.is_empty());
        assert!(coverage.kill_chain_phases.is_empty());
        assert!(coverage.confidence_level.is_none());
        assert!(coverage.max_severity.is_none());
        assert!(coverage.tlp.is_none());
    }

    #[test]
    fn test_detection_coverage_default() {
        let coverage = DetectionCoverage::default();
        assert!(coverage.mitre_techniques.is_empty());
        assert!(coverage.confidence_level.is_none());
    }

    #[test]
    fn test_detection_coverage_builder_mitre_techniques() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()]);

        assert_eq!(coverage.mitre_techniques.len(), 2);
        assert!(coverage.mitre_techniques.contains(&"T1071.004".to_string()));
        assert!(coverage.mitre_techniques.contains(&"T1110.003".to_string()));
    }

    #[test]
    fn test_detection_coverage_builder_mitre_tactics() {
        let coverage = DetectionCoverage::new().with_mitre_tactics(vec![
            "credential-access".to_string(),
            "lateral-movement".to_string(),
        ]);

        assert_eq!(coverage.mitre_tactics.len(), 2);
        assert!(coverage
            .mitre_tactics
            .contains(&"credential-access".to_string()));
        assert!(coverage
            .mitre_tactics
            .contains(&"lateral-movement".to_string()));
    }

    #[test]
    fn test_detection_coverage_builder_data_sources() {
        let coverage = DetectionCoverage::new().with_data_sources(vec![
            "process_creation".to_string(),
            "network_connection".to_string(),
        ]);

        assert_eq!(coverage.data_sources.len(), 2);
        assert!(coverage
            .data_sources
            .contains(&"process_creation".to_string()));
    }

    #[test]
    fn test_detection_coverage_builder_detection_rules() {
        let coverage = DetectionCoverage::new().with_detection_rules(vec![
            "sigma-rule-001".to_string(),
            "elastic-rule-002".to_string(),
        ]);

        assert_eq!(coverage.detection_rules.len(), 2);
        assert!(coverage
            .detection_rules
            .contains(&"sigma-rule-001".to_string()));
    }

    #[test]
    fn test_detection_coverage_builder_kill_chain_phases() {
        let coverage = DetectionCoverage::new()
            .with_kill_chain_phases(vec!["reconnaissance".to_string(), "delivery".to_string()]);

        assert_eq!(coverage.kill_chain_phases.len(), 2);
        assert!(coverage
            .kill_chain_phases
            .contains(&"reconnaissance".to_string()));
    }

    #[test]
    fn test_detection_coverage_builder_confidence() {
        let coverage = DetectionCoverage::new().with_confidence(ConfidenceLevel::High);

        assert_eq!(coverage.confidence_level, Some(ConfidenceLevel::High));
    }

    #[test]
    fn test_detection_coverage_builder_severity() {
        let coverage = DetectionCoverage::new().with_severity(Severity::Critical);

        assert_eq!(coverage.max_severity, Some(Severity::Critical));
    }

    #[test]
    fn test_detection_coverage_builder_tlp() {
        let coverage = DetectionCoverage::new().with_tlp(TLPLevel::Amber);

        assert_eq!(coverage.tlp, Some(TLPLevel::Amber));
    }

    #[test]
    fn test_detection_coverage_builder_chaining() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_mitre_tactics(vec!["credential-access".to_string()])
            .with_data_sources(vec!["process_creation".to_string()])
            .with_detection_rules(vec!["sigma-rule-001".to_string()])
            .with_kill_chain_phases(vec!["delivery".to_string()])
            .with_confidence(ConfidenceLevel::High)
            .with_severity(Severity::Critical)
            .with_tlp(TLPLevel::Red);

        assert_eq!(coverage.mitre_techniques.len(), 1);
        assert_eq!(coverage.mitre_tactics.len(), 1);
        assert_eq!(coverage.data_sources.len(), 1);
        assert_eq!(coverage.detection_rules.len(), 1);
        assert_eq!(coverage.kill_chain_phases.len(), 1);
        assert_eq!(coverage.confidence_level, Some(ConfidenceLevel::High));
        assert_eq!(coverage.max_severity, Some(Severity::Critical));
        assert_eq!(coverage.tlp, Some(TLPLevel::Red));
    }

    #[test]
    fn test_covers_technique_found() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()]);

        assert!(coverage.covers_technique("T1071.004"));
        assert!(coverage.covers_technique("T1110.003"));
    }

    #[test]
    fn test_covers_technique_not_found() {
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);

        assert!(!coverage.covers_technique("T9999"));
        assert!(!coverage.covers_technique(""));
    }

    #[test]
    fn test_covers_technique_case_sensitive() {
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);

        assert!(coverage.covers_technique("T1071.004"));
        assert!(!coverage.covers_technique("t1071.004"));
        assert!(!coverage.covers_technique("T1071.004 ")); // trailing space
    }

    #[test]
    fn test_covers_technique_empty_list() {
        let coverage = DetectionCoverage::new();
        assert!(!coverage.covers_technique("T1071.004"));
    }

    #[test]
    fn test_covers_tactic_found() {
        let coverage = DetectionCoverage::new().with_mitre_tactics(vec![
            "credential-access".to_string(),
            "lateral-movement".to_string(),
        ]);

        assert!(coverage.covers_tactic("credential-access"));
        assert!(coverage.covers_tactic("lateral-movement"));
    }

    #[test]
    fn test_covers_tactic_not_found() {
        let coverage =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);

        assert!(!coverage.covers_tactic("unknown-tactic"));
        assert!(!coverage.covers_tactic(""));
    }

    #[test]
    fn test_covers_tactic_case_sensitive() {
        let coverage =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);

        assert!(coverage.covers_tactic("credential-access"));
        assert!(!coverage.covers_tactic("Credential-Access"));
        assert!(!coverage.covers_tactic("CREDENTIAL-ACCESS"));
    }

    #[test]
    fn test_covers_tactic_empty_list() {
        let coverage = DetectionCoverage::new();
        assert!(!coverage.covers_tactic("credential-access"));
    }

    #[test]
    fn test_detection_coverage_json_serialization() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_mitre_tactics(vec!["credential-access".to_string()])
            .with_confidence(ConfidenceLevel::High)
            .with_severity(Severity::Critical)
            .with_tlp(TLPLevel::Amber);

        let json = serde_json::to_string(&coverage).unwrap();
        let deserialized: DetectionCoverage = serde_json::from_str(&json).unwrap();

        assert_eq!(coverage, deserialized);
    }

    #[test]
    fn test_detection_coverage_json_empty_vecs_skipped() {
        let coverage = DetectionCoverage::new().with_confidence(ConfidenceLevel::High);

        let json = serde_json::to_string(&coverage).unwrap();

        // Empty vectors should be skipped in serialization
        assert!(!json.contains("mitre_techniques"));
        assert!(!json.contains("mitre_tactics"));
        assert!(!json.contains("data_sources"));
        assert!(!json.contains("detection_rules"));
        assert!(!json.contains("kill_chain_phases"));
        assert!(json.contains("confidence_level"));
    }

    #[test]
    fn test_detection_coverage_json_none_fields_skipped() {
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);

        let json = serde_json::to_string(&coverage).unwrap();

        // None fields should be skipped in serialization
        assert!(!json.contains("confidence_level"));
        assert!(!json.contains("max_severity"));
        assert!(!json.contains("tlp"));
        assert!(json.contains("mitre_techniques"));
    }

    #[test]
    fn test_detection_coverage_json_deserialization_with_defaults() {
        // JSON with only some fields should deserialize with defaults for missing fields
        let json = r#"{"mitre_techniques":["T1071.004"]}"#;
        let coverage: DetectionCoverage = serde_json::from_str(json).unwrap();

        assert_eq!(coverage.mitre_techniques, vec!["T1071.004".to_string()]);
        assert!(coverage.mitre_tactics.is_empty());
        assert!(coverage.data_sources.is_empty());
        assert!(coverage.confidence_level.is_none());
    }

    #[test]
    fn test_detection_coverage_equality() {
        let coverage1 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let coverage2 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let coverage3 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::Low);

        assert_eq!(coverage1, coverage2);
        assert_ne!(coverage1, coverage3);
    }

    #[test]
    fn test_detection_coverage_clone() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let cloned = coverage.clone();
        assert_eq!(coverage, cloned);
    }

    // ========================================================================
    // TableEntry Tests
    // ========================================================================

    #[test]
    fn test_table_entry_new() {
        let table = TableEntry::new("network_activity", 4001);

        assert!(table.id.is_none());
        assert_eq!(table.table_name, "network_activity");
        assert!(table.schema_name.is_none());
        assert_eq!(table.class_uid, 4001);
        assert_eq!(table.ocsf_version, "1.3.0");
        assert_eq!(table.dialect, WarehouseDialect::Snowflake);
        assert!(table.is_active);
        assert!(table.metadata.is_empty());
        assert!(table.detection_coverage.is_none());
    }

    #[test]
    fn test_table_entry_with_schema() {
        let table = TableEntry::new("network_activity", 4001).with_schema("ocsf");

        assert_eq!(table.schema_name, Some("ocsf".to_string()));
    }

    #[test]
    fn test_table_entry_with_ocsf_version() {
        let table = TableEntry::new("network_activity", 4001).with_ocsf_version("1.4.0");

        assert_eq!(table.ocsf_version, "1.4.0");
    }

    #[test]
    fn test_table_entry_with_dialect() {
        let table =
            TableEntry::new("network_activity", 4001).with_dialect(WarehouseDialect::BigQuery);

        assert_eq!(table.dialect, WarehouseDialect::BigQuery);
    }

    #[test]
    fn test_table_entry_with_metadata() {
        let table = TableEntry::new("network_activity", 4001)
            .with_metadata("owner", "security-team")
            .with_metadata("environment", "production");

        assert_eq!(table.metadata.len(), 2);
        assert_eq!(
            table.metadata.get("owner"),
            Some(&"security-team".to_string())
        );
        assert_eq!(
            table.metadata.get("environment"),
            Some(&"production".to_string())
        );
    }

    #[test]
    fn test_table_entry_with_detection_coverage() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let table =
            TableEntry::new("network_activity", 4001).with_detection_coverage(coverage.clone());

        assert_eq!(table.detection_coverage, Some(coverage));
    }

    #[test]
    fn test_table_entry_with_id() {
        let table = TableEntry::new("network_activity", 4001).with_id(TableId::new(42));

        assert_eq!(table.id, Some(TableId::new(42)));
    }

    #[test]
    fn test_table_entry_builder_chaining() {
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);

        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.4.0")
            .with_dialect(WarehouseDialect::Databricks)
            .with_metadata("owner", "security-team")
            .with_detection_coverage(coverage);

        assert_eq!(table.table_name, "network_activity");
        assert_eq!(table.class_uid, 4001);
        assert_eq!(table.schema_name, Some("ocsf".to_string()));
        assert_eq!(table.ocsf_version, "1.4.0");
        assert_eq!(table.dialect, WarehouseDialect::Databricks);
        assert_eq!(
            table.metadata.get("owner"),
            Some(&"security-team".to_string())
        );
        assert!(table.detection_coverage.is_some());
    }

    #[test]
    fn test_table_entry_deactivate() {
        let mut table = TableEntry::new("network_activity", 4001);
        assert!(table.is_active);

        let original_updated_at = table.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        table.deactivate();

        assert!(!table.is_active);
        assert!(table.updated_at >= original_updated_at);
    }

    #[test]
    fn test_table_entry_activate() {
        let mut table = TableEntry::new("network_activity", 4001);
        table.is_active = false;

        let original_updated_at = table.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        table.activate();

        assert!(table.is_active);
        assert!(table.updated_at >= original_updated_at);
    }

    #[test]
    fn test_table_entry_qualified_name_with_schema() {
        let table = TableEntry::new("network_activity", 4001).with_schema("ocsf");

        assert_eq!(table.qualified_name(), "ocsf.network_activity");
    }

    #[test]
    fn test_table_entry_qualified_name_without_schema() {
        let table = TableEntry::new("network_activity", 4001);

        assert_eq!(table.qualified_name(), "network_activity");
    }

    #[test]
    fn test_table_entry_json_serialization() {
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.4.0")
            .with_dialect(WarehouseDialect::Snowflake)
            .with_metadata("owner", "security-team")
            .with_detection_coverage(coverage);

        let json = serde_json::to_string(&table).unwrap();
        let deserialized: TableEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(table.table_name, deserialized.table_name);
        assert_eq!(table.class_uid, deserialized.class_uid);
        assert_eq!(table.schema_name, deserialized.schema_name);
        assert_eq!(table.ocsf_version, deserialized.ocsf_version);
        assert_eq!(table.dialect, deserialized.dialect);
        assert_eq!(table.is_active, deserialized.is_active);
        assert_eq!(table.metadata, deserialized.metadata);
        assert_eq!(table.detection_coverage, deserialized.detection_coverage);
    }

    #[test]
    fn test_table_entry_json_optional_fields_skipped() {
        let table = TableEntry::new("network_activity", 4001);

        let json = serde_json::to_string(&table).unwrap();

        // Optional fields should be skipped when None/empty
        assert!(!json.contains("\"id\""));
        assert!(!json.contains("schema_name"));
        assert!(!json.contains("metadata"));
        assert!(!json.contains("detection_coverage"));
    }

    #[test]
    fn test_table_entry_json_deserialization_minimal() {
        // Minimal JSON with only required fields
        let json = r#"{
            "table_name": "network_activity",
            "class_uid": 4001,
            "ocsf_version": "1.3.0",
            "dialect": "snowflake",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "is_active": true
        }"#;

        let table: TableEntry = serde_json::from_str(json).unwrap();

        assert_eq!(table.table_name, "network_activity");
        assert_eq!(table.class_uid, 4001);
        assert!(table.id.is_none());
        assert!(table.schema_name.is_none());
        assert!(table.metadata.is_empty());
        assert!(table.detection_coverage.is_none());
    }

    #[test]
    fn test_table_entry_equality() {
        let table1 = TableEntry::new("network_activity", 4001).with_schema("ocsf");

        let table2 = TableEntry::new("network_activity", 4001).with_schema("ocsf");

        // Note: created_at/updated_at will differ, so we compare specific fields
        assert_eq!(table1.table_name, table2.table_name);
        assert_eq!(table1.class_uid, table2.class_uid);
        assert_eq!(table1.schema_name, table2.schema_name);
    }

    #[test]
    fn test_table_entry_clone() {
        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_metadata("owner", "security-team");

        let cloned = table.clone();

        assert_eq!(table.table_name, cloned.table_name);
        assert_eq!(table.class_uid, cloned.class_uid);
        assert_eq!(table.schema_name, cloned.schema_name);
        assert_eq!(table.metadata, cloned.metadata);
    }

    #[test]
    fn test_table_entry_index_record_trait() {
        let mut table = TableEntry::new("network_activity", 4001);

        // Test record_type
        assert_eq!(TableEntry::record_type(), "table_entry");

        // Test id() when None
        assert!(table.id().is_none());

        // Test set_id and id()
        table.set_id(RecordId::new(42));
        assert_eq!(table.id(), Some(RecordId::new(42)));
        assert_eq!(table.id, Some(TableId::new(42)));
    }

    #[test]
    fn test_table_entry_various_class_uids() {
        // Test with various valid class UIDs
        let table1 = TableEntry::new("table1", 1);
        assert_eq!(table1.class_uid, 1);

        let table2 = TableEntry::new("table2", 4001);
        assert_eq!(table2.class_uid, 4001);

        let table3 = TableEntry::new("table3", u32::MAX);
        assert_eq!(table3.class_uid, u32::MAX);
    }

    #[test]
    fn test_table_entry_with_all_dialects() {
        let dialects = vec![
            WarehouseDialect::Snowflake,
            WarehouseDialect::Databricks,
            WarehouseDialect::BigQuery,
            WarehouseDialect::Postgres,
        ];

        for dialect in dialects {
            let table = TableEntry::new("test_table", 4001).with_dialect(dialect.clone());
            assert_eq!(table.dialect, dialect);
        }
    }
}
