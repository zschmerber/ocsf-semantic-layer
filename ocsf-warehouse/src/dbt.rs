//! dbt semantic layer YAML generation.
//!
//! This module generates dbt semantic layer artifacts including:
//! - semantic_manifest.yml
//! - model SQL files
//! - sources.yml

use ocsf_semantic::{SemanticEntity, SemanticMetric, SemanticModel, WarehouseDialect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// dbt semantic layer artifacts.
#[derive(Debug, Clone, Default)]
pub struct DBTArtifacts {
    /// The semantic manifest YAML content.
    pub semantic_manifest: String,
    /// Model SQL files (model name -> SQL content).
    pub models: HashMap<String, String>,
    /// The sources YAML content.
    pub sources: String,
}

impl DBTArtifacts {
    /// Creates new empty artifacts.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the semantic manifest is valid YAML.
    pub fn is_valid_yaml(&self) -> bool {
        serde_yaml::from_str::<serde_yaml::Value>(&self.semantic_manifest).is_ok()
            && serde_yaml::from_str::<serde_yaml::Value>(&self.sources).is_ok()
    }
}

/// dbt semantic model definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTSemanticModel {
    /// Model name.
    pub name: String,
    /// Model description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// The dbt model reference.
    pub model: String,
    /// Default time dimension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_time_dimension: Option<String>,
    /// Entity definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<DBTEntity>,
    /// Dimension definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimensions: Vec<DBTDimension>,
    /// Measure definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measures: Vec<DBTMeasure>,
}

/// dbt entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTEntity {
    /// Entity name.
    pub name: String,
    /// Entity type (primary, foreign, unique).
    #[serde(rename = "type")]
    pub entity_type: String,
    /// Expression for the entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
}

/// dbt dimension definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTDimension {
    /// Dimension name.
    pub name: String,
    /// Dimension type (categorical, time).
    #[serde(rename = "type")]
    pub dimension_type: String,
    /// Expression for the dimension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    /// Time granularity for time dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_granularity: Option<String>,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// dbt measure definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTMeasure {
    /// Measure name.
    pub name: String,
    /// Aggregation type.
    pub agg: String,
    /// Expression for the measure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Whether this creates a metric.
    #[serde(default)]
    pub create_metric: bool,
}

/// dbt source definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTSource {
    /// Source name.
    pub name: String,
    /// Database name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    /// Schema name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Tables in this source.
    pub tables: Vec<DBTSourceTable>,
}

/// dbt source table definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTSourceTable {
    /// Table name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Column definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<DBTColumn>,
}

/// dbt column definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBTColumn {
    /// Column name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// Configuration for dbt artifact generation.
#[derive(Debug, Clone)]
pub struct DBTConfig {
    /// Source name for OCSF tables.
    pub source_name: String,
    /// Database name.
    pub database: Option<String>,
    /// Schema name.
    pub schema: Option<String>,
    /// Table prefix for OCSF tables.
    pub table_prefix: String,
}

impl Default for DBTConfig {
    fn default() -> Self {
        Self {
            source_name: "ocsf".to_string(),
            database: None,
            schema: None,
            table_prefix: "ocsf_".to_string(),
        }
    }
}

/// Generates dbt semantic layer artifacts.
pub struct DBTGenerator {
    config: DBTConfig,
    _dialect: WarehouseDialect,
}

impl DBTGenerator {
    /// Creates a new dbt generator.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            config: DBTConfig::default(),
            _dialect: dialect,
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: DBTConfig) -> Self {
        self.config = config;
        self
    }

    /// Generates dbt artifacts from a semantic model.
    pub fn generate(&self, model: &SemanticModel) -> DBTArtifacts {
        let semantic_manifest = self.generate_semantic_manifest(model);
        let models = self.generate_models(model);
        let sources = self.generate_sources(model);

        DBTArtifacts {
            semantic_manifest,
            models,
            sources,
        }
    }

    /// Generates the semantic manifest YAML.
    fn generate_semantic_manifest(&self, model: &SemanticModel) -> String {
        let mut semantic_models = Vec::new();

        for entity in &model.entities {
            let dbt_model = self.entity_to_dbt_semantic_model(entity, model);
            semantic_models.push(dbt_model);
        }

        let manifest = serde_yaml::to_value(&semantic_models).unwrap_or_default();
        let wrapper = serde_yaml::Mapping::from_iter([(
            serde_yaml::Value::String("semantic_models".to_string()),
            manifest,
        )]);

        serde_yaml::to_string(&wrapper).unwrap_or_default()
    }

    /// Converts a semantic entity to a dbt semantic model.
    fn entity_to_dbt_semantic_model(
        &self,
        entity: &SemanticEntity,
        model: &SemanticModel,
    ) -> DBTSemanticModel {
        let mut dimensions = Vec::new();
        let entities = vec![DBTEntity {
            name: entity.name.clone(),
            entity_type: "primary".to_string(),
            expr: None,
        }];

        // Convert attributes to dimensions
        for attr in &entity.attributes {
            if attr.is_dimension {
                let expr = attr
                    .ocsf_mapping
                    .field
                    .clone()
                    .or_else(|| attr.ocsf_mapping.expression.clone());

                dimensions.push(DBTDimension {
                    name: attr.name.clone(),
                    dimension_type: "categorical".to_string(),
                    expr,
                    time_granularity: None,
                    description: attr.description.clone(),
                });
            }
        }

        // Add time dimension
        dimensions.push(DBTDimension {
            name: "event_time".to_string(),
            dimension_type: "time".to_string(),
            expr: Some("time".to_string()),
            time_granularity: Some("day".to_string()),
            description: "Event timestamp".to_string(),
        });

        // Convert metrics to measures
        let measures: Vec<DBTMeasure> = model
            .metrics
            .iter()
            .filter(|m| self.metric_applies_to_entity(m, entity))
            .map(|m| self.metric_to_dbt_measure(m))
            .collect();

        DBTSemanticModel {
            name: entity.name.clone(),
            description: entity.description.clone(),
            model: format!("ref('{}{}')", self.config.table_prefix, entity.name),
            default_time_dimension: Some("event_time".to_string()),
            entities,
            dimensions,
            measures,
        }
    }

    /// Checks if a metric applies to an entity.
    fn metric_applies_to_entity(&self, metric: &SemanticMetric, entity: &SemanticEntity) -> bool {
        // A metric applies if any of its dimensions are attributes of the entity
        metric.dimensions.iter().any(|dim| {
            entity.attributes.iter().any(|attr| attr.name == *dim)
        })
    }

    /// Converts a semantic metric to a dbt measure.
    fn metric_to_dbt_measure(&self, metric: &SemanticMetric) -> DBTMeasure {
        let agg = match metric.aggregation {
            ocsf_semantic::Aggregation::Count => "count",
            ocsf_semantic::Aggregation::Sum => "sum",
            ocsf_semantic::Aggregation::Avg => "average",
            ocsf_semantic::Aggregation::Min => "min",
            ocsf_semantic::Aggregation::Max => "max",
            ocsf_semantic::Aggregation::CountDistinct => "count_distinct",
        };

        let expr = metric
            .measure
            .field
            .clone()
            .or_else(|| metric.measure.expression.clone());

        DBTMeasure {
            name: metric.name.clone(),
            agg: agg.to_string(),
            expr,
            description: metric.description.clone(),
            create_metric: true,
        }
    }

    /// Generates model SQL files.
    fn generate_models(&self, model: &SemanticModel) -> HashMap<String, String> {
        let mut models = HashMap::new();

        for entity in &model.entities {
            let model_name = format!("{}{}", self.config.table_prefix, entity.name);
            let sql = self.generate_entity_model_sql(entity);
            models.insert(model_name, sql);
        }

        models
    }

    /// Generates SQL for an entity model.
    fn generate_entity_model_sql(&self, entity: &SemanticEntity) -> String {
        let mut sql = String::new();
        sql.push_str("{{ config(materialized='view') }}\n\n");
        sql.push_str("SELECT\n");

        // Add attribute selections
        let mut selections = Vec::new();
        for attr in &entity.attributes {
            let expr = if let Some(ref field) = attr.ocsf_mapping.field {
                format!("    {} AS {}", field, attr.name)
            } else if let Some(ref expression) = attr.ocsf_mapping.expression {
                format!("    {} AS {}", expression, attr.name)
            } else {
                format!("    NULL AS {}", attr.name)
            };
            selections.push(expr);
        }

        // Add base columns
        selections.push("    time".to_string());
        selections.push("    metadata_uid".to_string());
        selections.push("    class_uid".to_string());

        sql.push_str(&selections.join(",\n"));
        sql.push_str("\nFROM {{ source('");
        sql.push_str(&self.config.source_name);
        sql.push_str("', '");

        // Use first source event class as the base table
        if let Some(class_uid) = entity.source_event_classes.first() {
            sql.push_str(&format!("ocsf_class_{}", class_uid));
        } else {
            sql.push_str("ocsf_events");
        }

        sql.push_str("') }}\n");

        sql
    }

    /// Generates the sources YAML.
    fn generate_sources(&self, model: &SemanticModel) -> String {
        let mut tables = Vec::new();

        // Add tables for each entity's source event classes
        for entity in &model.entities {
            for class_uid in &entity.source_event_classes {
                let table = DBTSourceTable {
                    name: format!("ocsf_class_{}", class_uid),
                    description: format!("OCSF event class {}", class_uid),
                    columns: Vec::new(),
                };
                tables.push(table);
            }
        }

        // Add observables table if configured
        if model.observable_config.extract_to_table {
            tables.push(DBTSourceTable {
                name: model.observable_config.table_name.clone(),
                description: "OCSF observables table for hot path analytics".to_string(),
                columns: Vec::new(),
            });
        }

        let source = DBTSource {
            name: self.config.source_name.clone(),
            database: self.config.database.clone(),
            schema: self.config.schema.clone(),
            tables,
        };

        let sources = serde_yaml::to_value(vec![source]).unwrap_or_default();
        let wrapper = serde_yaml::Mapping::from_iter([(
            serde_yaml::Value::String("sources".to_string()),
            sources,
        )]);

        serde_yaml::to_string(&wrapper).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::{
        Aggregation, OCSFMapping, ObservableConfig, SemanticAttribute, SemanticType,
        TimeGranularity,
    };

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test-model")
            .with_ocsf_version("1.4.0")
            .add_entity(
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
                        SemanticAttribute::new("source_ip")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("src_endpoint.ip"))
                            .as_dimension(),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_attempts")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("metadata_uid")
                    .with_dimensions(vec!["user_email".to_string()])
                    .with_time_granularities(vec![TimeGranularity::Hour, TimeGranularity::Day]),
            )
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5],
            })
    }

    #[test]
    fn test_dbt_generator_creates_artifacts() {
        let model = create_test_model();
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        assert!(!artifacts.semantic_manifest.is_empty());
        assert!(!artifacts.models.is_empty());
        assert!(!artifacts.sources.is_empty());
    }

    #[test]
    fn test_dbt_artifacts_valid_yaml() {
        let model = create_test_model();
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        assert!(artifacts.is_valid_yaml());
    }

    #[test]
    fn test_semantic_manifest_contains_entities() {
        let model = create_test_model();
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        assert!(artifacts.semantic_manifest.contains("authentication_event"));
        assert!(artifacts.semantic_manifest.contains("semantic_models"));
    }

    #[test]
    fn test_sources_contains_tables() {
        let model = create_test_model();
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        assert!(artifacts.sources.contains("sources"));
        assert!(artifacts.sources.contains("ocsf_class_3002"));
        assert!(artifacts.sources.contains("ocsf_observables"));
    }

    #[test]
    fn test_models_contain_sql() {
        let model = create_test_model();
        let generator = DBTGenerator::new(WarehouseDialect::Snowflake);
        let artifacts = generator.generate(&model);

        let model_sql = artifacts.models.get("ocsf_authentication_event");
        assert!(model_sql.is_some());

        let sql = model_sql.unwrap();
        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM"));
        assert!(sql.contains("source"));
    }
}
