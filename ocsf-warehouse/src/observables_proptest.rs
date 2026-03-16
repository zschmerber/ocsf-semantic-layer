//! Property-based tests for observables table schema generation.
//!
//! **Feature: ocsf-semantic-layer, Property 17: Observables Table Schema Completeness**
//! **Validates: Requirements 11.1, 11.2**

use proptest::prelude::*;
use ocsf_semantic::WarehouseDialect;

use crate::observables::{
    ObservablesTableConfig, ObservablesTableGenerator, REQUIRED_OBSERVABLE_COLUMNS,
};

/// Strategy for generating warehouse dialects.
fn dialect_strategy() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::Postgres),
    ]
}

/// Strategy for generating valid table names.
fn table_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,30}".prop_map(|s| s)
}

/// Strategy for generating observables table configurations.
fn config_strategy() -> impl Strategy<Value = ObservablesTableConfig> {
    (
        table_name_strategy(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(table_name, add_indexes, add_ingestion_time)| {
            ObservablesTableConfig::new(table_name)
                .with_indexes(add_indexes)
                .with_ingestion_time(add_ingestion_time)
        })
}

proptest! {
    /// **Property 17: Observables Table Schema Completeness**
    ///
    /// *For any* warehouse dialect, the generated observables table schema should
    /// contain columns for: observable_id, type_id, type_name, value, event_uid,
    /// event_class_uid, event_time, and attribute_path.
    ///
    /// **Validates: Requirements 11.1, 11.2**
    #[test]
    fn prop_observables_table_has_all_required_columns(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        // Verify all required columns are present
        prop_assert!(
            schema.has_required_columns(),
            "Missing required columns for dialect {:?}: {:?}",
            dialect,
            schema.missing_required_columns()
        );

        // Verify each required column individually
        for col_name in REQUIRED_OBSERVABLE_COLUMNS {
            prop_assert!(
                schema.table.has_column(col_name),
                "Missing required column '{}' for dialect {:?}",
                col_name,
                dialect
            );
        }
    }

    /// Property: observable_id column should be the primary key.
    #[test]
    fn prop_observable_id_is_primary_key(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        let observable_id_col = schema.table.get_column("observable_id");
        prop_assert!(observable_id_col.is_some(), "observable_id column not found");

        let col = observable_id_col.unwrap();
        prop_assert!(col.is_primary_key, "observable_id should be primary key");
        prop_assert!(!col.nullable, "observable_id should not be nullable");
    }

    /// Property: type_id column should be non-nullable integer.
    #[test]
    fn prop_type_id_is_non_nullable_integer(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        let type_id_col = schema.table.get_column("type_id");
        prop_assert!(type_id_col.is_some(), "type_id column not found");

        let col = type_id_col.unwrap();
        prop_assert!(!col.nullable, "type_id should not be nullable");
        
        // Verify it's an integer type (dialect-specific)
        let valid_int_types = ["INTEGER", "INT64", "BIGINT"];
        prop_assert!(
            valid_int_types.iter().any(|t| col.column_type.contains(t)),
            "type_id should be an integer type, got: {}",
            col.column_type
        );
    }

    /// Property: event_uid column should be non-nullable for reverse lookups.
    #[test]
    fn prop_event_uid_is_non_nullable(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        let event_uid_col = schema.table.get_column("event_uid");
        prop_assert!(event_uid_col.is_some(), "event_uid column not found");

        let col = event_uid_col.unwrap();
        prop_assert!(!col.nullable, "event_uid should not be nullable for reverse lookups");
    }

    /// Property: event_time column should be non-nullable timestamp.
    #[test]
    fn prop_event_time_is_non_nullable_timestamp(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        let event_time_col = schema.table.get_column("event_time");
        prop_assert!(event_time_col.is_some(), "event_time column not found");

        let col = event_time_col.unwrap();
        prop_assert!(!col.nullable, "event_time should not be nullable");
        
        // Verify it's a timestamp type (dialect-specific)
        let valid_ts_types = ["TIMESTAMP", "TIMESTAMP_NTZ"];
        prop_assert!(
            valid_ts_types.iter().any(|t| col.column_type.contains(t)),
            "event_time should be a timestamp type, got: {}",
            col.column_type
        );
    }

    /// Property: Custom configuration should preserve required columns.
    #[test]
    fn prop_custom_config_preserves_required_columns(
        dialect in dialect_strategy(),
        config in config_strategy()
    ) {
        let generator = ObservablesTableGenerator::new(dialect).with_config(config);
        let schema = generator.generate();

        prop_assert!(
            schema.has_required_columns(),
            "Custom config should not remove required columns. Missing: {:?}",
            schema.missing_required_columns()
        );
    }

    /// Property: Table should have partitioning configured.
    #[test]
    fn prop_table_has_partitioning(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        prop_assert!(
            !schema.table.partition_by.is_empty(),
            "Observables table should have partitioning configured"
        );
        prop_assert!(
            schema.table.partition_by.contains(&"event_time".to_string()),
            "Observables table should be partitioned by event_time"
        );
    }

    /// Property: Table should have clustering configured for performance.
    #[test]
    fn prop_table_has_clustering(dialect in dialect_strategy()) {
        let generator = ObservablesTableGenerator::new(dialect);
        let schema = generator.generate();

        prop_assert!(
            !schema.table.cluster_by.is_empty(),
            "Observables table should have clustering configured"
        );
        prop_assert!(
            schema.table.cluster_by.contains(&"type_id".to_string()),
            "Observables table should be clustered by type_id for threat intel matching"
        );
    }

    /// Property: When indexes are enabled, threat intel matching indexes should be present.
    #[test]
    fn prop_indexes_for_threat_intel_matching(dialect in dialect_strategy()) {
        let config = ObservablesTableConfig::default().with_indexes(true);
        let generator = ObservablesTableGenerator::new(dialect).with_config(config);
        let schema = generator.generate();

        // Should have index for type_id + value (threat intel matching)
        let has_type_value_index = schema.indexes.iter().any(|idx| {
            idx.columns.contains(&"type_id".to_string()) && 
            idx.columns.contains(&"value".to_string())
        });
        prop_assert!(
            has_type_value_index,
            "Should have index on (type_id, value) for threat intel matching"
        );

        // Should have index for event_uid (reverse lookups)
        let has_event_uid_index = schema.indexes.iter().any(|idx| {
            idx.columns.contains(&"event_uid".to_string())
        });
        prop_assert!(
            has_event_uid_index,
            "Should have index on event_uid for reverse lookups"
        );
    }

    /// Property: Generated CREATE statement should be non-empty and contain table name.
    #[test]
    fn prop_create_statement_is_valid(
        dialect in dialect_strategy(),
        config in config_strategy()
    ) {
        let generator = ObservablesTableGenerator::new(dialect).with_config(config.clone());
        let schema = generator.generate();

        let sql = schema.to_create_statements();

        prop_assert!(!sql.is_empty(), "CREATE statement should not be empty");
        prop_assert!(
            sql.contains("CREATE TABLE"),
            "Should contain CREATE TABLE"
        );
        prop_assert!(
            sql.contains(&config.table_name),
            "Should contain table name '{}'",
            config.table_name
        );
    }
}
