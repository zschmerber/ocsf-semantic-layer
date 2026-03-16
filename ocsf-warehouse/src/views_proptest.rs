//! Property-based tests for SQL view generation.
//!
//! **Feature: ocsf-semantic-layer, Property 12: SQL View Syntax Validity**
//! **Validates: Requirements 7.5**

use proptest::prelude::*;
use ocsf_semantic::{
    OCSFMapping, SemanticAttribute, SemanticEntity, SemanticModel, SemanticType,
    WarehouseDialect,
};

use crate::views::ViewGenerator;

/// Strategy for generating warehouse dialects.
fn dialect_strategy() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::Postgres),
    ]
}

/// Strategy for generating valid entity names.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s)
}

/// Strategy for generating valid attribute names.
fn attribute_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,15}".prop_map(|s| s)
}

/// Strategy for generating field paths.
fn field_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z][a-z0-9_]{1,10}".prop_map(|s| s),
        ("[a-z][a-z0-9_]{1,10}", "[a-z][a-z0-9_]{1,10}")
            .prop_map(|(a, b)| format!("{}.{}", a, b)),
    ]
}

/// Strategy for generating semantic attributes.
fn attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (attribute_name_strategy(), field_path_strategy()).prop_map(|(name, field)| {
        SemanticAttribute::new(name)
            .with_type(SemanticType::String)
            .with_mapping(OCSFMapping::from_field(field))
    })
}

/// Strategy for generating semantic entities.
fn entity_strategy() -> impl Strategy<Value = SemanticEntity> {
    (
        entity_name_strategy(),
        prop::collection::vec(attribute_strategy(), 1..5),
        prop::collection::vec(1000u32..9999u32, 0..3),
    )
        .prop_map(|(name, attributes, class_uids)| {
            SemanticEntity::new(name)
                .with_attributes(attributes)
                .with_source_event_classes(class_uids)
        })
}

/// Strategy for generating semantic models.
fn model_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        entity_name_strategy(),
        prop::collection::vec(entity_strategy(), 1..3),
    )
        .prop_map(|(name, entities)| SemanticModel::new(name).with_entities(entities))
}

proptest! {
    /// **Property 12: SQL View Syntax Validity**
    ///
    /// *For any* semantic entity and warehouse dialect, the generated SQL view
    /// should be syntactically valid for that dialect.
    ///
    /// **Validates: Requirements 7.5**
    #[test]
    fn prop_sql_view_is_syntactically_valid(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        // Verify basic SQL structure
        prop_assert!(
            generator.validate_syntax(&sql),
            "Generated SQL is not syntactically valid for dialect {:?}:\n{}",
            dialect,
            sql
        );
    }

    /// Property: View SQL should contain CREATE VIEW statement.
    #[test]
    fn prop_view_contains_create_statement(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        let sql_upper = sql.to_uppercase();
        prop_assert!(
            sql_upper.contains("CREATE") && sql_upper.contains("VIEW"),
            "SQL should contain CREATE VIEW"
        );
    }

    /// Property: View SQL should contain SELECT and FROM clauses.
    #[test]
    fn prop_view_contains_select_from(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        let sql_upper = sql.to_uppercase();
        prop_assert!(sql_upper.contains("SELECT"), "SQL should contain SELECT");
        prop_assert!(sql_upper.contains("FROM"), "SQL should contain FROM");
    }

    /// Property: View SQL should end with semicolon.
    #[test]
    fn prop_view_ends_with_semicolon(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        prop_assert!(sql.ends_with(';'), "SQL should end with semicolon");
    }

    /// Property: View should include all entity attributes.
    #[test]
    fn prop_view_includes_all_attributes(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        for attr in &entity.attributes {
            prop_assert!(
                sql.contains(&attr.name),
                "SQL should include attribute: {}",
                attr.name
            );
        }
    }

    /// Property: View should filter by class_uid when source_event_classes is set.
    #[test]
    fn prop_view_filters_by_class_uid(
        dialect in dialect_strategy(),
        entity in entity_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let sql = generator.generate_entity_view(&entity);

        if !entity.source_event_classes.is_empty() {
            prop_assert!(
                sql.contains("class_uid"),
                "SQL should filter by class_uid when source_event_classes is set"
            );
        }
    }

    /// Property: Generated views for a model should all be valid.
    #[test]
    fn prop_all_model_views_are_valid(
        dialect in dialect_strategy(),
        model in model_strategy()
    ) {
        let generator = ViewGenerator::new(dialect);
        let views = generator.generate(&model);

        for (name, sql) in &views.views {
            prop_assert!(
                generator.validate_syntax(sql),
                "View {} is not syntactically valid:\n{}",
                name,
                sql
            );
        }
    }
}
