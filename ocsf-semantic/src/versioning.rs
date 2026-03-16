//! Semantic model versioning and change tracking.
//!
//! This module provides versioning support for semantic models, including
//! version parsing, comparison, and change tracking between model versions.
//!
//! # Features
//!
//! - Semantic versioning (major.minor.patch) with pre-release and build metadata
//! - Model comparison to detect changes between versions
//! - Change history tracking with timestamps and descriptions
//! - Version bump suggestions based on change types
//!
//! # Example
//!
//! ```
//! use ocsf_semantic::versioning::{ModelVersion, VersionedModel, ModificationRecord};
//! use ocsf_semantic::model::SemanticModel;
//!
//! // Create a versioned model
//! let model = SemanticModel::new("my-model");
//! let mut versioned = VersionedModel::new(model);
//!
//! // Record a modification
//! versioned.record_modification("Added new entity", Some("Initial entity setup"));
//!
//! // Get modification history
//! let history = versioned.modification_history();
//! assert_eq!(history.len(), 1);
//! ```

use std::cmp::Ordering;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entity::SemanticEntity;
use crate::metric::SemanticMetric;
use crate::model::SemanticModel;

/// Error type for version parsing.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum VersionError {
    /// Invalid version format.
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    /// Invalid version component.
    #[error("Invalid version component '{component}': {reason}")]
    InvalidComponent { component: String, reason: String },
}

/// A semantic version for models.
///
/// Follows semantic versioning (major.minor.patch) with optional pre-release
/// and build metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Major version (breaking changes).
    pub major: u32,
    /// Minor version (new features, backward compatible).
    pub minor: u32,
    /// Patch version (bug fixes).
    pub patch: u32,
    /// Optional pre-release identifier (e.g., "alpha", "beta.1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_release: Option<String>,
    /// Optional build metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

impl Default for ModelVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
            pre_release: None,
            build: None,
        }
    }
}

impl ModelVersion {
    /// Creates a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }

    /// Sets the pre-release identifier.
    pub fn with_pre_release(mut self, pre_release: impl Into<String>) -> Self {
        self.pre_release = Some(pre_release.into());
        self
    }

    /// Sets the build metadata.
    pub fn with_build(mut self, build: impl Into<String>) -> Self {
        self.build = Some(build.into());
        self
    }

    /// Increments the major version and resets minor and patch.
    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self.pre_release = None;
    }

    /// Increments the minor version and resets patch.
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
        self.pre_release = None;
    }

    /// Increments the patch version.
    pub fn bump_patch(&mut self) {
        self.patch += 1;
        self.pre_release = None;
    }

    /// Returns true if this is a pre-release version.
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
}

impl fmt::Display for ModelVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl FromStr for ModelVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle build metadata
        let (version_pre, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        // Handle pre-release
        let (version, pre_release) = if let Some(idx) = version_pre.find('-') {
            (&version_pre[..idx], Some(version_pre[idx + 1..].to_string()))
        } else {
            (version_pre, None)
        };

        // Parse version numbers
        let parts: Vec<&str> = version.split('.').collect();
        
        // Support both "1.0" and "1.0.0" formats
        if parts.len() < 2 || parts.len() > 3 {
            return Err(VersionError::InvalidFormat(format!(
                "Expected 2 or 3 version components, got {}",
                parts.len()
            )));
        }

        let major = parts[0].parse().map_err(|_| VersionError::InvalidComponent {
            component: parts[0].to_string(),
            reason: "not a valid number".to_string(),
        })?;

        let minor = parts[1].parse().map_err(|_| VersionError::InvalidComponent {
            component: parts[1].to_string(),
            reason: "not a valid number".to_string(),
        })?;

        let patch = if parts.len() > 2 {
            parts[2].parse().map_err(|_| VersionError::InvalidComponent {
                component: parts[2].to_string(),
                reason: "not a valid number".to_string(),
            })?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }
}

impl PartialOrd for ModelVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Pre-release versions have lower precedence
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Type of change between model versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelChangeType {
    /// Entity was added.
    EntityAdded,
    /// Entity was removed.
    EntityRemoved,
    /// Entity was modified.
    EntityModified,
    /// Metric was added.
    MetricAdded,
    /// Metric was removed.
    MetricRemoved,
    /// Metric was modified.
    MetricModified,
    /// Observable config changed.
    ObservableConfigChanged,
    /// Model metadata changed.
    MetadataChanged,
}

/// A single change between model versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelChange {
    /// Type of change.
    pub change_type: ModelChangeType,
    /// Name of the affected element.
    pub element_name: String,
    /// Description of the change.
    pub description: String,
    /// Whether this is a breaking change.
    pub is_breaking: bool,
}

impl ModelChange {
    /// Creates a new model change.
    pub fn new(
        change_type: ModelChangeType,
        element_name: impl Into<String>,
        description: impl Into<String>,
        is_breaking: bool,
    ) -> Self {
        Self {
            change_type,
            element_name: element_name.into(),
            description: description.into(),
            is_breaking,
        }
    }
}

/// Diff between two model versions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelDiff {
    /// Version of the old model.
    pub old_version: String,
    /// Version of the new model.
    pub new_version: String,
    /// List of changes.
    pub changes: Vec<ModelChange>,
}

impl ModelDiff {
    /// Creates a new empty diff.
    pub fn new(old_version: impl Into<String>, new_version: impl Into<String>) -> Self {
        Self {
            old_version: old_version.into(),
            new_version: new_version.into(),
            changes: Vec::new(),
        }
    }

    /// Adds a change to the diff.
    pub fn add_change(&mut self, change: ModelChange) {
        self.changes.push(change);
    }

    /// Returns true if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns true if there are any breaking changes.
    pub fn has_breaking_changes(&self) -> bool {
        self.changes.iter().any(|c| c.is_breaking)
    }

    /// Returns all breaking changes.
    pub fn breaking_changes(&self) -> impl Iterator<Item = &ModelChange> {
        self.changes.iter().filter(|c| c.is_breaking)
    }

    /// Returns the count of each change type.
    pub fn change_summary(&self) -> std::collections::HashMap<ModelChangeType, usize> {
        let mut summary = std::collections::HashMap::new();
        for change in &self.changes {
            *summary.entry(change.change_type).or_insert(0) += 1;
        }
        summary
    }
}

/// Compares two semantic models and returns the differences.
pub fn compare_models(old: &SemanticModel, new: &SemanticModel) -> ModelDiff {
    let mut diff = ModelDiff::new(&old.version, &new.version);

    // Compare entities
    compare_entities(&old.entities, &new.entities, &mut diff);

    // Compare metrics
    compare_metrics(&old.metrics, &new.metrics, &mut diff);

    // Compare observable config
    if old.observable_config != new.observable_config {
        diff.add_change(ModelChange::new(
            ModelChangeType::ObservableConfigChanged,
            "observable_config",
            "Observable configuration changed",
            false,
        ));
    }

    // Compare metadata
    if old.name != new.name || old.description != new.description || old.ocsf_version != new.ocsf_version {
        diff.add_change(ModelChange::new(
            ModelChangeType::MetadataChanged,
            "metadata",
            "Model metadata changed",
            false,
        ));
    }

    diff
}

/// Compares entity lists and adds changes to the diff.
fn compare_entities(old: &[SemanticEntity], new: &[SemanticEntity], diff: &mut ModelDiff) {
    let old_names: std::collections::HashSet<_> = old.iter().map(|e| &e.name).collect();
    let new_names: std::collections::HashSet<_> = new.iter().map(|e| &e.name).collect();

    // Find added entities
    for name in new_names.difference(&old_names) {
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityAdded,
            *name,
            format!("Entity '{}' was added", name),
            false,
        ));
    }

    // Find removed entities (breaking change)
    for name in old_names.difference(&new_names) {
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityRemoved,
            *name,
            format!("Entity '{}' was removed", name),
            true, // Removing entities is a breaking change
        ));
    }

    // Find modified entities
    for name in old_names.intersection(&new_names) {
        let old_entity = old.iter().find(|e| &e.name == *name).unwrap();
        let new_entity = new.iter().find(|e| &e.name == *name).unwrap();

        if old_entity != new_entity {
            let is_breaking = is_entity_change_breaking(old_entity, new_entity);
            diff.add_change(ModelChange::new(
                ModelChangeType::EntityModified,
                *name,
                format!("Entity '{}' was modified", name),
                is_breaking,
            ));
        }
    }
}

/// Determines if an entity change is breaking.
fn is_entity_change_breaking(old: &SemanticEntity, new: &SemanticEntity) -> bool {
    // Removing attributes is breaking
    let old_attrs: std::collections::HashSet<_> = old.attributes.iter().map(|a| &a.name).collect();
    let new_attrs: std::collections::HashSet<_> = new.attributes.iter().map(|a| &a.name).collect();

    if old_attrs.difference(&new_attrs).next().is_some() {
        return true;
    }

    // Removing source event classes is breaking
    for class in &old.source_event_classes {
        if !new.source_event_classes.contains(class) {
            return true;
        }
    }

    // Removing relationships is breaking
    let old_rels: std::collections::HashSet<_> = old.relationships.iter().map(|r| &r.name).collect();
    let new_rels: std::collections::HashSet<_> = new.relationships.iter().map(|r| &r.name).collect();

    if old_rels.difference(&new_rels).next().is_some() {
        return true;
    }

    false
}

/// Compares metric lists and adds changes to the diff.
fn compare_metrics(old: &[SemanticMetric], new: &[SemanticMetric], diff: &mut ModelDiff) {
    let old_names: std::collections::HashSet<_> = old.iter().map(|m| &m.name).collect();
    let new_names: std::collections::HashSet<_> = new.iter().map(|m| &m.name).collect();

    // Find added metrics
    for name in new_names.difference(&old_names) {
        diff.add_change(ModelChange::new(
            ModelChangeType::MetricAdded,
            *name,
            format!("Metric '{}' was added", name),
            false,
        ));
    }

    // Find removed metrics (breaking change)
    for name in old_names.difference(&new_names) {
        diff.add_change(ModelChange::new(
            ModelChangeType::MetricRemoved,
            *name,
            format!("Metric '{}' was removed", name),
            true, // Removing metrics is a breaking change
        ));
    }

    // Find modified metrics
    for name in old_names.intersection(&new_names) {
        let old_metric = old.iter().find(|m| &m.name == *name).unwrap();
        let new_metric = new.iter().find(|m| &m.name == *name).unwrap();

        if old_metric != new_metric {
            let is_breaking = is_metric_change_breaking(old_metric, new_metric);
            diff.add_change(ModelChange::new(
                ModelChangeType::MetricModified,
                *name,
                format!("Metric '{}' was modified", name),
                is_breaking,
            ));
        }
    }
}

/// Determines if a metric change is breaking.
fn is_metric_change_breaking(old: &SemanticMetric, new: &SemanticMetric) -> bool {
    // Changing aggregation type is breaking
    if old.aggregation != new.aggregation {
        return true;
    }

    // Removing dimensions is breaking
    for dim in &old.dimensions {
        if !new.dimensions.contains(dim) {
            return true;
        }
    }

    // Removing time granularities is breaking
    for gran in &old.time_granularities {
        if !new.time_granularities.contains(gran) {
            return true;
        }
    }

    false
}

/// Suggests the next version based on changes.
pub fn suggest_version_bump(current: &ModelVersion, diff: &ModelDiff) -> ModelVersion {
    let mut new_version = current.clone();

    if diff.has_breaking_changes() {
        new_version.bump_major();
    } else if diff.changes.iter().any(|c| {
        matches!(
            c.change_type,
            ModelChangeType::EntityAdded | ModelChangeType::MetricAdded
        )
    }) {
        new_version.bump_minor();
    } else if diff.has_changes() {
        new_version.bump_patch();
    }

    new_version
}

// ============================================================================
// Modification History Tracking
// ============================================================================

/// A record of a modification to the semantic model.
///
/// Tracks when changes were made, what changed, and optionally why.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModificationRecord {
    /// ISO 8601 timestamp of when the modification occurred.
    pub timestamp: String,

    /// Unix timestamp in seconds for ordering.
    #[serde(default)]
    pub unix_timestamp: u64,

    /// Summary of what changed.
    pub summary: String,

    /// Optional detailed description of the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The version after this modification (if version was bumped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_after: Option<String>,

    /// Detailed changes if available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<ModelChange>,
}

impl ModificationRecord {
    /// Creates a new modification record with the current timestamp.
    pub fn new(summary: impl Into<String>) -> Self {
        let (timestamp, unix_timestamp) = current_timestamp();
        Self {
            timestamp,
            unix_timestamp,
            summary: summary.into(),
            description: None,
            version_after: None,
            changes: Vec::new(),
        }
    }

    /// Creates a modification record with a specific timestamp (for testing).
    pub fn with_timestamp(
        summary: impl Into<String>,
        timestamp: impl Into<String>,
        unix_timestamp: u64,
    ) -> Self {
        Self {
            timestamp: timestamp.into(),
            unix_timestamp,
            summary: summary.into(),
            description: None,
            version_after: None,
            changes: Vec::new(),
        }
    }

    /// Adds a description to the modification record.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the version after this modification.
    pub fn with_version_after(mut self, version: impl Into<String>) -> Self {
        self.version_after = Some(version.into());
        self
    }

    /// Adds detailed changes to the record.
    pub fn with_changes(mut self, changes: Vec<ModelChange>) -> Self {
        self.changes = changes;
        self
    }

    /// Adds a single change to the record.
    pub fn add_change(mut self, change: ModelChange) -> Self {
        self.changes.push(change);
        self
    }
}

/// Generates the current timestamp in ISO 8601 format and Unix timestamp.
fn current_timestamp() -> (String, u64) {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let unix_secs = duration.as_secs();
    
    // Simple ISO 8601 format without external dependencies
    // Format: YYYY-MM-DDTHH:MM:SSZ (UTC)
    let secs_per_day = 86400u64;
    let secs_per_hour = 3600u64;
    let secs_per_minute = 60u64;
    
    // Days since Unix epoch
    let days = unix_secs / secs_per_day;
    let remaining = unix_secs % secs_per_day;
    
    let hours = remaining / secs_per_hour;
    let remaining = remaining % secs_per_hour;
    let minutes = remaining / secs_per_minute;
    let seconds = remaining % secs_per_minute;
    
    // Calculate year, month, day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days);
    
    let iso = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    );
    
    (iso, unix_secs)
}

/// Converts days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Simplified calculation - good enough for timestamps
    let mut remaining_days = days as i64;
    let mut year = 1970i32;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1u32;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }
    
    let day = remaining_days as u32 + 1;
    
    (year as u32, month, day)
}

/// Returns true if the year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ============================================================================
// Versioned Model
// ============================================================================

/// A semantic model with version tracking and modification history.
///
/// Wraps a `SemanticModel` with:
/// - Parsed semantic version
/// - Modification history with timestamps and descriptions
/// - Snapshot capability for version comparison
///
/// # Example
///
/// ```
/// use ocsf_semantic::versioning::VersionedModel;
/// use ocsf_semantic::model::SemanticModel;
/// use ocsf_semantic::entity::SemanticEntity;
///
/// let model = SemanticModel::new("my-model").with_version("1.0.0");
/// let mut versioned = VersionedModel::new(model);
///
/// // Make changes and record them
/// versioned.model_mut().entities.push(SemanticEntity::new("new_entity"));
/// versioned.record_modification("Added new_entity", Some("Initial entity for user tracking"));
///
/// // Check history
/// assert_eq!(versioned.modification_history().len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedModel {
    /// The underlying semantic model.
    #[serde(flatten)]
    model: SemanticModel,

    /// Modification history.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    modification_history: Vec<ModificationRecord>,

    /// Previous model snapshot for comparison (not serialized).
    #[serde(skip)]
    previous_snapshot: Option<Box<SemanticModel>>,
}

impl VersionedModel {
    /// Creates a new versioned model from a semantic model.
    pub fn new(model: SemanticModel) -> Self {
        Self {
            model,
            modification_history: Vec::new(),
            previous_snapshot: None,
        }
    }

    /// Creates a versioned model with existing history.
    pub fn with_history(model: SemanticModel, history: Vec<ModificationRecord>) -> Self {
        Self {
            model,
            modification_history: history,
            previous_snapshot: None,
        }
    }

    /// Returns a reference to the underlying model.
    pub fn model(&self) -> &SemanticModel {
        &self.model
    }

    /// Returns a mutable reference to the underlying model.
    pub fn model_mut(&mut self) -> &mut SemanticModel {
        &mut self.model
    }

    /// Returns the modification history.
    pub fn modification_history(&self) -> &[ModificationRecord] {
        &self.modification_history
    }

    /// Returns the current version as a parsed `ModelVersion`.
    pub fn version(&self) -> Result<ModelVersion, VersionError> {
        self.model.version.parse()
    }

    /// Takes a snapshot of the current model state for later comparison.
    pub fn take_snapshot(&mut self) {
        self.previous_snapshot = Some(Box::new(self.model.clone()));
    }

    /// Records a modification to the model.
    ///
    /// If a snapshot was taken, this will also compute the diff and
    /// suggest a version bump.
    pub fn record_modification(
        &mut self,
        summary: impl Into<String>,
        description: Option<&str>,
    ) {
        let mut record = ModificationRecord::new(summary);
        
        if let Some(desc) = description {
            record = record.with_description(desc);
        }

        // If we have a snapshot, compute the diff
        if let Some(ref snapshot) = self.previous_snapshot {
            let diff = compare_models(snapshot, &self.model);
            if diff.has_changes() {
                record = record.with_changes(diff.changes.clone());
                
                // Suggest and apply version bump
                if let Ok(current_version) = self.version() {
                    let new_version = suggest_version_bump(&current_version, &diff);
                    self.model.version = new_version.to_string();
                    record = record.with_version_after(new_version.to_string());
                }
            }
        }

        record = record.with_version_after(self.model.version.clone());
        self.modification_history.push(record);
        self.previous_snapshot = None;
    }

    /// Records a modification with explicit changes (without snapshot comparison).
    pub fn record_modification_with_changes(
        &mut self,
        summary: impl Into<String>,
        description: Option<&str>,
        changes: Vec<ModelChange>,
    ) {
        let mut record = ModificationRecord::new(summary)
            .with_changes(changes)
            .with_version_after(self.model.version.clone());
        
        if let Some(desc) = description {
            record = record.with_description(desc);
        }

        self.modification_history.push(record);
    }

    /// Bumps the version based on the change type.
    pub fn bump_version(&mut self, bump_type: VersionBumpType) -> Result<(), VersionError> {
        let mut version = self.version()?;
        match bump_type {
            VersionBumpType::Major => version.bump_major(),
            VersionBumpType::Minor => version.bump_minor(),
            VersionBumpType::Patch => version.bump_patch(),
        }
        self.model.version = version.to_string();
        Ok(())
    }

    /// Compares this model with another model.
    pub fn compare_with(&self, other: &SemanticModel) -> ModelDiff {
        compare_models(&self.model, other)
    }

    /// Compares this model with the previous snapshot.
    pub fn compare_with_snapshot(&self) -> Option<ModelDiff> {
        self.previous_snapshot
            .as_ref()
            .map(|snapshot| compare_models(snapshot, &self.model))
    }

    /// Serializes the versioned model to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize versioned model to YAML")
    }

    /// Deserializes a versioned model from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse versioned model from YAML")
    }

    /// Saves the versioned model to a YAML file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path.as_ref(), yaml)
            .with_context(|| format!("Failed to write versioned model to {:?}", path.as_ref()))?;
        Ok(())
    }

    /// Loads a versioned model from a YAML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let yaml = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read versioned model from {:?}", path.as_ref()))?;
        Self::from_yaml(&yaml)
    }

    /// Returns the most recent modification record.
    pub fn last_modification(&self) -> Option<&ModificationRecord> {
        self.modification_history.last()
    }

    /// Returns modifications within a time range (by Unix timestamp).
    pub fn modifications_in_range(&self, start: u64, end: u64) -> Vec<&ModificationRecord> {
        self.modification_history
            .iter()
            .filter(|r| r.unix_timestamp >= start && r.unix_timestamp <= end)
            .collect()
    }

    /// Clears the modification history.
    pub fn clear_history(&mut self) {
        self.modification_history.clear();
    }
}

/// Type of version bump to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionBumpType {
    /// Major version bump (breaking changes).
    Major,
    /// Minor version bump (new features).
    Minor,
    /// Patch version bump (bug fixes).
    Patch,
}

// ============================================================================
// Version History Summary
// ============================================================================

/// A summary of version history for display purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistorySummary {
    /// Current version.
    pub current_version: String,
    /// Total number of modifications.
    pub total_modifications: usize,
    /// Number of breaking changes in history.
    pub breaking_changes: usize,
    /// Timestamp of first modification.
    pub first_modification: Option<String>,
    /// Timestamp of last modification.
    pub last_modification: Option<String>,
}

impl VersionedModel {
    /// Generates a summary of the version history.
    pub fn history_summary(&self) -> VersionHistorySummary {
        let breaking_changes = self
            .modification_history
            .iter()
            .flat_map(|r| &r.changes)
            .filter(|c| c.is_breaking)
            .count();

        VersionHistorySummary {
            current_version: self.model.version.clone(),
            total_modifications: self.modification_history.len(),
            breaking_changes,
            first_modification: self.modification_history.first().map(|r| r.timestamp.clone()),
            last_modification: self.modification_history.last().map(|r| r.timestamp.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{SemanticAttribute, SemanticEntity};
    use crate::metric::{Aggregation, SemanticMetric};
    use crate::model::ObservableConfig;

    #[test]
    fn test_model_version_new() {
        let v = ModelVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre_release.is_none());
        assert!(v.build.is_none());
    }

    #[test]
    fn test_model_version_with_pre_release() {
        let v = ModelVersion::new(1, 0, 0).with_pre_release("alpha");
        assert_eq!(v.pre_release, Some("alpha".to_string()));
        assert!(v.is_pre_release());
    }

    #[test]
    fn test_model_version_with_build() {
        let v = ModelVersion::new(1, 0, 0).with_build("20240101");
        assert_eq!(v.build, Some("20240101".to_string()));
    }

    #[test]
    fn test_model_version_bump_major() {
        let mut v = ModelVersion::new(1, 2, 3).with_pre_release("beta");
        v.bump_major();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert!(v.pre_release.is_none());
    }

    #[test]
    fn test_model_version_bump_minor() {
        let mut v = ModelVersion::new(1, 2, 3);
        v.bump_minor();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 3);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_model_version_bump_patch() {
        let mut v = ModelVersion::new(1, 2, 3);
        v.bump_patch();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 4);
    }

    #[test]
    fn test_model_version_display() {
        assert_eq!(ModelVersion::new(1, 2, 3).to_string(), "1.2.3");
        assert_eq!(
            ModelVersion::new(1, 0, 0).with_pre_release("alpha").to_string(),
            "1.0.0-alpha"
        );
        assert_eq!(
            ModelVersion::new(1, 0, 0).with_build("123").to_string(),
            "1.0.0+123"
        );
        assert_eq!(
            ModelVersion::new(1, 0, 0)
                .with_pre_release("beta")
                .with_build("456")
                .to_string(),
            "1.0.0-beta+456"
        );
    }

    #[test]
    fn test_model_version_parse() {
        assert_eq!(
            "1.2.3".parse::<ModelVersion>().unwrap(),
            ModelVersion::new(1, 2, 3)
        );
        assert_eq!(
            "1.0".parse::<ModelVersion>().unwrap(),
            ModelVersion::new(1, 0, 0)
        );
        assert_eq!(
            "2.1.0-alpha".parse::<ModelVersion>().unwrap(),
            ModelVersion::new(2, 1, 0).with_pre_release("alpha")
        );
        assert_eq!(
            "1.0.0+build123".parse::<ModelVersion>().unwrap(),
            ModelVersion::new(1, 0, 0).with_build("build123")
        );
        assert_eq!(
            "1.0.0-rc.1+build".parse::<ModelVersion>().unwrap(),
            ModelVersion::new(1, 0, 0)
                .with_pre_release("rc.1")
                .with_build("build")
        );
    }

    #[test]
    fn test_model_version_parse_errors() {
        assert!("invalid".parse::<ModelVersion>().is_err());
        assert!("1".parse::<ModelVersion>().is_err());
        assert!("1.2.3.4".parse::<ModelVersion>().is_err());
        assert!("a.b.c".parse::<ModelVersion>().is_err());
    }

    #[test]
    fn test_model_version_ordering() {
        assert!(ModelVersion::new(1, 0, 0) < ModelVersion::new(2, 0, 0));
        assert!(ModelVersion::new(1, 0, 0) < ModelVersion::new(1, 1, 0));
        assert!(ModelVersion::new(1, 0, 0) < ModelVersion::new(1, 0, 1));
        assert!(
            ModelVersion::new(1, 0, 0).with_pre_release("alpha")
                < ModelVersion::new(1, 0, 0)
        );
        assert!(
            ModelVersion::new(1, 0, 0).with_pre_release("alpha")
                < ModelVersion::new(1, 0, 0).with_pre_release("beta")
        );
    }

    #[test]
    fn test_compare_models_no_changes() {
        let model = SemanticModel::new("test");
        let diff = compare_models(&model, &model);
        assert!(!diff.has_changes());
        assert!(!diff.has_breaking_changes());
    }

    #[test]
    fn test_compare_models_entity_added() {
        let old = SemanticModel::new("test");
        let new = SemanticModel::new("test").add_entity(SemanticEntity::new("new_entity"));

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(!diff.has_breaking_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::EntityAdded && c.element_name == "new_entity"
        }));
    }

    #[test]
    fn test_compare_models_entity_removed() {
        let old = SemanticModel::new("test").add_entity(SemanticEntity::new("old_entity"));
        let new = SemanticModel::new("test");

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.has_breaking_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::EntityRemoved
                && c.element_name == "old_entity"
                && c.is_breaking
        }));
    }

    #[test]
    fn test_compare_models_entity_modified() {
        let old = SemanticModel::new("test")
            .add_entity(SemanticEntity::new("entity").with_caption("Old Caption"));
        let new = SemanticModel::new("test")
            .add_entity(SemanticEntity::new("entity").with_caption("New Caption"));

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::EntityModified && c.element_name == "entity"
        }));
    }

    #[test]
    fn test_compare_models_metric_added() {
        let old = SemanticModel::new("test");
        let new = SemanticModel::new("test").add_metric(SemanticMetric::new("new_metric"));

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(!diff.has_breaking_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::MetricAdded && c.element_name == "new_metric"
        }));
    }

    #[test]
    fn test_compare_models_metric_removed() {
        let old = SemanticModel::new("test").add_metric(SemanticMetric::new("old_metric"));
        let new = SemanticModel::new("test");

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.has_breaking_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::MetricRemoved
                && c.element_name == "old_metric"
                && c.is_breaking
        }));
    }

    #[test]
    fn test_compare_models_breaking_entity_change() {
        // Removing an attribute is a breaking change
        let old = SemanticModel::new("test").add_entity(
            SemanticEntity::new("entity")
                .add_attribute(SemanticAttribute::new("attr1"))
                .add_attribute(SemanticAttribute::new("attr2")),
        );
        let new = SemanticModel::new("test")
            .add_entity(SemanticEntity::new("entity").add_attribute(SemanticAttribute::new("attr1")));

        let diff = compare_models(&old, &new);
        assert!(diff.has_breaking_changes());
    }

    #[test]
    fn test_compare_models_breaking_metric_change() {
        // Changing aggregation type is a breaking change
        let old = SemanticModel::new("test")
            .add_metric(SemanticMetric::new("metric").with_aggregation(Aggregation::Count));
        let new = SemanticModel::new("test")
            .add_metric(SemanticMetric::new("metric").with_aggregation(Aggregation::Sum));

        let diff = compare_models(&old, &new);
        assert!(diff.has_breaking_changes());
    }

    #[test]
    fn test_compare_models_observable_config_changed() {
        let old = SemanticModel::new("test");
        let new = SemanticModel::new("test").with_observable_config(ObservableConfig {
            extract_to_table: true,
            table_name: "observables".to_string(),
            include_types: vec![1, 2, 3],
        });

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::ObservableConfigChanged
        }));
    }

    #[test]
    fn test_compare_models_metadata_changed() {
        let old = SemanticModel::new("test").with_description("Old description");
        let new = SemanticModel::new("test").with_description("New description");

        let diff = compare_models(&old, &new);
        assert!(diff.has_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::MetadataChanged
        }));
    }

    #[test]
    fn test_suggest_version_bump_breaking() {
        let current = ModelVersion::new(1, 2, 3);
        let mut diff = ModelDiff::new("1.2.3", "1.2.3");
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityRemoved,
            "entity",
            "Removed",
            true,
        ));

        let suggested = suggest_version_bump(&current, &diff);
        assert_eq!(suggested, ModelVersion::new(2, 0, 0));
    }

    #[test]
    fn test_suggest_version_bump_feature() {
        let current = ModelVersion::new(1, 2, 3);
        let mut diff = ModelDiff::new("1.2.3", "1.2.3");
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityAdded,
            "entity",
            "Added",
            false,
        ));

        let suggested = suggest_version_bump(&current, &diff);
        assert_eq!(suggested, ModelVersion::new(1, 3, 0));
    }

    #[test]
    fn test_suggest_version_bump_patch() {
        let current = ModelVersion::new(1, 2, 3);
        let mut diff = ModelDiff::new("1.2.3", "1.2.3");
        diff.add_change(ModelChange::new(
            ModelChangeType::MetadataChanged,
            "metadata",
            "Changed",
            false,
        ));

        let suggested = suggest_version_bump(&current, &diff);
        assert_eq!(suggested, ModelVersion::new(1, 2, 4));
    }

    #[test]
    fn test_suggest_version_bump_no_changes() {
        let current = ModelVersion::new(1, 2, 3);
        let diff = ModelDiff::new("1.2.3", "1.2.3");

        let suggested = suggest_version_bump(&current, &diff);
        assert_eq!(suggested, current);
    }

    #[test]
    fn test_model_diff_change_summary() {
        let mut diff = ModelDiff::new("1.0", "2.0");
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityAdded,
            "e1",
            "Added",
            false,
        ));
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityAdded,
            "e2",
            "Added",
            false,
        ));
        diff.add_change(ModelChange::new(
            ModelChangeType::MetricRemoved,
            "m1",
            "Removed",
            true,
        ));

        let summary = diff.change_summary();
        assert_eq!(summary.get(&ModelChangeType::EntityAdded), Some(&2));
        assert_eq!(summary.get(&ModelChangeType::MetricRemoved), Some(&1));
    }

    #[test]
    fn test_model_diff_breaking_changes() {
        let mut diff = ModelDiff::new("1.0", "2.0");
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityAdded,
            "e1",
            "Added",
            false,
        ));
        diff.add_change(ModelChange::new(
            ModelChangeType::EntityRemoved,
            "e2",
            "Removed",
            true,
        ));
        diff.add_change(ModelChange::new(
            ModelChangeType::MetricRemoved,
            "m1",
            "Removed",
            true,
        ));

        let breaking: Vec<_> = diff.breaking_changes().collect();
        assert_eq!(breaking.len(), 2);
    }

    // ========================================================================
    // ModificationRecord Tests
    // ========================================================================

    #[test]
    fn test_modification_record_new() {
        let record = ModificationRecord::new("Added new entity");
        assert_eq!(record.summary, "Added new entity");
        assert!(record.description.is_none());
        assert!(record.version_after.is_none());
        assert!(record.changes.is_empty());
        assert!(!record.timestamp.is_empty());
        assert!(record.unix_timestamp > 0);
    }

    #[test]
    fn test_modification_record_with_description() {
        let record = ModificationRecord::new("Added new entity")
            .with_description("This entity tracks user authentication events");
        
        assert_eq!(record.summary, "Added new entity");
        assert_eq!(
            record.description,
            Some("This entity tracks user authentication events".to_string())
        );
    }

    #[test]
    fn test_modification_record_with_version_after() {
        let record = ModificationRecord::new("Bumped version")
            .with_version_after("2.0.0");
        
        assert_eq!(record.version_after, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_modification_record_with_changes() {
        let changes = vec![
            ModelChange::new(ModelChangeType::EntityAdded, "entity1", "Added", false),
            ModelChange::new(ModelChangeType::MetricAdded, "metric1", "Added", false),
        ];
        
        let record = ModificationRecord::new("Added entities and metrics")
            .with_changes(changes);
        
        assert_eq!(record.changes.len(), 2);
    }

    #[test]
    fn test_modification_record_serialization() {
        let record = ModificationRecord::with_timestamp("Test change", "2024-01-15T10:30:00Z", 1705315800)
            .with_description("Test description")
            .with_version_after("1.1.0")
            .with_changes(vec![
                ModelChange::new(ModelChangeType::EntityAdded, "test_entity", "Added", false),
            ]);

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: ModificationRecord = serde_json::from_str(&json).unwrap();
        
        assert_eq!(record.summary, deserialized.summary);
        assert_eq!(record.description, deserialized.description);
        assert_eq!(record.version_after, deserialized.version_after);
        assert_eq!(record.changes.len(), deserialized.changes.len());
    }

    // ========================================================================
    // VersionedModel Tests
    // ========================================================================

    #[test]
    fn test_versioned_model_new() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let versioned = VersionedModel::new(model);
        
        assert_eq!(versioned.model().name, "test-model");
        assert_eq!(versioned.model().version, "1.0.0");
        assert!(versioned.modification_history().is_empty());
    }

    #[test]
    fn test_versioned_model_record_modification() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        versioned.record_modification("Initial setup", Some("Created the model"));
        
        assert_eq!(versioned.modification_history().len(), 1);
        let record = &versioned.modification_history()[0];
        assert_eq!(record.summary, "Initial setup");
        assert_eq!(record.description, Some("Created the model".to_string()));
    }

    #[test]
    fn test_versioned_model_snapshot_and_compare() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        // Take snapshot before changes
        versioned.take_snapshot();
        
        // Make changes
        versioned.model_mut().entities.push(SemanticEntity::new("new_entity"));
        
        // Compare with snapshot
        let diff = versioned.compare_with_snapshot();
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert!(diff.has_changes());
        assert!(diff.changes.iter().any(|c| c.change_type == ModelChangeType::EntityAdded));
    }

    #[test]
    fn test_versioned_model_auto_version_bump_minor() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        // Take snapshot, add entity, record modification
        versioned.take_snapshot();
        versioned.model_mut().entities.push(SemanticEntity::new("new_entity"));
        versioned.record_modification("Added new entity", None);
        
        // Version should be bumped to 1.1.0 (minor bump for new feature)
        assert_eq!(versioned.model().version, "1.1.0");
        
        // History should contain the change
        let record = versioned.last_modification().unwrap();
        assert!(record.changes.iter().any(|c| c.change_type == ModelChangeType::EntityAdded));
    }

    #[test]
    fn test_versioned_model_auto_version_bump_major() {
        let model = SemanticModel::new("test-model")
            .with_version("1.0.0")
            .add_entity(SemanticEntity::new("old_entity"));
        let mut versioned = VersionedModel::new(model);
        
        // Take snapshot, remove entity (breaking change)
        versioned.take_snapshot();
        versioned.model_mut().entities.clear();
        versioned.record_modification("Removed entity", Some("Breaking change"));
        
        // Version should be bumped to 2.0.0 (major bump for breaking change)
        assert_eq!(versioned.model().version, "2.0.0");
    }

    #[test]
    fn test_versioned_model_bump_version_manual() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        versioned.bump_version(VersionBumpType::Minor).unwrap();
        assert_eq!(versioned.model().version, "1.1.0");
        
        versioned.bump_version(VersionBumpType::Patch).unwrap();
        assert_eq!(versioned.model().version, "1.1.1");
        
        versioned.bump_version(VersionBumpType::Major).unwrap();
        assert_eq!(versioned.model().version, "2.0.0");
    }

    #[test]
    fn test_versioned_model_yaml_roundtrip() {
        let model = SemanticModel::new("test-model")
            .with_version("1.2.3")
            .with_description("Test model")
            .add_entity(SemanticEntity::new("entity1"));
        
        let mut versioned = VersionedModel::new(model);
        versioned.record_modification("Initial creation", Some("Created for testing"));
        
        let yaml = versioned.to_yaml().unwrap();
        let loaded = VersionedModel::from_yaml(&yaml).unwrap();
        
        assert_eq!(versioned.model().name, loaded.model().name);
        assert_eq!(versioned.model().version, loaded.model().version);
        assert_eq!(versioned.modification_history().len(), loaded.modification_history().len());
    }

    #[test]
    fn test_versioned_model_with_history() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let history = vec![
            ModificationRecord::with_timestamp("Change 1", "2024-01-01T00:00:00Z", 1704067200),
            ModificationRecord::with_timestamp("Change 2", "2024-01-02T00:00:00Z", 1704153600),
        ];
        
        let versioned = VersionedModel::with_history(model, history);
        assert_eq!(versioned.modification_history().len(), 2);
    }

    #[test]
    fn test_versioned_model_modifications_in_range() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let history = vec![
            ModificationRecord::with_timestamp("Change 1", "2024-01-01T00:00:00Z", 1000),
            ModificationRecord::with_timestamp("Change 2", "2024-01-02T00:00:00Z", 2000),
            ModificationRecord::with_timestamp("Change 3", "2024-01-03T00:00:00Z", 3000),
        ];
        
        let versioned = VersionedModel::with_history(model, history);
        
        let in_range = versioned.modifications_in_range(1500, 2500);
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].summary, "Change 2");
    }

    #[test]
    fn test_versioned_model_history_summary() {
        let model = SemanticModel::new("test-model").with_version("2.0.0");
        let history = vec![
            ModificationRecord::with_timestamp("Change 1", "2024-01-01T00:00:00Z", 1000)
                .with_changes(vec![
                    ModelChange::new(ModelChangeType::EntityAdded, "e1", "Added", false),
                ]),
            ModificationRecord::with_timestamp("Change 2", "2024-01-02T00:00:00Z", 2000)
                .with_changes(vec![
                    ModelChange::new(ModelChangeType::EntityRemoved, "e2", "Removed", true),
                ]),
        ];
        
        let versioned = VersionedModel::with_history(model, history);
        let summary = versioned.history_summary();
        
        assert_eq!(summary.current_version, "2.0.0");
        assert_eq!(summary.total_modifications, 2);
        assert_eq!(summary.breaking_changes, 1);
        assert_eq!(summary.first_modification, Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(summary.last_modification, Some("2024-01-02T00:00:00Z".to_string()));
    }

    #[test]
    fn test_versioned_model_clear_history() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        versioned.record_modification("Change 1", None);
        versioned.record_modification("Change 2", None);
        assert_eq!(versioned.modification_history().len(), 2);
        
        versioned.clear_history();
        assert!(versioned.modification_history().is_empty());
    }

    #[test]
    fn test_versioned_model_compare_with() {
        let model1 = SemanticModel::new("test-model")
            .with_version("1.0.0")
            .add_entity(SemanticEntity::new("entity1"));
        let model2 = SemanticModel::new("test-model")
            .with_version("1.1.0")
            .add_entity(SemanticEntity::new("entity1"))
            .add_entity(SemanticEntity::new("entity2"));
        
        let versioned = VersionedModel::new(model1);
        let diff = versioned.compare_with(&model2);
        
        assert!(diff.has_changes());
        assert!(diff.changes.iter().any(|c| {
            c.change_type == ModelChangeType::EntityAdded && c.element_name == "entity2"
        }));
    }

    #[test]
    fn test_versioned_model_record_modification_with_changes() {
        let model = SemanticModel::new("test-model").with_version("1.0.0");
        let mut versioned = VersionedModel::new(model);
        
        let changes = vec![
            ModelChange::new(ModelChangeType::EntityAdded, "entity1", "Added entity1", false),
            ModelChange::new(ModelChangeType::MetricAdded, "metric1", "Added metric1", false),
        ];
        
        versioned.record_modification_with_changes(
            "Added entities and metrics",
            Some("Bulk addition"),
            changes,
        );
        
        let record = versioned.last_modification().unwrap();
        assert_eq!(record.summary, "Added entities and metrics");
        assert_eq!(record.changes.len(), 2);
    }

    // ========================================================================
    // Timestamp Tests
    // ========================================================================

    #[test]
    fn test_days_to_ymd() {
        // Unix epoch
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        
        // 2024-01-01 (19723 days since epoch - corrected)
        assert_eq!(days_to_ymd(19723), (2024, 1, 1));
        
        // 2000-02-29 (leap year) - 11016 days since epoch
        assert_eq!(days_to_ymd(11016), (2000, 2, 29));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(is_leap_year(2000));  // Divisible by 400
        assert!(is_leap_year(2024));  // Divisible by 4
        assert!(!is_leap_year(2023)); // Not divisible by 4
    }

    #[test]
    fn test_current_timestamp_format() {
        let (timestamp, unix_ts) = current_timestamp();
        
        // Check ISO 8601 format
        assert!(timestamp.contains('T'));
        assert!(timestamp.ends_with('Z'));
        assert_eq!(timestamp.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
        
        // Unix timestamp should be reasonable (after 2020)
        assert!(unix_ts > 1577836800); // 2020-01-01
    }
}
