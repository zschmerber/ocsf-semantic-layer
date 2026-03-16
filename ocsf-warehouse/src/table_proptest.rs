//! Property-based tests for table schema generation.

use proptest::prelude::*;
use ocsf_semantic::WarehouseDialect;

use crate::table::{ColumnDefinition, TableDefinition, TypeMapper};

/// Strategy for generating warehouse dialects.
fn dialect_strategy() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::Postgres),
    ]
}

/// Strategy for generating OCSF type names.
fn ocsf_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("string_t".to_string()),
        Just("integer_t".to_string()),
        Just("boolean_t".to_string()),
        Just("timestamp_t".to_string()),
        Just("float_t".to_string()),
        Just("json_t".to_string()),
    ]
}

proptest! {
    /// Property: Type mapper should always return a non-empty type string.
    #[test]
    fn prop_type_mapper_returns_non_empty(
        dialect in dialect_strategy(),
        ocsf_type in ocsf_type_strategy(),
        is_array in any::<bool>()
    ) {
        let mapper = TypeMapper::new(dialect);
        let result = mapper.map_ocsf_type(&ocsf_type, is_array);
        
        prop_assert!(!result.is_empty(), "Type mapper returned empty string");
    }

    /// Property: Table definition should generate valid CREATE statement.
    #[test]
    fn prop_table_generates_valid_create(dialect in dialect_strategy()) {
        let table = TableDefinition::new("test_table")
            .add_column(ColumnDefinition::new("id", "INTEGER").as_primary_key())
            .add_column(ColumnDefinition::new("name", "VARCHAR"));

        let sql = table.to_create_statement(dialect);

        prop_assert!(sql.contains("CREATE TABLE"), "Missing CREATE TABLE");
        prop_assert!(sql.contains("test_table"), "Missing table name");
        prop_assert!(sql.ends_with(';'), "Missing semicolon");
    }

    /// Property: Primary key columns should be NOT NULL.
    #[test]
    fn prop_primary_key_is_not_null(dialect in dialect_strategy()) {
        let table = TableDefinition::new("test")
            .add_column(ColumnDefinition::new("pk_col", "INTEGER").as_primary_key());

        let sql = table.to_create_statement(dialect);

        prop_assert!(sql.contains("NOT NULL"), "Primary key should be NOT NULL");
    }
}
