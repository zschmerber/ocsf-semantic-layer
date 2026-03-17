//! Schema-driven semantic layer generation.
//!
//! This module generates semantic entities, dimensions, and metrics from
//! the compiled OCSF schema by leveraging schema metadata.

use crate::{
    DefaultDimensionInferrer, DimensionInferrer, FieldPath, OCSFMapping, ObservableConfig,
    PathGenerator, SemanticAttribute, SemanticEntity, SemanticModel, SemanticType,
};
use crate::entity::HierarchyLevel;
use crate::metric::MetricType;
use ocsf_core::{CompiledAttribute, CompiledClass, CompiledObject, CompiledRequirement, CompiledSchema};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// High-usage objects that should become entities by default.
pub const HIGH_USAGE_OBJECTS: &[&str] = &[
    "user",
    "device",
    "actor",
    "network_endpoint",
    "file",
    "process",
    "cloud",
    "account",
    "group",
    "product",
    "metadata",
];

/// Errors during entity generation.
#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Class not found: {0}")]
    ClassNotFound(String),

    #[error("Circular reference detected: {path:?}")]
    CircularReference { path: Vec<String> },

    #[error("Max nesting depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },

    #[error("Path error: {0}")]
    PathError(#[from] crate::field_path::PathError),
}

/// Configuration for entity generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Categories to include (empty = all).
    #[serde(default)]
    pub category_filter: Vec<String>,

    /// Specific classes to include (empty = all).
    #[serde(default)]
    pub class_filter: Vec<String>,

    /// Specific objects to include (empty = high-usage).
    #[serde(default)]
    pub object_filter: Vec<String>,

    /// Exclude patterns (object/class names to skip).
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Maximum nesting depth for path generation.
    #[serde(default = "default_max_depth")]
    pub max_nesting_depth: usize,

    /// Include deprecated attributes.
    #[serde(default)]
    pub include_deprecated: bool,

    /// Only include required/recommended attributes.
    #[serde(default)]
    pub required_only: bool,
}

fn default_max_depth() -> usize {
    3
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            category_filter: vec![],
            class_filter: vec![],
            object_filter: vec![],
            exclude_patterns: vec![],
            max_nesting_depth: 3,
            include_deprecated: false,
            required_only: false,
        }
    }
}

/// Statistics about generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationStats {
    pub objects_processed: usize,
    pub classes_processed: usize,
    pub entities_generated: usize,
    pub dimensions_inferred: usize,
    pub metrics_suggested: usize,
}

/// Metadata about the generation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// Source schema version.
    pub schema_version: String,

    /// Source schema file path.
    #[serde(default)]
    pub schema_path: String,

    /// Generation timestamp (Unix seconds).
    pub generated_at: u64,

    /// Generation configuration used.
    pub config: GenerationConfig,

    /// Statistics about generation.
    pub stats: GenerationStats,
}

/// Source type for generated entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitySourceType {
    /// Generated from an OCSF object.
    Object,
    /// Generated from an OCSF class.
    Class,
}

/// Extended entity with generation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedEntity {
    /// The semantic entity.
    #[serde(flatten)]
    pub entity: SemanticEntity,

    /// Source type (object or class).
    pub source_type: EntitySourceType,

    /// Source class UID (for class entities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_class_uid: Option<u32>,

    /// Source category name (for class entities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_category: Option<String>,
}

/// Schema-driven entity generator.
pub struct SchemaGenerator<'a> {
    schema: &'a CompiledSchema,
    config: GenerationConfig,
    dimension_inferrer: DefaultDimensionInferrer,
}

impl<'a> SchemaGenerator<'a> {
    /// Create a new schema generator.
    pub fn new(schema: &'a CompiledSchema, config: GenerationConfig) -> Self {
        Self {
            schema,
            config,
            dimension_inferrer: DefaultDimensionInferrer::new(),
        }
    }

    /// Generate entities from OCSF objects.
    pub fn generate_from_objects(&self) -> Vec<GeneratedEntity> {
        let mut entities = Vec::new();
        let objects_to_process = self.get_objects_to_process();

        for object_name in objects_to_process {
            if let Some(object) = self.schema.get_object(&object_name) {
                if let Some(entity) = self.generate_object_entity(&object_name, object) {
                    entities.push(entity);
                }
            }
        }

        entities
    }

    /// Generate entities from OCSF classes.
    pub fn generate_from_classes(&self) -> Vec<GeneratedEntity> {
        let mut entities = Vec::new();

        for (class_name, class) in self.schema.all_classes() {
            if self.should_include_class(class_name, class) {
                if let Some(entity) = self.generate_class_entity(class_name, class) {
                    entities.push(entity);
                }
            }
        }

        entities
    }

    /// Generate a complete semantic model.
    pub fn generate_model(&self) -> (SemanticModel, GenerationMetadata) {
        let object_entities = self.generate_from_objects();
        let class_entities = self.generate_from_classes();

        let mut all_entities: Vec<SemanticEntity> = Vec::new();
        let mut dimensions_count = 0;

        for ge in object_entities.iter().chain(class_entities.iter()) {
            dimensions_count += ge.entity.attributes.iter().filter(|a| a.is_dimension).count();
            all_entities.push(ge.entity.clone());
        }

        let model = SemanticModel::new("generated-from-ocsf")
            .with_ocsf_version(self.schema.version.clone())
            .with_description("Auto-generated semantic layer from OCSF schema")
            .with_entities(all_entities)
            .with_observable_config(ObservableConfig::default());

        let metadata = GenerationMetadata {
            schema_version: self.schema.version.clone(),
            schema_path: String::new(),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            config: self.config.clone(),
            stats: GenerationStats {
                objects_processed: object_entities.len(),
                classes_processed: class_entities.len(),
                entities_generated: object_entities.len() + class_entities.len(),
                dimensions_inferred: dimensions_count,
                metrics_suggested: 0,
            },
        };

        (model, metadata)
    }

    fn get_objects_to_process(&self) -> Vec<String> {
        if !self.config.object_filter.is_empty() {
            self.config.object_filter.clone()
        } else {
            HIGH_USAGE_OBJECTS
                .iter()
                .filter(|name| {
                    self.schema.get_object(name).is_some()
                        && !self.is_excluded(name)
                })
                .map(|s| s.to_string())
                .collect()
        }
    }

    fn should_include_class(&self, name: &str, class: &CompiledClass) -> bool {
        // Check exclude patterns
        if self.is_excluded(name) {
            return false;
        }

        // Check category filter
        if !self.config.category_filter.is_empty()
            && !self.config.category_filter.contains(&class.category)
        {
            return false;
        }

        // Check class filter
        if !self.config.class_filter.is_empty()
            && !self.config.class_filter.contains(&name.to_string())
        {
            return false;
        }

        true
    }

    fn is_excluded(&self, name: &str) -> bool {
        self.config
            .exclude_patterns
            .iter()
            .any(|pattern| name.contains(pattern))
    }

    fn generate_object_entity(
        &self,
        name: &str,
        object: &CompiledObject,
    ) -> Option<GeneratedEntity> {
        let mut attributes = Vec::new();

        for (attr_name, attr) in &object.attributes {
            if self.should_include_attribute(attr) {
                if let Some(sem_attr) = self.convert_attribute(attr_name, attr, None) {
                    attributes.push(sem_attr);
                }
            }
        }

        if attributes.is_empty() {
            return None;
        }

        // Task 7.2: Infer geographic hierarchy from attribute names (best-effort)
        Self::infer_geographic_hierarchy(&mut attributes);

        let entity = SemanticEntity::new(name)
            .with_caption(object.caption.clone())
            .with_description(object.description.clone())
            .with_attributes(attributes);

        Some(GeneratedEntity {
            entity,
            source_type: EntitySourceType::Object,
            source_class_uid: None,
            source_category: None,
        })
    }

    /// Infer geographic hierarchy from attribute names.
    ///
    /// When attributes contain geographic field name patterns (country, region, city, state, continent),
    /// generates a HierarchyLevel chain on the top-level geographic attribute.
    /// Best-effort: silently skips if no patterns match.
    fn infer_geographic_hierarchy(attributes: &mut [SemanticAttribute]) {
        // Geographic hierarchy order: continent > country > region/state > city
        let geo_patterns: &[(&str, &str)] = &[
            ("continent", "Continent"),
            ("country", "Country"),
            ("region", "Region"),
            ("state", "State"),
            ("city", "City"),
        ];

        // Find which geographic attributes exist
        let mut found_levels: Vec<HierarchyLevel> = Vec::new();
        for &(pattern, display_name) in geo_patterns {
            if let Some(attr) = attributes.iter().find(|a| a.name.contains(pattern)) {
                found_levels.push(HierarchyLevel {
                    name: display_name.to_string(),
                    attribute_ref: attr.name.clone(),
                });
            }
        }

        // Need at least 2 levels for a meaningful hierarchy
        if found_levels.len() < 2 {
            return;
        }

        // Attach hierarchy to the first (top-level) geographic attribute
        let top_attr_name = found_levels[0].attribute_ref.clone();
        if let Some(attr) = attributes.iter_mut().find(|a| a.name == top_attr_name) {
            attr.hierarchy = found_levels;
            attr.is_dimension = true;
        }
    }

    fn generate_class_entity(
        &self,
        name: &str,
        class: &CompiledClass,
    ) -> Option<GeneratedEntity> {
        let mut attributes = Vec::new();
        let path_gen = PathGenerator::new(self.schema, self.config.max_nesting_depth);

        for (attr_name, attr) in &class.attributes {
            if self.should_include_attribute(attr) {
                // For object types, flatten nested attributes
                if attr.attr_type == "object_t" {
                    if let Some(ref obj_type) = attr.object_type {
                        if let Ok(paths) = path_gen.generate_paths(obj_type) {
                            for gen_path in paths {
                                let full_path = FieldPath::from_segments(vec![
                                    crate::field_path::PathSegment::new(attr_name.clone(), attr.is_array),
                                ])
                                .child(gen_path.path.to_string(), false);

                                let leaf_name = gen_path.path.leaf_name().unwrap_or(attr_name);
                                let flattened_name = format!("{}_{}", attr_name, leaf_name);

                                if let Some(sem_attr) = self.convert_attribute(
                                    &flattened_name,
                                    &gen_path.attribute,
                                    Some(full_path.to_string()),
                                ) {
                                    attributes.push(sem_attr);
                                }
                            }
                        }
                    }
                } else if let Some(sem_attr) = self.convert_attribute(attr_name, attr, Some(attr_name.clone())) {
                    attributes.push(sem_attr);
                }
            }
        }

        if attributes.is_empty() {
            return None;
        }

        let _category_uid = self.schema.get_category_uid_for_class(class);

        let entity = SemanticEntity::new(name)
            .with_caption(class.caption.clone())
            .with_description(class.description.clone())
            .with_source_event_classes(vec![class.uid])
            .with_attributes(attributes);

        Some(GeneratedEntity {
            entity,
            source_type: EntitySourceType::Class,
            source_class_uid: Some(class.uid),
            source_category: Some(class.category.clone()),
        })
    }

    fn should_include_attribute(&self, attr: &CompiledAttribute) -> bool {
        // Skip deprecated unless configured
        if !self.config.include_deprecated && attr.deprecated.is_some() {
            return false;
        }

        // Skip optional if required_only
        if self.config.required_only && attr.requirement == CompiledRequirement::Optional {
            return false;
        }

        // Skip object types (they get flattened)
        if attr.attr_type == "object_t" {
            return true; // Will be handled specially
        }

        true
    }

    fn convert_attribute(
        &self,
        name: &str,
        attr: &CompiledAttribute,
        ocsf_path: Option<String>,
    ) -> Option<SemanticAttribute> {
        // Skip object types in direct conversion
        if attr.attr_type == "object_t" {
            return None;
        }

        let dim_info = self.dimension_inferrer.infer_with_name(name, attr);
        let sem_type = self.convert_type(&attr.attr_type, attr.is_array);

        let mapping = if let Some(path) = ocsf_path {
            OCSFMapping::from_field(path)
        } else {
            OCSFMapping::from_field(name)
        };

        let mut sem_attr = SemanticAttribute::new(name)
            .with_caption(attr.caption.clone())
            .with_description(attr.description.clone())
            .with_type(sem_type)
            .with_mapping(mapping);

        if dim_info.is_dimension {
            sem_attr = sem_attr.as_dimension();
        }

        Some(sem_attr)
    }

    fn convert_type(&self, ocsf_type: &str, is_array: bool) -> SemanticType {
        let base_type = match ocsf_type {
            "string_t" | "hostname_t" | "ip_t" | "mac_t" | "email_t" | "url_t" | "path_t"
            | "uuid_t" | "subnet_t" | "port_t" | "fingerprint_t" | "username_t" => {
                SemanticType::String
            }
            "integer_t" | "long_t" => SemanticType::Integer,
            "float_t" | "double_t" => SemanticType::Float,
            "boolean_t" => SemanticType::Boolean,
            "timestamp_t" | "datetime_t" => SemanticType::Timestamp,
            "json_t" | "object_t" => SemanticType::Json,
            _ => SemanticType::String,
        };

        if is_array {
            SemanticType::Array(Box::new(base_type))
        } else {
            base_type
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema() -> CompiledSchema {
        let mut schema = CompiledSchema::default();
        schema.version = "1.6.0".to_string();

        // Add a test object
        let mut user_attrs = std::collections::HashMap::new();
        user_attrs.insert(
            "name".to_string(),
            CompiledAttribute {
                caption: "Name".to_string(),
                description: "User name".to_string(),
                attr_type: "string_t".to_string(),
                type_name: "String".to_string(),
                requirement: CompiledRequirement::Required,
                group: Some("primary".to_string()),
                object_type: None,
                object_name: None,
                is_array: false,
                enum_values: std::collections::HashMap::new(),
                observable: Some(10),
                sibling: None,
                profiles: None,
                deprecated: None,
            },
        );
        user_attrs.insert(
            "email_addr".to_string(),
            CompiledAttribute {
                caption: "Email".to_string(),
                description: "Email address".to_string(),
                attr_type: "email_t".to_string(),
                type_name: "Email".to_string(),
                requirement: CompiledRequirement::Recommended,
                group: None,
                object_type: None,
                object_name: None,
                is_array: false,
                enum_values: std::collections::HashMap::new(),
                observable: Some(5),
                sibling: None,
                profiles: None,
                deprecated: None,
            },
        );

        schema.objects.insert(
            "user".to_string(),
            CompiledObject {
                name: "user".to_string(),
                caption: "User".to_string(),
                description: "User object".to_string(),
                extends: Some("object".to_string()),
                attributes: user_attrs,
            },
        );

        schema
    }

    #[test]
    fn test_generate_from_objects() {
        let schema = create_test_schema();
        let config = GenerationConfig::default();
        let generator = SchemaGenerator::new(&schema, config);

        let entities = generator.generate_from_objects();
        assert!(!entities.is_empty());

        let user_entity = entities.iter().find(|e| e.entity.name == "user");
        assert!(user_entity.is_some());

        let user = user_entity.unwrap();
        assert_eq!(user.source_type, EntitySourceType::Object);
        assert!(user.entity.attributes.iter().any(|a| a.name == "name"));
    }

    #[test]
    fn test_dimension_inference_in_generation() {
        let schema = create_test_schema();
        let config = GenerationConfig::default();
        let generator = SchemaGenerator::new(&schema, config);

        let entities = generator.generate_from_objects();
        let user_entity = entities.iter().find(|e| e.entity.name == "user").unwrap();

        // Name should be a dimension (required + observable)
        let name_attr = user_entity.entity.get_attribute("name").unwrap();
        assert!(name_attr.is_dimension);

        // Email should be a dimension (observable)
        let email_attr = user_entity.entity.get_attribute("email_addr").unwrap();
        assert!(email_attr.is_dimension);
    }

    #[test]
    fn test_object_filter() {
        let schema = create_test_schema();
        let config = GenerationConfig {
            object_filter: vec!["nonexistent".to_string()],
            ..Default::default()
        };
        let generator = SchemaGenerator::new(&schema, config);

        let entities = generator.generate_from_objects();
        assert!(entities.is_empty());
    }

    #[test]
    fn test_exclude_patterns() {
        let schema = create_test_schema();
        let config = GenerationConfig {
            exclude_patterns: vec!["user".to_string()],
            ..Default::default()
        };
        let generator = SchemaGenerator::new(&schema, config);

        let entities = generator.generate_from_objects();
        assert!(!entities.iter().any(|e| e.entity.name == "user"));
    }

    #[test]
    fn test_type_conversion() {
        let schema = create_test_schema();
        let config = GenerationConfig::default();
        let generator = SchemaGenerator::new(&schema, config);

        assert_eq!(
            generator.convert_type("string_t", false),
            SemanticType::String
        );
        assert_eq!(
            generator.convert_type("integer_t", false),
            SemanticType::Integer
        );
        assert_eq!(
            generator.convert_type("timestamp_t", false),
            SemanticType::Timestamp
        );
        assert_eq!(
            generator.convert_type("boolean_t", false),
            SemanticType::Boolean
        );

        // Array type
        assert_eq!(
            generator.convert_type("string_t", true),
            SemanticType::Array(Box::new(SemanticType::String))
        );
    }

    #[test]
    fn test_generate_model() {
        let schema = create_test_schema();
        let config = GenerationConfig::default();
        let generator = SchemaGenerator::new(&schema, config);

        let (model, metadata) = generator.generate_model();

        assert_eq!(model.ocsf_version, "1.6.0");
        assert!(!model.entities.is_empty());
        assert!(metadata.stats.entities_generated > 0);
    }

    #[test]
    fn test_high_usage_objects() {
        assert!(HIGH_USAGE_OBJECTS.contains(&"user"));
        assert!(HIGH_USAGE_OBJECTS.contains(&"device"));
        assert!(HIGH_USAGE_OBJECTS.contains(&"actor"));
        assert!(HIGH_USAGE_OBJECTS.contains(&"file"));
        assert!(HIGH_USAGE_OBJECTS.contains(&"process"));
    }
}


/// Metric suggester that generates metrics based on class attributes.
pub struct MetricSuggester<'a> {
    schema: &'a CompiledSchema,
}

impl<'a> MetricSuggester<'a> {
    /// Create a new metric suggester.
    pub fn new(schema: &'a CompiledSchema) -> Self {
        Self { schema }
    }

    /// Suggest metrics for a class.
    pub fn suggest_metrics(&self, class_name: &str) -> Vec<crate::SemanticMetric> {
        let class = match self.schema.get_class(class_name) {
            Some(c) => c,
            None => return vec![],
        };

        let mut metrics = Vec::new();
        let default_dims = self.get_default_dimensions(class);

        // Always suggest event count
        // Task 7.1: Count → Additive
        metrics.push(
            crate::SemanticMetric::new(format!("{}_count", class_name))
                .with_caption(format!("{} Count", class.caption))
                .with_description(format!("Count of {} events", class.caption))
                .with_aggregation(crate::Aggregation::Count)
                .with_metric_type(MetricType::Additive)
                .with_field_measure("metadata.uid")
                .with_dimensions(default_dims.clone())
                .with_time_granularities(vec![
                    crate::TimeGranularity::Minute,
                    crate::TimeGranularity::Hour,
                    crate::TimeGranularity::Day,
                ]),
        );

        // Check for duration attribute
        if class.attributes.contains_key("duration") {
            // Task 7.1: Avg → NonAdditive
            metrics.push(
                crate::SemanticMetric::new(format!("{}_avg_duration", class_name))
                    .with_caption(format!("Average {} Duration", class.caption))
                    .with_description(format!("Average duration of {} events", class.caption))
                    .with_aggregation(crate::Aggregation::Avg)
                    .with_metric_type(MetricType::NonAdditive)
                    .with_field_measure("duration")
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );
        }

        // Check for status_id (success/failure rate)
        if class.attributes.contains_key("status_id") {
            // Task 7.1: Avg → NonAdditive
            metrics.push(
                crate::SemanticMetric::new(format!("{}_success_rate", class_name))
                    .with_caption(format!("{} Success Rate", class.caption))
                    .with_description(format!("Success rate of {} events", class.caption))
                    .with_aggregation(crate::Aggregation::Avg)
                    .with_metric_type(MetricType::NonAdditive)
                    .with_expression_measure("CASE WHEN status_id = 1 THEN 1.0 ELSE 0.0 END")
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );

            // Task 7.3: Generate calculated success rate metric
            // Base metric: success_count (Count of successful events)
            metrics.push(
                crate::SemanticMetric::new(format!("{}_success_count", class_name))
                    .with_caption(format!("{} Success Count", class.caption))
                    .with_description(format!("Count of successful {} events", class.caption))
                    .with_aggregation(crate::Aggregation::Count)
                    .with_metric_type(MetricType::Additive)
                    .with_expression_measure("CASE WHEN status_id = 1 THEN 1 END")
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );

            // Base metric: total_count (Count of all events)
            metrics.push(
                crate::SemanticMetric::new(format!("{}_total_count", class_name))
                    .with_caption(format!("{} Total Count", class.caption))
                    .with_description(format!("Total count of {} events", class.caption))
                    .with_aggregation(crate::Aggregation::Count)
                    .with_metric_type(MetricType::Additive)
                    .with_field_measure("metadata.uid")
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );

            // Calculated metric: success_rate derived from the two base metrics
            metrics.push(
                crate::SemanticMetric::new(format!("{}_calculated_success_rate", class_name))
                    .with_caption(format!("{} Calculated Success Rate", class.caption))
                    .with_description(format!(
                        "Calculated success rate of {} events (success_count / total_count)",
                        class.caption
                    ))
                    .with_aggregation(crate::Aggregation::Avg)
                    .with_metric_type(MetricType::NonAdditive)
                    .with_formula(format!(
                        "{}_success_count / {}_total_count",
                        class_name, class_name
                    ))
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );
        }

        // Check for severity_id
        if class.attributes.contains_key("severity_id") {
            // Task 7.1: Count → Additive
            metrics.push(
                crate::SemanticMetric::new(format!("{}_high_severity_count", class_name))
                    .with_caption(format!("High Severity {} Count", class.caption))
                    .with_description(format!(
                        "Count of {} events with high severity (>= 4)",
                        class.caption
                    ))
                    .with_aggregation(crate::Aggregation::Count)
                    .with_metric_type(MetricType::Additive)
                    .with_expression_measure("CASE WHEN severity_id >= 4 THEN 1 END")
                    .with_dimensions(default_dims.clone())
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );
        }

        // Check for disposition_id
        if class.attributes.contains_key("disposition_id") {
            // Task 7.1: Count → Additive
            metrics.push(
                crate::SemanticMetric::new(format!("{}_blocked_count", class_name))
                    .with_caption(format!("Blocked {} Count", class.caption))
                    .with_description(format!("Count of blocked {} events", class.caption))
                    .with_aggregation(crate::Aggregation::Count)
                    .with_metric_type(MetricType::Additive)
                    .with_expression_measure("CASE WHEN disposition_id = 2 THEN 1 END")
                    .with_dimensions(default_dims)
                    .with_time_granularities(vec![
                        crate::TimeGranularity::Hour,
                        crate::TimeGranularity::Day,
                    ]),
            );
        }

        metrics
    }

    /// Suggest metrics for all classes.
    pub fn suggest_all_metrics(&self) -> Vec<crate::SemanticMetric> {
        let mut all_metrics = Vec::new();
        for (class_name, _) in self.schema.all_classes() {
            all_metrics.extend(self.suggest_metrics(class_name));
        }
        all_metrics
    }

    fn get_default_dimensions(&self, class: &CompiledClass) -> Vec<String> {
        let mut dims = Vec::new();

        for (attr_name, attr) in &class.attributes {
            if !attr.enum_values.is_empty()
                || attr.group.as_deref() == Some("primary")
                || attr.group.as_deref() == Some("classification")
            {
                dims.push(attr_name.clone());
            }
        }

        // Limit to reasonable number of dimensions
        dims.truncate(10);
        dims
    }
}

#[cfg(test)]
mod metric_suggester_tests {
    use super::*;

    fn create_class_with_status() -> CompiledSchema {
        let mut schema = CompiledSchema::default();
        schema.version = "1.6.0".to_string();

        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "status_id".to_string(),
            CompiledAttribute {
                caption: "Status ID".to_string(),
                description: "Status".to_string(),
                attr_type: "integer_t".to_string(),
                type_name: "Integer".to_string(),
                requirement: CompiledRequirement::Required,
                group: Some("primary".to_string()),
                object_type: None,
                object_name: None,
                is_array: false,
                enum_values: {
                    let mut enums = std::collections::HashMap::new();
                    enums.insert(
                        "1".to_string(),
                        ocsf_core::CompiledEnumValue {
                            caption: "Success".to_string(),
                            description: String::new(),
                            deprecated: None,
                        },
                    );
                    enums
                },
                observable: None,
                sibling: Some("status".to_string()),
                profiles: None,
                deprecated: None,
            },
        );
        attrs.insert(
            "severity_id".to_string(),
            CompiledAttribute {
                caption: "Severity ID".to_string(),
                description: "Severity".to_string(),
                attr_type: "integer_t".to_string(),
                type_name: "Integer".to_string(),
                requirement: CompiledRequirement::Recommended,
                group: Some("classification".to_string()),
                object_type: None,
                object_name: None,
                is_array: false,
                enum_values: std::collections::HashMap::new(),
                observable: None,
                sibling: None,
                profiles: None,
                deprecated: None,
            },
        );

        schema.classes.insert(
            "authentication".to_string(),
            CompiledClass {
                uid: 3002,
                name: "authentication".to_string(),
                caption: "Authentication".to_string(),
                description: "Authentication events".to_string(),
                category: "iam".to_string(),
                extends: Some("iam".to_string()),
                attributes: attrs,
                profiles: vec![],
            },
        );

        schema
    }

    #[test]
    fn test_suggest_count_metric() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");
        assert!(!metrics.is_empty());

        let count_metric = metrics.iter().find(|m| m.name == "authentication_count");
        assert!(count_metric.is_some());

        let count = count_metric.unwrap();
        assert_eq!(count.aggregation, crate::Aggregation::Count);
    }

    #[test]
    fn test_suggest_success_rate_metric() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");

        let success_rate = metrics
            .iter()
            .find(|m| m.name == "authentication_success_rate");
        assert!(success_rate.is_some());

        let rate = success_rate.unwrap();
        assert_eq!(rate.aggregation, crate::Aggregation::Avg);
        assert!(rate.measure.expression.is_some());
    }

    #[test]
    fn test_suggest_severity_metric() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");

        let severity = metrics
            .iter()
            .find(|m| m.name == "authentication_high_severity_count");
        assert!(severity.is_some());
    }

    #[test]
    fn test_default_dimensions() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");
        let count_metric = metrics.iter().find(|m| m.name == "authentication_count").unwrap();

        // Should include status_id (enum) and severity_id (classification group)
        assert!(!count_metric.dimensions.is_empty());
    }

    #[test]
    fn test_nonexistent_class() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("nonexistent");
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_metric_type_based_on_aggregation() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");

        // Count metrics should be Additive
        let count = metrics.iter().find(|m| m.name == "authentication_count").unwrap();
        assert_eq!(count.metric_type, MetricType::Additive);

        let severity = metrics.iter().find(|m| m.name == "authentication_high_severity_count").unwrap();
        assert_eq!(severity.metric_type, MetricType::Additive);

        // Avg metrics should be NonAdditive
        let success_rate = metrics.iter().find(|m| m.name == "authentication_success_rate").unwrap();
        assert_eq!(success_rate.metric_type, MetricType::NonAdditive);
    }

    #[test]
    fn test_calculated_success_rate_metric() {
        let schema = create_class_with_status();
        let suggester = MetricSuggester::new(&schema);

        let metrics = suggester.suggest_metrics("authentication");

        // Should have success_count base metric
        let success_count = metrics.iter().find(|m| m.name == "authentication_success_count");
        assert!(success_count.is_some());
        let sc = success_count.unwrap();
        assert_eq!(sc.aggregation, crate::Aggregation::Count);
        assert_eq!(sc.metric_type, MetricType::Additive);
        assert!(sc.measure.expression.is_some());

        // Should have total_count base metric
        let total_count = metrics.iter().find(|m| m.name == "authentication_total_count");
        assert!(total_count.is_some());
        let tc = total_count.unwrap();
        assert_eq!(tc.aggregation, crate::Aggregation::Count);
        assert_eq!(tc.metric_type, MetricType::Additive);

        // Should have calculated success rate metric with formula
        let calc = metrics.iter().find(|m| m.name == "authentication_calculated_success_rate");
        assert!(calc.is_some());
        let calc = calc.unwrap();
        assert_eq!(calc.metric_type, MetricType::NonAdditive);
        assert!(calc.is_calculated());
        assert_eq!(
            calc.formula.as_deref(),
            Some("authentication_success_count / authentication_total_count")
        );
    }

    #[test]
    fn test_geographic_hierarchy_inference() {
        let mut schema = CompiledSchema::default();
        schema.version = "1.6.0".to_string();

        let mut attrs = std::collections::HashMap::new();
        for (name, caption) in &[("country", "Country"), ("region", "Region"), ("city", "City")] {
            attrs.insert(
                name.to_string(),
                CompiledAttribute {
                    caption: caption.to_string(),
                    description: format!("{} field", caption),
                    attr_type: "string_t".to_string(),
                    type_name: "String".to_string(),
                    requirement: CompiledRequirement::Optional,
                    group: None,
                    object_type: None,
                    object_name: None,
                    is_array: false,
                    enum_values: std::collections::HashMap::new(),
                    observable: None,
                    sibling: None,
                    profiles: None,
                    deprecated: None,
                },
            );
        }

        schema.objects.insert(
            "location".to_string(),
            CompiledObject {
                name: "location".to_string(),
                caption: "Location".to_string(),
                description: "Geographic location".to_string(),
                extends: None,
                attributes: attrs,
            },
        );

        let config = GenerationConfig {
            object_filter: vec!["location".to_string()],
            ..Default::default()
        };
        let generator = SchemaGenerator::new(&schema, config);
        let entities = generator.generate_from_objects();

        assert_eq!(entities.len(), 1);
        let entity = &entities[0].entity;

        // The country attribute should have a hierarchy with country → region → city
        let country_attr = entity.get_attribute("country").unwrap();
        assert!(!country_attr.hierarchy.is_empty());
        assert!(country_attr.is_dimension);
        assert_eq!(country_attr.hierarchy.len(), 3);
        assert_eq!(country_attr.hierarchy[0].name, "Country");
        assert_eq!(country_attr.hierarchy[0].attribute_ref, "country");
        assert_eq!(country_attr.hierarchy[1].name, "Region");
        assert_eq!(country_attr.hierarchy[1].attribute_ref, "region");
        assert_eq!(country_attr.hierarchy[2].name, "City");
        assert_eq!(country_attr.hierarchy[2].attribute_ref, "city");
    }

    #[test]
    fn test_no_hierarchy_when_insufficient_geo_fields() {
        let mut schema = CompiledSchema::default();
        schema.version = "1.6.0".to_string();

        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "country".to_string(),
            CompiledAttribute {
                caption: "Country".to_string(),
                description: "Country field".to_string(),
                attr_type: "string_t".to_string(),
                type_name: "String".to_string(),
                requirement: CompiledRequirement::Optional,
                group: None,
                object_type: None,
                object_name: None,
                is_array: false,
                enum_values: std::collections::HashMap::new(),
                observable: None,
                sibling: None,
                profiles: None,
                deprecated: None,
            },
        );

        schema.objects.insert(
            "simple_loc".to_string(),
            CompiledObject {
                name: "simple_loc".to_string(),
                caption: "Simple Location".to_string(),
                description: "Location with only one geo field".to_string(),
                extends: None,
                attributes: attrs,
            },
        );

        let config = GenerationConfig {
            object_filter: vec!["simple_loc".to_string()],
            ..Default::default()
        };
        let generator = SchemaGenerator::new(&schema, config);
        let entities = generator.generate_from_objects();

        assert_eq!(entities.len(), 1);
        let entity = &entities[0].entity;
        let country_attr = entity.get_attribute("country").unwrap();
        // Only one geo field — no hierarchy should be generated
        assert!(country_attr.hierarchy.is_empty());
    }
}
