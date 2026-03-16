//! Property-based tests for graph generation completeness.
//!
//! **Feature: ocsf-semantic-layer, Property 5: Graph Generation Completeness**
//! **Validates: Requirements 4.1, 4.2**
//!
//! For any semantic model with entities and OCSF schema, the generated visualization
//! graph should contain nodes for all entities and edges to all mapped OCSF event classes.

use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::HashMap;

use ocsf_core::{Category, EventClass, OCSFSchema, ObservableDefinition};
use ocsf_semantic::{
    Cardinality, EntityRelationship, ObservableConfig, SemanticAttribute, SemanticEntity,
    SemanticModel,
};

use crate::generator::{generate_graph, GeneratorOptions};
use crate::graph::{EdgeType, NodeType, ViewMode};

// ============================================================================
// Generators
// ============================================================================

/// Generate a valid identifier string (lowercase letters and underscores).
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,19}".prop_map(|s| s.to_string())
}

/// Generate a human-readable caption.
fn caption_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,29}".prop_map(|s| s.to_string())
}

/// Generate a description string.
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,]{0,50}".prop_map(|s| s.to_string())
}

/// Generate a category UID (1-10 for simplicity).
fn category_uid_strategy() -> impl Strategy<Value = u32> {
    1u32..10
}

/// Generate a class UID (1000-9999).
fn class_uid_strategy() -> impl Strategy<Value = u32> {
    1000u32..9999
}

/// Generate an observable type_id (1-100).
fn observable_type_id_strategy() -> impl Strategy<Value = u32> {
    1u32..100
}

/// Generate an OCSF Category.
fn category_strategy() -> impl Strategy<Value = Category> {
    (
        category_uid_strategy(),
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

/// Generate an OCSF EventClass.
fn event_class_strategy(category_uid: u32) -> impl Strategy<Value = EventClass> {
    (
        class_uid_strategy(),
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
    )
        .prop_map(move |(class_uid, name, caption, description)| EventClass {
            class_uid,
            category_uid,
            name,
            caption,
            description,
            attributes: HashMap::new(),
            observables: vec![],
            extends: None,
            profiles: vec![],
        })
}

/// Generate an ObservableDefinition.
fn observable_definition_strategy() -> impl Strategy<Value = ObservableDefinition> {
    (
        observable_type_id_strategy(),
        caption_strategy(),
        description_strategy(),
    )
        .prop_map(|(type_id, type_name, description)| {
            ObservableDefinition::by_type(type_id, type_name).with_description(description)
        })
}

/// Generate a SemanticAttribute.
fn semantic_attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (identifier_strategy(), caption_strategy()).prop_map(|(name, caption)| {
        SemanticAttribute::new(name)
            .with_caption(caption)
            .with_field_mapping("some.field.path")
    })
}

/// Generate an EntityRelationship.
fn entity_relationship_strategy(
    target_entities: Vec<String>,
) -> impl Strategy<Value = EntityRelationship> {
    if target_entities.is_empty() {
        // Return a dummy strategy that won't be used
        Just(EntityRelationship::new("dummy", "dummy", "a.b = c.d")).boxed()
    } else {
        (
            identifier_strategy(),
            proptest::sample::select(target_entities),
        )
            .prop_map(|(name, target)| {
                EntityRelationship::new(name, target, "a.b = c.d").with_cardinality(Cardinality::OneToMany)
            })
            .boxed()
    }
}

/// Generate a SemanticEntity with references to existing classes and observables.
fn semantic_entity_strategy(
    available_classes: Vec<u32>,
    available_observables: Vec<u32>,
) -> impl Strategy<Value = SemanticEntity> {
    let classes = available_classes.clone();
    let observables = available_observables.clone();

    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        vec(semantic_attribute_strategy(), 0..3),
        // Select subset of available classes
        proptest::sample::subsequence(classes.clone(), 0..=classes.len().min(3)),
        // Select subset of available observables
        proptest::sample::subsequence(observables.clone(), 0..=observables.len().min(3)),
    )
        .prop_map(
            |(name, caption, description, attributes, source_classes, covers_obs)| {
                SemanticEntity::new(name)
                    .with_caption(caption)
                    .with_description(description)
                    .with_attributes(attributes)
                    .with_source_event_classes(source_classes)
                    .with_covers_observables(covers_obs)
            },
        )
}

/// Generate a complete test scenario with schema and model.
fn schema_and_model_strategy() -> impl Strategy<Value = (OCSFSchema, SemanticModel)> {
    // First generate categories
    vec(category_strategy(), 1..4).prop_flat_map(|categories| {
        let category_uids: Vec<u32> = categories.iter().map(|c| c.uid).collect();

        // Generate event classes for each category
        let class_strategies: Vec<_> = category_uids
            .iter()
            .map(|&uid| vec(event_class_strategy(uid), 1..3))
            .collect();

        (Just(categories), class_strategies)
            .prop_flat_map(|(categories, class_vecs)| {
                let all_classes: Vec<EventClass> = class_vecs.into_iter().flatten().collect();
                let class_uids: Vec<u32> = all_classes.iter().map(|c| c.class_uid).collect();

                // Generate observables
                vec(observable_definition_strategy(), 1..5).prop_flat_map(move |observables| {
                    let observable_type_ids: Vec<u32> =
                        observables.iter().map(|o| o.type_id).collect();

                    let class_uids_clone = class_uids.clone();
                    let observable_ids_clone = observable_type_ids.clone();
                    let categories_clone = categories.clone();
                    let all_classes_clone = all_classes.clone();
                    let observables_clone = observables.clone();

                    // Generate entities
                    vec(
                        semantic_entity_strategy(class_uids_clone.clone(), observable_ids_clone.clone()),
                        1..4,
                    )
                    .prop_map(move |entities| {
                        // Build schema
                        let mut schema = OCSFSchema::new("1.4.0");
                        for cat in &categories_clone {
                            schema.add_category(cat.clone());
                        }
                        for class in &all_classes_clone {
                            schema.add_event_class(class.clone());
                        }
                        for obs in &observables_clone {
                            schema.add_observable(obs.clone());
                        }

                        // Build model
                        let model = SemanticModel::new("test-model")
                            .with_version("1.0")
                            .with_ocsf_version("1.4.0")
                            .with_entities(entities)
                            .with_observable_config(ObservableConfig {
                                extract_to_table: true,
                                table_name: "ocsf_observables".to_string(),
                                include_types: observable_type_ids.clone(),
                            });

                        (schema, model)
                    })
                })
            })
    })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 5: Graph Generation Completeness**
    ///
    /// For any semantic model with entities and OCSF schema, the generated
    /// visualization graph should contain nodes for all entities.
    #[test]
    fn graph_contains_all_entity_nodes((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        // Every entity in the model should have a corresponding node
        for entity in &model.entities {
            let node_id = format!("entity_{}", entity.name);
            prop_assert!(
                graph.get_node(&node_id).is_some(),
                "Graph should contain node for entity '{}' (expected node_id: {})",
                entity.name,
                node_id
            );

            // Verify node type
            let node = graph.get_node(&node_id).unwrap();
            prop_assert_eq!(
                node.node_type,
                NodeType::SemanticEntity,
                "Entity node should have SemanticEntity type"
            );
        }
    }

    /// **Feature: ocsf-semantic-layer, Property 5: Graph Generation Completeness**
    ///
    /// For any semantic model with entities and OCSF schema, the generated
    /// visualization graph should contain edges to all mapped OCSF event classes.
    #[test]
    fn graph_contains_all_mapping_edges((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        // Every source_event_class mapping should have a corresponding edge
        for entity in &model.entities {
            let entity_id = format!("entity_{}", entity.name);

            for &class_uid in &entity.source_event_classes {
                // Only check if the class exists in the schema
                if schema.get_event_class(class_uid).is_some() {
                    let class_id = format!("class_{}", class_uid);

                    // Find the mapping edge
                    let has_edge = graph
                        .edges_of_type(EdgeType::MapsTo)
                        .any(|e| e.source == entity_id && e.target == class_id);

                    prop_assert!(
                        has_edge,
                        "Graph should contain MapsTo edge from entity '{}' to class_{}",
                        entity.name,
                        class_uid
                    );
                }
            }
        }
    }

    /// **Feature: ocsf-semantic-layer, Property 5: Graph Generation Completeness**
    ///
    /// For any semantic model, the graph should contain nodes for all OCSF
    /// event classes that are referenced by entities.
    #[test]
    fn graph_contains_referenced_class_nodes((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        // Collect all referenced class UIDs
        let referenced_classes: std::collections::HashSet<u32> = model
            .entities
            .iter()
            .flat_map(|e| e.source_event_classes.iter().copied())
            .collect();

        // Each referenced class should have a node (if it exists in schema)
        for class_uid in referenced_classes {
            if schema.get_event_class(class_uid).is_some() {
                let node_id = format!("class_{}", class_uid);
                prop_assert!(
                    graph.get_node(&node_id).is_some(),
                    "Graph should contain node for referenced class_{}",
                    class_uid
                );

                // Verify node type
                let node = graph.get_node(&node_id).unwrap();
                prop_assert_eq!(
                    node.node_type,
                    NodeType::OcsfClass,
                    "Class node should have OcsfClass type"
                );
            }
        }
    }

    /// **Feature: ocsf-semantic-layer, Property 5: Graph Generation Completeness**
    ///
    /// For any semantic model, the graph should contain nodes for all categories
    /// that contain referenced event classes.
    #[test]
    fn graph_contains_referenced_category_nodes((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        // Collect all referenced category UIDs
        let referenced_categories: std::collections::HashSet<u32> = model
            .entities
            .iter()
            .flat_map(|e| e.source_event_classes.iter())
            .filter_map(|&class_uid| {
                schema.get_event_class(class_uid).map(|ec| ec.category_uid)
            })
            .collect();

        // Each referenced category should have a node
        for category_uid in referenced_categories {
            let node_id = format!("category_{}", category_uid);
            prop_assert!(
                graph.get_node(&node_id).is_some(),
                "Graph should contain node for referenced category_{}",
                category_uid
            );

            // Verify node type
            let node = graph.get_node(&node_id).unwrap();
            prop_assert_eq!(
                node.node_type,
                NodeType::OcsfCategory,
                "Category node should have OcsfCategory type"
            );
        }
    }

    /// The number of entity nodes should equal the number of entities in the model.
    #[test]
    fn entity_node_count_matches_model((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        let entity_node_count = graph.nodes_of_type(NodeType::SemanticEntity).count();
        prop_assert_eq!(
            entity_node_count,
            model.entities.len(),
            "Number of entity nodes should match number of entities in model"
        );
    }

    /// The number of MapsTo edges should equal the total number of source_event_class
    /// mappings across all entities (for classes that exist in schema).
    #[test]
    fn mapping_edge_count_matches_mappings((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        // Count expected mappings (only for classes that exist in schema)
        let expected_mappings: usize = model
            .entities
            .iter()
            .flat_map(|e| e.source_event_classes.iter())
            .filter(|&&class_uid| schema.get_event_class(class_uid).is_some())
            .count();

        let actual_mappings = graph.edges_of_type(EdgeType::MapsTo).count();
        prop_assert_eq!(
            actual_mappings,
            expected_mappings,
            "Number of MapsTo edges should match number of valid mappings"
        );
    }

    /// Graph metadata should reflect the schema and model versions.
    #[test]
    fn graph_metadata_contains_versions((schema, model) in schema_and_model_strategy()) {
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange);
        let graph = generate_graph(&model, &schema, options);

        prop_assert_eq!(
            graph.metadata.schema_version,
            Some(schema.version.clone()),
            "Graph metadata should contain schema version"
        );
        prop_assert_eq!(
            graph.metadata.model_version,
            Some(model.version.clone()),
            "Graph metadata should contain model version"
        );
        prop_assert_eq!(
            graph.metadata.view_mode,
            ViewMode::Interchange,
            "Graph metadata should reflect view mode"
        );
    }
}
