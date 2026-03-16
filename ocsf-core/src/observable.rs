//! Observable extraction and analysis.
//!
//! This module provides functionality to extract and catalog observables
//! from OCSF schema definitions, supporting extraction by type, attribute,
//! object, event class, and attribute path.

use std::collections::HashMap;

use crate::schema::{
    Attribute, OCSFObject, OCSFSchema, ObservableDefinition,
    ObservableDefinitionType,
};

// Re-export EventClass for use in tests
#[cfg(test)]
use crate::schema::EventClass;

/// A catalog of observables extracted from an OCSF schema.
#[derive(Debug, Clone, Default)]
pub struct ObservableCatalog {
    /// All observable definitions indexed by type_id.
    pub by_type_id: HashMap<u32, Vec<ObservableEntry>>,

    /// Observable type names indexed by type_id.
    pub type_names: HashMap<u32, String>,

    /// Observables defined by attribute (attribute name -> entries).
    pub by_attribute: HashMap<String, Vec<ObservableEntry>>,

    /// Observables defined by object (object name -> entries).
    pub by_object: HashMap<String, Vec<ObservableEntry>>,

    /// Observables defined by event class (class_uid -> entries).
    pub by_event_class: HashMap<u32, Vec<ObservableEntry>>,

    /// Observables defined by path (path -> entries).
    pub by_path: HashMap<String, Vec<ObservableEntry>>,

    /// All observable entries.
    pub all_entries: Vec<ObservableEntry>,
}

/// An entry in the observable catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservableEntry {
    /// The observable type_id.
    pub type_id: u32,

    /// The observable type name.
    pub type_name: String,

    /// How this observable was defined.
    pub definition_type: ObservableDefinitionType,

    /// Source of this observable (attribute name, object name, event class, or path).
    pub source: ObservableSource,

    /// Description of the observable.
    pub description: String,
}

/// The source of an observable definition.
#[derive(Debug, Clone, PartialEq)]
pub enum ObservableSource {
    /// Observable defined on a base attribute.
    Attribute { name: String },

    /// Observable defined on an object.
    Object { name: String },

    /// Observable defined on an event class.
    EventClass { class_uid: u32, name: String },

    /// Observable defined by an attribute path.
    Path { path: String, context: String },

    /// Observable defined by type only (no specific source).
    Type,
}

impl ObservableCatalog {
    /// Creates a new empty observable catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of observable entries.
    pub fn len(&self) -> usize {
        self.all_entries.len()
    }

    /// Returns true if the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.all_entries.is_empty()
    }

    /// Returns all unique type_ids in the catalog.
    pub fn type_ids(&self) -> impl Iterator<Item = &u32> {
        self.by_type_id.keys()
    }

    /// Returns the type name for a given type_id.
    pub fn get_type_name(&self, type_id: u32) -> Option<&str> {
        self.type_names.get(&type_id).map(|s| s.as_str())
    }

    /// Returns all entries for a given type_id.
    pub fn get_by_type_id(&self, type_id: u32) -> Option<&Vec<ObservableEntry>> {
        self.by_type_id.get(&type_id)
    }

    /// Returns all entries for a given attribute name.
    pub fn get_by_attribute(&self, name: &str) -> Option<&Vec<ObservableEntry>> {
        self.by_attribute.get(name)
    }

    /// Returns all entries for a given object name.
    pub fn get_by_object(&self, name: &str) -> Option<&Vec<ObservableEntry>> {
        self.by_object.get(name)
    }

    /// Returns all entries for a given event class.
    pub fn get_by_event_class(&self, class_uid: u32) -> Option<&Vec<ObservableEntry>> {
        self.by_event_class.get(&class_uid)
    }

    /// Returns all entries for a given path.
    pub fn get_by_path(&self, path: &str) -> Option<&Vec<ObservableEntry>> {
        self.by_path.get(path)
    }

    /// Adds an entry to the catalog.
    fn add_entry(&mut self, entry: ObservableEntry) {
        // Update type_names
        if let std::collections::hash_map::Entry::Vacant(e) = self.type_names.entry(entry.type_id) {
            e.insert(entry.type_name.clone());
        }

        // Add to by_type_id
        self.by_type_id
            .entry(entry.type_id)
            .or_default()
            .push(entry.clone());

        // Add to appropriate source index
        match &entry.source {
            ObservableSource::Attribute { name } => {
                self.by_attribute
                    .entry(name.clone())
                    .or_default()
                    .push(entry.clone());
            }
            ObservableSource::Object { name } => {
                self.by_object
                    .entry(name.clone())
                    .or_default()
                    .push(entry.clone());
            }
            ObservableSource::EventClass { class_uid, .. } => {
                self.by_event_class
                    .entry(*class_uid)
                    .or_default()
                    .push(entry.clone());
            }
            ObservableSource::Path { path, .. } => {
                self.by_path
                    .entry(path.clone())
                    .or_default()
                    .push(entry.clone());
            }
            ObservableSource::Type => {}
        }

        // Add to all_entries
        self.all_entries.push(entry);
    }
}

/// Observable extractor for building an observable catalog from an OCSF schema.
#[derive(Debug)]
pub struct ObservableExtractor<'a> {
    schema: &'a OCSFSchema,
}

impl<'a> ObservableExtractor<'a> {
    /// Creates a new observable extractor for the given schema.
    pub fn new(schema: &'a OCSFSchema) -> Self {
        Self { schema }
    }

    /// Extracts all observables from the schema and builds a catalog.
    pub fn extract(&self) -> ObservableCatalog {
        let mut catalog = ObservableCatalog::new();

        // Extract from base attributes
        self.extract_from_attributes(&mut catalog);

        // Extract from objects
        self.extract_from_objects(&mut catalog);

        // Extract from event classes
        self.extract_from_event_classes(&mut catalog);

        // Extract from schema-level observables
        self.extract_from_schema_observables(&mut catalog);

        catalog
    }

    /// Extracts observables from base attributes in the dictionary.
    fn extract_from_attributes(&self, catalog: &mut ObservableCatalog) {
        for (name, attr) in &self.schema.attributes {
            if let Some(type_id) = attr.observable {
                let entry = ObservableEntry {
                    type_id,
                    type_name: self.get_observable_type_name(type_id, attr),
                    definition_type: ObservableDefinitionType::ByAttribute,
                    source: ObservableSource::Attribute { name: name.clone() },
                    description: attr.description.clone(),
                };
                catalog.add_entry(entry);
            }
        }
    }

    /// Extracts observables from objects.
    fn extract_from_objects(&self, catalog: &mut ObservableCatalog) {
        for (obj_name, object) in &self.schema.objects {
            // Extract from object-level observables
            for obs in &object.observables {
                let entry = self.observable_def_to_entry(
                    obs,
                    ObservableSource::Object { name: obj_name.clone() },
                );
                catalog.add_entry(entry);
            }

            // Extract from object attributes
            self.extract_from_object_attributes(catalog, object);
        }
    }

    /// Extracts observables from object attributes.
    fn extract_from_object_attributes(&self, catalog: &mut ObservableCatalog, object: &OCSFObject) {
        for (attr_name, attr) in &object.attributes {
            if let Some(type_id) = attr.observable {
                let path = format!("{}.{}", object.name, attr_name);
                let entry = ObservableEntry {
                    type_id,
                    type_name: self.get_observable_type_name(type_id, attr),
                    definition_type: ObservableDefinitionType::ByPath,
                    source: ObservableSource::Path {
                        path,
                        context: format!("object:{}", object.name),
                    },
                    description: attr.description.clone(),
                };
                catalog.add_entry(entry);
            }
        }
    }

    /// Extracts observables from event classes.
    fn extract_from_event_classes(&self, catalog: &mut ObservableCatalog) {
        for (class_uid, event_class) in &self.schema.event_classes {
            // Extract from event class-level observables
            for obs in &event_class.observables {
                let source = match &obs.definition_type {
                    ObservableDefinitionType::ByPath => {
                        ObservableSource::Path {
                            path: obs.source_path.clone().unwrap_or_default(),
                            context: format!("event_class:{}", event_class.name),
                        }
                    }
                    _ => ObservableSource::EventClass {
                        class_uid: *class_uid,
                        name: event_class.name.clone(),
                    },
                };
                let entry = self.observable_def_to_entry(obs, source);
                catalog.add_entry(entry);
            }
        }
    }

    /// Extracts observables from schema-level observable definitions.
    fn extract_from_schema_observables(&self, catalog: &mut ObservableCatalog) {
        for obs in &self.schema.observables {
            let source = match &obs.definition_type {
                ObservableDefinitionType::ByType => ObservableSource::Type,
                ObservableDefinitionType::ByAttribute => {
                    ObservableSource::Attribute {
                        name: obs.source_path.clone().unwrap_or_default(),
                    }
                }
                ObservableDefinitionType::ByObject => {
                    ObservableSource::Object {
                        name: obs.source_path.clone().unwrap_or_default(),
                    }
                }
                ObservableDefinitionType::ByEventClass => {
                    ObservableSource::EventClass {
                        class_uid: obs.source_event_class.unwrap_or(0),
                        name: String::new(),
                    }
                }
                ObservableDefinitionType::ByPath => {
                    ObservableSource::Path {
                        path: obs.source_path.clone().unwrap_or_default(),
                        context: "schema".to_string(),
                    }
                }
            };
            let entry = self.observable_def_to_entry(obs, source);
            catalog.add_entry(entry);
        }
    }

    /// Converts an ObservableDefinition to an ObservableEntry.
    fn observable_def_to_entry(
        &self,
        obs: &ObservableDefinition,
        source: ObservableSource,
    ) -> ObservableEntry {
        ObservableEntry {
            type_id: obs.type_id,
            type_name: obs.type_name.clone(),
            definition_type: obs.definition_type,
            source,
            description: obs.description.clone(),
        }
    }

    /// Gets the observable type name, using the attribute caption as fallback.
    fn get_observable_type_name(&self, type_id: u32, attr: &Attribute) -> String {
        // Try to find a matching observable definition with a type name
        for obs in &self.schema.observables {
            if obs.type_id == type_id && !obs.type_name.is_empty() {
                return obs.type_name.clone();
            }
        }

        // Fall back to attribute caption or a generic name
        if !attr.caption.is_empty() {
            attr.caption.clone()
        } else {
            format!("Observable Type {}", type_id)
        }
    }
}

/// Extracts all observables from an OCSF schema.
///
/// This is a convenience function that creates an extractor and extracts
/// all observables in one call.
pub fn extract_observables(schema: &OCSFSchema) -> ObservableCatalog {
    ObservableExtractor::new(schema).extract()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AttributeRef, Category, Requirement};

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");

        // Add base attributes with observables
        schema.add_attribute(Attribute {
            name: "ip_address".to_string(),
            attr_type: "string_t".to_string(),
            caption: "IP Address".to_string(),
            description: "An IP address".to_string(),
            requirement: Requirement::Optional,
            observable: Some(2),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        schema.add_attribute(Attribute {
            name: "email_addr".to_string(),
            attr_type: "string_t".to_string(),
            caption: "Email Address".to_string(),
            description: "An email address".to_string(),
            requirement: Requirement::Optional,
            observable: Some(5),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        schema.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "A user name".to_string(),
            requirement: Requirement::Optional,
            observable: Some(10),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        // Add an object with observable attributes
        let mut user_attrs = HashMap::new();
        user_attrs.insert(
            "name".to_string(),
            Attribute {
                name: "name".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Name".to_string(),
                description: "User name".to_string(),
                requirement: Requirement::Required,
                observable: Some(10),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );
        user_attrs.insert(
            "email_addr".to_string(),
            Attribute {
                name: "email_addr".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Email".to_string(),
                description: "User email".to_string(),
                requirement: Requirement::Optional,
                observable: Some(5),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );

        schema.add_object(OCSFObject {
            name: "user".to_string(),
            caption: "User".to_string(),
            description: "User object".to_string(),
            attributes: user_attrs,
            extends: None,
            observables: vec![],
        });

        // Add a category
        schema.add_category(Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        });

        // Add an event class with observables
        let mut ec_attrs = HashMap::new();
        ec_attrs.insert(
            "user".to_string(),
            AttributeRef {
                name: "user".to_string(),
                requirement: Some(Requirement::Required),
                description: None,
                group: Some("primary".to_string()),
            },
        );

        schema.add_event_class(EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Authentication events".to_string(),
            attributes: ec_attrs,
            observables: vec![
                ObservableDefinition::by_path(10, "User Name", "actor.user.name"),
                ObservableDefinition::by_path(2, "IP Address", "src_endpoint.ip"),
            ],
            extends: None,
            profiles: vec![],
        });

        // Add schema-level observables
        schema.add_observable(
            ObservableDefinition::by_type(22, "Hostname")
                .with_description("A hostname observable"),
        );

        schema
    }

    #[test]
    fn test_extract_from_attributes() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Should have extracted observables from base attributes
        assert!(catalog.get_by_attribute("ip_address").is_some());
        assert!(catalog.get_by_attribute("email_addr").is_some());
        assert!(catalog.get_by_attribute("user_name").is_some());

        let ip_entries = catalog.get_by_attribute("ip_address").unwrap();
        assert_eq!(ip_entries.len(), 1);
        assert_eq!(ip_entries[0].type_id, 2);
    }

    #[test]
    fn test_extract_from_objects() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Should have extracted observables from object attributes
        let user_name_path = catalog.get_by_path("user.name");
        assert!(user_name_path.is_some());

        let user_email_path = catalog.get_by_path("user.email_addr");
        assert!(user_email_path.is_some());
    }

    #[test]
    fn test_extract_from_event_classes() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Should have extracted observables from event class
        let auth_path = catalog.get_by_path("actor.user.name");
        assert!(auth_path.is_some());

        let ip_path = catalog.get_by_path("src_endpoint.ip");
        assert!(ip_path.is_some());
    }

    #[test]
    fn test_extract_schema_observables() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Should have extracted schema-level observables
        let hostname_entries = catalog.get_by_type_id(22);
        assert!(hostname_entries.is_some());
        assert_eq!(hostname_entries.unwrap()[0].type_name, "Hostname");
    }

    #[test]
    fn test_type_id_index() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Check type_id indexing
        assert!(catalog.get_by_type_id(2).is_some()); // IP Address
        assert!(catalog.get_by_type_id(5).is_some()); // Email
        assert!(catalog.get_by_type_id(10).is_some()); // User Name
        assert!(catalog.get_by_type_id(22).is_some()); // Hostname

        // Type 2 (IP) should have multiple entries
        let ip_entries = catalog.get_by_type_id(2).unwrap();
        assert!(ip_entries.len() >= 2); // From attribute and event class path
    }

    #[test]
    fn test_type_names() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        assert_eq!(catalog.get_type_name(22), Some("Hostname"));
    }

    #[test]
    fn test_catalog_len() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Should have multiple entries
        assert!(!catalog.is_empty());
        assert!(catalog.len() > 0);
    }

    #[test]
    fn test_empty_schema() {
        let schema = OCSFSchema::new("1.0.0");
        let catalog = extract_observables(&schema);

        assert!(catalog.is_empty());
        assert_eq!(catalog.len(), 0);
    }

    #[test]
    fn test_definition_types() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);

        // Check that different definition types are captured
        let mut has_by_attribute = false;
        let mut has_by_path = false;
        let mut has_by_type = false;

        for entry in &catalog.all_entries {
            match entry.definition_type {
                ObservableDefinitionType::ByAttribute => has_by_attribute = true,
                ObservableDefinitionType::ByPath => has_by_path = true,
                ObservableDefinitionType::ByType => has_by_type = true,
                _ => {}
            }
        }

        assert!(has_by_attribute, "Should have ByAttribute observables");
        assert!(has_by_path, "Should have ByPath observables");
        assert!(has_by_type, "Should have ByType observables");
    }
}
