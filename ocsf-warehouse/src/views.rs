//! SQL view generation for semantic entities.
//!
//! This module generates CREATE VIEW statements for semantic entities,
//! including OCSF field mappings and expressions.

use ocsf_semantic::{SemanticEntity, SemanticModel, WarehouseDialect};
use std::collections::HashMap;

/// SQL view definitions.
#[derive(Debug, Clone, Default)]
pub struct ViewDefinitions {
    /// View definitions (view name -> SQL).
    pub views: HashMap<String, String>,
}

impl ViewDefinitions {
    /// Creates new empty view definitions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a view definition.
    pub fn add(&mut self, name: String, sql: String) {
        self.views.insert(name, sql);
    }

    /// Gets a view by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.views.get(name)
    }

    /// Returns all view SQL statements concatenated.
    pub fn to_sql(&self) -> String {
        self.views.values().cloned().collect::<Vec<_>>().join("\n\n")
    }
}

/// Configuration for view generation.
#[derive(Debug, Clone)]
pub struct ViewConfig {
    /// View name prefix.
    pub view_prefix: String,
    /// Source table prefix.
    pub table_prefix: String,
    /// Schema name.
    pub schema: Option<String>,
    /// Whether to use CREATE OR REPLACE.
    pub create_or_replace: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            view_prefix: "v_".to_string(),
            table_prefix: "ocsf_".to_string(),
            schema: None,
            create_or_replace: true,
        }
    }
}

/// Generates SQL views for semantic entities.
pub struct ViewGenerator {
    dialect: WarehouseDialect,
    config: ViewConfig,
}

impl ViewGenerator {
    /// Creates a new view generator.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            config: ViewConfig::default(),
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: ViewConfig) -> Self {
        self.config = config;
        self
    }

    /// Generates views for all entities in a semantic model.
    pub fn generate(&self, model: &SemanticModel) -> ViewDefinitions {
        let mut views = ViewDefinitions::new();

        for entity in &model.entities {
            let view_name = format!("{}{}", self.config.view_prefix, entity.name);
            let sql = self.generate_entity_view(entity);
            views.add(view_name, sql);
        }

        views
    }

    /// Generates a view for a single entity.
    pub fn generate_entity_view(&self, entity: &SemanticEntity) -> String {
        let view_name = format!("{}{}", self.config.view_prefix, entity.name);
        let qualified_view_name = self.qualify_name(&view_name);

        let mut sql = String::new();

        // View header
        let create_stmt = if self.config.create_or_replace {
            "CREATE OR REPLACE VIEW"
        } else {
            "CREATE VIEW"
        };
        sql.push_str(&format!("{} {} AS\n", create_stmt, qualified_view_name));

        // SELECT clause
        sql.push_str("SELECT\n");

        let mut columns = vec![
            "    metadata_uid".to_string(),
            "    class_uid".to_string(),
            "    category_uid".to_string(),
            "    time".to_string(),
            "    severity_id".to_string(),
            "    status_id".to_string(),
        ];

        // Add entity attributes with their mappings
        for attr in &entity.attributes {
            let column_expr = self.attribute_to_column_expr(attr);
            columns.push(column_expr);
        }

        sql.push_str(&columns.join(",\n"));
        sql.push('\n');

        // FROM clause
        let source_table = self.get_source_table(entity);
        sql.push_str(&format!("FROM {}", source_table));

        // WHERE clause for class_uid filter
        if !entity.source_event_classes.is_empty() {
            sql.push('\n');
            if entity.source_event_classes.len() == 1 {
                sql.push_str(&format!(
                    "WHERE class_uid = {}",
                    entity.source_event_classes[0]
                ));
            } else {
                let class_uids: Vec<String> = entity
                    .source_event_classes
                    .iter()
                    .map(|uid| uid.to_string())
                    .collect();
                sql.push_str(&format!("WHERE class_uid IN ({})", class_uids.join(", ")));
            }
        }

        sql.push(';');
        sql
    }

    /// Converts an attribute to a column expression.
    fn attribute_to_column_expr(&self, attr: &ocsf_semantic::SemanticAttribute) -> String {
        let expr = if let Some(ref field) = attr.ocsf_mapping.field {
            self.field_to_sql_expr(field)
        } else if let Some(ref expression) = attr.ocsf_mapping.expression {
            expression.clone()
        } else {
            "NULL".to_string()
        };

        format!("    {} AS {}", expr, self.dialect.quote_identifier(&attr.name))
    }

    /// Converts a field path to a SQL expression.
    fn field_to_sql_expr(&self, field: &str) -> String {
        // Handle nested field paths (e.g., "actor.user.email_addr")
        if field.contains('.') {
            self.dialect.json_extract("raw_data", field)
        } else {
            field.to_string()
        }
    }

    /// Gets the source table for an entity.
    fn get_source_table(&self, entity: &SemanticEntity) -> String {
        // Use a union of all source event class tables, or a generic events table
        if entity.source_event_classes.is_empty() {
            self.qualify_name(&format!("{}events", self.config.table_prefix))
        } else if entity.source_event_classes.len() == 1 {
            let class_uid = entity.source_event_classes[0];
            self.qualify_name(&format!("{}class_{}", self.config.table_prefix, class_uid))
        } else {
            // For multiple classes, use a generic events table with WHERE filter
            self.qualify_name(&format!("{}events", self.config.table_prefix))
        }
    }

    /// Qualifies a name with schema if configured.
    fn qualify_name(&self, name: &str) -> String {
        if let Some(ref schema) = self.config.schema {
            format!("{}.{}", schema, name)
        } else {
            name.to_string()
        }
    }

    /// Validates that a generated view SQL is syntactically valid.
    pub fn validate_syntax(&self, sql: &str) -> bool {
        // Basic syntax validation
        let sql_upper = sql.to_uppercase();
        sql_upper.contains("CREATE") 
            && sql_upper.contains("VIEW")
            && sql_upper.contains("SELECT")
            && sql_upper.contains("FROM")
            && sql.ends_with(';')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::{OCSFMapping, SemanticAttribute, SemanticType};

    fn create_test_entity() -> SemanticEntity {
        SemanticEntity::new("authentication_event")
            .with_description("Authentication events")
            .with_source_event_classes(vec![3002])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_type(SemanticType::String)
                    .with_mapping(OCSFMapping::from_field("actor.user.email_addr"))
                    .as_dimension(),
            )
            .add_attribute(
                SemanticAttribute::new("auth_result")
                    .with_type(SemanticType::String)
                    .with_mapping(OCSFMapping::from_expression(
                        "CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END",
                    )),
            )
    }

    #[test]
    fn test_view_generator_creates_view() {
        let entity = create_test_entity();
        let generator = ViewGenerator::new(WarehouseDialect::Snowflake);
        let sql = generator.generate_entity_view(&entity);

        assert!(sql.contains("CREATE OR REPLACE VIEW"));
        assert!(sql.contains("v_authentication_event"));
        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM"));
    }

    #[test]
    fn test_view_contains_attributes() {
        let entity = create_test_entity();
        let generator = ViewGenerator::new(WarehouseDialect::Snowflake);
        let sql = generator.generate_entity_view(&entity);

        assert!(sql.contains("user_email"));
        assert!(sql.contains("auth_result"));
    }

    #[test]
    fn test_view_contains_class_uid_filter() {
        let entity = create_test_entity();
        let generator = ViewGenerator::new(WarehouseDialect::Snowflake);
        let sql = generator.generate_entity_view(&entity);

        assert!(sql.contains("WHERE class_uid = 3002"));
    }

    #[test]
    fn test_view_syntax_validation() {
        let entity = create_test_entity();
        let generator = ViewGenerator::new(WarehouseDialect::Postgres);
        let sql = generator.generate_entity_view(&entity);

        assert!(generator.validate_syntax(&sql));
    }

    #[test]
    fn test_view_definitions_collection() {
        let model = SemanticModel::new("test")
            .add_entity(create_test_entity());

        let generator = ViewGenerator::new(WarehouseDialect::BigQuery);
        let views = generator.generate(&model);

        assert!(views.get("v_authentication_event").is_some());
    }

    #[test]
    fn test_dialect_specific_json_extraction() {
        let entity = create_test_entity();

        // BigQuery
        let bq_gen = ViewGenerator::new(WarehouseDialect::BigQuery);
        let bq_sql = bq_gen.generate_entity_view(&entity);
        assert!(bq_sql.contains("JSON_EXTRACT_SCALAR"));

        // Snowflake
        let sf_gen = ViewGenerator::new(WarehouseDialect::Snowflake);
        let sf_sql = sf_gen.generate_entity_view(&entity);
        assert!(sf_sql.contains("raw_data:"));

        // Postgres
        let pg_gen = ViewGenerator::new(WarehouseDialect::Postgres);
        let pg_sql = pg_gen.generate_entity_view(&entity);
        assert!(pg_sql.contains("->>"));
    }
}


/// Refresh strategy for materialized views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RefreshStrategy {
    /// Manual refresh only.
    #[default]
    Manual,
    /// Automatic refresh on schedule.
    Scheduled,
    /// Incremental refresh.
    Incremental,
    /// Refresh on commit (for supported warehouses).
    OnCommit,
}

use serde::{Deserialize, Serialize};

/// Configuration for materialized view generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedViewConfig {
    /// View name.
    pub name: String,
    /// Refresh strategy.
    pub refresh_strategy: RefreshStrategy,
    /// Refresh interval in minutes (for scheduled refresh).
    pub refresh_interval_minutes: Option<u32>,
    /// Cluster columns for optimization.
    pub cluster_by: Vec<String>,
    /// Partition column.
    pub partition_by: Option<String>,
}

impl Default for MaterializedViewConfig {
    fn default() -> Self {
        Self {
            name: "mv_ocsf_observables".to_string(),
            refresh_strategy: RefreshStrategy::default(),
            cluster_by: vec!["type_id".to_string(), "value".to_string()],
            partition_by: Some("event_time".to_string()),
            refresh_interval_minutes: None,
        }
    }
}

/// Materialized view definition.
#[derive(Debug, Clone)]
pub struct MaterializedViewDefinition {
    /// View name.
    pub name: String,
    /// The SQL for the view.
    pub sql: String,
    /// Refresh strategy.
    pub refresh_strategy: RefreshStrategy,
    /// Dialect used.
    pub dialect: WarehouseDialect,
}

impl MaterializedViewDefinition {
    /// Generates the CREATE MATERIALIZED VIEW statement.
    pub fn to_create_statement(&self) -> String {
        self.sql.clone()
    }

    /// Generates the REFRESH statement.
    pub fn to_refresh_statement(&self) -> String {
        match self.dialect {
            WarehouseDialect::Snowflake => {
                format!("ALTER MATERIALIZED VIEW {} REFRESH;", self.name)
            }
            WarehouseDialect::BigQuery => {
                format!("CALL BQ.REFRESH_MATERIALIZED_VIEW('{}');", self.name)
            }
            WarehouseDialect::Databricks => {
                format!("REFRESH MATERIALIZED VIEW {};", self.name)
            }
            WarehouseDialect::Postgres => {
                format!("REFRESH MATERIALIZED VIEW {};", self.name)
            }
        }
    }
}

/// Generates materialized views for observables.
pub struct MaterializedViewGenerator {
    dialect: WarehouseDialect,
    config: MaterializedViewConfig,
    source_table: String,
}

impl MaterializedViewGenerator {
    /// Creates a new materialized view generator.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            config: MaterializedViewConfig::default(),
            source_table: "ocsf_events".to_string(),
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: MaterializedViewConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the source table.
    pub fn with_source_table(mut self, table: impl Into<String>) -> Self {
        self.source_table = table.into();
        self
    }

    /// Generates the materialized view for observables.
    pub fn generate_observables_mv(&self) -> MaterializedViewDefinition {
        let sql = self.generate_mv_sql();
        
        MaterializedViewDefinition {
            name: self.config.name.clone(),
            sql,
            refresh_strategy: self.config.refresh_strategy,
            dialect: self.dialect,
        }
    }

    /// Generates the SQL for the materialized view.
    fn generate_mv_sql(&self) -> String {
        let mut sql = String::new();

        // Add comment
        sql.push_str(&format!(
            "{} Materialized view for observables - optimized for threat intel matching\n",
            self.dialect.comment_syntax()
        ));

        // CREATE statement varies by dialect
        match self.dialect {
            WarehouseDialect::Snowflake => {
                sql.push_str(&format!(
                    "CREATE OR REPLACE MATERIALIZED VIEW {}\n",
                    self.config.name
                ));
                if !self.config.cluster_by.is_empty() {
                    sql.push_str(&format!(
                        "CLUSTER BY ({})\n",
                        self.config.cluster_by.join(", ")
                    ));
                }
            }
            WarehouseDialect::BigQuery => {
                sql.push_str(&format!(
                    "CREATE MATERIALIZED VIEW IF NOT EXISTS {}\n",
                    self.config.name
                ));
                if let Some(ref partition) = self.config.partition_by {
                    sql.push_str(&format!("PARTITION BY DATE({})\n", partition));
                }
                if !self.config.cluster_by.is_empty() {
                    sql.push_str(&format!(
                        "CLUSTER BY {}\n",
                        self.config.cluster_by.join(", ")
                    ));
                }
                sql.push_str("OPTIONS(\n");
                sql.push_str("  enable_refresh = true,\n");
                match self.config.refresh_strategy {
                    RefreshStrategy::Scheduled => {
                        let interval = self.config.refresh_interval_minutes.unwrap_or(60);
                        sql.push_str(&format!("  refresh_interval_minutes = {}\n", interval));
                    }
                    _ => {
                        sql.push_str("  refresh_interval_minutes = 60\n");
                    }
                }
                sql.push_str(")\n");
            }
            WarehouseDialect::Databricks => {
                sql.push_str(&format!(
                    "CREATE MATERIALIZED VIEW IF NOT EXISTS {}\n",
                    self.config.name
                ));
                if let Some(ref partition) = self.config.partition_by {
                    sql.push_str(&format!("PARTITIONED BY ({})\n", partition));
                }
            }
            WarehouseDialect::Postgres => {
                sql.push_str(&format!(
                    "CREATE MATERIALIZED VIEW IF NOT EXISTS {}\n",
                    self.config.name
                ));
            }
        }

        sql.push_str("AS\n");
        sql.push_str(&self.generate_select_sql());
        sql.push(';');

        sql
    }

    /// Generates the SELECT SQL for the materialized view.
    fn generate_select_sql(&self) -> String {
        format!(
            r#"SELECT
    CONCAT(metadata_uid, '_', attribute_path) AS observable_id,
    o.type_id,
    o.type_name,
    o.value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    o.attribute_path,
    {} AS ingestion_time
FROM {},
    UNNEST(observables) AS o
WHERE o.value IS NOT NULL"#,
            self.dialect.current_timestamp_fn(),
            self.source_table
        )
    }
}

#[cfg(test)]
mod materialized_view_tests {
    use super::*;

    #[test]
    fn test_materialized_view_generator() {
        let generator = MaterializedViewGenerator::new(WarehouseDialect::Snowflake);
        let mv = generator.generate_observables_mv();

        assert!(mv.sql.contains("CREATE"));
        assert!(mv.sql.contains("MATERIALIZED VIEW"));
        assert!(mv.sql.contains("mv_ocsf_observables"));
    }

    #[test]
    fn test_materialized_view_refresh_statement() {
        let generator = MaterializedViewGenerator::new(WarehouseDialect::Postgres);
        let mv = generator.generate_observables_mv();

        let refresh = mv.to_refresh_statement();
        assert!(refresh.contains("REFRESH MATERIALIZED VIEW"));
    }

    #[test]
    fn test_bigquery_materialized_view() {
        let config = MaterializedViewConfig {
            name: "test_mv".to_string(),
            refresh_strategy: RefreshStrategy::Scheduled,
            refresh_interval_minutes: Some(30),
            cluster_by: vec!["type_id".to_string()],
            partition_by: Some("event_time".to_string()),
        };

        let generator = MaterializedViewGenerator::new(WarehouseDialect::BigQuery)
            .with_config(config);
        let mv = generator.generate_observables_mv();

        assert!(mv.sql.contains("PARTITION BY"));
        assert!(mv.sql.contains("CLUSTER BY"));
        assert!(mv.sql.contains("refresh_interval_minutes = 30"));
    }

    #[test]
    fn test_snowflake_materialized_view() {
        let generator = MaterializedViewGenerator::new(WarehouseDialect::Snowflake);
        let mv = generator.generate_observables_mv();

        assert!(mv.sql.contains("CREATE OR REPLACE MATERIALIZED VIEW"));
        assert!(mv.sql.contains("CLUSTER BY"));
    }
}
