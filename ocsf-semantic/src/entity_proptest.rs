//! Property-based tests for semantic entity serialization round-trip.
//!
//! **Feature: ocsf-semantic-layer, Property 2: Semantic Entity Definition Round-Trip**
//! **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 9.1, 9.4**
//!
//! For any valid semantic entity definition (with event class mappings, attributes,
//! expressions, and relationships), saving to YAML then loading should produce an
//! equivalent entity with all properties preserved.

use proptest::prelude::*;
use proptest::collection::vec;

use crate::entity::{
    Cardinality, EntityRelationship, OCSFMapping, RelationshipType, SemanticAttribute, SemanticEntity, SemanticType,
    ThreatRelevance,
};
use crate::model::SemanticModel;

// ============================================================================
// Generators for semantic entity types
// ============================================================================

/// Generate a valid identifier string (lowercase letters and underscores).
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,29}".prop_map(|s| s.to_string())
}

/// Generate a human-readable caption.
fn caption_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,49}".prop_map(|s| s.to_string())
}

/// Generate a description string.
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,;:!?'-]{0,200}".prop_map(|s| s.to_string())
}

/// Generate a SemanticType enum value.
fn semantic_type_strategy() -> impl Strategy<Value = SemanticType> {
    prop_oneof![
        Just(SemanticType::String),
        Just(SemanticType::Integer),
        Just(SemanticType::Float),
        Just(SemanticType::Boolean),
        Just(SemanticType::Timestamp),
        Just(SemanticType::Json),
        // Nested array types
        Just(SemanticType::Array(Box::new(SemanticType::String))),
        Just(SemanticType::Array(Box::new(SemanticType::Integer))),
    ]
}

/// Generate a Cardinality enum value.
fn cardinality_strategy() -> impl Strategy<Value = Cardinality> {
    prop_oneof![
        Just(Cardinality::OneToOne),
        Just(Cardinality::OneToMany),
        Just(Cardinality::ManyToOne),
        Just(Cardinality::ManyToMany),
    ]
}

/// Generate a RelationshipType enum value.
fn relationship_type_strategy() -> impl Strategy<Value = Option<RelationshipType>> {
    prop_oneof![
        Just(None),
        Just(Some(RelationshipType::OriginatedFrom)),
        Just(Some(RelationshipType::TargetedTo)),
        Just(Some(RelationshipType::PerformedBy)),
        Just(Some(RelationshipType::Contains)),
        Just(Some(RelationshipType::ReferencedIn)),
    ]
}

/// Generate an OCSF field path.
fn field_path_strategy() -> impl Strategy<Value = String> {
    "[a-z_]+\\.[a-z_]+".prop_map(|s| s.to_string())
}

/// Generate a SQL expression.
fn expression_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "COALESCE\\([a-z_]+, [a-z_]+\\)".prop_map(|s| s.to_string()),
        "CASE WHEN [a-z_]+ = [0-9]+ THEN '[a-z]+' ELSE '[a-z]+' END".prop_map(|s| s.to_string()),
        "[a-z_]+ \\|\\| ' ' \\|\\| [a-z_]+".prop_map(|s| s.to_string()),
    ]
}

/// Generate an OCSFMapping.
fn ocsf_mapping_strategy() -> impl Strategy<Value = OCSFMapping> {
    prop_oneof![
        // Field-based mapping
        field_path_strategy().prop_map(|field| OCSFMapping::from_field(field)),
        // Expression-based mapping
        expression_strategy().prop_map(|expr| OCSFMapping::from_expression(expr)),
        // Field with join condition
        (field_path_strategy(), "[a-z_]+ = [a-z_]+".prop_map(|s| s.to_string()))
            .prop_map(|(field, join)| OCSFMapping::from_field(field).with_join_condition(join)),
    ]
}

/// Generate sample values.
fn sample_values_strategy() -> impl Strategy<Value = Vec<String>> {
    vec("[a-zA-Z0-9@._-]{1,30}".prop_map(|s| s.to_string()), 0..3)
}

/// Generate synonyms for LLM-friendly attributes.
fn synonyms_strategy() -> impl Strategy<Value = Vec<String>> {
    vec("[a-zA-Z][a-zA-Z0-9 _-]{0,20}".prop_map(|s| s.to_string()), 0..5)
}

/// Generate optional security context.
fn security_context_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[A-Za-z0-9 .,;:!?'-]{10,100}".prop_map(|s| Some(s.to_string())),
    ]
}

/// Generate optional value pattern (regex).
fn value_pattern_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some(r"^[a-zA-Z0-9]+$".to_string())),
        Just(Some(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$".to_string())),
        Just(Some(r"^[a-zA-Z0-9][a-zA-Z0-9-_.]+$".to_string())),
    ]
}

/// Generate optional ThreatRelevance.
fn threat_relevance_strategy() -> impl Strategy<Value = Option<ThreatRelevance>> {
    prop_oneof![
        Just(None),
        (
            vec("[A-Za-z ]{5,30}".prop_map(|s| s.to_string()), 0..3),
            vec("T[0-9]{4}(\\.[0-9]{3})?".prop_map(|s| s.to_string()), 0..3),
        )
            .prop_map(|(use_cases, mitre_techniques)| {
                Some(ThreatRelevance {
                    use_cases,
                    mitre_techniques,
                })
            }),
    ]
}

/// Generate a SemanticAttribute.
fn semantic_attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        semantic_type_strategy(),
        ocsf_mapping_strategy(),
        any::<bool>(),
        sample_values_strategy(),
        synonyms_strategy(),
        security_context_strategy(),
        value_pattern_strategy(),
        any::<bool>(),
        threat_relevance_strategy(),
    )
        .prop_map(
            |(name, caption, description, attr_type, ocsf_mapping, is_dimension, sample_values,
              synonyms, security_context, value_pattern, is_observable, threat_relevance)| {
                SemanticAttribute {
                    name,
                    caption,
                    description,
                    attr_type,
                    ocsf_mapping,
                    is_dimension,
                    sample_values,
                    synonyms,
                    security_context,
                    value_pattern,
                    is_observable,
                    threat_relevance,
                }
            },
        )
}

/// Generate an EntityRelationship.
fn entity_relationship_strategy() -> impl Strategy<Value = EntityRelationship> {
    (
        identifier_strategy(),
        identifier_strategy(),
        relationship_type_strategy(),
        cardinality_strategy(),
        "[a-z_]+\\.[a-z_]+ = [a-z_]+\\.[a-z_]+".prop_map(|s| s.to_string()),
        description_strategy(),
    )
        .prop_map(|(name, target_entity, relationship_type, cardinality, join_condition, description)| EntityRelationship {
            name,
            target_entity,
            relationship_type,
            cardinality,
            join_condition,
            description,
        })
}

/// Generate a SemanticEntity.
fn semantic_entity_strategy() -> impl Strategy<Value = SemanticEntity> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        vec(1000u32..10000, 0..5),
        vec(semantic_attribute_strategy(), 0..5),
        vec(entity_relationship_strategy(), 0..3),
        vec(1u32..100, 0..5),
    )
        .prop_map(
            |(
                name,
                caption,
                description,
                source_event_classes,
                attributes,
                relationships,
                covers_observables,
            )| {
                SemanticEntity {
                    name,
                    caption,
                    description,
                    source_event_classes,
                    attributes,
                    relationships,
                    covers_observables,
                }
            },
        )
}

/// Generate a SemanticModel with entities.
fn semantic_model_with_entities_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        identifier_strategy(),
        description_strategy(),
        "[0-9]+\\.[0-9]+".prop_map(|s| s.to_string()),
        "[0-9]+\\.[0-9]+\\.[0-9]+".prop_map(|s| s.to_string()),
        vec(semantic_entity_strategy(), 1..5),
    )
        .prop_map(|(name, description, version, ocsf_version, entities)| {
            SemanticModel::new(name)
                .with_description(description)
                .with_version(version)
                .with_ocsf_version(ocsf_version)
                .with_entities(entities)
        })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 2: Semantic Entity Definition Round-Trip**
    ///
    /// For any valid SemanticType, serializing to JSON then deserializing should
    /// produce an equivalent SemanticType.
    #[test]
    fn semantic_type_json_roundtrip(st in semantic_type_strategy()) {
        let json = serde_json::to_string(&st).expect("SemanticType should serialize to JSON");
        let deserialized: SemanticType = serde_json::from_str(&json).expect("JSON should deserialize to SemanticType");
        prop_assert_eq!(st, deserialized, "SemanticType round-trip should preserve value");
    }

    /// For any valid Cardinality, serializing to JSON then deserializing should
    /// produce an equivalent Cardinality.
    #[test]
    fn cardinality_json_roundtrip(card in cardinality_strategy()) {
        let json = serde_json::to_string(&card).expect("Cardinality should serialize to JSON");
        let deserialized: Cardinality = serde_json::from_str(&json).expect("JSON should deserialize to Cardinality");
        prop_assert_eq!(card, deserialized, "Cardinality round-trip should preserve value");
    }

    /// For any valid OCSFMapping, serializing to JSON then deserializing should
    /// produce an equivalent OCSFMapping with field, expression, and join_condition preserved.
    #[test]
    fn ocsf_mapping_json_roundtrip(mapping in ocsf_mapping_strategy()) {
        let json = serde_json::to_string(&mapping).expect("OCSFMapping should serialize to JSON");
        let deserialized: OCSFMapping = serde_json::from_str(&json).expect("JSON should deserialize to OCSFMapping");
        prop_assert_eq!(mapping, deserialized, "OCSFMapping round-trip should preserve all fields");
    }

    /// For any valid SemanticAttribute, serializing to JSON then deserializing should
    /// produce an equivalent SemanticAttribute with all properties preserved.
    #[test]
    fn semantic_attribute_json_roundtrip(attr in semantic_attribute_strategy()) {
        let json = serde_json::to_string(&attr).expect("SemanticAttribute should serialize to JSON");
        let deserialized: SemanticAttribute = serde_json::from_str(&json).expect("JSON should deserialize to SemanticAttribute");
        prop_assert_eq!(attr, deserialized, "SemanticAttribute round-trip should preserve all fields");
    }

    /// For any valid EntityRelationship, serializing to JSON then deserializing should
    /// produce an equivalent EntityRelationship with cardinality preserved.
    #[test]
    fn entity_relationship_json_roundtrip(rel in entity_relationship_strategy()) {
        let json = serde_json::to_string(&rel).expect("EntityRelationship should serialize to JSON");
        let deserialized: EntityRelationship = serde_json::from_str(&json).expect("JSON should deserialize to EntityRelationship");
        prop_assert_eq!(rel, deserialized, "EntityRelationship round-trip should preserve all fields");
    }

    /// **Feature: ocsf-semantic-layer, Property 2: Semantic Entity Definition Round-Trip**
    ///
    /// For any valid SemanticEntity, serializing to JSON then deserializing should
    /// produce an equivalent entity with all event class mappings, attributes,
    /// expressions, and relationships preserved.
    #[test]
    fn semantic_entity_json_roundtrip(entity in semantic_entity_strategy()) {
        let json = serde_json::to_string(&entity).expect("SemanticEntity should serialize to JSON");
        let deserialized: SemanticEntity = serde_json::from_str(&json).expect("JSON should deserialize to SemanticEntity");

        // Verify all fields
        prop_assert_eq!(&entity.name, &deserialized.name, "Name should be preserved");
        prop_assert_eq!(&entity.caption, &deserialized.caption, "Caption should be preserved");
        prop_assert_eq!(&entity.description, &deserialized.description, "Description should be preserved");
        prop_assert_eq!(&entity.source_event_classes, &deserialized.source_event_classes, "Source event classes should be preserved");
        prop_assert_eq!(entity.attributes.len(), deserialized.attributes.len(), "Attributes count should be preserved");
        prop_assert_eq!(entity.relationships.len(), deserialized.relationships.len(), "Relationships count should be preserved");
        prop_assert_eq!(&entity.covers_observables, &deserialized.covers_observables, "Covers observables should be preserved");

        // Full equality check
        prop_assert_eq!(entity, deserialized, "Full entity round-trip should preserve all data");
    }

    /// **Feature: ocsf-semantic-layer, Property 2: Semantic Entity Definition Round-Trip**
    ///
    /// For any valid SemanticEntity, serializing to YAML then deserializing should
    /// produce an equivalent entity with all properties preserved.
    #[test]
    fn semantic_entity_yaml_roundtrip(entity in semantic_entity_strategy()) {
        let yaml = serde_yaml::to_string(&entity).expect("SemanticEntity should serialize to YAML");
        let deserialized: SemanticEntity = serde_yaml::from_str(&yaml).expect("YAML should deserialize to SemanticEntity");
        prop_assert_eq!(entity, deserialized, "YAML round-trip should preserve all data");
    }

    /// **Feature: ocsf-semantic-layer, Property 2: Semantic Entity Definition Round-Trip**
    ///
    /// For any valid SemanticModel with entities, serializing to YAML then deserializing
    /// should produce an equivalent model with all entities and their properties preserved.
    /// This validates Requirements 9.1 (YAML persistence) and 9.4 (import/export).
    #[test]
    fn semantic_model_yaml_roundtrip(model in semantic_model_with_entities_strategy()) {
        let yaml = model.to_yaml().expect("SemanticModel should serialize to YAML");
        let deserialized = SemanticModel::from_yaml(&yaml).expect("YAML should deserialize to SemanticModel");

        // Verify model metadata
        prop_assert_eq!(&model.name, &deserialized.name, "Model name should be preserved");
        prop_assert_eq!(&model.version, &deserialized.version, "Model version should be preserved");
        prop_assert_eq!(&model.ocsf_version, &deserialized.ocsf_version, "OCSF version should be preserved");
        prop_assert_eq!(&model.description, &deserialized.description, "Description should be preserved");

        // Verify entities count
        prop_assert_eq!(
            model.entities.len(),
            deserialized.entities.len(),
            "Entities count should be preserved"
        );

        // Verify each entity
        for (original, deser) in model.entities.iter().zip(deserialized.entities.iter()) {
            prop_assert_eq!(original, deser, "Entity content should be preserved");
        }

        // Full equality check
        prop_assert_eq!(model, deserialized, "Full model round-trip should preserve all data");
    }

    /// For any valid SemanticAttribute with expression-based mapping, the expression
    /// should be preserved through serialization round-trip.
    #[test]
    fn expression_mapping_preserved(
        name in identifier_strategy(),
        expr in expression_strategy()
    ) {
        let attr = SemanticAttribute::new(name)
            .with_expression_mapping(expr.clone());

        let yaml = serde_yaml::to_string(&attr).expect("Should serialize to YAML");
        let deserialized: SemanticAttribute = serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

        prop_assert_eq!(
            attr.ocsf_mapping.expression,
            deserialized.ocsf_mapping.expression,
            "Expression mapping should be preserved"
        );
    }

    /// For any valid EntityRelationship with cardinality, the cardinality
    /// should be preserved through serialization round-trip.
    #[test]
    fn relationship_cardinality_preserved(rel in entity_relationship_strategy()) {
        let yaml = serde_yaml::to_string(&rel).expect("Should serialize to YAML");
        let deserialized: EntityRelationship = serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

        prop_assert_eq!(
            rel.cardinality,
            deserialized.cardinality,
            "Cardinality should be preserved"
        );
    }
}
