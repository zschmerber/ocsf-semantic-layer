//! Cube.js schema generation.
//!
//! This module generates Cube.js schema files from semantic models,
//! mapping entities to cubes and metrics to measures.

use ocsf_semantic::{SemanticEntity, SemanticMetric, SemanticModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cube.js schema artifacts.
#[derive(Debug, Clone, Default)]
pub struct CubeArtifacts {
    /// Cube definitions (cube name -> JavaScript/TypeScript content).
    pub cubes: HashMap<String, String>,
}

impl CubeArtifacts {
    /// Creates new empty artifacts.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all cube definitions are valid JavaScript.
    pub fn is_valid_js(&self) -> bool {
        // Basic validation: check for cube() function and required structure
        self.cubes.values().all(|content| {
            content.contains("cube(") && content.contains("sql:")
        })
    }
}

/// Cube.js measure definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeMeasure {
    /// Measure name.
    pub name: String,
    /// Measure type (count, sum, avg, min, max, countDistinct).
    pub measure_type: String,
    /// SQL expression.
    pub sql: String,
    /// Description.
    pub description: String,
}

/// Cube.js dimension definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeDimension {
    /// Dimension name.
    pub name: String,
    /// Dimension type (string, number, time, boolean).
    pub dimension_type: String,
    /// SQL expression.
    pub sql: String,
    /// Description.
    pub description: String,
    /// Whether this is a primary key.
    pub primary_key: bool,
}

/// Configuration for Cube.js generation.
#[derive(Debug, Clone)]
pub struct CubeConfig {
    /// Table prefix for OCSF tables.
    pub table_prefix: String,
    /// Schema name.
    pub schema: Option<String>,
    /// Whether to use TypeScript syntax.
    pub use_typescript: bool,
}

impl Default for CubeConfig {
    fn default() -> Self {
        Self {
            table_prefix: "ocsf_".to_string(),
            schema: None,
            use_typescript: false,
        }
    }
}

/// Generates Cube.js schema files.
pub struct CubeGenerator {
    config: CubeConfig,
}

impl CubeGenerator {
    /// Creates a new Cube.js generator.
    pub fn new() -> Self {
        Self {
            config: CubeConfig::default(),
        }
    }

    /// Sets the configuration.
    pub fn with_config(mut self, config: CubeConfig) -> Self {
        self.config = config;
        self
    }

    /// Generates Cube.js artifacts from a semantic model.
    pub fn generate(&self, model: &SemanticModel) -> CubeArtifacts {
        let mut cubes = HashMap::new();

        for entity in &model.entities {
            let cube_name = self.entity_to_cube_name(&entity.name);
            let cube_content = self.generate_cube(entity, model);
            cubes.insert(cube_name, cube_content);
        }

        CubeArtifacts { cubes }
    }

    /// Converts an entity name to a cube name.
    fn entity_to_cube_name(&self, entity_name: &str) -> String {
        // Convert snake_case to PascalCase
        entity_name
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }

    /// Generates a cube definition for an entity.
    fn generate_cube(&self, entity: &SemanticEntity, model: &SemanticModel) -> String {
        let cube_name = self.entity_to_cube_name(&entity.name);
        let table_name = format!("{}{}", self.config.table_prefix, entity.name);

        let mut lines: Vec<String> = Vec::new();

        // Cube header
        lines.push(format!("cube(`{}`, {{", cube_name));
        
        // SQL table reference
        let sql_table = if let Some(ref schema) = self.config.schema {
            format!("{}.{}", schema, table_name)
        } else {
            table_name
        };
        lines.push(format!("  sql: `SELECT * FROM {}`,", sql_table));
        lines.push(String::new());

        // Title and description
        lines.push(format!("  title: `{}`,", entity.caption));
        if !entity.description.is_empty() {
            lines.push(format!("  description: `{}`,", entity.description));
        }
        lines.push(String::new());

        // Measures
        lines.push("  measures: {".to_string());
        
        // Add count measure by default
        lines.push("    count: {".to_string());
        lines.push("      type: `count`,".to_string());
        lines.push("    },".to_string());

        // Add metrics as measures
        for metric in &model.metrics {
            if self.metric_applies_to_entity(metric, entity) {
                lines.push(self.generate_measure(metric));
            }
        }
        
        lines.push("  },".to_string());
        lines.push(String::new());

        // Dimensions
        lines.push("  dimensions: {".to_string());
        
        // Add primary key dimension
        lines.push("    id: {".to_string());
        lines.push("      sql: `metadata_uid`,".to_string());
        lines.push("      type: `string`,".to_string());
        lines.push("      primaryKey: true,".to_string());
        lines.push("    },".to_string());

        // Add entity attributes as dimensions
        for attr in &entity.attributes {
            lines.push(self.generate_dimension(attr));
        }

        // Add time dimension
        lines.push("    time: {".to_string());
        lines.push("      sql: `time`,".to_string());
        lines.push("      type: `time`,".to_string());
        lines.push("    },".to_string());
        
        lines.push("  },".to_string());

        // Pre-aggregations (optional)
        lines.push(String::new());
        lines.push("  preAggregations: {".to_string());
        lines.push("    // Define pre-aggregations here for better performance".to_string());
        lines.push("  },".to_string());

        // Close cube
        lines.push("});".to_string());

        lines.join("\n")
    }

    /// Checks if a metric applies to an entity.
    fn metric_applies_to_entity(&self, metric: &SemanticMetric, entity: &SemanticEntity) -> bool {
        metric.dimensions.iter().any(|dim| {
            entity.attributes.iter().any(|attr| attr.name == *dim)
        })
    }

    /// Generates a measure definition.
    fn generate_measure(&self, metric: &SemanticMetric) -> String {
        let measure_type = match metric.aggregation {
            ocsf_semantic::Aggregation::Count => "count",
            ocsf_semantic::Aggregation::Sum => "sum",
            ocsf_semantic::Aggregation::Avg => "avg",
            ocsf_semantic::Aggregation::Min => "min",
            ocsf_semantic::Aggregation::Max => "max",
            ocsf_semantic::Aggregation::CountDistinct => "countDistinct",
        };

        let sql = metric
            .measure
            .field
            .clone()
            .or_else(|| metric.measure.expression.clone())
            .unwrap_or_else(|| "*".to_string());

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("    {}: {{", metric.name));
        lines.push(format!("      type: `{}`,", measure_type));
        lines.push(format!("      sql: `{}`,", sql));
        if !metric.description.is_empty() {
            lines.push(format!("      description: `{}`,", metric.description));
        }
        lines.push("    },".to_string());

        lines.join("\n")
    }

    /// Generates a dimension definition.
    fn generate_dimension(&self, attr: &ocsf_semantic::SemanticAttribute) -> String {
        let dimension_type = match attr.attr_type {
            ocsf_semantic::SemanticType::String => "string",
            ocsf_semantic::SemanticType::Integer => "number",
            ocsf_semantic::SemanticType::Float => "number",
            ocsf_semantic::SemanticType::Boolean => "boolean",
            ocsf_semantic::SemanticType::Timestamp => "time",
            ocsf_semantic::SemanticType::Json => "string",
            ocsf_semantic::SemanticType::Array(_) => "string",
        };

        let sql = attr
            .ocsf_mapping
            .field
            .clone()
            .or_else(|| attr.ocsf_mapping.expression.clone())
            .unwrap_or_else(|| attr.name.clone());

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("    {}: {{", attr.name));
        lines.push(format!("      sql: `{}`,", sql));
        lines.push(format!("      type: `{}`,", dimension_type));
        if !attr.description.is_empty() {
            lines.push(format!("      description: `{}`,", attr.description));
        }
        lines.push("    },".to_string());

        lines.join("\n")
    }
}

impl Default for CubeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::{Aggregation, OCSFMapping, SemanticAttribute, SemanticType, TimeGranularity};

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_caption("Authentication Event")
                    .with_description("Authentication events")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr"))
                            .as_dimension(),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_attempts")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("metadata_uid")
                    .with_dimensions(vec!["user_email".to_string()])
                    .with_time_granularities(vec![TimeGranularity::Hour]),
            )
    }

    #[test]
    fn test_cube_generator_creates_artifacts() {
        let model = create_test_model();
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        assert!(!artifacts.cubes.is_empty());
        assert!(artifacts.cubes.contains_key("AuthenticationEvent"));
    }

    #[test]
    fn test_cube_artifacts_valid_js() {
        let model = create_test_model();
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        assert!(artifacts.is_valid_js());
    }

    #[test]
    fn test_cube_contains_measures() {
        let model = create_test_model();
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        let cube = artifacts.cubes.get("AuthenticationEvent").unwrap();
        assert!(cube.contains("measures:"));
        assert!(cube.contains("count:"));
        assert!(cube.contains("auth_attempts:"));
    }

    #[test]
    fn test_cube_contains_dimensions() {
        let model = create_test_model();
        let generator = CubeGenerator::new();
        let artifacts = generator.generate(&model);

        let cube = artifacts.cubes.get("AuthenticationEvent").unwrap();
        assert!(cube.contains("dimensions:"));
        assert!(cube.contains("user_email:"));
        assert!(cube.contains("time:"));
    }

    #[test]
    fn test_entity_to_cube_name() {
        let generator = CubeGenerator::new();
        assert_eq!(
            generator.entity_to_cube_name("authentication_event"),
            "AuthenticationEvent"
        );
        assert_eq!(generator.entity_to_cube_name("user"), "User");
    }
}
