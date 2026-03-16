//! Schema version detection and comparison.
//!
//! This module provides functionality to parse OCSF schema version strings
//! and generate diffs between schema versions.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::schema::OCSFSchema;

/// Error type for version parsing.
#[derive(Debug, Error)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Invalid version component: {0}")]
    InvalidComponent(String),
}

/// A parsed OCSF schema version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Patch version number.
    pub patch: u32,
    /// Optional pre-release identifier (e.g., "rc1", "beta").
    pub prerelease: Option<String>,
}

impl SchemaVersion {
    /// Creates a new schema version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }

    /// Creates a new schema version with a pre-release identifier.
    pub fn with_prerelease(major: u32, minor: u32, patch: u32, prerelease: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: Some(prerelease.into()),
        }
    }

    /// Parses a version string.
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        s.parse()
    }

    /// Returns true if this is a pre-release version.
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    /// Returns the version as a tuple for comparison.
    pub fn as_tuple(&self) -> (u32, u32, u32) {
        (self.major, self.minor, self.patch)
    }
}

impl FromStr for SchemaVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove leading 'v' if present
        let s = s.strip_prefix('v').unwrap_or(s);

        // Split on '-' to separate version from prerelease
        let (version_part, prerelease) = match s.split_once('-') {
            Some((v, p)) => (v, Some(p.to_string())),
            None => (s, None),
        };

        // Parse version components
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidComponent(parts[0].to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidComponent(parts[1].to_string()))?;
        let patch = if parts.len() > 2 {
            parts[2]
                .parse()
                .map_err(|_| VersionError::InvalidComponent(parts[2].to_string()))?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        match self.as_tuple().cmp(&other.as_tuple()) {
            Ordering::Equal => {
                // Pre-release versions are less than release versions
                match (&self.prerelease, &other.prerelease) {
                    (None, None) => Ordering::Equal,
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(a), Some(b)) => a.cmp(b),
                }
            }
            other => other,
        }
    }
}

/// The type of change in a schema diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// A new element was added.
    Added,
    /// An existing element was removed.
    Removed,
    /// An existing element was modified.
    Modified,
}

/// A single change in a schema diff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaChange {
    /// The type of change.
    pub change_type: ChangeType,
    /// The element type (category, event_class, object, attribute).
    pub element_type: String,
    /// The element identifier (name or UID).
    pub element_id: String,
    /// Description of the change.
    pub description: String,
}

impl SchemaChange {
    /// Creates a new schema change.
    pub fn new(
        change_type: ChangeType,
        element_type: impl Into<String>,
        element_id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            change_type,
            element_type: element_type.into(),
            element_id: element_id.into(),
            description: description.into(),
        }
    }

    /// Creates an "added" change.
    pub fn added(element_type: impl Into<String>, element_id: impl Into<String>) -> Self {
        let element_type = element_type.into();
        let element_id = element_id.into();
        Self::new(
            ChangeType::Added,
            element_type.clone(),
            element_id.clone(),
            format!("Added {} '{}'", element_type, element_id),
        )
    }

    /// Creates a "removed" change.
    pub fn removed(element_type: impl Into<String>, element_id: impl Into<String>) -> Self {
        let element_type = element_type.into();
        let element_id = element_id.into();
        Self::new(
            ChangeType::Removed,
            element_type.clone(),
            element_id.clone(),
            format!("Removed {} '{}'", element_type, element_id),
        )
    }

    /// Creates a "modified" change.
    pub fn modified(
        element_type: impl Into<String>,
        element_id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(
            ChangeType::Modified,
            element_type,
            element_id,
            description,
        )
    }
}

/// A diff between two schema versions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaDiff {
    /// The source (older) version.
    pub from_version: String,
    /// The target (newer) version.
    pub to_version: String,
    /// List of changes.
    pub changes: Vec<SchemaChange>,
}

impl SchemaDiff {
    /// Creates a new empty schema diff.
    pub fn new(from_version: impl Into<String>, to_version: impl Into<String>) -> Self {
        Self {
            from_version: from_version.into(),
            to_version: to_version.into(),
            changes: Vec::new(),
        }
    }

    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns the number of changes.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Adds a change to the diff.
    pub fn add_change(&mut self, change: SchemaChange) {
        self.changes.push(change);
    }

    /// Returns all added elements.
    pub fn added(&self) -> impl Iterator<Item = &SchemaChange> {
        self.changes.iter().filter(|c| c.change_type == ChangeType::Added)
    }

    /// Returns all removed elements.
    pub fn removed(&self) -> impl Iterator<Item = &SchemaChange> {
        self.changes.iter().filter(|c| c.change_type == ChangeType::Removed)
    }

    /// Returns all modified elements.
    pub fn modified(&self) -> impl Iterator<Item = &SchemaChange> {
        self.changes.iter().filter(|c| c.change_type == ChangeType::Modified)
    }

    /// Returns changes for a specific element type.
    pub fn for_element_type<'a>(&'a self, element_type: &'a str) -> impl Iterator<Item = &'a SchemaChange> {
        self.changes.iter().filter(move |c| c.element_type == element_type)
    }
}

/// Compares two OCSF schemas and generates a diff.
pub fn compare_schemas(from: &OCSFSchema, to: &OCSFSchema) -> SchemaDiff {
    let mut diff = SchemaDiff::new(&from.version, &to.version);

    // Compare categories
    compare_categories(from, to, &mut diff);

    // Compare event classes
    compare_event_classes(from, to, &mut diff);

    // Compare objects
    compare_objects(from, to, &mut diff);

    // Compare base attributes
    compare_attributes(from, to, &mut diff);

    diff
}

/// Compares categories between two schemas.
fn compare_categories(from: &OCSFSchema, to: &OCSFSchema, diff: &mut SchemaDiff) {
    let from_uids: HashSet<_> = from.categories.keys().collect();
    let to_uids: HashSet<_> = to.categories.keys().collect();

    // Added categories
    for uid in to_uids.difference(&from_uids) {
        if let Some(cat) = to.categories.get(*uid) {
            diff.add_change(SchemaChange::added("category", &cat.name));
        }
    }

    // Removed categories
    for uid in from_uids.difference(&to_uids) {
        if let Some(cat) = from.categories.get(*uid) {
            diff.add_change(SchemaChange::removed("category", &cat.name));
        }
    }

    // Modified categories
    for uid in from_uids.intersection(&to_uids) {
        let from_cat = from.categories.get(*uid).unwrap();
        let to_cat = to.categories.get(*uid).unwrap();

        if from_cat.caption != to_cat.caption || from_cat.description != to_cat.description {
            diff.add_change(SchemaChange::modified(
                "category",
                &to_cat.name,
                "Caption or description changed",
            ));
        }
    }
}

/// Compares event classes between two schemas.
fn compare_event_classes(from: &OCSFSchema, to: &OCSFSchema, diff: &mut SchemaDiff) {
    let from_uids: HashSet<_> = from.event_classes.keys().collect();
    let to_uids: HashSet<_> = to.event_classes.keys().collect();

    // Added event classes
    for uid in to_uids.difference(&from_uids) {
        if let Some(ec) = to.event_classes.get(*uid) {
            diff.add_change(SchemaChange::added("event_class", &ec.name));
        }
    }

    // Removed event classes
    for uid in from_uids.difference(&to_uids) {
        if let Some(ec) = from.event_classes.get(*uid) {
            diff.add_change(SchemaChange::removed("event_class", &ec.name));
        }
    }

    // Modified event classes
    for uid in from_uids.intersection(&to_uids) {
        let from_ec = from.event_classes.get(*uid).unwrap();
        let to_ec = to.event_classes.get(*uid).unwrap();

        // Check for attribute changes
        let from_attrs: HashSet<_> = from_ec.attributes.keys().collect();
        let to_attrs: HashSet<_> = to_ec.attributes.keys().collect();

        if from_attrs != to_attrs {
            diff.add_change(SchemaChange::modified(
                "event_class",
                &to_ec.name,
                "Attributes changed",
            ));
        } else if from_ec.caption != to_ec.caption || from_ec.description != to_ec.description {
            diff.add_change(SchemaChange::modified(
                "event_class",
                &to_ec.name,
                "Caption or description changed",
            ));
        }
    }
}

/// Compares objects between two schemas.
fn compare_objects(from: &OCSFSchema, to: &OCSFSchema, diff: &mut SchemaDiff) {
    let from_names: HashSet<_> = from.objects.keys().collect();
    let to_names: HashSet<_> = to.objects.keys().collect();

    // Added objects
    for name in to_names.difference(&from_names) {
        diff.add_change(SchemaChange::added("object", *name));
    }

    // Removed objects
    for name in from_names.difference(&to_names) {
        diff.add_change(SchemaChange::removed("object", *name));
    }

    // Modified objects
    for name in from_names.intersection(&to_names) {
        let from_obj = from.objects.get(*name).unwrap();
        let to_obj = to.objects.get(*name).unwrap();

        // Check for attribute changes
        let from_attrs: HashSet<_> = from_obj.attributes.keys().collect();
        let to_attrs: HashSet<_> = to_obj.attributes.keys().collect();

        if from_attrs != to_attrs {
            diff.add_change(SchemaChange::modified(
                "object",
                *name,
                "Attributes changed",
            ));
        }
    }
}

/// Compares base attributes between two schemas.
fn compare_attributes(from: &OCSFSchema, to: &OCSFSchema, diff: &mut SchemaDiff) {
    let from_names: HashSet<_> = from.attributes.keys().collect();
    let to_names: HashSet<_> = to.attributes.keys().collect();

    // Added attributes
    for name in to_names.difference(&from_names) {
        diff.add_change(SchemaChange::added("attribute", *name));
    }

    // Removed attributes
    for name in from_names.difference(&to_names) {
        diff.add_change(SchemaChange::removed("attribute", *name));
    }

    // Modified attributes
    for name in from_names.intersection(&to_names) {
        let from_attr = from.attributes.get(*name).unwrap();
        let to_attr = to.attributes.get(*name).unwrap();

        if from_attr.attr_type != to_attr.attr_type {
            diff.add_change(SchemaChange::modified(
                "attribute",
                *name,
                format!("Type changed from '{}' to '{}'", from_attr.attr_type, to_attr.attr_type),
            ));
        } else if from_attr.requirement != to_attr.requirement {
            diff.add_change(SchemaChange::modified(
                "attribute",
                *name,
                "Requirement level changed",
            ));
        }
    }
}

/// Detects the version of an OCSF schema.
pub fn detect_version(schema: &OCSFSchema) -> Result<SchemaVersion, VersionError> {
    SchemaVersion::parse(&schema.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Attribute, Category, EventClass, OCSFObject, Requirement};
    use std::collections::HashMap;

    #[test]
    fn test_version_parsing() {
        let v = SchemaVersion::parse("1.4.0").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 4);
        assert_eq!(v.patch, 0);
        assert!(v.prerelease.is_none());
    }

    #[test]
    fn test_version_parsing_with_v_prefix() {
        let v = SchemaVersion::parse("v1.4.0").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 4);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_parsing_with_prerelease() {
        let v = SchemaVersion::parse("1.4.0-rc1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 4);
        assert_eq!(v.patch, 0);
        assert_eq!(v.prerelease, Some("rc1".to_string()));
    }

    #[test]
    fn test_version_parsing_two_parts() {
        let v = SchemaVersion::parse("1.4").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 4);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SchemaVersion::parse("1.3.0").unwrap();
        let v2 = SchemaVersion::parse("1.4.0").unwrap();
        let v3 = SchemaVersion::parse("1.4.0-rc1").unwrap();
        let v4 = SchemaVersion::parse("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v3 < v2); // Pre-release is less than release
        assert!(v2 < v4);
    }

    #[test]
    fn test_version_display() {
        let v = SchemaVersion::new(1, 4, 0);
        assert_eq!(v.to_string(), "1.4.0");

        let v_pre = SchemaVersion::with_prerelease(1, 4, 0, "rc1");
        assert_eq!(v_pre.to_string(), "1.4.0-rc1");
    }

    #[test]
    fn test_schema_diff_empty() {
        let schema1 = OCSFSchema::new("1.0.0");
        let schema2 = OCSFSchema::new("1.0.0");

        let diff = compare_schemas(&schema1, &schema2);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_schema_diff_added_category() {
        let schema1 = OCSFSchema::new("1.0.0");
        let mut schema2 = OCSFSchema::new("1.1.0");

        schema2.add_category(Category {
            uid: 1,
            name: "system".to_string(),
            caption: "System Activity".to_string(),
            description: "System events".to_string(),
            event_classes: vec![],
        });

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff.added().count(), 1);

        let change = diff.added().next().unwrap();
        assert_eq!(change.element_type, "category");
        assert_eq!(change.element_id, "system");
    }

    #[test]
    fn test_schema_diff_removed_event_class() {
        let mut schema1 = OCSFSchema::new("1.0.0");
        let schema2 = OCSFSchema::new("1.1.0");

        schema1.add_event_class(EventClass {
            class_uid: 1001,
            category_uid: 1,
            name: "process_activity".to_string(),
            caption: "Process Activity".to_string(),
            description: "Process events".to_string(),
            attributes: HashMap::new(),
            observables: vec![],
            extends: None,
            profiles: vec![],
        });

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.removed().count(), 1);

        let change = diff.removed().next().unwrap();
        assert_eq!(change.element_type, "event_class");
        assert_eq!(change.element_id, "process_activity");
    }

    #[test]
    fn test_schema_diff_modified_attribute() {
        let mut schema1 = OCSFSchema::new("1.0.0");
        let mut schema2 = OCSFSchema::new("1.1.0");

        schema1.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "The user name".to_string(),
            requirement: Requirement::Optional,
            observable: None,
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        schema2.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "The user name".to_string(),
            requirement: Requirement::Required, // Changed
            observable: None,
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.modified().count(), 1);

        let change = diff.modified().next().unwrap();
        assert_eq!(change.element_type, "attribute");
        assert_eq!(change.element_id, "user_name");
    }

    #[test]
    fn test_schema_diff_added_object() {
        let schema1 = OCSFSchema::new("1.0.0");
        let mut schema2 = OCSFSchema::new("1.1.0");

        schema2.add_object(OCSFObject {
            name: "user".to_string(),
            caption: "User".to_string(),
            description: "User object".to_string(),
            attributes: HashMap::new(),
            extends: None,
            observables: vec![],
        });

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.for_element_type("object").count(), 1);
    }

    #[test]
    fn test_detect_version() {
        let schema = OCSFSchema::new("1.4.0");
        let version = detect_version(&schema).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 4);
        assert_eq!(version.patch, 0);
    }
}
