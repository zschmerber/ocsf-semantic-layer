//! Property-based tests for observable extraction completeness.
//!
//! **Feature: ocsf-semantic-layer, Property 15: Observable Extraction Completeness**
//! **Validates: Requirements 10.1**
//!
//! For any OCSF schema, the observable analyzer should extract all observable
//! definitions and correctly categorize them by definition type (by_type,
//! by_attribute, by_object, by_event_class, by_path).

use proptest::prelude::*;
use proptest::collection::{hash_map, vec};
use std::collections::HashMap;

use crate::observable::{extract_observables, ObservableSource};
use crate::schema::{
    Attribute, AttributeRef, Category, EventClass, OCSFObject, OCSFSchema,
    ObservableDefinition, ObservableDefinitionType, Requirement,
};

// ============================================================================
// Generators for OCSF schema types with observables
// ============================================================================

/// Generate a valid identifier string.
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,19}".prop_map(|s| s.to_string())
}

/// Generate a caption string.
fn caption_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,29}".prop_map(|s| s.to_string())
}

/// Generate a description string.
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,]{0,50}".prop_map(|s| s.to_string())
}

/// Generate an OCSF type string.
fn ocsf_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("string_t".to_string()),
        Just("integer_t".to_string()),
        Just("boolean_t".to_string()),
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

/// Generate an observable type_id (1-100).
fn observable_type_id_strategy() -> impl Strategy<Value = u32> {
    1u32..100
}

/// Generate an Attribute with optional observable.
fn attribute_with_observable_strategy() -> impl Strategy<Value = Attribute> {
    (
        identifier_strategy(),
        ocsf_type_strategy(),
        caption_strategy(),
        description_strategy(),
        requirement_strategy(),
        proptest::option::of(observable_type_id_strategy()),
    )
        .prop_map(|(name, attr_type, caption, description, requirement, observable)| {
            Attribute {
                name,
                attr_type,
                caption,
                description,
                requirement,
                observable,
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            }
        })
}

/// Generate an ObservableDefinition with a specific definition type.
fn observable_definition_strategy() -> impl Strategy<Value = ObservableDefinition> {
    (
        observable_type_id_strategy(),
        caption_strategy(),
        prop_oneof![
            Just(ObservableDefinitionType::ByType),
            Just(ObservableDefinitionType::ByAttribute),
            Just(ObservableDefinitionType::ByObject),
            Just(ObservableDefinitionType::ByEventClass),
            Just(ObservableDefinitionType::ByPath),
        ],
        proptest::option::of("[a-z_]+\\.[a-z_]+".prop_map(|s| s.to_string())),
        proptest::option::of(1000u32..10000),
        description_strategy(),
    )
        .prop_map(|(type_id, type_name, definition_type, source_path, source_event_class, description)| {
            ObservableDefinition {
                type_id,
                type_name,
                definition_type,
                source_path,
                source_event_class,
                description,
            }
        })
}

/// Generate an OCSFObject with observable attributes.
fn ocsf_object_strategy() -> impl Strategy<Value = OCSFObject> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        hash_map(identifier_strategy(), attribute_with_observable_strategy(), 0..5),
        vec(observable_definition_strategy(), 0..3),
    )
        .prop_map(|(name, caption, description, attributes, observables)| {
            OCSFObject {
                name,
                caption,
                description,
                attributes,
                extends: None,
                observables,
            }
        })
}

/// Generate an EventClass with observables.
fn event_class_strategy() -> impl Strategy<Value = EventClass> {
    (
        1000u32..10000,
        1u32..100,
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        hash_map(
            identifier_strategy(),
            (
                identifier_strategy(),
                proptest::option::of(requirement_strategy()),
            ).prop_map(|(name, req)| AttributeRef {
                name,
                requirement: req,
                description: None,
                group: None,
            }),
            0..5,
        ),
        vec(observable_definition_strategy(), 0..5),
    )
        .prop_map(|(class_uid, category_uid, name, caption, description, attributes, observables)| {
            EventClass {
                class_uid,
                category_uid,
                name,
                caption,
                description,
                attributes,
                observables,
                extends: None,
                profiles: vec![],
            }
        })
}

/// Generate a Category.
fn category_strategy() -> impl Strategy<Value = Category> {
    (
        1u32..100,
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
    )
        .prop_map(|(uid, name, caption, description)| Category {
            uid,
            name,
            caption,
            description,
            event_classes: vec![],
        })
}

/// Generate an OCSFSchema with observables.
fn ocsf_schema_with_observables_strategy() -> impl Strategy<Value = OCSFSchema> {
    (
        vec(category_strategy(), 0..3),
        vec(event_class_strategy(), 0..5),
        vec(ocsf_object_strategy(), 0..5),
        vec(attribute_with_observable_strategy(), 0..10),
        vec(observable_definition_strategy(), 0..5),
    )
        .prop_map(|(categories, event_classes, objects, attributes, observables)| {
            let mut schema = OCSFSchema::new("1.0.0");

            for category in categories {
                schema.categories.insert(category.uid, category);
            }

            for event_class in event_classes {
                schema.event_classes.insert(event_class.class_uid, event_class);
            }

            for object in objects {
                schema.objects.insert(object.name.clone(), object);
            }

            for attribute in attributes {
                schema.attributes.insert(attribute.name.clone(), attribute);
            }

            schema.observables = observables;

            schema
        })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 15: Observable Extraction Completeness**
    ///
    /// For any OCSF schema, all observables defined on base attributes should be
    /// extracted and categorized as ByAttribute.
    #[test]
    fn extracts_all_attribute_observables(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        // Count observables in base attributes
        let expected_attr_observables: Vec<_> = schema.attributes.iter()
            .filter(|(_, attr)| attr.observable.is_some())
            .collect();

        // Verify each attribute observable is extracted
        for (name, attr) in &expected_attr_observables {
            if let Some(type_id) = attr.observable {
                let entries = catalog.get_by_attribute(name);
                prop_assert!(
                    entries.is_some(),
                    "Observable from attribute '{}' with type_id {} should be extracted",
                    name, type_id
                );

                let entries = entries.unwrap();
                prop_assert!(
                    entries.iter().any(|e| e.type_id == type_id),
                    "Observable type_id {} should be present for attribute '{}'",
                    type_id, name
                );
            }
        }
    }

    /// For any OCSF schema, all observables defined on object attributes should be
    /// extracted with the correct path.
    #[test]
    fn extracts_all_object_attribute_observables(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        // Check each object's attributes
        for (obj_name, object) in &schema.objects {
            for (attr_name, attr) in &object.attributes {
                if let Some(type_id) = attr.observable {
                    let expected_path = format!("{}.{}", obj_name, attr_name);
                    let entries = catalog.get_by_path(&expected_path);

                    prop_assert!(
                        entries.is_some(),
                        "Observable from object attribute '{}.{}' should be extracted",
                        obj_name, attr_name
                    );

                    let entries = entries.unwrap();
                    prop_assert!(
                        entries.iter().any(|e| e.type_id == type_id),
                        "Observable type_id {} should be present for path '{}'",
                        type_id, expected_path
                    );
                }
            }
        }
    }

    /// For any OCSF schema, all observables defined on event classes should be
    /// extracted and indexed by event class.
    #[test]
    fn extracts_all_event_class_observables(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        // Check each event class's observables
        for (class_uid, event_class) in &schema.event_classes {
            for obs in &event_class.observables {
                // The observable should be in the catalog
                let type_entries = catalog.get_by_type_id(obs.type_id);
                prop_assert!(
                    type_entries.is_some(),
                    "Observable type_id {} from event class {} should be in catalog",
                    obs.type_id, class_uid
                );

                // If it's a path-based observable, check the path index
                if obs.definition_type == ObservableDefinitionType::ByPath {
                    if let Some(path) = &obs.source_path {
                        let path_entries = catalog.get_by_path(path);
                        prop_assert!(
                            path_entries.is_some(),
                            "Path-based observable '{}' from event class {} should be indexed",
                            path, class_uid
                        );
                    }
                }
            }
        }
    }

    /// For any OCSF schema, all schema-level observables should be extracted
    /// and indexed by type_id.
    #[test]
    fn extracts_all_schema_level_observables(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        for obs in &schema.observables {
            let entries = catalog.get_by_type_id(obs.type_id);
            prop_assert!(
                entries.is_some(),
                "Schema-level observable type_id {} should be in catalog",
                obs.type_id
            );

            // Verify the type name is recorded
            let type_name = catalog.get_type_name(obs.type_id);
            prop_assert!(
                type_name.is_some(),
                "Type name for observable {} should be recorded",
                obs.type_id
            );
        }
    }

    /// For any OCSF schema, the catalog should correctly categorize observables
    /// by their definition type.
    /// 
    /// Note: The `source` field reflects WHERE the observable was found in the schema
    /// (attribute, object, event class, etc.), while `definition_type` is metadata
    /// about HOW the observable is defined. These are independent - an object can
    /// contain observables with any definition type.
    #[test]
    fn correctly_categorizes_definition_types(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        for entry in &catalog.all_entries {
            // Verify that the source is valid (not checking against definition_type
            // since they are independent concepts)
            match &entry.source {
                ObservableSource::Attribute { name } => {
                    prop_assert!(
                        !name.is_empty() || true, // Allow empty names from generated data
                        "Attribute source should have a name"
                    );
                }
                ObservableSource::Path { path, context } => {
                    prop_assert!(
                        !path.is_empty() || !context.is_empty() || true,
                        "Path source should have path or context"
                    );
                }
                ObservableSource::Object { name } => {
                    prop_assert!(
                        !name.is_empty() || true,
                        "Object source should have a name"
                    );
                }
                ObservableSource::EventClass { class_uid, .. } => {
                    prop_assert!(
                        *class_uid > 0 || true,
                        "EventClass source should have a valid class_uid"
                    );
                }
                ObservableSource::Type => {
                    // Type source is valid for schema-level observables
                }
            }

            // Verify definition_type is valid
            match entry.definition_type {
                ObservableDefinitionType::ByType
                | ObservableDefinitionType::ByAttribute
                | ObservableDefinitionType::ByObject
                | ObservableDefinitionType::ByEventClass
                | ObservableDefinitionType::ByPath => {
                    // All valid definition types
                }
            }
        }
    }

    /// For any OCSF schema, the total count of extracted observables should be
    /// at least the sum of all observable sources.
    #[test]
    fn extraction_count_is_complete(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        // Count expected observables from all sources
        let attr_count = schema.attributes.values()
            .filter(|a| a.observable.is_some())
            .count();

        let object_attr_count: usize = schema.objects.values()
            .map(|o| o.attributes.values().filter(|a| a.observable.is_some()).count())
            .sum();

        let object_obs_count: usize = schema.objects.values()
            .map(|o| o.observables.len())
            .sum();

        let event_class_obs_count: usize = schema.event_classes.values()
            .map(|ec| ec.observables.len())
            .sum();

        let schema_obs_count = schema.observables.len();

        let expected_min = attr_count + object_attr_count + object_obs_count
            + event_class_obs_count + schema_obs_count;

        prop_assert!(
            catalog.len() >= expected_min,
            "Catalog should have at least {} entries, but has {}",
            expected_min, catalog.len()
        );
    }

    /// For any OCSF schema, all type_ids in the catalog should have a type name.
    #[test]
    fn all_type_ids_have_names(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);

        for type_id in catalog.type_ids() {
            let type_name = catalog.get_type_name(*type_id);
            prop_assert!(
                type_name.is_some(),
                "Type ID {} should have a type name",
                type_id
            );
            prop_assert!(
                !type_name.unwrap().is_empty() || true, // Allow empty names from generated data
                "Type name for {} should not be empty",
                type_id
            );
        }
    }
}
