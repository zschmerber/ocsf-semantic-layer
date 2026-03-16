//! OCSF schema types.
//!
//! This module contains the core OCSF schema data structures for representing
//! categories, event classes, objects, attributes, and observables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Requirement level for an OCSF attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Requirement {
    /// The attribute must be present.
    Required,
    /// The attribute should be present when available.
    Recommended,
    /// The attribute may be present.
    #[default]
    Optional,
}


/// An OCSF attribute definition.
///
/// Attributes are the building blocks of OCSF objects and event classes,
/// defining individual fields with their types, descriptions, and requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// The attribute name (e.g., "user_name", "ip_address").
    pub name: String,

    /// The OCSF type (e.g., "string_t", "integer_t", "timestamp_t").
    #[serde(rename = "type")]
    pub attr_type: String,

    /// Human-readable caption for the attribute.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the attribute.
    #[serde(default)]
    pub description: String,

    /// Whether the attribute is required, recommended, or optional.
    #[serde(default)]
    pub requirement: Requirement,

    /// Observable type_id if this attribute is an observable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observable: Option<u32>,

    /// Whether this attribute is an array.
    #[serde(default)]
    pub is_array: bool,

    /// Reference to an object type if this attribute is an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,

    /// Enumeration values if this is an enum type.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub enum_values: HashMap<String, EnumValue>,

    /// Default value for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}


/// An enumeration value for enum-type attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumValue {
    /// The numeric value of the enum.
    pub value: i64,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Description of this enum value.
    #[serde(default)]
    pub description: String,
}

/// A reference to an attribute within an event class or object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeRef {
    /// The attribute name.
    pub name: String,

    /// Override requirement level for this context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement: Option<Requirement>,

    /// Override description for this context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Group this attribute belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// An OCSF object definition.
///
/// Objects are reusable structures that can be embedded in event classes
/// or other objects (e.g., User, Device, Process).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OCSFObject {
    /// The object name (e.g., "user", "device", "process").
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the object.
    #[serde(default)]
    pub description: String,

    /// Attributes contained in this object.
    #[serde(default)]
    pub attributes: HashMap<String, Attribute>,

    /// Parent object if this extends another object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Observable definitions for this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observables: Vec<ObservableDefinition>,
}


/// The type of observable definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservableDefinitionType {
    /// Observable defined by its type_id.
    ByType,
    /// Observable defined by an attribute name.
    ByAttribute,
    /// Observable defined by an object type.
    ByObject,
    /// Observable defined by an event class.
    ByEventClass,
    /// Observable defined by an attribute path.
    ByPath,
}

/// An OCSF observable definition.
///
/// Observables are security-relevant data elements that can be extracted
/// from events for threat intelligence matching and analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableDefinition {
    /// The observable type_id (e.g., 2 for IP addresses, 5 for emails).
    pub type_id: u32,

    /// Human-readable type name (e.g., "IP Address", "Email Address").
    pub type_name: String,

    /// How this observable is defined.
    pub definition_type: ObservableDefinitionType,

    /// The attribute path for path-based observables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,

    /// The source event class for event-class-based observables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_event_class: Option<u32>,

    /// Description of this observable.
    #[serde(default)]
    pub description: String,
}

impl ObservableDefinition {
    /// Creates a new observable definition by type.
    pub fn by_type(type_id: u32, type_name: impl Into<String>) -> Self {
        Self {
            type_id,
            type_name: type_name.into(),
            definition_type: ObservableDefinitionType::ByType,
            source_path: None,
            source_event_class: None,
            description: String::new(),
        }
    }

    /// Creates a new observable definition by attribute.
    pub fn by_attribute(type_id: u32, type_name: impl Into<String>) -> Self {
        Self {
            type_id,
            type_name: type_name.into(),
            definition_type: ObservableDefinitionType::ByAttribute,
            source_path: None,
            source_event_class: None,
            description: String::new(),
        }
    }

    /// Creates a new observable definition by path.
    pub fn by_path(type_id: u32, type_name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            type_id,
            type_name: type_name.into(),
            definition_type: ObservableDefinitionType::ByPath,
            source_path: Some(path.into()),
            source_event_class: None,
            description: String::new(),
        }
    }

    /// Creates a new observable definition by event class.
    pub fn by_event_class(type_id: u32, type_name: impl Into<String>, class_uid: u32) -> Self {
        Self {
            type_id,
            type_name: type_name.into(),
            definition_type: ObservableDefinitionType::ByEventClass,
            source_path: None,
            source_event_class: Some(class_uid),
            description: String::new(),
        }
    }

    /// Creates a new observable definition by object.
    pub fn by_object(type_id: u32, type_name: impl Into<String>) -> Self {
        Self {
            type_id,
            type_name: type_name.into(),
            definition_type: ObservableDefinitionType::ByObject,
            source_path: None,
            source_event_class: None,
            description: String::new(),
        }
    }

    /// Sets the description for this observable.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}


/// An OCSF event class definition.
///
/// Event classes are templates that define the structure of security events
/// (e.g., Authentication, Network Activity, File Activity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventClass {
    /// The unique class identifier (class_uid).
    pub class_uid: u32,

    /// The category this event class belongs to.
    pub category_uid: u32,

    /// The event class name (e.g., "authentication", "network_activity").
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the event class.
    #[serde(default)]
    pub description: String,

    /// Attribute references for this event class.
    #[serde(default)]
    pub attributes: HashMap<String, AttributeRef>,

    /// Observable definitions for this event class.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observables: Vec<ObservableDefinition>,

    /// Parent event class if this extends another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Profiles applied to this event class.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
}

/// An OCSF category definition.
///
/// Categories group related event classes together
/// (e.g., System Activity, Identity & Access Management).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    /// The unique category identifier (category_uid).
    pub uid: u32,

    /// The category name (e.g., "system", "iam").
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the category.
    #[serde(default)]
    pub description: String,

    /// Event class UIDs belonging to this category.
    #[serde(default)]
    pub event_classes: Vec<u32>,
}

/// The complete OCSF schema representation.
///
/// This is the normalized internal representation of the OCSF schema
/// suitable for semantic mapping and analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OCSFSchema {
    /// The OCSF schema version.
    pub version: String,

    /// Categories indexed by their UID.
    #[serde(default)]
    pub categories: HashMap<u32, Category>,

    /// Event classes indexed by their class_uid.
    #[serde(default)]
    pub event_classes: HashMap<u32, EventClass>,

    /// Objects indexed by their name.
    #[serde(default)]
    pub objects: HashMap<String, OCSFObject>,

    /// Base attributes indexed by their name.
    #[serde(default)]
    pub attributes: HashMap<String, Attribute>,

    /// All observable definitions extracted from the schema.
    #[serde(default)]
    pub observables: Vec<ObservableDefinition>,
}

impl Default for OCSFSchema {
    fn default() -> Self {
        Self::new("0.0.0")
    }
}

impl OCSFSchema {
    /// Creates a new empty schema with the given version.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            categories: HashMap::new(),
            event_classes: HashMap::new(),
            objects: HashMap::new(),
            attributes: HashMap::new(),
            observables: Vec::new(),
        }
    }

    /// Adds a category to the schema.
    pub fn add_category(&mut self, category: Category) {
        self.categories.insert(category.uid, category);
    }

    /// Adds an event class to the schema.
    pub fn add_event_class(&mut self, event_class: EventClass) {
        // Also update the category's event_classes list
        if let Some(category) = self.categories.get_mut(&event_class.category_uid) {
            if !category.event_classes.contains(&event_class.class_uid) {
                category.event_classes.push(event_class.class_uid);
            }
        }
        self.event_classes.insert(event_class.class_uid, event_class);
    }

    /// Adds an object to the schema.
    pub fn add_object(&mut self, object: OCSFObject) {
        self.objects.insert(object.name.clone(), object);
    }

    /// Adds a base attribute to the schema.
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.insert(attribute.name.clone(), attribute);
    }

    /// Adds an observable definition to the schema.
    pub fn add_observable(&mut self, observable: ObservableDefinition) {
        self.observables.push(observable);
    }

    /// Gets a category by UID.
    pub fn get_category(&self, uid: u32) -> Option<&Category> {
        self.categories.get(&uid)
    }

    /// Gets an event class by class_uid.
    pub fn get_event_class(&self, class_uid: u32) -> Option<&EventClass> {
        self.event_classes.get(&class_uid)
    }

    /// Gets an object by name.
    pub fn get_object(&self, name: &str) -> Option<&OCSFObject> {
        self.objects.get(name)
    }

    /// Gets a base attribute by name.
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }

    /// Returns all categories.
    pub fn all_categories(&self) -> impl Iterator<Item = &Category> {
        self.categories.values()
    }

    /// Returns all event classes.
    pub fn all_event_classes(&self) -> impl Iterator<Item = &EventClass> {
        self.event_classes.values()
    }

    /// Returns all objects.
    pub fn all_objects(&self) -> impl Iterator<Item = &OCSFObject> {
        self.objects.values()
    }

    /// Returns all base attributes.
    pub fn all_attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.values()
    }

    /// Returns all observable definitions.
    pub fn all_observables(&self) -> &[ObservableDefinition] {
        &self.observables
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_serialization() {
        let attr = Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "The name of the user".to_string(),
            requirement: Requirement::Required,
            observable: Some(10),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        };

        let json = serde_json::to_string(&attr).unwrap();
        let deserialized: Attribute = serde_json::from_str(&json).unwrap();
        assert_eq!(attr, deserialized);
    }

    #[test]
    fn test_observable_definition_types() {
        let by_type = ObservableDefinition::by_type(2, "IP Address");
        assert_eq!(by_type.definition_type, ObservableDefinitionType::ByType);

        let by_path = ObservableDefinition::by_path(2, "IP Address", "src_endpoint.ip");
        assert_eq!(by_path.definition_type, ObservableDefinitionType::ByPath);
        assert_eq!(by_path.source_path, Some("src_endpoint.ip".to_string()));

        let by_event_class = ObservableDefinition::by_event_class(5, "Email", 3002);
        assert_eq!(by_event_class.definition_type, ObservableDefinitionType::ByEventClass);
        assert_eq!(by_event_class.source_event_class, Some(3002));
    }

    #[test]
    fn test_category_serialization() {
        let category = Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "Identity and access management events".to_string(),
            event_classes: vec![3001, 3002, 3003],
        };

        let json = serde_json::to_string(&category).unwrap();
        let deserialized: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }

    #[test]
    fn test_event_class_serialization() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "user".to_string(),
            AttributeRef {
                name: "user".to_string(),
                requirement: Some(Requirement::Required),
                description: None,
                group: Some("primary".to_string()),
            },
        );

        let event_class = EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Authentication events".to_string(),
            attributes,
            observables: vec![ObservableDefinition::by_type(10, "User Name")],
            extends: Some("base_event".to_string()),
            profiles: vec!["security_controls".to_string()],
        };

        let json = serde_json::to_string(&event_class).unwrap();
        let deserialized: EventClass = serde_json::from_str(&json).unwrap();
        assert_eq!(event_class, deserialized);
    }

    #[test]
    fn test_ocsf_object_serialization() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "name".to_string(),
            Attribute {
                name: "name".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Name".to_string(),
                description: "The user name".to_string(),
                requirement: Requirement::Required,
                observable: Some(10),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );

        let object = OCSFObject {
            name: "user".to_string(),
            caption: "User".to_string(),
            description: "The user object".to_string(),
            attributes,
            extends: None,
            observables: vec![],
        };

        let json = serde_json::to_string(&object).unwrap();
        let deserialized: OCSFObject = serde_json::from_str(&json).unwrap();
        assert_eq!(object, deserialized);
    }

    #[test]
    fn test_ocsf_schema_operations() {
        let mut schema = OCSFSchema::new("1.4.0");

        // Add a category
        let category = Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        };
        schema.add_category(category);

        // Add an event class
        let event_class = EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Auth events".to_string(),
            attributes: HashMap::new(),
            observables: vec![],
            extends: None,
            profiles: vec![],
        };
        schema.add_event_class(event_class);

        // Verify the category was updated
        let cat = schema.get_category(3).unwrap();
        assert!(cat.event_classes.contains(&3002));

        // Verify event class retrieval
        let ec = schema.get_event_class(3002).unwrap();
        assert_eq!(ec.name, "authentication");
    }

    #[test]
    fn test_schema_serialization_roundtrip() {
        let mut schema = OCSFSchema::new("1.4.0");

        schema.add_category(Category {
            uid: 1,
            name: "system".to_string(),
            caption: "System Activity".to_string(),
            description: "System events".to_string(),
            event_classes: vec![],
        });

        schema.add_observable(ObservableDefinition::by_type(2, "IP Address")
            .with_description("IP address observable"));

        let json = serde_json::to_string_pretty(&schema).unwrap();
        let deserialized: OCSFSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(schema.version, deserialized.version);
        assert_eq!(schema.categories.len(), deserialized.categories.len());
        assert_eq!(schema.observables.len(), deserialized.observables.len());
    }
}
