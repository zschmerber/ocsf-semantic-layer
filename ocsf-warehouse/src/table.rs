//! OCSF table schema generation.
//!
//! This module generates CREATE TABLE statements for OCSF event classes,
//! mapping OCSF types to warehouse-specific types.

use ocsf_core::{Attribute, EventClass, OCSFSchema, Requirement};
use ocsf_semantic::WarehouseDialect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A column definition for a warehouse table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column name.
    pub name: String,
    /// Column type (warehouse-specific).
    pub column_type: String,
    /// Whether the column is nullable.
    pub nullable: bool,
    /// Column description/comment.
    pub description: String,
    /// Whether this is a primary key column.
    pub is_primary_key: bool,
}

impl ColumnDefinition {
    /// Creates a new column definition.
    pub fn new(name: impl Into<String>, column_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            column_type: column_type.into(),
            nullable: true,
            description: String::new(),
            is_primary_key: false,
        }
    }

    /// Sets the nullable flag.
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Marks this column as a primary key.
    pub fn as_primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.nullable = false;
        self
    }
}

/// A table definition for a warehouse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableDefinition {
    /// Table name.
    pub name: String,
    /// Table description/comment.
    pub description: String,
    /// Column definitions.
    pub columns: Vec<ColumnDefinition>,
    /// Partition columns.
    pub partition_by: Vec<String>,
    /// Cluster columns.
    pub cluster_by: Vec<String>,
}

impl TableDefinition {
    /// Creates a new table definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            columns: Vec::new(),
            partition_by: Vec::new(),
            cluster_by: Vec::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a column.
    pub fn add_column(mut self, column: ColumnDefinition) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the columns.
    pub fn with_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.columns = columns;
        self
    }

    /// Sets the partition columns.
    pub fn with_partition_by(mut self, columns: Vec<String>) -> Self {
        self.partition_by = columns;
        self
    }

    /// Sets the cluster columns.
    pub fn with_cluster_by(mut self, columns: Vec<String>) -> Self {
        self.cluster_by = columns;
        self
    }

    /// Generates the CREATE TABLE statement for the given dialect.
    pub fn to_create_statement(&self, dialect: WarehouseDialect) -> String {
        let mut sql = String::new();

        // Add table comment if present
        if !self.description.is_empty() {
            sql.push_str(&format!("{} {}\n", dialect.comment_syntax(), self.description));
        }

        sql.push_str(&format!("CREATE TABLE {} (\n", dialect.quote_identifier(&self.name)));

        // Add columns
        let column_defs: Vec<String> = self
            .columns
            .iter()
            .map(|col| {
                let mut col_def = format!(
                    "    {} {}",
                    dialect.quote_identifier(&col.name),
                    col.column_type
                );
                if !col.nullable {
                    col_def.push_str(" NOT NULL");
                }
                col_def
            })
            .collect();

        sql.push_str(&column_defs.join(",\n"));

        // Add primary key constraint if any
        let pk_columns: Vec<&str> = self
            .columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .collect();

        if !pk_columns.is_empty() {
            sql.push_str(",\n    PRIMARY KEY (");
            sql.push_str(&pk_columns.join(", "));
            sql.push(')');
        }

        sql.push_str("\n)");

        // Add partitioning (dialect-specific)
        if !self.partition_by.is_empty() {
            match dialect {
                WarehouseDialect::BigQuery => {
                    sql.push_str(&format!(
                        "\nPARTITION BY DATE({})",
                        self.partition_by[0]
                    ));
                }
                WarehouseDialect::Snowflake => {
                    // Snowflake uses CLUSTER BY for micro-partitioning
                }
                WarehouseDialect::Databricks => {
                    sql.push_str(&format!(
                        "\nPARTITIONED BY ({})",
                        self.partition_by.join(", ")
                    ));
                }
                WarehouseDialect::Postgres => {
                    sql.push_str(&format!(
                        "\nPARTITION BY RANGE ({})",
                        self.partition_by.join(", ")
                    ));
                }
            }
        }

        // Add clustering
        if !self.cluster_by.is_empty() {
            match dialect {
                WarehouseDialect::BigQuery => {
                    sql.push_str(&format!(
                        "\nCLUSTER BY {}",
                        self.cluster_by.join(", ")
                    ));
                }
                WarehouseDialect::Snowflake => {
                    sql.push_str(&format!(
                        "\nCLUSTER BY ({})",
                        self.cluster_by.join(", ")
                    ));
                }
                _ => {}
            }
        }

        sql.push(';');
        sql
    }

    /// Returns true if this table has a column with the given name.
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.iter().any(|c| c.name == name)
    }

    /// Gets a column by name.
    pub fn get_column(&self, name: &str) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// Maps OCSF types to warehouse-specific SQL types.
pub struct TypeMapper {
    dialect: WarehouseDialect,
}

impl TypeMapper {
    /// Creates a new type mapper for the given dialect.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self { dialect }
    }

    /// Maps an OCSF type to a warehouse SQL type.
    pub fn map_ocsf_type(&self, ocsf_type: &str, is_array: bool) -> String {
        let base_type = self.map_base_type(ocsf_type);
        
        if is_array {
            self.wrap_array_type(&base_type)
        } else {
            base_type
        }
    }

    /// Maps a base OCSF type to a warehouse type.
    fn map_base_type(&self, ocsf_type: &str) -> String {
        match ocsf_type {
            "string_t" | "hostname_t" | "ip_t" | "mac_t" | "email_t" 
            | "url_t" | "file_name_t" | "path_t" | "uuid_t" | "subnet_t"
            | "process_name_t" | "username_t" | "country_t"
            | "city_t" | "region_t" | "fingerprint_t" | "hash_t" => {
                self.dialect.string_type().to_string()
            }
            "integer_t" | "long_t" | "port_t" => {
                self.dialect.integer_type().to_string()
            }
            "float_t" | "double_t" => {
                match self.dialect {
                    WarehouseDialect::BigQuery => "FLOAT64".to_string(),
                    WarehouseDialect::Snowflake => "FLOAT".to_string(),
                    WarehouseDialect::Databricks => "DOUBLE".to_string(),
                    WarehouseDialect::Postgres => "DOUBLE PRECISION".to_string(),
                }
            }
            "boolean_t" => self.dialect.boolean_type().to_string(),
            "timestamp_t" | "datetime_t" => self.dialect.timestamp_type().to_string(),
            "json_t" | "object_t" => self.dialect.json_type().to_string(),
            "bytestring_t" | "binary_t" => {
                match self.dialect {
                    WarehouseDialect::BigQuery => "BYTES".to_string(),
                    WarehouseDialect::Snowflake => "BINARY".to_string(),
                    WarehouseDialect::Databricks => "BINARY".to_string(),
                    WarehouseDialect::Postgres => "BYTEA".to_string(),
                }
            }
            // Default to string for unknown types
            _ => self.dialect.string_type().to_string(),
        }
    }

    /// Wraps a type in an array type for the dialect.
    fn wrap_array_type(&self, base_type: &str) -> String {
        match self.dialect {
            WarehouseDialect::BigQuery => format!("ARRAY<{}>", base_type),
            WarehouseDialect::Snowflake => "ARRAY".to_string(),
            WarehouseDialect::Databricks => format!("ARRAY<{}>", base_type),
            WarehouseDialect::Postgres => format!("{}[]", base_type),
        }
    }
}

/// Generates OCSF table schemas for a warehouse.
pub struct TableGenerator {
    dialect: WarehouseDialect,
    type_mapper: TypeMapper,
    table_prefix: String,
}

impl TableGenerator {
    /// Creates a new table generator for the given dialect.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            type_mapper: TypeMapper::new(dialect),
            table_prefix: "ocsf_".to_string(),
        }
    }

    /// Sets the table name prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.table_prefix = prefix.into();
        self
    }

    /// Generates a table definition for an OCSF event class.
    pub fn generate_event_class_table(
        &self,
        event_class: &EventClass,
        schema: &OCSFSchema,
    ) -> TableDefinition {
        let table_name = format!("{}{}", self.table_prefix, event_class.name);
        
        let mut table = TableDefinition::new(&table_name)
            .with_description(&event_class.description);

        // Add base event columns
        table = self.add_base_event_columns(table);

        // Add event class specific columns
        for (attr_name, attr_ref) in &event_class.attributes {
            // Look up the attribute definition
            if let Some(attr) = schema.get_attribute(attr_name) {
                let column = self.attribute_to_column(attr, attr_ref.requirement);
                table.columns.push(column);
            } else {
                // Create a column with default type if attribute not found
                let column = ColumnDefinition::new(
                    attr_name,
                    self.dialect.string_type(),
                )
                .with_nullable(attr_ref.requirement != Some(Requirement::Required));
                table.columns.push(column);
            }
        }

        // Add partitioning and clustering
        table = table
            .with_partition_by(vec!["time".to_string()])
            .with_cluster_by(vec!["class_uid".to_string(), "category_uid".to_string()]);

        table
    }

    /// Adds base event columns common to all OCSF events.
    fn add_base_event_columns(&self, mut table: TableDefinition) -> TableDefinition {
        // Metadata columns
        table = table
            .add_column(
                ColumnDefinition::new("metadata_uid", self.dialect.string_type())
                    .with_description("Unique event identifier")
                    .as_primary_key(),
            )
            .add_column(
                ColumnDefinition::new("metadata_version", self.dialect.string_type())
                    .with_description("OCSF version"),
            )
            .add_column(
                ColumnDefinition::new("metadata_product", self.dialect.json_type())
                    .with_description("Product information"),
            );

        // Classification columns
        table = table
            .add_column(
                ColumnDefinition::new("class_uid", self.dialect.integer_type())
                    .with_description("Event class UID")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("class_name", self.dialect.string_type())
                    .with_description("Event class name"),
            )
            .add_column(
                ColumnDefinition::new("category_uid", self.dialect.integer_type())
                    .with_description("Category UID")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("category_name", self.dialect.string_type())
                    .with_description("Category name"),
            )
            .add_column(
                ColumnDefinition::new("type_uid", self.dialect.integer_type())
                    .with_description("Type UID (class_uid * 100 + activity_id)"),
            )
            .add_column(
                ColumnDefinition::new("type_name", self.dialect.string_type())
                    .with_description("Type name"),
            )
            .add_column(
                ColumnDefinition::new("activity_id", self.dialect.integer_type())
                    .with_description("Activity ID"),
            )
            .add_column(
                ColumnDefinition::new("activity_name", self.dialect.string_type())
                    .with_description("Activity name"),
            );

        // Time columns
        table = table
            .add_column(
                ColumnDefinition::new("time", self.dialect.timestamp_type())
                    .with_description("Event timestamp")
                    .with_nullable(false),
            )
            .add_column(
                ColumnDefinition::new("time_dt", self.dialect.timestamp_type())
                    .with_description("Event datetime"),
            );

        // Severity columns
        table = table
            .add_column(
                ColumnDefinition::new("severity_id", self.dialect.integer_type())
                    .with_description("Severity ID"),
            )
            .add_column(
                ColumnDefinition::new("severity", self.dialect.string_type())
                    .with_description("Severity name"),
            );

        // Status columns
        table = table
            .add_column(
                ColumnDefinition::new("status_id", self.dialect.integer_type())
                    .with_description("Status ID"),
            )
            .add_column(
                ColumnDefinition::new("status", self.dialect.string_type())
                    .with_description("Status name"),
            )
            .add_column(
                ColumnDefinition::new("status_code", self.dialect.string_type())
                    .with_description("Status code"),
            )
            .add_column(
                ColumnDefinition::new("status_detail", self.dialect.string_type())
                    .with_description("Status detail"),
            );

        // Message and raw data
        table = table
            .add_column(
                ColumnDefinition::new("message", self.dialect.string_type())
                    .with_description("Event message"),
            )
            .add_column(
                ColumnDefinition::new("raw_data", self.dialect.string_type())
                    .with_description("Raw event data"),
            );

        // Observables array
        table = table.add_column(
            ColumnDefinition::new(
                "observables",
                self.type_mapper.map_ocsf_type("json_t", true),
            )
            .with_description("Array of observable objects"),
        );

        table
    }

    /// Converts an OCSF attribute to a column definition.
    fn attribute_to_column(
        &self,
        attr: &Attribute,
        requirement_override: Option<Requirement>,
    ) -> ColumnDefinition {
        let column_type = self.type_mapper.map_ocsf_type(&attr.attr_type, attr.is_array);
        let requirement = requirement_override.unwrap_or(attr.requirement);
        
        ColumnDefinition::new(&attr.name, column_type)
            .with_description(&attr.description)
            .with_nullable(requirement != Requirement::Required)
    }

    /// Generates table definitions for all event classes in a schema.
    pub fn generate_all_tables(&self, schema: &OCSFSchema) -> Vec<TableDefinition> {
        schema
            .all_event_classes()
            .map(|ec| self.generate_event_class_table(ec, schema))
            .collect()
    }

    /// Generates CREATE TABLE statements for all event classes.
    pub fn generate_all_create_statements(&self, schema: &OCSFSchema) -> String {
        let tables = self.generate_all_tables(schema);
        tables
            .iter()
            .map(|t| t.to_create_statement(self.dialect))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Collection of generated table definitions.
#[derive(Debug, Clone, Default)]
pub struct TableDefinitions {
    /// All table definitions.
    pub tables: Vec<TableDefinition>,
    /// Index by table name.
    pub by_name: HashMap<String, usize>,
}

impl TableDefinitions {
    /// Creates a new empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a table definition.
    pub fn add(&mut self, table: TableDefinition) {
        let idx = self.tables.len();
        self.by_name.insert(table.name.clone(), idx);
        self.tables.push(table);
    }

    /// Gets a table by name.
    pub fn get(&self, name: &str) -> Option<&TableDefinition> {
        self.by_name.get(name).map(|&idx| &self.tables[idx])
    }

    /// Returns all tables.
    pub fn all(&self) -> &[TableDefinition] {
        &self.tables
    }

    /// Returns the number of tables.
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Returns true if there are no tables.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");
        
        // Add a test attribute
        schema.add_attribute(Attribute {
            name: "user".to_string(),
            attr_type: "object_t".to_string(),
            caption: "User".to_string(),
            description: "The user object".to_string(),
            requirement: Requirement::Recommended,
            observable: None,
            is_array: false,
            object_type: Some("user".to_string()),
            enum_values: HashMap::new(),
            default: None,
        });

        // Add a test event class
        let mut attributes = HashMap::new();
        attributes.insert(
            "user".to_string(),
            ocsf_core::AttributeRef {
                name: "user".to_string(),
                requirement: Some(Requirement::Required),
                description: None,
                group: None,
            },
        );

        schema.add_event_class(EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Authentication events".to_string(),
            attributes,
            observables: vec![],
            extends: None,
            profiles: vec![],
        });

        schema
    }

    #[test]
    fn test_type_mapper_string_types() {
        let mapper = TypeMapper::new(WarehouseDialect::Snowflake);
        assert_eq!(mapper.map_ocsf_type("string_t", false), "VARCHAR");
        assert_eq!(mapper.map_ocsf_type("hostname_t", false), "VARCHAR");
        assert_eq!(mapper.map_ocsf_type("ip_t", false), "VARCHAR");
    }

    #[test]
    fn test_type_mapper_numeric_types() {
        let mapper = TypeMapper::new(WarehouseDialect::BigQuery);
        assert_eq!(mapper.map_ocsf_type("integer_t", false), "INT64");
        assert_eq!(mapper.map_ocsf_type("float_t", false), "FLOAT64");
        assert_eq!(mapper.map_ocsf_type("boolean_t", false), "BOOL");
    }

    #[test]
    fn test_type_mapper_array_types() {
        let mapper = TypeMapper::new(WarehouseDialect::BigQuery);
        assert_eq!(mapper.map_ocsf_type("string_t", true), "ARRAY<STRING>");
        
        let mapper = TypeMapper::new(WarehouseDialect::Postgres);
        assert_eq!(mapper.map_ocsf_type("string_t", true), "TEXT[]");
    }

    #[test]
    fn test_column_definition() {
        let col = ColumnDefinition::new("test_col", "VARCHAR")
            .with_description("Test column")
            .with_nullable(false)
            .as_primary_key();

        assert_eq!(col.name, "test_col");
        assert_eq!(col.column_type, "VARCHAR");
        assert!(!col.nullable);
        assert!(col.is_primary_key);
    }

    #[test]
    fn test_table_definition_create_statement() {
        let table = TableDefinition::new("test_table")
            .with_description("Test table")
            .add_column(
                ColumnDefinition::new("id", "INTEGER")
                    .as_primary_key(),
            )
            .add_column(
                ColumnDefinition::new("name", "VARCHAR")
                    .with_nullable(true),
            );

        let sql = table.to_create_statement(WarehouseDialect::Snowflake);
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("\"test_table\""));
        assert!(sql.contains("\"id\" INTEGER NOT NULL"));
        assert!(sql.contains("\"name\" VARCHAR"));
        assert!(sql.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn test_table_generator_event_class() {
        let schema = create_test_schema();
        let generator = TableGenerator::new(WarehouseDialect::Snowflake);
        
        let event_class = schema.get_event_class(3002).unwrap();
        let table = generator.generate_event_class_table(event_class, &schema);

        assert_eq!(table.name, "ocsf_authentication");
        assert!(table.has_column("metadata_uid"));
        assert!(table.has_column("class_uid"));
        assert!(table.has_column("time"));
        assert!(table.has_column("user"));
    }

    #[test]
    fn test_table_definitions_collection() {
        let mut defs = TableDefinitions::new();
        
        defs.add(TableDefinition::new("table1"));
        defs.add(TableDefinition::new("table2"));

        assert_eq!(defs.len(), 2);
        assert!(defs.get("table1").is_some());
        assert!(defs.get("table2").is_some());
        assert!(defs.get("table3").is_none());
    }

    #[test]
    fn test_dialect_specific_create_statements() {
        let table = TableDefinition::new("events")
            .add_column(ColumnDefinition::new("id", "INTEGER").as_primary_key())
            .add_column(ColumnDefinition::new("time", "TIMESTAMP"))
            .with_partition_by(vec!["time".to_string()])
            .with_cluster_by(vec!["id".to_string()]);

        // BigQuery
        let bq_sql = table.to_create_statement(WarehouseDialect::BigQuery);
        assert!(bq_sql.contains("PARTITION BY DATE(time)"));
        assert!(bq_sql.contains("CLUSTER BY id"));

        // Snowflake
        let sf_sql = table.to_create_statement(WarehouseDialect::Snowflake);
        assert!(sf_sql.contains("CLUSTER BY (id)"));
    }
}
