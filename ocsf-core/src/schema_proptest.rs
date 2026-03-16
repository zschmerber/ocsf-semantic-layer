//! Property-based tests for OCSF schema serialization round-trip.
//!
//! **Feature: ocsf-semantic-layer, Property 1: Schema Parsing Completeness and Fidelity**
//! **Validates: Requirements 1.2, 1.3, 1.5**
//!
//! For any valid OCSF schema JSON, parsing then serializing back to JSON should
//! produce an equivalent schema with all categories, event classes, objects,
//! attributes, and their metadata (types, requirements, descriptions) preserved.

use proptest::prelude::*;
use proptest::collection::{hash_map, vec};

use crate::schema::{
    Attribute, AttributeRef, Category, EnumValue, EventClass, OCSFObject,
    OCSFSchema, ObservableDefinition, ObservableDefinitionType, Requirement,
};

// ============================================================================
// Generators for OCSF schema types
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

/// Generate a version string.
fn version_strategy() -> impl Strategy<Value = String> {
    (1u32..10, 0u32..20, 0u32..10).prop_map(|(major, minor, patch)| {
        format!("{}.{}.{}", major, minor, patch)
    })
}

/// Generate an OCSF type string.
fn ocsf_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("string_t".to_string()),
        Just("integer_t".to_string()),
        Just("long_t".to_string()),
        Just("boolean_t".to_string()),
        Just("timestamp_t".to_string()),
        Just("float_t".to_string()),
        Just("json_t".to_string()),
        Just("object_t".to_string()),
    ]
}

/// Generate a Requirement enum value.
fn requirement_strategy() -> impl Strategy<Value = Requirement> {
    prop_oneof![
        Just(Requirement::Required),
        Just(Requirement::Recommended),
        Just(Requirement::Optional),
    ]
}

/// Generate an ObservableDefinitionType enum value.
fn observable_definition_type_strategy() -> impl Strategy<Value = ObservableDefinitionType> {
    prop_oneof![
        Just(ObservableDefinitionType::ByType),
        Just(ObservableDefinitionType::ByAttribute),
        Just(ObservableDefinitionType::ByObject),
        Just(ObservableDefinitionType::ByEventClass),
        Just(ObservableDefinitionType::ByPath),
    ]
}

/// Generate an EnumValue.
fn enum_value_strategy() -> impl Strategy<Value = EnumValue> {
    (any::<i64>(), caption_strategy(), description_strategy()).prop_map(
        |(value, caption, description)| EnumValue {
            value,
            caption,
            description,
        },
    )
}

/// Generate an Attribute.
fn attribute_strategy() -> impl Strategy<Value = Attribute> {
    (
        identifier_strategy(),
        ocsf_type_strategy(),
        caption_strategy(),
        description_strategy(),
        requirement_strategy(),
        proptest::option::of(1u32..100),
        any::<bool>(),
        proptest::option::of(identifier_strategy()),
        hash_map(identifier_strategy(), enum_value_strategy(), 0..3),
    )
        .prop_map(
            |(
                name,
                attr_type,
                caption,
                description,
                requirement,
                observable,
                is_array,
                object_type,
                enum_values,
            )| {
                Attribute {
                    name,
                    attr_type,
                    caption,
                    description,
                    requirement,
                    observable,
                    is_array,
                    object_type,
                    enum_values,
                    default: None, // Skip default for simplicity in round-trip
                }
            },
        )
}

/// Generate an AttributeRef.
fn attribute_ref_strategy() -> impl Strategy<Value = AttributeRef> {
    (
        identifier_strategy(),
        proptest::option::of(requirement_strategy()),
        proptest::option::of(description_strategy()),
        proptest::option::of(identifier_strategy()),
    )
        .prop_map(|(name, requirement, description, group)| AttributeRef {
            name,
            requirement,
            description,
            group,
        })
}

/// Generate an ObservableDefinition.
fn observable_definition_strategy() -> impl Strategy<Value = ObservableDefinition> {
    (
        1u32..100,
        caption_strategy(),
        observable_definition_type_strategy(),
        proptest::option::of("[a-z_]+\\.[a-z_]+".prop_map(|s| s.to_string())),
        proptest::option::of(1000u32..10000),
        description_strategy(),
    )
        .prop_map(
            |(type_id, type_name, definition_type, source_path, source_event_class, description)| {
                ObservableDefinition {
                    type_id,
                    type_name,
                    definition_type,
                    source_path,
                    source_event_class,
                    description,
                }
            },
        )
}

/// Generate a Category.
fn category_strategy() -> impl Strategy<Value = Category> {
    (
        1u32..100,
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        vec(1000u32..10000, 0..5),
    )
        .prop_map(|(uid, name, caption, description, event_classes)| Category {
            uid,
            name,
            caption,
            description,
            event_classes,
        })
}

/// Generate an EventClass.
fn event_class_strategy() -> impl Strategy<Value = EventClass> {
    (
        1000u32..10000,
        1u32..100,
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        hash_map(identifier_strategy(), attribute_ref_strategy(), 0..5),
        vec(observable_definition_strategy(), 0..3),
        proptest::option::of(identifier_strategy()),
        vec(identifier_strategy(), 0..3),
    )
        .prop_map(
            |(
                class_uid,
                category_uid,
                name,
                caption,
                description,
                attributes,
                observables,
                extends,
                profiles,
            )| {
                EventClass {
                    class_uid,
                    category_uid,
                    name,
                    caption,
                    description,
                    attributes,
                    observables,
                    extends,
                    profiles,
                }
            },
        )
}

/// Generate an OCSFObject.
fn ocsf_object_strategy() -> impl Strategy<Value = OCSFObject> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        hash_map(identifier_strategy(), attribute_strategy(), 0..5),
        proptest::option::of(identifier_strategy()),
        vec(observable_definition_strategy(), 0..3),
    )
        .prop_map(
            |(name, caption, description, attributes, extends, observables)| OCSFObject {
                name,
                caption,
                description,
                attributes,
                extends,
                observables,
            },
        )
}

/// Generate an OCSFSchema.
fn ocsf_schema_strategy() -> impl Strategy<Value = OCSFSchema> {
    (
        version_strategy(),
        vec(category_strategy(), 0..5),
        vec(event_class_strategy(), 0..5),
        vec(ocsf_object_strategy(), 0..5),
        vec(attribute_strategy(), 0..5),
        vec(observable_definition_strategy(), 0..5),
    )
        .prop_map(
            |(version, categories, event_classes, objects, attributes, observables)| {
                let mut schema = OCSFSchema::new(version);

                // Add categories
                for category in categories {
                    schema.categories.insert(category.uid, category);
                }

                // Add event classes
                for event_class in event_classes {
                    schema.event_classes.insert(event_class.class_uid, event_class);
                }

                // Add objects
                for object in objects {
                    schema.objects.insert(object.name.clone(), object);
                }

                // Add attributes
                for attribute in attributes {
                    schema.attributes.insert(attribute.name.clone(), attribute);
                }

                // Add observables
                schema.observables = observables;

                schema
            },
        )
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 1: Schema Parsing Completeness and Fidelity**
    ///
    /// For any valid Attribute, serializing to JSON then deserializing should
    /// produce an equivalent Attribute with all metadata preserved.
    #[test]
    fn attribute_roundtrip(attr in attribute_strategy()) {
        let json = serde_json::to_string(&attr).expect("Attribute should serialize to JSON");
        let deserialized: Attribute = serde_json::from_str(&json).expect("JSON should deserialize to Attribute");
        prop_assert_eq!(attr, deserialized, "Attribute round-trip should preserve all fields");
    }

    /// For any valid EnumValue, serializing to JSON then deserializing should
    /// produce an equivalent EnumValue.
    #[test]
    fn enum_value_roundtrip(ev in enum_value_strategy()) {
        let json = serde_json::to_string(&ev).expect("EnumValue should serialize to JSON");
        let deserialized: EnumValue = serde_json::from_str(&json).expect("JSON should deserialize to EnumValue");
        prop_assert_eq!(ev, deserialized, "EnumValue round-trip should preserve all fields");
    }

    /// For any valid AttributeRef, serializing to JSON then deserializing should
    /// produce an equivalent AttributeRef.
    #[test]
    fn attribute_ref_roundtrip(attr_ref in attribute_ref_strategy()) {
        let json = serde_json::to_string(&attr_ref).expect("AttributeRef should serialize to JSON");
        let deserialized: AttributeRef = serde_json::from_str(&json).expect("JSON should deserialize to AttributeRef");
        prop_assert_eq!(attr_ref, deserialized, "AttributeRef round-trip should preserve all fields");
    }

    /// For any valid ObservableDefinition, serializing to JSON then deserializing should
    /// produce an equivalent ObservableDefinition with definition type and metadata preserved.
    #[test]
    fn observable_definition_roundtrip(obs in observable_definition_strategy()) {
        let json = serde_json::to_string(&obs).expect("ObservableDefinition should serialize to JSON");
        let deserialized: ObservableDefinition = serde_json::from_str(&json).expect("JSON should deserialize to ObservableDefinition");
        prop_assert_eq!(obs, deserialized, "ObservableDefinition round-trip should preserve all fields");
    }

    /// For any valid Category, serializing to JSON then deserializing should
    /// produce an equivalent Category with all event class references preserved.
    #[test]
    fn category_roundtrip(category in category_strategy()) {
        let json = serde_json::to_string(&category).expect("Category should serialize to JSON");
        let deserialized: Category = serde_json::from_str(&json).expect("JSON should deserialize to Category");
        prop_assert_eq!(category, deserialized, "Category round-trip should preserve all fields");
    }

    /// For any valid EventClass, serializing to JSON then deserializing should
    /// produce an equivalent EventClass with all attributes, observables, and metadata preserved.
    #[test]
    fn event_class_roundtrip(event_class in event_class_strategy()) {
        let json = serde_json::to_string(&event_class).expect("EventClass should serialize to JSON");
        let deserialized: EventClass = serde_json::from_str(&json).expect("JSON should deserialize to EventClass");
        prop_assert_eq!(event_class, deserialized, "EventClass round-trip should preserve all fields");
    }

    /// For any valid OCSFObject, serializing to JSON then deserializing should
    /// produce an equivalent OCSFObject with all attributes and observables preserved.
    #[test]
    fn ocsf_object_roundtrip(object in ocsf_object_strategy()) {
        let json = serde_json::to_string(&object).expect("OCSFObject should serialize to JSON");
        let deserialized: OCSFObject = serde_json::from_str(&json).expect("JSON should deserialize to OCSFObject");
        prop_assert_eq!(object, deserialized, "OCSFObject round-trip should preserve all fields");
    }

    /// **Feature: ocsf-semantic-layer, Property 1: Schema Parsing Completeness and Fidelity**
    ///
    /// For any valid OCSFSchema, serializing to JSON then deserializing should
    /// produce an equivalent schema with all categories, event classes, objects,
    /// attributes, and observables preserved.
    #[test]
    fn ocsf_schema_roundtrip(schema in ocsf_schema_strategy()) {
        let json = serde_json::to_string(&schema).expect("OCSFSchema should serialize to JSON");
        let deserialized: OCSFSchema = serde_json::from_str(&json).expect("JSON should deserialize to OCSFSchema");

        // Verify version
        prop_assert_eq!(&schema.version, &deserialized.version, "Version should be preserved");

        // Verify categories count and content
        prop_assert_eq!(
            schema.categories.len(),
            deserialized.categories.len(),
            "Categories count should be preserved"
        );
        for (uid, category) in &schema.categories {
            let deser_cat = deserialized.categories.get(uid)
                .expect("Category should exist in deserialized schema");
            prop_assert_eq!(category, deser_cat, "Category content should be preserved");
        }

        // Verify event classes count and content
        prop_assert_eq!(
            schema.event_classes.len(),
            deserialized.event_classes.len(),
            "Event classes count should be preserved"
        );
        for (class_uid, event_class) in &schema.event_classes {
            let deser_ec = deserialized.event_classes.get(class_uid)
                .expect("EventClass should exist in deserialized schema");
            prop_assert_eq!(event_class, deser_ec, "EventClass content should be preserved");
        }

        // Verify objects count and content
        prop_assert_eq!(
            schema.objects.len(),
            deserialized.objects.len(),
            "Objects count should be preserved"
        );
        for (name, object) in &schema.objects {
            let deser_obj = deserialized.objects.get(name)
                .expect("Object should exist in deserialized schema");
            prop_assert_eq!(object, deser_obj, "Object content should be preserved");
        }

        // Verify attributes count and content
        prop_assert_eq!(
            schema.attributes.len(),
            deserialized.attributes.len(),
            "Attributes count should be preserved"
        );
        for (name, attribute) in &schema.attributes {
            let deser_attr = deserialized.attributes.get(name)
                .expect("Attribute should exist in deserialized schema");
            prop_assert_eq!(attribute, deser_attr, "Attribute content should be preserved");
        }

        // Verify observables count and content
        prop_assert_eq!(
            schema.observables.len(),
            deserialized.observables.len(),
            "Observables count should be preserved"
        );

        // Full equality check
        prop_assert_eq!(schema, deserialized, "Full schema round-trip should preserve all data");
    }

    /// For any valid OCSFSchema, serializing to pretty JSON then deserializing should
    /// produce an equivalent schema (testing pretty-print format).
    #[test]
    fn ocsf_schema_pretty_roundtrip(schema in ocsf_schema_strategy()) {
        let json = serde_json::to_string_pretty(&schema).expect("OCSFSchema should serialize to pretty JSON");
        let deserialized: OCSFSchema = serde_json::from_str(&json).expect("Pretty JSON should deserialize to OCSFSchema");
        prop_assert_eq!(schema, deserialized, "Pretty JSON round-trip should preserve all data");
    }

    /// For any valid Requirement enum, serializing to JSON then deserializing should
    /// produce the same Requirement value.
    #[test]
    fn requirement_roundtrip(req in requirement_strategy()) {
        let json = serde_json::to_string(&req).expect("Requirement should serialize to JSON");
        let deserialized: Requirement = serde_json::from_str(&json).expect("JSON should deserialize to Requirement");
        prop_assert_eq!(req, deserialized, "Requirement round-trip should preserve value");
    }

    /// For any valid ObservableDefinitionType enum, serializing to JSON then deserializing
    /// should produce the same ObservableDefinitionType value.
    #[test]
    fn observable_definition_type_roundtrip(odt in observable_definition_type_strategy()) {
        let json = serde_json::to_string(&odt).expect("ObservableDefinitionType should serialize to JSON");
        let deserialized: ObservableDefinitionType = serde_json::from_str(&json).expect("JSON should deserialize to ObservableDefinitionType");
        prop_assert_eq!(odt, deserialized, "ObservableDefinitionType round-trip should preserve value");
    }
}
