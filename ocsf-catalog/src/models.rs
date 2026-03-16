//! Data models for catalog entries, lineage, sync results, and plugin info.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ocsf_semantic::entity::{SemanticAttribute, EntityRelationship};

/// Unique identifier for a catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CatalogEntryId(pub u64);

/// Detection coverage metadata embedded in a catalog entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionCoverage {
    pub mitre_techniques: Vec<String>,
    pub mitre_tactics: Vec<String>,
    pub data_sources: Vec<String>,
    pub detection_rules: Vec<String>,
    pub kill_chain_phases: Vec<String>,
    pub confidence_level: Option<String>,
    pub max_severity: Option<String>,
}

/// Source-level lineage record for a catalog entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogLineageRecord {
    pub source_system: String,
    pub source_table: String,
    pub description: String,
    pub record_count: Option<u64>,
}

/// Field-level lineage within a catalog entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogFieldLineage {
    pub source_field: String,
    pub target_field: String,
    pub transformation: Option<String>,
}

/// A single semantic catalog entry representing one OCSF entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Assigned ID (set after persistence).
    pub id: Option<CatalogEntryId>,
    /// Entity name.
    pub entity_name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Detailed description.
    pub description: String,
    /// OCSF schema version this entry is associated with.
    pub ocsf_version: String,
    /// Source OCSF event class UIDs.
    pub source_event_classes: Vec<u32>,
    /// Semantic attributes with OCSF mappings.
    pub attributes: Vec<SemanticAttribute>,
    /// Entity relationships.
    pub relationships: Vec<EntityRelationship>,
    /// Observable type IDs covered.
    pub covers_observables: Vec<u32>,
    /// Detection coverage metadata.
    pub detection_coverage: Option<DetectionCoverage>,
    /// Source lineage records.
    pub source_lineage: Vec<CatalogLineageRecord>,
    /// Field-level lineage records.
    pub field_lineage: Vec<CatalogFieldLineage>,
    /// Catalog version when this entry was last modified.
    pub catalog_version: u64,
    /// Timestamp of last modification.
    pub updated_at: DateTime<Utc>,
}

/// Result of a push sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub engine: String,
    pub entries_pushed: usize,
}

/// Result of a pull sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub engine: String,
    pub entries_pulled: usize,
    pub entries_merged: usize,
}

/// Result of a full bidirectional sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub engine: String,
    pub push: PushResult,
    pub pull: PullResult,
    pub conflicts_resolved: usize,
}

/// Diff between catalog state and engine metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDiff {
    pub engine: String,
    pub added: Vec<CatalogEntry>,
    pub modified: Vec<CatalogEntry>,
    pub removed: Vec<String>, // entity names
}

/// Info about a registered plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub engine: String,
    pub connected: bool,
}
