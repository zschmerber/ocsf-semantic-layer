//! Observables table schema generation.
//!
//! This module generates the dedicated observables table schema for hot path
//! analytics and threat intelligence matching.

use ocsf_semantic::WarehouseDialect;
use serde::{Deserialize, Serialize};

use crate::table::{ColumnDefinition, TableDefinition};

/// Required columns for the observables table.
/// These columns are mandated by Requirements 11.1 and 11.2.
pub const REQUIRED_OBSERVABLE_COLUMNS: &[&str] = &[
    "observable_id",
    "type_id",
    "type_name",
    "value",
    "event_uid",
    "event_class_uid",
    "event_time",
    "attribute_path",
];

/// Configuration for observables table generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservablesTableConfig {
    /// Table name.
    pub table_name: String,
    /// Whether to add indexes for threat intel matching.
    pub add_indexes: bool,
    /// Whether to add ingestion timestamp column.
    pub add_ingestion_time: bool,
    /// Additional columns to include.
    pub additional_columns: Vec<ColumnDefinition>,
}

impl Default for ObservablesTableConfig {
    fn default() -> Self {
        Self {
            table_name: "ocsf_observables".to_string(),
            add_indexes: true,
            add_ingestion_time: true,
            additional_columns: Vec::new(),
        }
    }
}

impl ObservablesTableConfig {
    /// Creates a new configuration with the given table name.
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            ..Default::default()
        }
    }

    /// Sets whether to add indexes.
    pub fn with_indexes(mut self, add_indexes: bool) -> Self {
        self.add_indexes = add_indexes;
        self
    }

    /// Sets whether to add ingestion time column.
    pub fn with_ingestion_time(mut self, add_ingestion_time: bool) -> Self {
        self.add_ingestion_time = add_ingestion_time;
        self
    }

    /// Adds an additional column.
    pub fn add_column(mut self, column: ColumnDefinition) -> Self {
        self.additional_columns.push(column);
        self
    }
}

/// Index definition for the observables table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexDefinition {
    /// Index name.
    pub name: String,
    /// Columns included in the index.
    pub columns: Vec<String>,
    /// Whether this is a unique index.
    pub unique: bool,
    /// Index description.
    pub description: String,
}

impl IndexDefinition {
    /// Creates a new index definition.
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            unique: false,
            description: String::new(),
        }
    }

    /// Marks this as a unique index.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Generates the CREATE INDEX statement for the given dialect.
    pub fn to_create_statement(&self, table_name: &str, dialect: WarehouseDialect) -> String {
        let unique_str = if self.unique { "UNIQUE " } else { "" };
        let columns = self.columns.join(", ");

        match dialect {
            WarehouseDialect::BigQuery => {
                // BigQuery doesn't support traditional indexes
                format!(
                    "{} BigQuery: Consider clustering by ({}) for index-like performance",
                    dialect.comment_syntax(),
                    columns
                )
            }
            WarehouseDialect::Snowflake => {
                // Snowflake uses clustering keys instead of indexes
                format!(
                    "{} Snowflake: Consider CLUSTER BY ({}) for index-like performance",
                    dialect.comment_syntax(),
                    columns
                )
            }
            WarehouseDialect::Databricks => {
                // Databricks uses Z-ORDER for optimization
                format!(
                    "{} Databricks: Consider ZORDER BY ({}) for index-like performance",
                    dialect.comment_syntax(),
                    columns
                )
            }
            WarehouseDialect::Postgres => {
                format!(
                    "CREATE {}INDEX {} ON {} ({});",
                    unique_str, self.name, table_name, columns
                )
            }
        }
    }
}

/// Generated observables table schema.
#[derive(Debug, Clone)]
pub struct ObservablesTableSchema {
    /// The table definition.
    pub table: TableDefinition,
    /// Index definitions.
    pub indexes: Vec<IndexDefinition>,
    /// The dialect used.
    pub dialect: WarehouseDialect,
}

impl ObservablesTableSchema {
    /// Generates the complete CREATE TABLE statement with indexes.
    pub fn to_create_statements(&self) -> String {
        let mut statements = vec![self.table.to_create_statement(self.dialect)];

        for index in &self.indexes {
            statements.push(index.to_create_statement(&self.table.name, self.dialect));
        }

        statements.join("\n\n")
    }

    /// Returns true if the table has all required columns.
    pub fn has_required_columns(&self) -> bool {
        REQUIRED_OBSERVABLE_COLUMNS
            .iter()
            .all(|col| self.table.has_column(col))
    }

    /// Returns the list of missing required columns.
    pub fn missing_required_columns(&self) -> Vec<&'static str> {
        REQUIRED_OBSERVABLE_COLUMNS
            .iter()
            .filter(|col| !self.table.has_column(col))
            .copied()
            .collect()
    }
}

/// Generates observables table schemas for warehouses.
pub struct ObservablesTableGenerator {
    dialect: WarehouseDialect,
    config: ObservablesTableConfig,
}

impl ObservablesTableGenerator {
    /// Creates a new generator with the given dialect.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            config: ObservablesTableConfig::default(),
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: ObservablesTableConfig) -> Self {
        self.config = config;
        self
    }

    /// Generates the observables table schema.
    pub fn generate(&self) -> ObservablesTableSchema {
        let table = self.generate_table();
        let indexes = if self.config.add_indexes {
            self.generate_indexes()
        } else {
            Vec::new()
        };

        ObservablesTableSchema {
            table,
            indexes,
            dialect: self.dialect,
        }
    }

    /// Generates the table definition.
    fn generate_table(&self) -> TableDefinition {
        let mut table = TableDefinition::new(&self.config.table_name)
            .with_description("Dedicated observables table for hot path analytics and threat intel matching");

        // Observable identification columns
        table = table
            .add_column(
                ColumnDefinition::new("observable_id", self.dialect.string_type())
                    .with_description("Unique ID for this observable instance")
                    .as_primary_key(),
            )
            .add_column(
                ColumnDefinition::new("type_id", self.dialect.integer_type())
                    .with_description("OCSF observable type_id")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("type_name", self.dialect.string_type())
                    .with_description("Human-readable type name")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("value", self.dialect.string_type())
                    .with_description("Observable value (for primitives)"),
            );

        // Source event reference columns for reverse lookup
        table = table
            .add_column(
                ColumnDefinition::new("event_uid", self.dialect.string_type())
                    .with_description("Source event metadata.uid")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("event_class_uid", self.dialect.integer_type())
                    .with_description("Source event class_uid")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("event_time", self.dialect.timestamp_type())
                    .with_description("Source event time")
                    .with_nullable(false),
            );

        // Path reference column
        table = table.add_column(
            ColumnDefinition::new("attribute_path", self.dialect.string_type())
                .with_description("Path to attribute in source event")
                .with_nullable(false),
        );

        // Optional ingestion time column
        if self.config.add_ingestion_time {
            table = table.add_column(
                ColumnDefinition::new("ingestion_time", self.dialect.timestamp_type())
                    .with_description("Time when the observable was ingested"),
            );
        }

        // Add any additional columns
        for col in &self.config.additional_columns {
            table.columns.push(col.clone());
        }

        // Add partitioning and clustering
        table = table
            .with_partition_by(vec!["event_time".to_string()])
            .with_cluster_by(vec!["type_id".to_string(), "value".to_string()]);

        table
    }

    /// Generates index definitions for threat intel matching.
    fn generate_indexes(&self) -> Vec<IndexDefinition> {
        vec![
            IndexDefinition::new(
                "idx_observables_type_value",
                vec!["type_id".to_string(), "value".to_string()],
            )
            .with_description("Index for fast threat intel lookups"),
            IndexDefinition::new(
                "idx_observables_event_uid",
                vec!["event_uid".to_string()],
            )
            .with_description("Index for reverse lookups"),
            IndexDefinition::new(
                "idx_observables_event_time",
                vec!["event_time".to_string()],
            )
            .with_description("Index for time-based queries"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observables_table_has_required_columns() {
        let generator = ObservablesTableGenerator::new(WarehouseDialect::Snowflake);
        let schema = generator.generate();

        assert!(schema.has_required_columns());
        assert!(schema.missing_required_columns().is_empty());
    }

    #[test]
    fn test_observables_table_columns() {
        let generator = ObservablesTableGenerator::new(WarehouseDialect::BigQuery);
        let schema = generator.generate();

        // Check all required columns exist
        for col_name in REQUIRED_OBSERVABLE_COLUMNS {
            assert!(
                schema.table.has_column(col_name),
                "Missing required column: {}",
                col_name
            );
        }

        // Check column types
        let type_id_col = schema.table.get_column("type_id").unwrap();
        assert_eq!(type_id_col.column_type, "INT64");
        assert!(!type_id_col.nullable);

        let value_col = schema.table.get_column("value").unwrap();
        assert_eq!(value_col.column_type, "STRING");
        assert!(value_col.nullable);
    }

    #[test]
    fn test_observables_table_indexes() {
        let generator = ObservablesTableGenerator::new(WarehouseDialect::Postgres);
        let schema = generator.generate();

        assert_eq!(schema.indexes.len(), 3);

        // Check type_value index
        let type_value_idx = schema
            .indexes
            .iter()
            .find(|i| i.name == "idx_observables_type_value")
            .unwrap();
        assert_eq!(type_value_idx.columns, vec!["type_id", "value"]);

        // Check event_uid index
        let event_uid_idx = schema
            .indexes
            .iter()
            .find(|i| i.name == "idx_observables_event_uid")
            .unwrap();
        assert_eq!(event_uid_idx.columns, vec!["event_uid"]);
    }

    #[test]
    fn test_observables_table_create_statement() {
        let generator = ObservablesTableGenerator::new(WarehouseDialect::Postgres);
        let schema = generator.generate();

        let sql = schema.to_create_statements();

        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("ocsf_observables"));
        assert!(sql.contains("observable_id"));
        assert!(sql.contains("type_id"));
        assert!(sql.contains("event_uid"));
        assert!(sql.contains("CREATE INDEX"));
    }

    #[test]
    fn test_observables_table_config() {
        let config = ObservablesTableConfig::new("custom_observables")
            .with_indexes(false)
            .with_ingestion_time(false)
            .add_column(ColumnDefinition::new("custom_col", "VARCHAR"));

        let generator = ObservablesTableGenerator::new(WarehouseDialect::Snowflake)
            .with_config(config);
        let schema = generator.generate();

        assert_eq!(schema.table.name, "custom_observables");
        assert!(schema.indexes.is_empty());
        assert!(!schema.table.has_column("ingestion_time"));
        assert!(schema.table.has_column("custom_col"));
    }

    #[test]
    fn test_observables_table_partitioning() {
        let generator = ObservablesTableGenerator::new(WarehouseDialect::BigQuery);
        let schema = generator.generate();

        assert_eq!(schema.table.partition_by, vec!["event_time"]);
        assert_eq!(schema.table.cluster_by, vec!["type_id", "value"]);
    }

    #[test]
    fn test_dialect_specific_index_statements() {
        let index = IndexDefinition::new("test_idx", vec!["col1".to_string(), "col2".to_string()]);

        // Postgres generates actual CREATE INDEX
        let pg_sql = index.to_create_statement("test_table", WarehouseDialect::Postgres);
        assert!(pg_sql.contains("CREATE INDEX test_idx ON test_table"));

        // BigQuery generates a comment
        let bq_sql = index.to_create_statement("test_table", WarehouseDialect::BigQuery);
        assert!(bq_sql.contains("--"));
        assert!(bq_sql.contains("clustering"));
    }

    #[test]
    fn test_all_dialects_generate_valid_schema() {
        let dialects = [
            WarehouseDialect::Snowflake,
            WarehouseDialect::BigQuery,
            WarehouseDialect::Databricks,
            WarehouseDialect::Postgres,
        ];

        for dialect in dialects {
            let generator = ObservablesTableGenerator::new(dialect);
            let schema = generator.generate();

            assert!(
                schema.has_required_columns(),
                "Dialect {:?} missing required columns: {:?}",
                dialect,
                schema.missing_required_columns()
            );
        }
    }
}
