//! Compiled OCSF schema types.
//!
//! This module contains types for parsing the compiled OCSF schema JSON
//! produced by the ocsf-schema-compiler. The compiled schema is the
//! authoritative source for generating semantic layer entities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors during schema parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Missing required field: {field} in {context}")]
    MissingField { field: String, context: String },

    #[error("Invalid schema version: {0}")]
    InvalidVersion(String),

    #[error("Unsupported schema version: {version}, supported: {supported:?}")]
    UnsupportedVersion {
        version: String,
        supported: Vec<String>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Requirement level for an attribute in the compiled schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CompiledRequirement {
    /// The attribute must be present.
    Required,
    /// The attribute should be present when available.
    Recommended,
    /// The attribute may be present.
    #[default]
    Optional,
}

/// Enum value definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledEnumValue {
    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Description of this enum value.
    #[serde(default)]
    pub description: String,

    /// Deprecation info if present.
    #[serde(rename = "@deprecated", skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<DeprecationInfo>,
}

/// Deprecation information for schema elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeprecationInfo {
    /// Deprecation message.
    #[serde(default)]
    pub message: String,

    /// Version when deprecated.
    #[serde(default)]
    pub since: String,
}

/// Attribute definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledAttribute {
    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description.
    #[serde(default)]
    pub description: String,

    /// The OCSF type (e.g., "string_t", "integer_t", "object_t").
    #[serde(rename = "type")]
    pub attr_type: String,

    /// Human-readable type name.
    #[serde(default)]
    pub type_name: String,

    /// Requirement level.
    #[serde(default)]
    pub requirement: CompiledRequirement,

    /// Attribute group (primary, context, classification, occurrence).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    /// Object type name for object_t types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,

    /// Object name (human-readable) for object_t types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,

    /// Whether this is an array attribute.
    #[serde(default)]
    pub is_array: bool,

    /// Enum values if this is an enum type.
    #[serde(rename = "enum", default, skip_serializing_if = "HashMap::is_empty")]
    pub enum_values: HashMap<String, CompiledEnumValue>,

    /// Observable type_id if this attribute is an observable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observable: Option<u32>,

    /// Sibling attribute name (e.g., activity_id has sibling activity_name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sibling: Option<String>,

    /// Profiles this attribute belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<String>>,

    /// Deprecation info if present.
    #[serde(rename = "@deprecated", skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<DeprecationInfo>,
}

/// Category definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledCategory {
    /// Unique category identifier.
    pub uid: u32,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description.
    #[serde(default)]
    pub description: String,
}

/// Categories container in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CategoriesContainer {
    /// Category attributes indexed by name.
    #[serde(default)]
    pub attributes: HashMap<String, CompiledCategory>,

    /// Caption for the categories section.
    #[serde(default)]
    pub caption: String,

    /// Description for the categories section.
    #[serde(default)]
    pub description: String,

    /// Name of the categories section.
    #[serde(default)]
    pub name: String,
}

/// Class (event type) definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledClass {
    /// Unique class identifier.
    pub uid: u32,

    /// Class name.
    #[serde(default)]
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description.
    #[serde(default)]
    pub description: String,

    /// Category name this class belongs to.
    #[serde(default)]
    pub category: String,

    /// Parent class if this extends another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Attributes for this class.
    #[serde(default)]
    pub attributes: HashMap<String, CompiledAttribute>,

    /// Profiles applied to this class.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
}

/// Object definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledObject {
    /// Object name.
    #[serde(default)]
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description.
    #[serde(default)]
    pub description: String,

    /// Parent object if this extends another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Attributes for this object.
    #[serde(default)]
    pub attributes: HashMap<String, CompiledAttribute>,
}

/// Profile definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledProfile {
    /// Profile name.
    #[serde(default)]
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description.
    #[serde(default)]
    pub description: String,

    /// Attributes added by this profile.
    #[serde(default)]
    pub attributes: HashMap<String, CompiledAttribute>,
}

/// Extension definition in the compiled schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledExtension {
    /// Extension name.
    #[serde(default)]
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Unique extension identifier.
    #[serde(default)]
    pub uid: u32,

    /// Extension version.
    #[serde(default)]
    pub version: String,

    /// Whether this is a platform extension.
    #[serde(rename = "platform_extension?", default)]
    pub platform_extension: bool,
}

/// The complete compiled OCSF schema representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledSchema {
    /// Schema version.
    pub version: String,

    /// Categories container.
    #[serde(default)]
    pub categories: CategoriesContainer,

    /// Classes (event types) indexed by name.
    #[serde(default)]
    pub classes: HashMap<String, CompiledClass>,

    /// Objects indexed by name.
    #[serde(default)]
    pub objects: HashMap<String, CompiledObject>,

    /// Profiles indexed by name.
    #[serde(default)]
    pub profiles: HashMap<String, CompiledProfile>,

    /// Extensions indexed by name.
    #[serde(default)]
    pub extensions: HashMap<String, CompiledExtension>,
}

impl Default for CompiledSchema {
    fn default() -> Self {
        Self {
            version: String::new(),
            categories: CategoriesContainer {
                attributes: HashMap::new(),
                caption: String::new(),
                description: String::new(),
                name: String::new(),
            },
            classes: HashMap::new(),
            objects: HashMap::new(),
            profiles: HashMap::new(),
            extensions: HashMap::new(),
        }
    }
}

impl CompiledSchema {
    /// Parse a compiled schema from JSON string.
    pub fn parse(json: &str) -> Result<Self, ParseError> {
        let schema: CompiledSchema = serde_json::from_str(json)?;
        Ok(schema)
    }

    /// Parse a compiled schema from a file path.
    pub fn parse_file(path: &Path) -> Result<Self, ParseError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Get a category by name.
    pub fn get_category(&self, name: &str) -> Option<&CompiledCategory> {
        self.categories.attributes.get(name)
    }

    /// Get a class by name.
    pub fn get_class(&self, name: &str) -> Option<&CompiledClass> {
        self.classes.get(name)
    }

    /// Get an object by name.
    pub fn get_object(&self, name: &str) -> Option<&CompiledObject> {
        self.objects.get(name)
    }

    /// Get a profile by name.
    pub fn get_profile(&self, name: &str) -> Option<&CompiledProfile> {
        self.profiles.get(name)
    }

    /// Returns all categories.
    pub fn all_categories(&self) -> impl Iterator<Item = (&String, &CompiledCategory)> {
        self.categories.attributes.iter()
    }

    /// Returns all classes.
    pub fn all_classes(&self) -> impl Iterator<Item = (&String, &CompiledClass)> {
        self.classes.iter()
    }

    /// Returns all objects.
    pub fn all_objects(&self) -> impl Iterator<Item = (&String, &CompiledObject)> {
        self.objects.iter()
    }

    /// Returns all profiles.
    pub fn all_profiles(&self) -> impl Iterator<Item = (&String, &CompiledProfile)> {
        self.profiles.iter()
    }

    /// Get the category UID for a class by looking up its category name.
    pub fn get_category_uid_for_class(&self, class: &CompiledClass) -> Option<u32> {
        self.get_category(&class.category).map(|c| c.uid)
    }

    /// Count statistics about the schema.
    pub fn stats(&self) -> SchemaStats {
        SchemaStats {
            categories: self.categories.attributes.len(),
            classes: self.classes.len(),
            objects: self.objects.len(),
            profiles: self.profiles.len(),
            extensions: self.extensions.len(),
        }
    }
}

/// Statistics about a compiled schema.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaStats {
    pub categories: usize,
    pub classes: usize,
    pub objects: usize,
    pub profiles: usize,
    pub extensions: usize,
}

impl std::fmt::Display for SchemaStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} categories, {} classes, {} objects, {} profiles, {} extensions",
            self.categories, self.classes, self.objects, self.profiles, self.extensions
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiled_requirement_default() {
        let req: CompiledRequirement = Default::default();
        assert_eq!(req, CompiledRequirement::Optional);
    }

    #[test]
    fn test_compiled_attribute_deserialization() {
        let json = r#"{
            "caption": "Activity ID",
            "description": "The normalized identifier of the activity.",
            "type": "integer_t",
            "type_name": "Integer",
            "requirement": "required",
            "group": "classification",
            "sibling": "activity_name",
            "enum": {
                "0": {"caption": "Unknown", "description": "Unknown activity"},
                "1": {"caption": "Create", "description": "Create activity"}
            }
        }"#;

        let attr: CompiledAttribute = serde_json::from_str(json).unwrap();
        assert_eq!(attr.caption, "Activity ID");
        assert_eq!(attr.attr_type, "integer_t");
        assert_eq!(attr.requirement, CompiledRequirement::Required);
        assert_eq!(attr.group, Some("classification".to_string()));
        assert_eq!(attr.sibling, Some("activity_name".to_string()));
        assert_eq!(attr.enum_values.len(), 2);
    }

    #[test]
    fn test_compiled_attribute_object_type() {
        let json = r#"{
            "caption": "Actor",
            "description": "The actor object.",
            "type": "object_t",
            "object_type": "actor",
            "object_name": "Actor",
            "requirement": "recommended"
        }"#;

        let attr: CompiledAttribute = serde_json::from_str(json).unwrap();
        assert_eq!(attr.attr_type, "object_t");
        assert_eq!(attr.object_type, Some("actor".to_string()));
        assert_eq!(attr.object_name, Some("Actor".to_string()));
    }

    #[test]
    fn test_compiled_category_deserialization() {
        let json = r#"{
            "uid": 3,
            "caption": "Identity & Access Management",
            "description": "IAM events"
        }"#;

        let cat: CompiledCategory = serde_json::from_str(json).unwrap();
        assert_eq!(cat.uid, 3);
        assert_eq!(cat.caption, "Identity & Access Management");
    }

    #[test]
    fn test_compiled_class_deserialization() {
        let json = r#"{
            "uid": 3002,
            "name": "authentication",
            "caption": "Authentication",
            "description": "Authentication events",
            "category": "iam",
            "extends": "iam",
            "profiles": ["cloud", "host"],
            "attributes": {}
        }"#;

        let class: CompiledClass = serde_json::from_str(json).unwrap();
        assert_eq!(class.uid, 3002);
        assert_eq!(class.name, "authentication");
        assert_eq!(class.category, "iam");
        assert_eq!(class.profiles, vec!["cloud", "host"]);
    }

    #[test]
    fn test_compiled_object_deserialization() {
        let json = r#"{
            "name": "user",
            "caption": "User",
            "description": "The user object",
            "extends": "object",
            "attributes": {
                "name": {
                    "caption": "Name",
                    "description": "The user name",
                    "type": "string_t",
                    "type_name": "String",
                    "requirement": "recommended"
                }
            }
        }"#;

        let obj: CompiledObject = serde_json::from_str(json).unwrap();
        assert_eq!(obj.name, "user");
        assert_eq!(obj.caption, "User");
        assert_eq!(obj.extends, Some("object".to_string()));
        assert!(obj.attributes.contains_key("name"));
    }

    #[test]
    fn test_schema_stats() {
        let schema = CompiledSchema::default();
        let stats = schema.stats();
        assert_eq!(stats.categories, 0);
        assert_eq!(stats.classes, 0);
        assert_eq!(stats.objects, 0);
    }

    #[test]
    fn test_deprecation_info() {
        let json = r#"{
            "caption": "Old Field",
            "description": "Deprecated field",
            "type": "string_t",
            "@deprecated": {
                "message": "Use new_field instead",
                "since": "1.5.0"
            }
        }"#;

        let attr: CompiledAttribute = serde_json::from_str(json).unwrap();
        assert!(attr.deprecated.is_some());
        let dep = attr.deprecated.unwrap();
        assert_eq!(dep.message, "Use new_field instead");
        assert_eq!(dep.since, "1.5.0");
    }
}


    #[test]
    fn test_parse_real_schema() {
        // Test parsing the real compiled OCSF schema
        let schema_path = std::path::Path::new("../demo/schema/ocsf-compiled-v1.6.0.json");
        if schema_path.exists() {
            let schema = CompiledSchema::parse_file(schema_path).expect("Failed to parse schema");
            
            // Verify version
            assert_eq!(schema.version, "1.6.0");
            
            // Verify we have categories
            assert!(!schema.categories.attributes.is_empty());
            
            // Verify we have classes
            assert!(!schema.classes.is_empty());
            
            // Verify we have objects
            assert!(!schema.objects.is_empty());
            
            // Check for known categories
            assert!(schema.get_category("iam").is_some());
            assert!(schema.get_category("system").is_some());
            assert!(schema.get_category("network").is_some());
            
            // Check for known classes
            assert!(schema.get_class("authentication").is_some());
            assert!(schema.get_class("account_change").is_some());
            
            // Check for known objects
            assert!(schema.get_object("user").is_some());
            assert!(schema.get_object("device").is_some());
            assert!(schema.get_object("actor").is_some());
            
            // Print stats
            let stats = schema.stats();
            println!("Schema stats: {}", stats);
            assert!(stats.categories >= 8);
            assert!(stats.classes >= 80);
            assert!(stats.objects >= 160);
        }
    }
