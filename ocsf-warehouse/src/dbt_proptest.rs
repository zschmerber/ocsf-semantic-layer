//! Property-based tests for dbt artifact generation.
//!
//! **Feature: ocsf-semantic-layer, Property 10: DBT Artifact Validity**
//! **Validates: Requirements 7.2**

use proptest::prelude::*;
use ocsf_semantic::{
    Aggregation, OCSFMapping, SemanticAttribute, SemanticEntity, SemanticMetric,
    SemanticModel, SemanticType, TimeGranularity, WarehouseDialect,
};

use crate::dbt::DBTGenerator;

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
        prop::collection::vec(1000u32..9999u32, 1..3),
    )
        .prop_map(|(name, attributes, class_uids)| {
            SemanticEntity::new(name)
                .with_attributes(attributes)
                .with_source_event_classes(class_uids)
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
                .with_time_granularities(vec![TimeGranularity::Hour, TimeGranularity::Day])
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
    /// **Property 10: DBT Artifact Validity**
    ///
    /// *For any* semantic model, the generated dbt semantic layer YAML should
    /// be valid YAML that parses without errors.
    ///
    /// **Validates: Requirements 7.2**
    #[test]
    fn prop_dbt_artifacts_are_valid_yaml(model in model_strategy()) {
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        // Verify semantic manifest is valid YAML
        let manifest_result = serde_yaml::from_str::<serde_yaml::Value>(&artifacts.semantic_manifest);
        prop_assert!(
            manifest_result.is_ok(),
            "Semantic manifest is not valid YAML: {:?}",
            manifest_result.err()
        );

        // Verify sources is valid YAML
        let sources_result = serde_yaml::from_str::<serde_yaml::Value>(&artifacts.sources);
        prop_assert!(
            sources_result.is_ok(),
            "Sources is not valid YAML: {:?}",
            sources_result.err()
        );
    }

    /// Property: dbt artifacts should contain semantic_models key.
    #[test]
    fn prop_dbt_manifest_has_semantic_models(model in model_strategy()) {
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        prop_assert!(
            artifacts.semantic_manifest.contains("semantic_models"),
            "Manifest should contain semantic_models key"
        );
    }

    /// Property: dbt sources should contain sources key.
    #[test]
    fn prop_dbt_sources_has_sources_key(model in model_strategy()) {
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        prop_assert!(
            artifacts.sources.contains("sources"),
            "Sources should contain sources key"
        );
    }

    /// Property: Each entity should have a corresponding model SQL file.
    #[test]
    fn prop_dbt_has_model_for_each_entity(model in model_strategy()) {
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        for entity in &model.entities {
            let model_name = format!("ocsf_{}", entity.name);
            prop_assert!(
                artifacts.models.contains_key(&model_name),
                "Missing model SQL for entity: {}",
                entity.name
            );
        }
    }

    /// Property: Model SQL files should contain SELECT and FROM.
    #[test]
    fn prop_dbt_model_sql_is_valid(model in model_strategy()) {
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        for (name, sql) in &artifacts.models {
            prop_assert!(
                sql.contains("SELECT"),
                "Model {} should contain SELECT",
                name
            );
            prop_assert!(
                sql.contains("FROM"),
                "Model {} should contain FROM",
                name
            );
        }
    }
}
