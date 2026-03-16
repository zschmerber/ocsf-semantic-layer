//! Core types for the OCSF Semantic Index.
//!
//! This module provides the foundational types used throughout the index,
//! including ID types, configuration, and common enums.

use std::fmt;

use serde::{Deserialize, Serialize};

// ============================================================================
// ID Types
// ============================================================================

/// Unique identifier for a registered table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableId(pub u64);

impl TableId {
    /// Creates a new TableId.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for TableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TableId({})", self.0)
    }
}

impl From<u64> for TableId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<TableId> for u64 {
    fn from(id: TableId) -> Self {
        id.0
    }
}

/// Unique identifier for a lineage record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageId(pub u64);

impl LineageId {
    /// Creates a new LineageId.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for LineageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LineageId({})", self.0)
    }
}

impl From<u64> for LineageId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<LineageId> for u64 {
    fn from(id: LineageId) -> Self {
        id.0
    }
}

/// Unique identifier for a partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u64);

impl PartitionId {
    /// Creates a new PartitionId.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartitionId({})", self.0)
    }
}

impl From<u64> for PartitionId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<PartitionId> for u64 {
    fn from(id: PartitionId) -> Self {
        id.0
    }
}

/// Generic record identifier used by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub u64);

impl RecordId {
    /// Creates a new RecordId.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RecordId({})", self.0)
    }
}

impl From<u64> for RecordId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<RecordId> for u64 {
    fn from(id: RecordId) -> Self {
        id.0
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for the semantic index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Maximum number of entries in the query cache.
    pub cache_max_entries: usize,

    /// Default time-to-live for cache entries in seconds.
    pub cache_default_ttl_secs: u64,

    /// Sample rate for statistics collection (0.0 to 1.0).
    pub statistics_sample_rate: f64,

    /// Maximum age for statistics before considered stale, in seconds.
    pub statistics_max_age_secs: u64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            cache_max_entries: 1000,
            cache_default_ttl_secs: 3600,   // 1 hour
            statistics_sample_rate: 0.1,    // 10% sample
            statistics_max_age_secs: 86400, // 24 hours
        }
    }
}

impl IndexConfig {
    /// Creates a new IndexConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum cache entries.
    pub fn with_cache_max_entries(mut self, max_entries: usize) -> Self {
        self.cache_max_entries = max_entries;
        self
    }

    /// Sets the default cache TTL in seconds.
    pub fn with_cache_default_ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.cache_default_ttl_secs = ttl_secs;
        self
    }

    /// Sets the statistics sample rate.
    pub fn with_statistics_sample_rate(mut self, rate: f64) -> Self {
        self.statistics_sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Sets the maximum statistics age in seconds.
    pub fn with_statistics_max_age_secs(mut self, max_age_secs: u64) -> Self {
        self.statistics_max_age_secs = max_age_secs;
        self
    }
}

// ============================================================================
// Enums
// ============================================================================

/// Warehouse dialect for SQL generation.
///
/// Re-exported from ocsf-semantic for convenience.
pub use ocsf_semantic::WarehouseDialect;

/// Confidence level for detection coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// Low confidence detection.
    Low,
    /// Medium confidence detection.
    Medium,
    /// High confidence detection.
    High,
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Severity level for detectable threats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational severity.
    Informational,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity.
    High,
    /// Critical severity.
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Informational => write!(f, "informational"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Traffic Light Protocol (TLP) classification level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TLPLevel {
    /// TLP:CLEAR - No restrictions on disclosure.
    Clear,
    /// TLP:GREEN - Limited disclosure within community.
    Green,
    /// TLP:AMBER - Limited disclosure within organization.
    Amber,
    /// TLP:AMBER+STRICT - Restricted to organization only.
    #[serde(rename = "amber_strict")]
    AmberStrict,
    /// TLP:RED - Not for disclosure.
    Red,
}

impl fmt::Display for TLPLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clear => write!(f, "TLP:CLEAR"),
            Self::Green => write!(f, "TLP:GREEN"),
            Self::Amber => write!(f, "TLP:AMBER"),
            Self::AmberStrict => write!(f, "TLP:AMBER+STRICT"),
            Self::Red => write!(f, "TLP:RED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_id_display() {
        let id = TableId::new(42);
        assert_eq!(format!("{}", id), "TableId(42)");
    }

    #[test]
    fn test_table_id_from_u64() {
        let id: TableId = 123u64.into();
        assert_eq!(id.0, 123);
    }

    #[test]
    fn test_lineage_id_display() {
        let id = LineageId::new(99);
        assert_eq!(format!("{}", id), "LineageId(99)");
    }

    #[test]
    fn test_partition_id_display() {
        let id = PartitionId::new(7);
        assert_eq!(format!("{}", id), "PartitionId(7)");
    }

    #[test]
    fn test_record_id_display() {
        let id = RecordId::new(1);
        assert_eq!(format!("{}", id), "RecordId(1)");
    }

    #[test]
    fn test_index_config_default() {
        let config = IndexConfig::default();
        assert_eq!(config.cache_max_entries, 1000);
        assert_eq!(config.cache_default_ttl_secs, 3600);
        assert_eq!(config.statistics_sample_rate, 0.1);
        assert_eq!(config.statistics_max_age_secs, 86400);
    }

    #[test]
    fn test_index_config_builder() {
        let config = IndexConfig::new()
            .with_cache_max_entries(500)
            .with_cache_default_ttl_secs(1800)
            .with_statistics_sample_rate(0.5)
            .with_statistics_max_age_secs(43200);

        assert_eq!(config.cache_max_entries, 500);
        assert_eq!(config.cache_default_ttl_secs, 1800);
        assert_eq!(config.statistics_sample_rate, 0.5);
        assert_eq!(config.statistics_max_age_secs, 43200);
    }

    #[test]
    fn test_index_config_sample_rate_clamping() {
        let config = IndexConfig::new().with_statistics_sample_rate(1.5);
        assert_eq!(config.statistics_sample_rate, 1.0);

        let config = IndexConfig::new().with_statistics_sample_rate(-0.5);
        assert_eq!(config.statistics_sample_rate, 0.0);
    }

    #[test]
    fn test_confidence_level_serialization() {
        let level = ConfidenceLevel::High;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"high\"");

        let deserialized: ConfidenceLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ConfidenceLevel::High);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"critical\"");

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Severity::Critical);
    }

    #[test]
    fn test_tlp_level_serialization() {
        let tlp = TLPLevel::AmberStrict;
        let json = serde_json::to_string(&tlp).unwrap();
        assert_eq!(json, "\"amber_strict\"");

        let deserialized: TLPLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TLPLevel::AmberStrict);
    }

    #[test]
    fn test_tlp_level_display() {
        assert_eq!(format!("{}", TLPLevel::Clear), "TLP:CLEAR");
        assert_eq!(format!("{}", TLPLevel::Green), "TLP:GREEN");
        assert_eq!(format!("{}", TLPLevel::Amber), "TLP:AMBER");
        assert_eq!(format!("{}", TLPLevel::AmberStrict), "TLP:AMBER+STRICT");
        assert_eq!(format!("{}", TLPLevel::Red), "TLP:RED");
    }
}
