//! Semantic entity definitions.
//!
//! This module contains the core semantic entity data structures for defining
//! business-level security concepts that map to OCSF event classes.

use serde::{Deserialize, Serialize};

/// Threat relevance metadata for security-focused attributes.
///
/// Provides context about how an attribute relates to threat detection
/// and maps to MITRE ATT&CK techniques.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ThreatRelevance {
    /// Security use cases this attribute supports (e.g., "DGA detection", "DNS tunneling").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub use_cases: Vec<String>,

    /// MITRE ATT&CK technique IDs (e.g., "T1071.004", "T1568.002").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mitre_techniques: Vec<String>,
}

impl ThreatRelevance {
    /// Creates a new empty threat relevance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds use cases.
    pub fn with_use_cases(mut self, use_cases: Vec<String>) -> Self {
        self.use_cases = use_cases;
        self
    }

    /// Adds MITRE ATT&CK techniques.
    pub fn with_mitre_techniques(mut self, techniques: Vec<String>) -> Self {
        self.mitre_techniques = techniques;
        self
    }

    /// Returns true if this threat relevance has any data.
    pub fn is_empty(&self) -> bool {
        self.use_cases.is_empty() && self.mitre_techniques.is_empty()
    }
}

/// Semantic type for entity attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SemanticType {
    /// String type.
    #[default]
    String,
    /// Integer type.
    Integer,
    /// Float/decimal type.
    Float,
    /// Boolean type.
    Boolean,
    /// Timestamp type.
    Timestamp,
    /// JSON/object type.
    Json,
    /// Array of another type.
    Array(Box<SemanticType>),
}


/// Mapping from semantic attribute to OCSF fields.
///
/// Supports simple field references, expressions for computed fields,
/// and join conditions for cross-class mappings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OCSFMapping {
    /// Simple field reference (e.g., "actor.user.name").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// Expression for computed fields (e.g., "COALESCE(src_endpoint.ip, src_endpoint.hostname)").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,

    /// Join condition for cross-class mappings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_condition: Option<String>,
}

impl OCSFMapping {
    /// Creates a new mapping from a simple field reference.
    pub fn from_field(field: impl Into<String>) -> Self {
        Self {
            field: Some(field.into()),
            expression: None,
            join_condition: None,
        }
    }

    /// Creates a new mapping from an expression.
    pub fn from_expression(expression: impl Into<String>) -> Self {
        Self {
            field: None,
            expression: Some(expression.into()),
            join_condition: None,
        }
    }

    /// Adds a join condition to the mapping.
    pub fn with_join_condition(mut self, condition: impl Into<String>) -> Self {
        self.join_condition = Some(condition.into());
        self
    }

    /// Returns true if this mapping has a field reference.
    pub fn has_field(&self) -> bool {
        self.field.is_some()
    }

    /// Returns true if this mapping has an expression.
    pub fn has_expression(&self) -> bool {
        self.expression.is_some()
    }

    /// Returns true if this mapping is empty (no field or expression).
    pub fn is_empty(&self) -> bool {
        self.field.is_none() && self.expression.is_none()
    }
}

/// A semantic attribute within an entity.
///
/// Attributes represent business-friendly fields that map to underlying OCSF fields.
/// Enhanced with LLM-friendly metadata for agentic SQL generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticAttribute {
    /// Business-friendly attribute name.
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the attribute.
    #[serde(default)]
    pub description: String,

    /// The semantic type of this attribute.
    #[serde(rename = "type", default)]
    pub attr_type: SemanticType,

    /// Mapping to OCSF fields.
    #[serde(default)]
    pub ocsf_mapping: OCSFMapping,

    /// Whether this attribute can be used as a dimension for slicing metrics.
    #[serde(default)]
    pub is_dimension: bool,

    /// Sample values for embedding generation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_values: Vec<String>,

    // ========================================
    // LLM-Friendly Metadata Fields
    // ========================================

    /// Alternative names/terms that LLMs might use to refer to this attribute.
    /// Enables synonym resolution for natural language queries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,

    /// Security-specific context explaining how this attribute is used in threat detection.
    /// Helps LLMs understand the security significance of the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_context: Option<String>,

    /// Regex pattern describing valid values for this attribute.
    /// Helps LLMs validate and generate appropriate filter conditions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_pattern: Option<String>,

    /// Whether this attribute represents an observable (IOC-relevant field).
    #[serde(default)]
    pub is_observable: bool,

    /// Threat relevance metadata including use cases and MITRE ATT&CK mappings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_relevance: Option<ThreatRelevance>,
}

impl SemanticAttribute {
    /// Creates a new semantic attribute with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            caption: String::new(),
            description: String::new(),
            attr_type: SemanticType::default(),
            ocsf_mapping: OCSFMapping::default(),
            is_dimension: false,
            sample_values: Vec::new(),
            synonyms: Vec::new(),
            security_context: None,
            value_pattern: None,
            is_observable: false,
            threat_relevance: None,
        }
    }

    /// Sets the caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the semantic type.
    pub fn with_type(mut self, attr_type: SemanticType) -> Self {
        self.attr_type = attr_type;
        self
    }

    /// Sets the OCSF mapping from a field reference.
    pub fn with_field_mapping(mut self, field: impl Into<String>) -> Self {
        self.ocsf_mapping = OCSFMapping::from_field(field);
        self
    }

    /// Sets the OCSF mapping from an expression.
    pub fn with_expression_mapping(mut self, expression: impl Into<String>) -> Self {
        self.ocsf_mapping = OCSFMapping::from_expression(expression);
        self
    }

    /// Sets the OCSF mapping.
    pub fn with_mapping(mut self, mapping: OCSFMapping) -> Self {
        self.ocsf_mapping = mapping;
        self
    }

    /// Marks this attribute as a dimension.
    pub fn as_dimension(mut self) -> Self {
        self.is_dimension = true;
        self
    }

    /// Adds sample values.
    pub fn with_sample_values(mut self, values: Vec<String>) -> Self {
        self.sample_values = values;
        self
    }

    /// Sets synonyms for LLM-friendly natural language queries.
    pub fn with_synonyms(mut self, synonyms: Vec<String>) -> Self {
        self.synonyms = synonyms;
        self
    }

    /// Sets security context explaining threat detection relevance.
    pub fn with_security_context(mut self, context: impl Into<String>) -> Self {
        self.security_context = Some(context.into());
        self
    }

    /// Sets the value pattern regex for validation.
    pub fn with_value_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.value_pattern = Some(pattern.into());
        self
    }

    /// Marks this attribute as an observable (IOC-relevant).
    pub fn as_observable(mut self) -> Self {
        self.is_observable = true;
        self
    }

    /// Sets threat relevance metadata.
    pub fn with_threat_relevance(mut self, relevance: ThreatRelevance) -> Self {
        self.threat_relevance = Some(relevance);
        self
    }
}

/// Cardinality of an entity relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum Cardinality {
    /// One-to-one relationship.
    OneToOne,
    /// One-to-many relationship.
    #[default]
    OneToMany,
    /// Many-to-one relationship.
    ManyToOne,
    /// Many-to-many relationship.
    ManyToMany,
}


/// Type of relationship between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum RelationshipType {
    /// Entity originated from another entity (e.g., DNS query from endpoint).
    OriginatedFrom,
    /// Entity targeted another entity (e.g., attack targeted a host).
    TargetedTo,
    /// Action performed by an entity (e.g., user performed authentication).
    PerformedBy,
    /// Entity contains another entity (e.g., network contains hosts).
    Contains,
    /// Entity is referenced in another entity (e.g., IOC referenced in event).
    #[default]
    ReferencedIn,
}


/// A relationship between semantic entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRelationship {
    /// Name of the relationship.
    pub name: String,

    /// Target entity name.
    pub target_entity: String,

    /// Type of relationship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship_type: Option<RelationshipType>,

    /// Cardinality of the relationship.
    #[serde(default)]
    pub cardinality: Cardinality,

    /// Join condition for the relationship.
    pub join_condition: String,

    /// Human-readable description of the relationship.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

impl EntityRelationship {
    /// Creates a new entity relationship.
    pub fn new(
        name: impl Into<String>,
        target_entity: impl Into<String>,
        join_condition: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            target_entity: target_entity.into(),
            relationship_type: None,
            cardinality: Cardinality::default(),
            join_condition: join_condition.into(),
            description: String::new(),
        }
    }

    /// Sets the relationship type.
    pub fn with_relationship_type(mut self, rel_type: RelationshipType) -> Self {
        self.relationship_type = Some(rel_type);
        self
    }

    /// Sets the cardinality.
    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Generates a SQL JOIN clause for this relationship.
    pub fn to_join_sql(&self, from_entity: &str) -> String {
        format!(
            "JOIN {} ON {}",
            self.target_entity,
            self.join_condition.replace("{from}", from_entity)
        )
    }
}

/// A semantic entity definition.
///
/// Entities represent business-level security concepts (e.g., User, Device, Threat)
/// that map to one or more OCSF event classes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticEntity {
    /// Entity name (e.g., "authentication_event", "user").
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the entity.
    #[serde(default)]
    pub description: String,

    /// OCSF event class UIDs this entity draws from.
    #[serde(default)]
    pub source_event_classes: Vec<u32>,

    /// Attributes of this entity.
    #[serde(default)]
    pub attributes: Vec<SemanticAttribute>,

    /// Relationships to other entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<EntityRelationship>,

    /// Observable type_ids this entity covers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covers_observables: Vec<u32>,
}

impl SemanticEntity {
    /// Creates a new semantic entity with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            caption: String::new(),
            description: String::new(),
            source_event_classes: Vec::new(),
            attributes: Vec::new(),
            relationships: Vec::new(),
            covers_observables: Vec::new(),
        }
    }

    /// Sets the caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the source event classes.
    pub fn with_source_event_classes(mut self, classes: Vec<u32>) -> Self {
        self.source_event_classes = classes;
        self
    }

    /// Adds a source event class.
    pub fn add_source_event_class(mut self, class_uid: u32) -> Self {
        self.source_event_classes.push(class_uid);
        self
    }

    /// Adds an attribute.
    pub fn add_attribute(mut self, attribute: SemanticAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Sets the attributes.
    pub fn with_attributes(mut self, attributes: Vec<SemanticAttribute>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Adds a relationship.
    pub fn add_relationship(mut self, relationship: EntityRelationship) -> Self {
        self.relationships.push(relationship);
        self
    }

    /// Sets the relationships.
    pub fn with_relationships(mut self, relationships: Vec<EntityRelationship>) -> Self {
        self.relationships = relationships;
        self
    }

    /// Sets the covered observables.
    pub fn with_covers_observables(mut self, observables: Vec<u32>) -> Self {
        self.covers_observables = observables;
        self
    }

    /// Gets an attribute by name.
    pub fn get_attribute(&self, name: &str) -> Option<&SemanticAttribute> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// Gets a relationship by name.
    pub fn get_relationship(&self, name: &str) -> Option<&EntityRelationship> {
        self.relationships.iter().find(|r| r.name == name)
    }

    /// Returns all dimension attributes.
    pub fn dimensions(&self) -> impl Iterator<Item = &SemanticAttribute> {
        self.attributes.iter().filter(|a| a.is_dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocsf_mapping_from_field() {
        let mapping = OCSFMapping::from_field("actor.user.name");
        assert_eq!(mapping.field, Some("actor.user.name".to_string()));
        assert!(mapping.expression.is_none());
        assert!(mapping.has_field());
        assert!(!mapping.has_expression());
        assert!(!mapping.is_empty());
    }

    #[test]
    fn test_ocsf_mapping_from_expression() {
        let mapping = OCSFMapping::from_expression("COALESCE(src_endpoint.ip, src_endpoint.hostname)");
        assert!(mapping.field.is_none());
        assert!(mapping.expression.is_some());
        assert!(!mapping.has_field());
        assert!(mapping.has_expression());
        assert!(!mapping.is_empty());
    }

    #[test]
    fn test_semantic_attribute_builder() {
        let attr = SemanticAttribute::new("user_email")
            .with_caption("User Email")
            .with_description("The email address of the user")
            .with_type(SemanticType::String)
            .with_field_mapping("actor.user.email_addr")
            .as_dimension()
            .with_sample_values(vec!["user@example.com".to_string()]);

        assert_eq!(attr.name, "user_email");
        assert_eq!(attr.caption, "User Email");
        assert!(attr.is_dimension);
        assert_eq!(attr.ocsf_mapping.field, Some("actor.user.email_addr".to_string()));
        assert_eq!(attr.sample_values, vec!["user@example.com"]);
    }

    #[test]
    fn test_entity_relationship() {
        let rel = EntityRelationship::new(
            "performed_by",
            "user",
            "authentication_event.user_email = user.email",
        )
        .with_cardinality(Cardinality::ManyToMany);

        assert_eq!(rel.name, "performed_by");
        assert_eq!(rel.target_entity, "user");
        assert_eq!(rel.cardinality, Cardinality::ManyToMany);
    }

    #[test]
    fn test_semantic_entity_builder() {
        let entity = SemanticEntity::new("authentication_event")
            .with_caption("Authentication Event")
            .with_description("User authentication attempts")
            .with_source_event_classes(vec![3002, 3003])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_field_mapping("actor.user.email_addr")
                    .as_dimension(),
            )
            .add_relationship(EntityRelationship::new(
                "performed_by",
                "user",
                "authentication_event.user_email = user.email",
            ))
            .with_covers_observables(vec![5, 10]);

        assert_eq!(entity.name, "authentication_event");
        assert_eq!(entity.source_event_classes, vec![3002, 3003]);
        assert_eq!(entity.attributes.len(), 1);
        assert_eq!(entity.relationships.len(), 1);
        assert_eq!(entity.covers_observables, vec![5, 10]);
    }

    #[test]
    fn test_semantic_entity_serialization() {
        let entity = SemanticEntity::new("user")
            .with_caption("User")
            .with_source_event_classes(vec![3001, 3002])
            .add_attribute(
                SemanticAttribute::new("email")
                    .with_type(SemanticType::String)
                    .with_field_mapping("user.email_addr")
                    .as_dimension(),
            );

        let json = serde_json::to_string(&entity).unwrap();
        let deserialized: SemanticEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn test_semantic_type_array() {
        let array_type = SemanticType::Array(Box::new(SemanticType::String));
        let attr = SemanticAttribute::new("tags").with_type(array_type.clone());

        let json = serde_json::to_string(&attr).unwrap();
        let deserialized: SemanticAttribute = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.attr_type, array_type);
    }

    #[test]
    fn test_entity_get_attribute() {
        let entity = SemanticEntity::new("test")
            .add_attribute(SemanticAttribute::new("attr1"))
            .add_attribute(SemanticAttribute::new("attr2").as_dimension());

        assert!(entity.get_attribute("attr1").is_some());
        assert!(entity.get_attribute("attr2").is_some());
        assert!(entity.get_attribute("nonexistent").is_none());

        let dims: Vec<_> = entity.dimensions().collect();
        assert_eq!(dims.len(), 1);
        assert_eq!(dims[0].name, "attr2");
    }

    #[test]
    fn test_threat_relevance() {
        let relevance = ThreatRelevance::new()
            .with_use_cases(vec!["DGA detection".to_string(), "DNS tunneling".to_string()])
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1568.002".to_string()]);

        assert_eq!(relevance.use_cases.len(), 2);
        assert_eq!(relevance.mitre_techniques.len(), 2);
        assert!(!relevance.is_empty());

        let empty = ThreatRelevance::new();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_semantic_attribute_llm_fields() {
        let attr = SemanticAttribute::new("query_hostname")
            .with_caption("Query Hostname")
            .with_description("The fully qualified domain name being queried")
            .with_type(SemanticType::String)
            .with_field_mapping("query.hostname")
            .as_dimension()
            .as_observable()
            .with_synonyms(vec![
                "domain".to_string(),
                "dns name".to_string(),
                "fqdn".to_string(),
            ])
            .with_security_context("Primary indicator for threat detection. Check against threat intel feeds.")
            .with_value_pattern(r"^[a-zA-Z0-9][a-zA-Z0-9-_.]+$")
            .with_sample_values(vec!["www.example.com".to_string()])
            .with_threat_relevance(
                ThreatRelevance::new()
                    .with_use_cases(vec!["DGA detection".to_string()])
                    .with_mitre_techniques(vec!["T1568.002".to_string()]),
            );

        assert_eq!(attr.name, "query_hostname");
        assert!(attr.is_observable);
        assert_eq!(attr.synonyms.len(), 3);
        assert!(attr.synonyms.contains(&"domain".to_string()));
        assert!(attr.security_context.is_some());
        assert!(attr.value_pattern.is_some());
        assert!(attr.threat_relevance.is_some());

        let relevance = attr.threat_relevance.as_ref().unwrap();
        assert_eq!(relevance.use_cases, vec!["DGA detection"]);
        assert_eq!(relevance.mitre_techniques, vec!["T1568.002"]);
    }

    #[test]
    fn test_semantic_attribute_llm_serialization_roundtrip() {
        let attr = SemanticAttribute::new("source_ip")
            .with_caption("Source IP Address")
            .with_type(SemanticType::String)
            .with_field_mapping("src_endpoint.ip")
            .as_dimension()
            .as_observable()
            .with_synonyms(vec!["client IP".to_string(), "origin IP".to_string()])
            .with_security_context("Identifies the potentially compromised host")
            .with_value_pattern(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$")
            .with_sample_values(vec!["10.0.0.50".to_string()])
            .with_threat_relevance(
                ThreatRelevance::new()
                    .with_use_cases(vec!["C2 detection".to_string()])
                    .with_mitre_techniques(vec!["T1071.004".to_string()]),
            );

        // Test JSON round-trip
        let json = serde_json::to_string(&attr).unwrap();
        let deserialized: SemanticAttribute = serde_json::from_str(&json).unwrap();
        assert_eq!(attr, deserialized);

        // Test YAML round-trip
        let yaml = serde_yaml::to_string(&attr).unwrap();
        let deserialized_yaml: SemanticAttribute = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(attr, deserialized_yaml);
    }

    #[test]
    fn test_semantic_attribute_empty_llm_fields_not_serialized() {
        let attr = SemanticAttribute::new("simple_attr")
            .with_caption("Simple Attribute")
            .with_type(SemanticType::String);

        let json = serde_json::to_string(&attr).unwrap();
        
        // Empty optional fields should not appear in JSON
        assert!(!json.contains("synonyms"));
        assert!(!json.contains("security_context"));
        assert!(!json.contains("value_pattern"));
        assert!(!json.contains("threat_relevance"));
        assert!(!json.contains("sample_values"));
    }
}
