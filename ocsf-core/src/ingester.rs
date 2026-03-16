//! OCSF schema ingestion from various sources.
//!
//! This module provides functionality to load and parse OCSF schema definitions
//! from local files or remote sources (GitHub).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::github::{GitHubConfig, GitHubFetcher};
use crate::schema::{
    Attribute, AttributeRef, Category, EventClass, OCSFObject, OCSFSchema,
    ObservableDefinition, ObservableDefinitionType, Requirement,
};

/// Source configuration for loading OCSF schema.
#[derive(Debug, Clone)]
pub enum SchemaSource {
    /// Load from a local directory path.
    Local { path: String },
    /// Load from GitHub repository.
    GitHub { version: Option<String> },
    /// Load from a URL.
    Url { url: String },
}

/// Schema ingester for loading and parsing OCSF schema definitions.
#[derive(Debug)]
pub struct SchemaIngester {
    schema: OCSFSchema,
}

impl Default for SchemaIngester {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaIngester {
    /// Creates a new empty schema ingester.
    pub fn new() -> Self {
        Self {
            schema: OCSFSchema::default(),
        }
    }

    /// Returns the loaded schema.
    pub fn schema(&self) -> &OCSFSchema {
        &self.schema
    }

    /// Consumes the ingester and returns the loaded schema.
    pub fn into_schema(self) -> OCSFSchema {
        self.schema
    }

    /// Loads OCSF schema from GitHub.
    ///
    /// Fetches the schema from the ocsf/ocsf-schema repository,
    /// caching it locally for future use.
    ///
    /// # Arguments
    /// * `version` - Optional version tag (e.g., "v1.4.0"). Uses main branch if None.
    pub async fn load_from_github(&mut self, version: Option<&str>) -> Result<()> {
        let config = match version {
            Some(v) => GitHubConfig::with_version(v),
            None => GitHubConfig::default(),
        };

        let fetcher = GitHubFetcher::new(config);
        let schema_path = fetcher.fetch().await
            .context("Failed to fetch schema from GitHub")?;

        self.load_from_local(&schema_path)
    }

    /// Loads OCSF schema from GitHub with custom configuration.
    pub async fn load_from_github_with_config(&mut self, config: GitHubConfig) -> Result<()> {
        let fetcher = GitHubFetcher::new(config);
        let schema_path = fetcher.fetch().await
            .context("Failed to fetch schema from GitHub")?;

        self.load_from_local(&schema_path)
    }

    /// Loads OCSF schema from the specified source.
    pub async fn load(&mut self, source: SchemaSource) -> Result<()> {
        match source {
            SchemaSource::Local { path } => self.load_from_local(&path),
            SchemaSource::GitHub { version } => self.load_from_github(version.as_deref()).await,
            SchemaSource::Url { url: _ } => {
                anyhow::bail!("URL source not yet implemented")
            }
        }
    }

    /// Loads OCSF schema from a local directory.
    ///
    /// The directory should contain the OCSF schema structure with:
    /// - `version.json` - Schema version information
    /// - `categories/` - Category definitions
    /// - `events/` - Event class definitions  
    /// - `objects/` - Object definitions
    /// - `dictionary.json` - Base attribute definitions
    pub fn load_from_local(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let base_path = path.as_ref();

        // Load version
        self.load_version(base_path)?;

        // Load dictionary (base attributes)
        self.load_dictionary(base_path)?;

        // Load objects
        self.load_objects(base_path)?;

        // Load categories
        self.load_categories(base_path)?;

        // Load event classes
        self.load_event_classes(base_path)?;

        Ok(())
    }

    /// Loads the schema version from version.json.
    fn load_version(&mut self, base_path: &Path) -> Result<()> {
        let version_path = base_path.join("version.json");
        if version_path.exists() {
            let content = std::fs::read_to_string(&version_path)
                .with_context(|| format!("Failed to read {}", version_path.display()))?;
            let version_info: VersionInfo = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", version_path.display()))?;
            self.schema.version = version_info.version;
        }
        Ok(())
    }

    /// Loads base attributes from dictionary.json.
    fn load_dictionary(&mut self, base_path: &Path) -> Result<()> {
        let dict_path = base_path.join("dictionary.json");
        if dict_path.exists() {
            let content = std::fs::read_to_string(&dict_path)
                .with_context(|| format!("Failed to read {}", dict_path.display()))?;
            let dict: DictionaryFile = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", dict_path.display()))?;

            for (name, raw_attr) in dict.attributes {
                let attr = self.convert_raw_attribute(&name, raw_attr);
                self.schema.add_attribute(attr);
            }
        }
        Ok(())
    }

    /// Loads objects from the objects/ directory.
    fn load_objects(&mut self, base_path: &Path) -> Result<()> {
        let objects_path = base_path.join("objects");
        if objects_path.exists() && objects_path.is_dir() {
            self.load_objects_recursive(&objects_path)?;
        }
        Ok(())
    }

    /// Recursively loads objects from a directory.
    fn load_objects_recursive(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.load_objects_recursive(&path)?;
            } else if path.extension().is_some_and(|ext| ext == "json") {
                self.load_object_file(&path)?;
            }
        }
        Ok(())
    }

    /// Loads a single object definition file.
    fn load_object_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let raw_obj: RawObject = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        let object = self.convert_raw_object(raw_obj);
        self.schema.add_object(object);
        Ok(())
    }

    /// Loads categories from the categories/ directory.
    fn load_categories(&mut self, base_path: &Path) -> Result<()> {
        let categories_path = base_path.join("categories.json");
        if categories_path.exists() {
            let content = std::fs::read_to_string(&categories_path)
                .with_context(|| format!("Failed to read {}", categories_path.display()))?;
            let raw_categories: RawCategoriesFile = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", categories_path.display()))?;

            for (name, raw_cat) in raw_categories.attributes {
                let category = Category {
                    uid: raw_cat.uid,
                    name,
                    caption: raw_cat.caption.unwrap_or_default(),
                    description: raw_cat.description.unwrap_or_default(),
                    event_classes: vec![],
                };
                self.schema.add_category(category);
            }
        }
        Ok(())
    }

    /// Loads event classes from the events/ directory.
    fn load_event_classes(&mut self, base_path: &Path) -> Result<()> {
        let events_path = base_path.join("events");
        if events_path.exists() && events_path.is_dir() {
            self.load_event_classes_recursive(&events_path)?;
        }
        Ok(())
    }

    /// Recursively loads event classes from a directory.
    fn load_event_classes_recursive(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.load_event_classes_recursive(&path)?;
            } else if path.extension().is_some_and(|ext| ext == "json") {
                self.load_event_class_file(&path)?;
            }
        }
        Ok(())
    }

    /// Loads a single event class definition file.
    fn load_event_class_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let raw_ec: RawEventClass = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        let event_class = self.convert_raw_event_class(raw_ec);
        self.schema.add_event_class(event_class);
        Ok(())
    }

    /// Converts a raw attribute from the schema files to our internal representation.
    fn convert_raw_attribute(&self, name: &str, raw: RawAttribute) -> Attribute {
        Attribute {
            name: name.to_string(),
            attr_type: raw.attr_type.unwrap_or_else(|| "string_t".to_string()),
            caption: raw.caption.unwrap_or_default(),
            description: raw.description.unwrap_or_default(),
            requirement: raw.requirement.map(|r| parse_requirement(&r)).unwrap_or_default(),
            observable: raw.observable,
            is_array: raw.is_array.unwrap_or(false),
            object_type: raw.object_type,
            enum_values: HashMap::new(),
            default: raw.default,
        }
    }

    /// Converts a raw object from the schema files to our internal representation.
    fn convert_raw_object(&self, raw: RawObject) -> OCSFObject {
        let mut attributes = HashMap::new();
        if let Some(raw_attrs) = raw.attributes {
            for (name, raw_attr) in raw_attrs {
                let attr = self.convert_raw_attribute(&name, raw_attr);
                attributes.insert(name, attr);
            }
        }

        let observables = raw.observables
            .map(|obs| obs.into_iter().map(|o| self.convert_raw_observable(o)).collect())
            .unwrap_or_default();

        OCSFObject {
            name: raw.name,
            caption: raw.caption.unwrap_or_default(),
            description: raw.description.unwrap_or_default(),
            attributes,
            extends: raw.extends,
            observables,
        }
    }

    /// Converts a raw event class from the schema files to our internal representation.
    fn convert_raw_event_class(&self, raw: RawEventClass) -> EventClass {
        let mut attributes = HashMap::new();
        if let Some(raw_attrs) = raw.attributes {
            for (name, raw_attr_ref) in raw_attrs {
                let attr_ref = AttributeRef {
                    name: name.clone(),
                    requirement: raw_attr_ref.requirement.map(|r| parse_requirement(&r)),
                    description: raw_attr_ref.description,
                    group: raw_attr_ref.group,
                };
                attributes.insert(name, attr_ref);
            }
        }

        let observables = raw.observables
            .map(|obs| obs.into_iter().map(|o| self.convert_raw_observable(o)).collect())
            .unwrap_or_default();

        EventClass {
            class_uid: raw.uid,
            category_uid: raw.category.parse().unwrap_or(0),
            name: raw.name,
            caption: raw.caption.unwrap_or_default(),
            description: raw.description.unwrap_or_default(),
            attributes,
            observables,
            extends: raw.extends,
            profiles: raw.profiles.unwrap_or_default(),
        }
    }

    /// Converts a raw observable definition to our internal representation.
    fn convert_raw_observable(&self, raw: RawObservable) -> ObservableDefinition {
        let definition_type = match raw.definition_type.as_deref() {
            Some("by_type") => ObservableDefinitionType::ByType,
            Some("by_attribute") => ObservableDefinitionType::ByAttribute,
            Some("by_object") => ObservableDefinitionType::ByObject,
            Some("by_event_class") => ObservableDefinitionType::ByEventClass,
            Some("by_path") => ObservableDefinitionType::ByPath,
            _ => ObservableDefinitionType::ByType,
        };

        ObservableDefinition {
            type_id: raw.type_id,
            type_name: raw.type_name.unwrap_or_default(),
            definition_type,
            source_path: raw.path,
            source_event_class: raw.event_class,
            description: raw.description.unwrap_or_default(),
        }
    }
}

/// Parses a requirement string to the Requirement enum.
fn parse_requirement(s: &str) -> Requirement {
    match s.to_lowercase().as_str() {
        "required" => Requirement::Required,
        "recommended" => Requirement::Recommended,
        _ => Requirement::Optional,
    }
}

// ============================================================================
// Raw schema file structures (for parsing OCSF JSON files)
// ============================================================================

#[derive(Debug, Deserialize)]
struct VersionInfo {
    version: String,
}

#[derive(Debug, Deserialize)]
struct DictionaryFile {
    #[serde(default)]
    attributes: HashMap<String, RawAttribute>,
}

#[derive(Debug, Deserialize)]
struct RawAttribute {
    #[serde(rename = "type")]
    attr_type: Option<String>,
    caption: Option<String>,
    description: Option<String>,
    requirement: Option<String>,
    observable: Option<u32>,
    is_array: Option<bool>,
    #[serde(rename = "object_type")]
    object_type: Option<String>,
    default: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RawObject {
    name: String,
    caption: Option<String>,
    description: Option<String>,
    extends: Option<String>,
    #[serde(default)]
    attributes: Option<HashMap<String, RawAttribute>>,
    observables: Option<Vec<RawObservable>>,
}

#[derive(Debug, Deserialize)]
struct RawCategoriesFile {
    #[serde(default)]
    attributes: HashMap<String, RawCategory>,
}

#[derive(Debug, Deserialize)]
struct RawCategory {
    uid: u32,
    caption: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawEventClass {
    uid: u32,
    name: String,
    category: String,
    caption: Option<String>,
    description: Option<String>,
    extends: Option<String>,
    #[serde(default)]
    attributes: Option<HashMap<String, RawAttributeRef>>,
    observables: Option<Vec<RawObservable>>,
    profiles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RawAttributeRef {
    requirement: Option<String>,
    description: Option<String>,
    group: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawObservable {
    type_id: u32,
    type_name: Option<String>,
    #[serde(rename = "type")]
    definition_type: Option<String>,
    path: Option<String>,
    event_class: Option<u32>,
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_schema_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create version.json
        fs::write(
            base.join("version.json"),
            r#"{"version": "1.4.0"}"#,
        ).unwrap();

        // Create dictionary.json
        fs::write(
            base.join("dictionary.json"),
            r#"{
                "attributes": {
                    "user_name": {
                        "type": "string_t",
                        "caption": "User Name",
                        "description": "The name of the user",
                        "requirement": "required",
                        "observable": 10
                    },
                    "ip_address": {
                        "type": "string_t",
                        "caption": "IP Address",
                        "description": "An IP address",
                        "observable": 2
                    }
                }
            }"#,
        ).unwrap();

        // Create categories.json
        fs::write(
            base.join("categories.json"),
            r#"{
                "attributes": {
                    "iam": {
                        "uid": 3,
                        "caption": "Identity & Access Management",
                        "description": "IAM events"
                    },
                    "system": {
                        "uid": 1,
                        "caption": "System Activity",
                        "description": "System events"
                    }
                }
            }"#,
        ).unwrap();

        // Create objects directory
        let objects_dir = base.join("objects");
        fs::create_dir(&objects_dir).unwrap();

        fs::write(
            objects_dir.join("user.json"),
            r#"{
                "name": "user",
                "caption": "User",
                "description": "The user object",
                "attributes": {
                    "name": {
                        "type": "string_t",
                        "caption": "Name",
                        "requirement": "required"
                    },
                    "email_addr": {
                        "type": "string_t",
                        "caption": "Email Address",
                        "observable": 5
                    }
                }
            }"#,
        ).unwrap();

        // Create events directory
        let events_dir = base.join("events");
        fs::create_dir(&events_dir).unwrap();

        let iam_dir = events_dir.join("iam");
        fs::create_dir(&iam_dir).unwrap();

        fs::write(
            iam_dir.join("authentication.json"),
            r#"{
                "uid": 3002,
                "name": "authentication",
                "category": "3",
                "caption": "Authentication",
                "description": "Authentication events",
                "extends": "base_event",
                "attributes": {
                    "user": {
                        "requirement": "required",
                        "group": "primary"
                    },
                    "src_endpoint": {
                        "requirement": "recommended"
                    }
                },
                "observables": [
                    {
                        "type_id": 10,
                        "type_name": "User Name",
                        "type": "by_path",
                        "path": "actor.user.name"
                    }
                ],
                "profiles": ["security_controls"]
            }"#,
        ).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_from_local() {
        let temp_dir = create_test_schema_dir();
        let mut ingester = SchemaIngester::new();

        ingester.load_from_local(temp_dir.path()).unwrap();
        let schema = ingester.schema();

        // Check version
        assert_eq!(schema.version, "1.4.0");

        // Check attributes
        assert!(schema.attributes.contains_key("user_name"));
        assert!(schema.attributes.contains_key("ip_address"));
        let user_name_attr = schema.get_attribute("user_name").unwrap();
        assert_eq!(user_name_attr.observable, Some(10));
        assert_eq!(user_name_attr.requirement, Requirement::Required);

        // Check categories
        assert_eq!(schema.categories.len(), 2);
        let iam_cat = schema.get_category(3).unwrap();
        assert_eq!(iam_cat.name, "iam");
        assert_eq!(iam_cat.caption, "Identity & Access Management");

        // Check objects
        assert!(schema.objects.contains_key("user"));
        let user_obj = schema.get_object("user").unwrap();
        assert_eq!(user_obj.caption, "User");
        assert!(user_obj.attributes.contains_key("name"));
        assert!(user_obj.attributes.contains_key("email_addr"));

        // Check event classes
        let auth_ec = schema.get_event_class(3002).unwrap();
        assert_eq!(auth_ec.name, "authentication");
        assert_eq!(auth_ec.category_uid, 3);
        assert!(auth_ec.attributes.contains_key("user"));
        assert_eq!(auth_ec.observables.len(), 1);
        assert_eq!(auth_ec.observables[0].type_id, 10);
        assert_eq!(auth_ec.observables[0].source_path, Some("actor.user.name".to_string()));
    }

    #[test]
    fn test_load_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut ingester = SchemaIngester::new();

        // Should not fail on empty directory
        ingester.load_from_local(temp_dir.path()).unwrap();
        let schema = ingester.schema();

        assert_eq!(schema.version, "0.0.0");
        assert!(schema.categories.is_empty());
        assert!(schema.event_classes.is_empty());
    }

    #[test]
    fn test_into_schema() {
        let temp_dir = create_test_schema_dir();
        let mut ingester = SchemaIngester::new();
        ingester.load_from_local(temp_dir.path()).unwrap();

        let schema = ingester.into_schema();
        assert_eq!(schema.version, "1.4.0");
    }
}
