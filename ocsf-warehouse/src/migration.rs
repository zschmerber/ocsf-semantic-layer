//! Migration script generation.
//!
//! This module generates migration scripts for schema changes,
//! comparing model versions and generating ALTER statements.

use ocsf_semantic::{SemanticModel, WarehouseDialect};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A migration script.
#[derive(Debug, Clone, Default)]
pub struct MigrationScript {
    /// Migration version/name.
    pub version: String,
    /// Description of the migration.
    pub description: String,
    /// Up migration statements.
    pub up_statements: Vec<String>,
    /// Down migration statements (rollback).
    pub down_statements: Vec<String>,
}

impl MigrationScript {
    /// Creates a new migration script.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            description: String::new(),
            up_statements: Vec::new(),
            down_statements: Vec::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds an up statement.
    pub fn add_up(&mut self, statement: String) {
        self.up_statements.push(statement);
    }

    /// Adds a down statement.
    pub fn add_down(&mut self, statement: String) {
        self.down_statements.push(statement);
    }

    /// Returns the up migration as SQL.
    pub fn up_sql(&self) -> String {
        let mut sql = String::new();
        sql.push_str(&format!("-- Migration: {}\n", self.version));
        if !self.description.is_empty() {
            sql.push_str(&format!("-- {}\n", self.description));
        }
        sql.push_str("-- UP\n\n");
        sql.push_str(&self.up_statements.join("\n\n"));
        sql
    }

    /// Returns the down migration as SQL.
    pub fn down_sql(&self) -> String {
        let mut sql = String::new();
        sql.push_str(&format!("-- Migration: {}\n", self.version));
        sql.push_str("-- DOWN (Rollback)\n\n");
        sql.push_str(&self.down_statements.join("\n\n"));
        sql
    }

    /// Returns true if this migration has any changes.
    pub fn has_changes(&self) -> bool {
        !self.up_statements.is_empty()
    }
}

/// Type of schema change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaChangeType {
    /// Entity was added.
    EntityAdded(String),
    /// Entity was removed.
    EntityRemoved(String),
    /// Entity was modified.
    EntityModified(String),
    /// Attribute was added to an entity.
    AttributeAdded(String, String),
    /// Attribute was removed from an entity.
    AttributeRemoved(String, String),
    /// Attribute was modified.
    AttributeModified(String, String),
    /// Metric was added.
    MetricAdded(String),
    /// Metric was removed.
    MetricRemoved(String),
    /// Metric was modified.
    MetricModified(String),
}

/// Detected changes between two model versions.
#[derive(Debug, Clone, Default)]
pub struct ModelDiff {
    /// List of changes.
    pub changes: Vec<SchemaChangeType>,
}

impl ModelDiff {
    /// Creates a new empty diff.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a change.
    pub fn add(&mut self, change: SchemaChangeType) {
        self.changes.push(change);
    }

    /// Returns true if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

/// Generates migration scripts.
pub struct MigrationGenerator {
    dialect: WarehouseDialect,
    view_prefix: String,
    table_prefix: String,
}

impl MigrationGenerator {
    /// Creates a new migration generator.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            view_prefix: "v_".to_string(),
            table_prefix: "ocsf_".to_string(),
        }
    }

    /// Sets the view prefix.
    pub fn with_view_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.view_prefix = prefix.into();
        self
    }

    /// Sets the table prefix.
    pub fn with_table_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.table_prefix = prefix.into();
        self
    }

    /// Compares two model versions and returns the differences.
    pub fn compare_models(&self, from: &SemanticModel, to: &SemanticModel) -> ModelDiff {
        let mut diff = ModelDiff::new();

        // Compare entities
        let from_entities: HashSet<&str> = from.entities.iter().map(|e| e.name.as_str()).collect();
        let to_entities: HashSet<&str> = to.entities.iter().map(|e| e.name.as_str()).collect();

        // Added entities
        for name in to_entities.difference(&from_entities) {
            diff.add(SchemaChangeType::EntityAdded(name.to_string()));
        }

        // Removed entities
        for name in from_entities.difference(&to_entities) {
            diff.add(SchemaChangeType::EntityRemoved(name.to_string()));
        }

        // Modified entities
        for name in from_entities.intersection(&to_entities) {
            let from_entity = from.get_entity(name).unwrap();
            let to_entity = to.get_entity(name).unwrap();

            // Compare attributes
            let from_attrs: HashSet<&str> = from_entity.attributes.iter().map(|a| a.name.as_str()).collect();
            let to_attrs: HashSet<&str> = to_entity.attributes.iter().map(|a| a.name.as_str()).collect();

            for attr_name in to_attrs.difference(&from_attrs) {
                diff.add(SchemaChangeType::AttributeAdded(
                    name.to_string(),
                    attr_name.to_string(),
                ));
            }

            for attr_name in from_attrs.difference(&to_attrs) {
                diff.add(SchemaChangeType::AttributeRemoved(
                    name.to_string(),
                    attr_name.to_string(),
                ));
            }

            // Check for modified attributes
            for attr_name in from_attrs.intersection(&to_attrs) {
                let from_attr = from_entity.get_attribute(attr_name).unwrap();
                let to_attr = to_entity.get_attribute(attr_name).unwrap();

                if from_attr != to_attr {
                    diff.add(SchemaChangeType::AttributeModified(
                        name.to_string(),
                        attr_name.to_string(),
                    ));
                }
            }
        }

        // Compare metrics
        let from_metrics: HashSet<&str> = from.metrics.iter().map(|m| m.name.as_str()).collect();
        let to_metrics: HashSet<&str> = to.metrics.iter().map(|m| m.name.as_str()).collect();

        for name in to_metrics.difference(&from_metrics) {
            diff.add(SchemaChangeType::MetricAdded(name.to_string()));
        }

        for name in from_metrics.difference(&to_metrics) {
            diff.add(SchemaChangeType::MetricRemoved(name.to_string()));
        }

        for name in from_metrics.intersection(&to_metrics) {
            let from_metric = from.get_metric(name).unwrap();
            let to_metric = to.get_metric(name).unwrap();

            if from_metric != to_metric {
                diff.add(SchemaChangeType::MetricModified(name.to_string()));
            }
        }

        diff
    }

    /// Generates a migration script from a diff.
    pub fn generate_migration(
        &self,
        from_version: &str,
        to_version: &str,
        diff: &ModelDiff,
        to_model: &SemanticModel,
    ) -> MigrationScript {
        let mut migration = MigrationScript::new(format!("{}_{}", from_version, to_version))
            .with_description(format!(
                "Migration from version {} to {}",
                from_version, to_version
            ));

        for change in &diff.changes {
            match change {
                SchemaChangeType::EntityAdded(name) => {
                    if let Some(entity) = to_model.get_entity(name) {
                        let view_name = format!("{}{}", self.view_prefix, name);
                        
                        // Generate CREATE VIEW for new entity
                        let create_sql = self.generate_create_view_sql(&view_name, entity);
                        migration.add_up(create_sql);

                        // Generate DROP VIEW for rollback
                        let drop_sql = format!("DROP VIEW IF EXISTS {};", view_name);
                        migration.add_down(drop_sql);
                    }
                }
                SchemaChangeType::EntityRemoved(name) => {
                    let view_name = format!("{}{}", self.view_prefix, name);
                    
                    // Generate DROP VIEW
                    let drop_sql = format!("DROP VIEW IF EXISTS {};", view_name);
                    migration.add_up(drop_sql);

                    // Note: Can't easily rollback a removed entity without the original definition
                    migration.add_down(format!(
                        "-- Cannot automatically restore removed view: {}",
                        view_name
                    ));
                }
                SchemaChangeType::EntityModified(name) | SchemaChangeType::AttributeAdded(name, _) 
                | SchemaChangeType::AttributeRemoved(name, _) | SchemaChangeType::AttributeModified(name, _) => {
                    if let Some(entity) = to_model.get_entity(name) {
                        let view_name = format!("{}{}", self.view_prefix, name);
                        
                        // Generate CREATE OR REPLACE VIEW
                        let create_sql = self.generate_create_view_sql(&view_name, entity);
                        migration.add_up(create_sql);

                        // Note: Rollback would need the previous version
                        migration.add_down(format!(
                            "-- Rollback requires previous version of view: {}",
                            view_name
                        ));
                    }
                }
                SchemaChangeType::MetricAdded(_) | SchemaChangeType::MetricRemoved(_) 
                | SchemaChangeType::MetricModified(_) => {
                    // Metrics don't directly affect SQL schema
                    // They affect semantic layer definitions (dbt, Cube.js)
                    migration.add_up(format!(
                        "-- Metric change: {:?} - Update semantic layer definitions",
                        change
                    ));
                }
            }
        }

        migration
    }

    /// Generates CREATE VIEW SQL for an entity.
    fn generate_create_view_sql(
        &self,
        view_name: &str,
        entity: &ocsf_semantic::SemanticEntity,
    ) -> String {
        let mut sql = String::new();
        sql.push_str(&format!(
            "CREATE OR REPLACE VIEW {} AS\n",
            self.dialect.quote_identifier(view_name)
        ));
        sql.push_str("SELECT\n");

        let mut columns = vec![
            "    metadata_uid".to_string(),
            "    class_uid".to_string(),
            "    time".to_string(),
        ];

        for attr in &entity.attributes {
            let expr = if let Some(ref field) = attr.ocsf_mapping.field {
                format!("    {} AS {}", field, attr.name)
            } else if let Some(ref expression) = attr.ocsf_mapping.expression {
                format!("    {} AS {}", expression, attr.name)
            } else {
                format!("    NULL AS {}", attr.name)
            };
            columns.push(expr);
        }

        sql.push_str(&columns.join(",\n"));
        sql.push_str(&format!(
            "\nFROM {}events",
            self.table_prefix
        ));

        if !entity.source_event_classes.is_empty() {
            let class_uids: Vec<String> = entity
                .source_event_classes
                .iter()
                .map(|uid| uid.to_string())
                .collect();
            sql.push_str(&format!("\nWHERE class_uid IN ({})", class_uids.join(", ")));
        }

        sql.push(';');
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

    fn create_v1_model() -> SemanticModel {
        SemanticModel::new("test")
            .with_version("1.0")
            .add_entity(
                SemanticEntity::new("user")
                    .add_attribute(
                        SemanticAttribute::new("email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("user.email_addr")),
                    ),
            )
    }

    fn create_v2_model() -> SemanticModel {
        SemanticModel::new("test")
            .with_version("2.0")
            .add_entity(
                SemanticEntity::new("user")
                    .add_attribute(
                        SemanticAttribute::new("email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("user.email_addr")),
                    )
                    .add_attribute(
                        SemanticAttribute::new("name")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("user.name")),
                    ),
            )
            .add_entity(
                SemanticEntity::new("device")
                    .add_attribute(
                        SemanticAttribute::new("hostname")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("device.hostname")),
                    ),
            )
    }

    #[test]
    fn test_compare_models_detects_added_entity() {
        let v1 = create_v1_model();
        let v2 = create_v2_model();

        let generator = MigrationGenerator::new(WarehouseDialect::Snowflake);
        let diff = generator.compare_models(&v1, &v2);

        assert!(diff.has_changes());
        assert!(diff.changes.contains(&SchemaChangeType::EntityAdded("device".to_string())));
    }

    #[test]
    fn test_compare_models_detects_added_attribute() {
        let v1 = create_v1_model();
        let v2 = create_v2_model();

        let generator = MigrationGenerator::new(WarehouseDialect::Snowflake);
        let diff = generator.compare_models(&v1, &v2);

        assert!(diff.changes.contains(&SchemaChangeType::AttributeAdded(
            "user".to_string(),
            "name".to_string()
        )));
    }

    #[test]
    fn test_generate_migration_creates_statements() {
        let v1 = create_v1_model();
        let v2 = create_v2_model();

        let generator = MigrationGenerator::new(WarehouseDialect::Postgres);
        let diff = generator.compare_models(&v1, &v2);
        let migration = generator.generate_migration("1.0", "2.0", &diff, &v2);

        assert!(migration.has_changes());
        assert!(!migration.up_statements.is_empty());

        let up_sql = migration.up_sql();
        assert!(up_sql.contains("CREATE"));
        assert!(up_sql.contains("VIEW"));
    }

    #[test]
    fn test_migration_script_sql_output() {
        let mut migration = MigrationScript::new("v1_v2")
            .with_description("Test migration");
        
        migration.add_up("CREATE VIEW test AS SELECT 1;".to_string());
        migration.add_down("DROP VIEW test;".to_string());

        let up_sql = migration.up_sql();
        assert!(up_sql.contains("Migration: v1_v2"));
        assert!(up_sql.contains("Test migration"));
        assert!(up_sql.contains("CREATE VIEW"));

        let down_sql = migration.down_sql();
        assert!(down_sql.contains("Rollback"));
        assert!(down_sql.contains("DROP VIEW"));
    }

    #[test]
    fn test_no_changes_when_models_identical() {
        let v1 = create_v1_model();
        let v1_copy = create_v1_model();

        let generator = MigrationGenerator::new(WarehouseDialect::BigQuery);
        let diff = generator.compare_models(&v1, &v1_copy);

        assert!(!diff.has_changes());
    }
}
