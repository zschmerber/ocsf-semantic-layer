//! Detection coverage summary types for the OCSF Semantic Index.
//!
//! This module provides types for aggregating and summarizing detection coverage
//! across all indexed tables. It's useful for gap analysis and detection engineering.
//!
//! # Example
//!
//! ```rust,ignore
//! use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
//! use ocsf_index::table_registry::DetectionCoverage;
//! use ocsf_index::backend::InMemoryBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let backend = InMemoryBackend::new();
//!     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
//!
//!     // Register tables with detection coverage
//!     let coverage1 = DetectionCoverage::new()
//!         .with_mitre_techniques(vec!["T1071.004".to_string()])
//!         .with_mitre_tactics(vec!["command-and-control".to_string()])
//!         .with_data_sources(vec!["network_connection".to_string()])
//!         .with_kill_chain_phases(vec!["delivery".to_string()]);
//!     index.register_table(
//!         TableEntry::new("network_activity", 4001).with_detection_coverage(coverage1)
//!     ).await?;
//!
//!     // Get aggregated coverage summary
//!     let summary = index.get_detection_coverage_summary().await?;
//!     println!("Tables with coverage: {}", summary.tables_with_coverage);
//!     println!("MITRE techniques covered: {}", summary.mitre_techniques.len());
//!
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Detection Coverage Summary Types
// ============================================================================

/// Summary of detection coverage across all indexed tables.
///
/// This struct aggregates detection coverage metadata from all active tables
/// in the index, providing a comprehensive view of:
/// - Which MITRE ATT&CK techniques are covered and by how many tables
/// - Which MITRE ATT&CK tactics are covered and their technique counts
/// - Which data sources are available across all tables
/// - Coverage by kill chain phase
///
/// This is useful for:
/// - Gap analysis: Identifying which techniques/tactics are not covered
/// - Detection engineering: Understanding current detection capabilities
/// - Compliance reporting: Documenting security monitoring coverage
///
/// # Example
///
/// ```rust
/// use ocsf_index::detection_coverage::{
///     DetectionCoverageSummary, MitreTechniqueCoverage, MitreTacticCoverage,
///     DataSourceCoverage, KillChainCoverage,
/// };
///
/// let summary = DetectionCoverageSummary {
///     tables_with_coverage: 5,
///     mitre_techniques: vec![
///         MitreTechniqueCoverage {
///             technique_id: "T1071.004".to_string(),
///             table_count: 2,
///             tables: vec!["network_activity".to_string(), "dns_logs".to_string()],
///         },
///     ],
///     mitre_tactics: vec![
///         MitreTacticCoverage {
///             tactic: "command-and-control".to_string(),
///             technique_count: 3,
///             table_count: 2,
///         },
///     ],
///     data_sources: vec![
///         DataSourceCoverage {
///             data_source: "network_connection".to_string(),
///             table_count: 3,
///             tables: vec!["network_activity".to_string(), "firewall_logs".to_string(), "proxy_logs".to_string()],
///         },
///     ],
///     kill_chain_coverage: vec![
///         KillChainCoverage {
///             phase: "delivery".to_string(),
///             table_count: 2,
///         },
///     ],
/// };
///
/// assert_eq!(summary.tables_with_coverage, 5);
/// assert_eq!(summary.mitre_techniques.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionCoverageSummary {
    /// Total number of tables with detection coverage metadata.
    ///
    /// This counts only active tables that have a non-empty `detection_coverage` field.
    pub tables_with_coverage: u64,

    /// All unique MITRE ATT&CK techniques covered across all tables.
    ///
    /// Each entry includes the technique ID, the count of tables covering it,
    /// and the list of table names.
    pub mitre_techniques: Vec<MitreTechniqueCoverage>,

    /// All unique MITRE ATT&CK tactics covered across all tables.
    ///
    /// Each entry includes the tactic name, the count of unique techniques
    /// associated with tables covering this tactic, and the count of tables.
    pub mitre_tactics: Vec<MitreTacticCoverage>,

    /// All unique data sources available across all tables.
    ///
    /// Each entry includes the data source name, the count of tables providing it,
    /// and the list of table names.
    pub data_sources: Vec<DataSourceCoverage>,

    /// Coverage by kill chain phase.
    ///
    /// Each entry includes the phase name and the count of tables covering it.
    pub kill_chain_coverage: Vec<KillChainCoverage>,
}

impl Default for DetectionCoverageSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionCoverageSummary {
    /// Creates a new empty DetectionCoverageSummary.
    pub fn new() -> Self {
        Self {
            tables_with_coverage: 0,
            mitre_techniques: Vec::new(),
            mitre_tactics: Vec::new(),
            data_sources: Vec::new(),
            kill_chain_coverage: Vec::new(),
        }
    }
}

/// Coverage information for a single MITRE ATT&CK technique.
///
/// This struct tracks which tables provide detection capability for a specific
/// MITRE technique, enabling gap analysis at the technique level.
///
/// # Example
///
/// ```rust
/// use ocsf_index::detection_coverage::MitreTechniqueCoverage;
///
/// let coverage = MitreTechniqueCoverage {
///     technique_id: "T1071.004".to_string(),
///     table_count: 2,
///     tables: vec!["network_activity".to_string(), "dns_logs".to_string()],
/// };
///
/// assert_eq!(coverage.technique_id, "T1071.004");
/// assert_eq!(coverage.table_count, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MitreTechniqueCoverage {
    /// The MITRE ATT&CK technique ID (e.g., "T1071.004", "T1110.003").
    pub technique_id: String,

    /// The number of tables that cover this technique.
    pub table_count: u64,

    /// The names of tables that cover this technique.
    pub tables: Vec<String>,
}

impl MitreTechniqueCoverage {
    /// Creates a new MitreTechniqueCoverage for the given technique ID.
    pub fn new(technique_id: impl Into<String>) -> Self {
        Self {
            technique_id: technique_id.into(),
            table_count: 0,
            tables: Vec::new(),
        }
    }

    /// Adds a table to this technique's coverage.
    pub fn add_table(&mut self, table_name: impl Into<String>) {
        let name = table_name.into();
        if !self.tables.contains(&name) {
            self.tables.push(name);
            self.table_count = self.tables.len() as u64;
        }
    }
}

/// Coverage information for a single MITRE ATT&CK tactic.
///
/// This struct tracks which tables provide detection capability for a specific
/// MITRE tactic, along with the count of unique techniques covered by those tables.
///
/// # Example
///
/// ```rust
/// use ocsf_index::detection_coverage::MitreTacticCoverage;
///
/// let coverage = MitreTacticCoverage {
///     tactic: "credential-access".to_string(),
///     technique_count: 5,
///     table_count: 3,
/// };
///
/// assert_eq!(coverage.tactic, "credential-access");
/// assert_eq!(coverage.technique_count, 5);
/// assert_eq!(coverage.table_count, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MitreTacticCoverage {
    /// The MITRE ATT&CK tactic name (e.g., "credential-access", "lateral-movement").
    pub tactic: String,

    /// The number of unique techniques covered by tables with this tactic.
    pub technique_count: u64,

    /// The number of tables that cover this tactic.
    pub table_count: u64,
}

impl MitreTacticCoverage {
    /// Creates a new MitreTacticCoverage for the given tactic.
    pub fn new(tactic: impl Into<String>) -> Self {
        Self {
            tactic: tactic.into(),
            technique_count: 0,
            table_count: 0,
        }
    }
}

/// Coverage information for a single data source.
///
/// This struct tracks which tables provide a specific data source,
/// enabling data source gap analysis.
///
/// # Example
///
/// ```rust
/// use ocsf_index::detection_coverage::DataSourceCoverage;
///
/// let coverage = DataSourceCoverage {
///     data_source: "network_connection".to_string(),
///     table_count: 3,
///     tables: vec![
///         "network_activity".to_string(),
///         "firewall_logs".to_string(),
///         "proxy_logs".to_string(),
///     ],
/// };
///
/// assert_eq!(coverage.data_source, "network_connection");
/// assert_eq!(coverage.table_count, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataSourceCoverage {
    /// The data source name (e.g., "process_creation", "network_connection").
    pub data_source: String,

    /// The number of tables that provide this data source.
    pub table_count: u64,

    /// The names of tables that provide this data source.
    pub tables: Vec<String>,
}

impl DataSourceCoverage {
    /// Creates a new DataSourceCoverage for the given data source.
    pub fn new(data_source: impl Into<String>) -> Self {
        Self {
            data_source: data_source.into(),
            table_count: 0,
            tables: Vec::new(),
        }
    }

    /// Adds a table to this data source's coverage.
    pub fn add_table(&mut self, table_name: impl Into<String>) {
        let name = table_name.into();
        if !self.tables.contains(&name) {
            self.tables.push(name);
            self.table_count = self.tables.len() as u64;
        }
    }
}

/// Coverage information for a single kill chain phase.
///
/// This struct tracks how many tables provide detection capability
/// for a specific kill chain phase.
///
/// # Example
///
/// ```rust
/// use ocsf_index::detection_coverage::KillChainCoverage;
///
/// let coverage = KillChainCoverage {
///     phase: "delivery".to_string(),
///     table_count: 2,
/// };
///
/// assert_eq!(coverage.phase, "delivery");
/// assert_eq!(coverage.table_count, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KillChainCoverage {
    /// The kill chain phase name (e.g., "reconnaissance", "delivery", "exploitation").
    pub phase: String,

    /// The number of tables that cover this kill chain phase.
    pub table_count: u64,
}

impl KillChainCoverage {
    /// Creates a new KillChainCoverage for the given phase.
    pub fn new(phase: impl Into<String>) -> Self {
        Self {
            phase: phase.into(),
            table_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // DetectionCoverageSummary Tests
    // ========================================================================

    #[test]
    fn test_detection_coverage_summary_new() {
        let summary = DetectionCoverageSummary::new();
        assert_eq!(summary.tables_with_coverage, 0);
        assert!(summary.mitre_techniques.is_empty());
        assert!(summary.mitre_tactics.is_empty());
        assert!(summary.data_sources.is_empty());
        assert!(summary.kill_chain_coverage.is_empty());
    }

    #[test]
    fn test_detection_coverage_summary_default() {
        let summary = DetectionCoverageSummary::default();
        assert_eq!(summary.tables_with_coverage, 0);
        assert!(summary.mitre_techniques.is_empty());
    }

    #[test]
    fn test_detection_coverage_summary_json_roundtrip() {
        let summary = DetectionCoverageSummary {
            tables_with_coverage: 5,
            mitre_techniques: vec![MitreTechniqueCoverage {
                technique_id: "T1071.004".to_string(),
                table_count: 2,
                tables: vec!["table1".to_string(), "table2".to_string()],
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
                    "table1".to_string(),
                    "table2".to_string(),
                    "table3".to_string(),
                ],
            }],
            kill_chain_coverage: vec![KillChainCoverage {
                phase: "delivery".to_string(),
                table_count: 2,
            }],
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: DetectionCoverageSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, deserialized);
    }

    // ========================================================================
    // MitreTechniqueCoverage Tests
    // ========================================================================

    #[test]
    fn test_mitre_technique_coverage_new() {
        let coverage = MitreTechniqueCoverage::new("T1071.004");
        assert_eq!(coverage.technique_id, "T1071.004");
        assert_eq!(coverage.table_count, 0);
        assert!(coverage.tables.is_empty());
    }

    #[test]
    fn test_mitre_technique_coverage_add_table() {
        let mut coverage = MitreTechniqueCoverage::new("T1071.004");
        coverage.add_table("network_activity");
        coverage.add_table("dns_logs");

        assert_eq!(coverage.table_count, 2);
        assert_eq!(coverage.tables.len(), 2);
        assert!(coverage.tables.contains(&"network_activity".to_string()));
        assert!(coverage.tables.contains(&"dns_logs".to_string()));
    }

    #[test]
    fn test_mitre_technique_coverage_add_table_deduplication() {
        let mut coverage = MitreTechniqueCoverage::new("T1071.004");
        coverage.add_table("network_activity");
        coverage.add_table("network_activity"); // Duplicate

        assert_eq!(coverage.table_count, 1);
        assert_eq!(coverage.tables.len(), 1);
    }

    #[test]
    fn test_mitre_technique_coverage_json_roundtrip() {
        let coverage = MitreTechniqueCoverage {
            technique_id: "T1071.004".to_string(),
            table_count: 2,
            tables: vec!["table1".to_string(), "table2".to_string()],
        };

        let json = serde_json::to_string(&coverage).unwrap();
        let deserialized: MitreTechniqueCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(coverage, deserialized);
    }

    // ========================================================================
    // MitreTacticCoverage Tests
    // ========================================================================

    #[test]
    fn test_mitre_tactic_coverage_new() {
        let coverage = MitreTacticCoverage::new("credential-access");
        assert_eq!(coverage.tactic, "credential-access");
        assert_eq!(coverage.technique_count, 0);
        assert_eq!(coverage.table_count, 0);
    }

    #[test]
    fn test_mitre_tactic_coverage_json_roundtrip() {
        let coverage = MitreTacticCoverage {
            tactic: "credential-access".to_string(),
            technique_count: 5,
            table_count: 3,
        };

        let json = serde_json::to_string(&coverage).unwrap();
        let deserialized: MitreTacticCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(coverage, deserialized);
    }

    // ========================================================================
    // DataSourceCoverage Tests
    // ========================================================================

    #[test]
    fn test_data_source_coverage_new() {
        let coverage = DataSourceCoverage::new("network_connection");
        assert_eq!(coverage.data_source, "network_connection");
        assert_eq!(coverage.table_count, 0);
        assert!(coverage.tables.is_empty());
    }

    #[test]
    fn test_data_source_coverage_add_table() {
        let mut coverage = DataSourceCoverage::new("network_connection");
        coverage.add_table("network_activity");
        coverage.add_table("firewall_logs");

        assert_eq!(coverage.table_count, 2);
        assert_eq!(coverage.tables.len(), 2);
        assert!(coverage.tables.contains(&"network_activity".to_string()));
        assert!(coverage.tables.contains(&"firewall_logs".to_string()));
    }

    #[test]
    fn test_data_source_coverage_add_table_deduplication() {
        let mut coverage = DataSourceCoverage::new("network_connection");
        coverage.add_table("network_activity");
        coverage.add_table("network_activity"); // Duplicate

        assert_eq!(coverage.table_count, 1);
        assert_eq!(coverage.tables.len(), 1);
    }

    #[test]
    fn test_data_source_coverage_json_roundtrip() {
        let coverage = DataSourceCoverage {
            data_source: "network_connection".to_string(),
            table_count: 2,
            tables: vec!["table1".to_string(), "table2".to_string()],
        };

        let json = serde_json::to_string(&coverage).unwrap();
        let deserialized: DataSourceCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(coverage, deserialized);
    }

    // ========================================================================
    // KillChainCoverage Tests
    // ========================================================================

    #[test]
    fn test_kill_chain_coverage_new() {
        let coverage = KillChainCoverage::new("delivery");
        assert_eq!(coverage.phase, "delivery");
        assert_eq!(coverage.table_count, 0);
    }

    #[test]
    fn test_kill_chain_coverage_json_roundtrip() {
        let coverage = KillChainCoverage {
            phase: "delivery".to_string(),
            table_count: 2,
        };

        let json = serde_json::to_string(&coverage).unwrap();
        let deserialized: KillChainCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(coverage, deserialized);
    }
}
