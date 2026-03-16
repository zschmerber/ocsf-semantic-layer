//! Property-based tests for Cube.js schema generation.
//!
//! **Feature: ocsf-semantic-layer, Property 11: Cube.js Schema Validity**
//! **Validates: Requirements 7.3**

use proptest::prelude::*;
use ocsf_semantic::{
    Aggregation, OCSFMapping, SemanticAttribute, SemanticEntity, SemanticMetric,
    SemanticModel, SemanticType, TimeGranularity,
};

use crate::cubejs::CubeGenerator;

/// Strategy for generating valid entity names.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s)
}

/// Strategy for generating valid attribute names.
fn attribute_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,15}".prop_map(|s| s)
}

/// Strategy for generating semantic attributes.
fn attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (attribute_name_strategy(), any::<bool>()).prop_map(|(name, is_dimension)| {
        let mut attr = SemanticAttribute::new(name.clone())
            .with_type(SemanticType::String)
            .with_mapping(OCSFMapping::from_field(format!("field.{}", name)));
        if is_dimension {
            attr = attr.as_dimension();
        }
        attr
    })
}

/// Strategy for generating semantic entities.
fn entity_strategy() -> impl Strategy<Value = SemanticEntity> {
    (
        entity_name_strategy(),
        prop::collection::vec(attribute_strategy(), 1..5),
    )
        .prop_map(|(name, attributes)| {
            SemanticEntity::new(name).with_attributes(attributes)
        })
}

/// Strategy for generating semantic metrics.
fn metric_strategy() -> impl Strategy<Value = SemanticMetric> {
    (
        attribute_name_strategy(),
        prop_oneof![
            Just(Aggregation::Count),
            Just(Aggregation::Sum),
            Just(Aggregation::Avg),
        ],
    )
        .prop_map(|(name, agg)| {
            SemanticMetric::new(name)
                .with_aggregation(agg)
                .with_field_measure("metadata_uid")
                .with_time_granularities(vec![TimeGranularity::Hour])
        })
}

/// Strategy for generating semantic models.
fn model_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        entity_name_strategy(),
        prop::collection::vec(entity_strategy(), 1..3),
        prop::collection::vec(metric_strategy(), 0..3),
    )
        .prop_map(|(name, entities, metrics)| {
            SemanticModel::new(name)
                .with_entities(entities)
                .with_metrics(metrics)
        })
}

proptest! {
    /// **Property 11: Cube.js Schema Validity**
    ///
    /// *For any* semantic model, the generated Cube.js schema should be valid
    /// JavaScript/TypeScript that parses without errors.
    ///
    /// **Validates: Requirements 7.3**
    #[test]
    fn prop_cubejs_schema_is_valid_js(model in model_strategy()) {
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        // Verify all cube definitions have valid structure
        prop_assert!(
            artifacts.is_valid_js(),
            "Cube.js schema should be valid JavaScript"
        );

        // Each cube should contain required elements
        for (name, content) in &artifacts.cubes {
            prop_assert!(
                content.contains("cube("),
                "Cube {} should contain cube() function",
                name
            );
            prop_assert!(
                content.contains("sql:"),
                "Cube {} should contain sql property",
                name
            );
            prop_assert!(
                content.contains("measures:"),
                "Cube {} should contain measures",
                name
            );
            prop_assert!(
                content.contains("dimensions:"),
                "Cube {} should contain dimensions",
                name
            );
        }
    }

    /// Property: Each entity should have a corresponding cube.
    #[test]
    fn prop_cubejs_has_cube_for_each_entity(model in model_strategy()) {
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        for entity in &model.entities {
            // Convert snake_case to PascalCase
            let cube_name: String = entity
                .name
                .split('_')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect();

            prop_assert!(
                artifacts.cubes.contains_key(&cube_name),
                "Missing cube for entity: {} (expected cube name: {})",
                entity.name,
                cube_name
            );
        }
    }

    /// Property: Cubes should have a count measure by default.
    #[test]
    fn prop_cubejs_has_count_measure(model in model_strategy()) {
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        for (name, content) in &artifacts.cubes {
            prop_assert!(
                content.contains("count:"),
                "Cube {} should have a count measure",
                name
            );
        }
    }

    /// Property: Cubes should have a time dimension.
    #[test]
    fn prop_cubejs_has_time_dimension(model in model_strategy()) {
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        for (name, content) in &artifacts.cubes {
            prop_assert!(
                content.contains("time:") && content.contains("type: `time`"),
                "Cube {} should have a time dimension",
                name
            );
        }
    }

    /// Property: Cubes should have a primary key dimension.
    #[test]
    fn prop_cubejs_has_primary_key(model in model_strategy()) {
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        for (name, content) in &artifacts.cubes {
            prop_assert!(
                content.contains("primaryKey: true"),
                "Cube {} should have a primary key dimension",
                name
            );
        }
    }
}
