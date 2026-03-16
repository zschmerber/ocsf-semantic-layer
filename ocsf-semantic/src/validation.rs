//! Semantic model validation.
//!
//! This module provides validation for semantic models against OCSF schemas,
//! including field reference validation, metric determinism checks, and
//! model versioning support.

use std::collections::HashSet;

use ocsf_core::OCSFSchema;
use thiserror::Error;

use crate::entity::{SemanticAttribute, SemanticEntity};
use crate::metric::SemanticMetric;
use crate::model::SemanticModel;

/// Validation error types.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// Invalid field reference in entity attribute mapping.
    #[error("Invalid field reference '{field}' in entity '{entity}', attribute '{attribute}'")]
    InvalidFieldReference {
        entity: String,
        attribute: String,
        field: String,
    },

    /// Invalid field reference in metric measure.
    #[error("Invalid field reference '{field}' in metric '{metric}'")]
    InvalidMetricFieldReference { metric: String, field: String },

    /// Non-deterministic expression in metric.
    #[error("Non-deterministic expression in metric '{metric}': {reason}")]
    NonDeterministicExpression { metric: String, reason: String },

    /// Invalid aggregation for the measure type.
    #[error("Invalid aggregation '{aggregation}' for metric '{metric}': {reason}")]
    InvalidAggregation {
        metric: String,
        aggregation: String,
        reason: String,
    },

    /// Duplicate entity name.
    #[error("Duplicate entity name: '{name}'")]
    DuplicateEntityName { name: String },

    /// Duplicate metric name.
    #[error("Duplicate metric name: '{name}'")]
    DuplicateMetricName { name: String },

    /// Invalid relationship target.
    #[error("Entity '{entity}' has relationship '{relationship}' targeting non-existent entity '{target}'")]
    InvalidRelationshipTarget {
        entity: String,
        relationship: String,
        target: String,
    },

    /// Invalid dimension reference in metric.
    #[error("Metric '{metric}' references non-existent dimension '{dimension}'")]
    InvalidDimensionReference { metric: String, dimension: String },
}

/// Result of validating a semantic model.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (non-fatal issues).
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Creates a new empty validation result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Adds an error to the result.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Adds a warning to the result.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Merges another validation result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Validator for semantic models.
pub struct SemanticModelValidator<'a> {
    schema: &'a OCSFSchema,
    /// Cache of valid field paths for performance.
    valid_paths_cache: HashSet<String>,
}

impl<'a> SemanticModelValidator<'a> {
    /// Creates a new validator with the given OCSF schema.
    pub fn new(schema: &'a OCSFSchema) -> Self {
        let mut validator = Self {
            schema,
            valid_paths_cache: HashSet::new(),
        };
        validator.build_valid_paths_cache();
        validator
    }

    /// Builds a cache of all valid field paths from the schema.
    fn build_valid_paths_cache(&mut self) {
        // Add base attributes
        for attr_name in self.schema.attributes.keys() {
            self.valid_paths_cache.insert(attr_name.clone());
        }

        // Add object paths recursively
        for (obj_name, obj) in &self.schema.objects {
            self.add_object_paths(obj_name, obj, &mut HashSet::new());
        }

        // Add common OCSF paths that are always valid
        self.add_common_ocsf_paths();
    }

    /// Recursively adds object attribute paths to the cache.
    fn add_object_paths(
        &mut self,
        prefix: &str,
        obj: &ocsf_core::OCSFObject,
        visited: &mut HashSet<String>,
    ) {
        // Prevent infinite recursion
        if visited.contains(&obj.name) {
            return;
        }
        visited.insert(obj.name.clone());

        for attr_name in obj.attributes.keys() {
            let path = format!("{}.{}", prefix, attr_name);
            self.valid_paths_cache.insert(path.clone());

            // If this attribute references another object, recurse
            if let Some(attr) = obj.attributes.get(attr_name) {
                if let Some(ref obj_type) = attr.object_type {
                    if let Some(nested_obj) = self.schema.objects.get(obj_type) {
                        self.add_object_paths(&path, nested_obj, visited);
                    }
                }
            }
        }
    }

    /// Adds common OCSF paths that are always valid.
    fn add_common_ocsf_paths(&mut self) {
        // Common top-level fields
        let common_paths = [
            "metadata",
            "metadata.uid",
            "metadata.version",
            "metadata.product",
            "metadata.logged_time",
            "metadata.original_time",
            "time",
            "severity",
            "severity_id",
            "status",
            "status_id",
            "message",
            "activity_id",
            "activity_name",
            "type_uid",
            "type_name",
            "class_uid",
            "class_name",
            "category_uid",
            "category_name",
            // Actor paths
            "actor",
            "actor.user",
            "actor.user.name",
            "actor.user.uid",
            "actor.user.email_addr",
            "actor.user.type",
            "actor.user.type_id",
            "actor.process",
            "actor.session",
            // Source/destination endpoints
            "src_endpoint",
            "src_endpoint.ip",
            "src_endpoint.port",
            "src_endpoint.hostname",
            "src_endpoint.mac",
            "dst_endpoint",
            "dst_endpoint.ip",
            "dst_endpoint.port",
            "dst_endpoint.hostname",
            "dst_endpoint.mac",
            // Device
            "device",
            "device.name",
            "device.uid",
            "device.ip",
            "device.hostname",
            "device.type",
            "device.type_id",
            // User
            "user",
            "user.name",
            "user.uid",
            "user.email_addr",
            "user.type",
            "user.type_id",
            // File
            "file",
            "file.name",
            "file.path",
            "file.size",
            "file.type",
            "file.type_id",
            "file.hashes",
            // Process
            "process",
            "process.name",
            "process.pid",
            "process.cmd_line",
            "process.file",
            // Network
            "connection_info",
            "connection_info.protocol_num",
            "connection_info.direction",
            "connection_info.direction_id",
            // Observables
            "observables",
            "observable_value",
        ];

        for path in common_paths {
            self.valid_paths_cache.insert(path.to_string());
        }
    }

    /// Validates a complete semantic model.
    pub fn validate(&self, model: &SemanticModel) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for duplicate names
        self.validate_unique_names(model, &mut result);

        // Validate entities
        for entity in &model.entities {
            self.validate_entity(entity, &model.entities, &mut result);
        }

        // Validate metrics
        for metric in &model.metrics {
            self.validate_metric(metric, &model.entities, &mut result);
        }

        result
    }

    /// Validates that entity and metric names are unique.
    fn validate_unique_names(&self, model: &SemanticModel, result: &mut ValidationResult) {
        let mut entity_names = HashSet::new();
        for entity in &model.entities {
            if !entity_names.insert(&entity.name) {
                result.add_error(ValidationError::DuplicateEntityName {
                    name: entity.name.clone(),
                });
            }
        }

        let mut metric_names = HashSet::new();
        for metric in &model.metrics {
            if !metric_names.insert(&metric.name) {
                result.add_error(ValidationError::DuplicateMetricName {
                    name: metric.name.clone(),
                });
            }
        }
    }

    /// Validates a single entity.
    fn validate_entity(
        &self,
        entity: &SemanticEntity,
        all_entities: &[SemanticEntity],
        result: &mut ValidationResult,
    ) {
        // Validate attribute mappings
        for attr in &entity.attributes {
            self.validate_attribute_mapping(entity, attr, result);
        }

        // Validate relationships
        let entity_names: HashSet<_> = all_entities.iter().map(|e| e.name.as_str()).collect();
        for rel in &entity.relationships {
            if !entity_names.contains(rel.target_entity.as_str()) {
                result.add_error(ValidationError::InvalidRelationshipTarget {
                    entity: entity.name.clone(),
                    relationship: rel.name.clone(),
                    target: rel.target_entity.clone(),
                });
            }
        }
    }

    /// Validates an attribute's OCSF mapping.
    fn validate_attribute_mapping(
        &self,
        entity: &SemanticEntity,
        attr: &SemanticAttribute,
        result: &mut ValidationResult,
    ) {
        // Validate field reference if present
        if let Some(ref field) = attr.ocsf_mapping.field {
            if !self.is_valid_field_path(field) {
                result.add_error(ValidationError::InvalidFieldReference {
                    entity: entity.name.clone(),
                    attribute: attr.name.clone(),
                    field: field.clone(),
                });
            }
        }

        // Validate expression field references if present
        if let Some(ref expression) = attr.ocsf_mapping.expression {
            for field in extract_field_references(expression) {
                if !self.is_valid_field_path(&field) {
                    result.add_error(ValidationError::InvalidFieldReference {
                        entity: entity.name.clone(),
                        attribute: attr.name.clone(),
                        field,
                    });
                }
            }
        }
    }

    /// Validates a metric definition.
    fn validate_metric(
        &self,
        metric: &SemanticMetric,
        entities: &[SemanticEntity],
        result: &mut ValidationResult,
    ) {
        // Validate measure field reference
        if let Some(ref field) = metric.measure.field {
            if !self.is_valid_field_path(field) {
                result.add_error(ValidationError::InvalidMetricFieldReference {
                    metric: metric.name.clone(),
                    field: field.clone(),
                });
            }
        }

        // Validate measure expression
        if let Some(ref expression) = metric.measure.expression {
            // Check for non-deterministic functions
            if let Some(reason) = check_non_deterministic(expression) {
                result.add_error(ValidationError::NonDeterministicExpression {
                    metric: metric.name.clone(),
                    reason,
                });
            }

            // Validate field references in expression
            for field in extract_field_references(expression) {
                if !self.is_valid_field_path(&field) {
                    result.add_error(ValidationError::InvalidMetricFieldReference {
                        metric: metric.name.clone(),
                        field,
                    });
                }
            }
        }

        // Validate aggregation compatibility
        self.validate_aggregation_compatibility(metric, result);

        // Validate dimension references
        self.validate_dimension_references(metric, entities, result);
    }

    /// Validates that the aggregation is compatible with the measure.
    fn validate_aggregation_compatibility(
        &self,
        metric: &SemanticMetric,
        result: &mut ValidationResult,
    ) {
        use crate::metric::Aggregation;

        // Sum and Avg require numeric fields
        if matches!(metric.aggregation, Aggregation::Sum | Aggregation::Avg) {
            if let Some(ref expression) = metric.measure.expression {
                // Check if expression produces a numeric result
                // This is a heuristic check - expressions with CASE WHEN returning numbers are OK
                let expr_lower = expression.to_lowercase();
                if expr_lower.contains("concat")
                    || (expr_lower.contains("'") && !expr_lower.contains("case"))
                {
                    result.add_error(ValidationError::InvalidAggregation {
                        metric: metric.name.clone(),
                        aggregation: format!("{:?}", metric.aggregation),
                        reason: "SUM/AVG aggregation requires numeric expression".to_string(),
                    });
                }
            }
        }
    }

    /// Validates that dimension references exist in entities.
    fn validate_dimension_references(
        &self,
        metric: &SemanticMetric,
        entities: &[SemanticEntity],
        result: &mut ValidationResult,
    ) {
        // Collect all dimension attributes from entities
        let available_dimensions: HashSet<_> = entities
            .iter()
            .flat_map(|e| e.attributes.iter())
            .filter(|a| a.is_dimension)
            .map(|a| a.name.as_str())
            .collect();

        // Check each metric dimension
        for dim in &metric.dimensions {
            if !available_dimensions.contains(dim.as_str()) {
                result.add_warning(format!(
                    "Metric '{}' references dimension '{}' which is not marked as a dimension in any entity",
                    metric.name, dim
                ));
            }
        }
    }

    /// Checks if a field path is valid according to the schema.
    pub fn is_valid_field_path(&self, path: &str) -> bool {
        // Check cache first
        if self.valid_paths_cache.contains(path) {
            return true;
        }

        // Check if it's a prefix of a valid path (for nested access)
        let path_prefix = format!("{}.", path);
        for valid_path in &self.valid_paths_cache {
            if valid_path.starts_with(&path_prefix) || valid_path == path {
                return true;
            }
        }

        // Check if the first segment is a valid object or attribute
        let segments: Vec<&str> = path.split('.').collect();
        if segments.is_empty() {
            return false;
        }

        // Check if root is a known object
        if self.schema.objects.contains_key(segments[0]) {
            return true;
        }

        // Check if root is a known attribute
        if self.schema.attributes.contains_key(segments[0]) {
            return true;
        }

        false
    }
}

/// Extracts field references from a SQL expression.
///
/// This is a simple heuristic that looks for identifiers that look like
/// OCSF field paths (containing dots or matching known patterns).
pub fn extract_field_references(expression: &str) -> Vec<String> {
    let mut fields = Vec::new();

    // Remove string literals to avoid false positives
    let expr = remove_string_literals(expression);

    // Split by common SQL operators and keywords
    let tokens: Vec<&str> = expr
        .split(|c: char| {
            c.is_whitespace()
                || c == '('
                || c == ')'
                || c == ','
                || c == '='
                || c == '!'
                || c == '<'
                || c == '>'
                || c == '+'
                || c == '-'
                || c == '*'
                || c == '/'
        })
        .filter(|s| !s.is_empty())
        .collect();

    // SQL keywords to ignore
    let sql_keywords: HashSet<&str> = [
        "select", "from", "where", "and", "or", "not", "in", "is", "null", "case", "when", "then",
        "else", "end", "as", "like", "between", "exists", "having", "group", "by", "order", "asc",
        "desc", "limit", "offset", "join", "left", "right", "inner", "outer", "on", "union", "all",
        "distinct", "count", "sum", "avg", "min", "max", "coalesce", "nullif", "cast", "true",
        "false",
    ]
    .into_iter()
    .collect();

    for token in tokens {
        let token_lower = token.to_lowercase();

        // Skip SQL keywords
        if sql_keywords.contains(token_lower.as_str()) {
            continue;
        }

        // Skip numeric literals
        if token.parse::<f64>().is_ok() {
            continue;
        }

        // Include tokens that look like field paths (contain dots or are valid identifiers)
        if token.contains('.') || is_valid_identifier(token) {
            // Skip if it looks like a function call (followed by parenthesis in original)
            if !expression.contains(&format!("{}(", token)) {
                fields.push(token.to_string());
            }
        }
    }

    fields
}

/// Removes string literals from an expression.
fn remove_string_literals(expr: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    for c in expr.chars() {
        if in_string {
            if c == string_char {
                in_string = false;
            }
        } else if c == '\'' || c == '"' {
            in_string = true;
            string_char = c;
        } else {
            result.push(c);
        }
    }

    result
}

/// Checks if a string is a valid SQL identifier.
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
}

/// Checks if an expression contains non-deterministic functions.
///
/// Returns Some(reason) if non-deterministic, None otherwise.
pub fn check_non_deterministic(expression: &str) -> Option<String> {
    let expr_lower = expression.to_lowercase();

    // List of non-deterministic SQL functions
    let non_deterministic_functions = [
        ("random()", "RANDOM() returns different values on each call"),
        ("rand()", "RAND() returns different values on each call"),
        ("uuid()", "UUID() generates different values on each call"),
        (
            "gen_random_uuid()",
            "GEN_RANDOM_UUID() generates different values on each call",
        ),
        ("now()", "NOW() returns current time which changes"),
        (
            "current_timestamp",
            "CURRENT_TIMESTAMP returns current time which changes",
        ),
        (
            "current_date",
            "CURRENT_DATE returns current date which changes",
        ),
        (
            "current_time",
            "CURRENT_TIME returns current time which changes",
        ),
        (
            "getdate()",
            "GETDATE() returns current time which changes",
        ),
        (
            "sysdate",
            "SYSDATE returns current time which changes",
        ),
        (
            "newid()",
            "NEWID() generates different values on each call",
        ),
    ];

    for (func, reason) in non_deterministic_functions {
        if expr_lower.contains(func) {
            return Some(reason.to_string());
        }
    }

    None
}

/// Validates a single entity against the schema.
///
/// This is a convenience function for validating a single entity without
/// a full model context.
pub fn validate_entity(
    entity: &SemanticEntity,
    schema: &OCSFSchema,
) -> Vec<ValidationError> {
    let validator = SemanticModelValidator::new(schema);
    let mut result = ValidationResult::new();
    validator.validate_entity(entity, &[], &mut result);
    result.errors
}

/// Validates a single metric against the schema.
///
/// This is a convenience function for validating a single metric without
/// a full model context.
pub fn validate_metric(
    metric: &SemanticMetric,
    schema: &OCSFSchema,
) -> Vec<ValidationError> {
    let validator = SemanticModelValidator::new(schema);
    let mut result = ValidationResult::new();
    validator.validate_metric(metric, &[], &mut result);
    result.errors
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityRelationship, SemanticAttribute};
    use crate::metric::Aggregation;
    use ocsf_core::{Attribute, Category, EventClass, OCSFObject, Requirement};
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");

        // Add a category
        schema.add_category(Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        });

        // Add base attributes
        schema.add_attribute(Attribute {
            name: "time".to_string(),
            attr_type: "timestamp_t".to_string(),
            caption: "Time".to_string(),
            description: "Event time".to_string(),
            requirement: Requirement::Required,
            observable: None,
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        schema.add_attribute(Attribute {
            name: "severity_id".to_string(),
            attr_type: "integer_t".to_string(),
            caption: "Severity ID".to_string(),
            description: "Severity level".to_string(),
            requirement: Requirement::Required,
            observable: None,
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        // Add user object
        let mut user_attrs = HashMap::new();
        user_attrs.insert(
            "name".to_string(),
            Attribute {
                name: "name".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Name".to_string(),
                description: "User name".to_string(),
                requirement: Requirement::Required,
                observable: Some(10),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );
        user_attrs.insert(
            "email_addr".to_string(),
            Attribute {
                name: "email_addr".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Email".to_string(),
                description: "Email address".to_string(),
                requirement: Requirement::Optional,
                observable: Some(5),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );
        user_attrs.insert(
            "uid".to_string(),
            Attribute {
                name: "uid".to_string(),
                attr_type: "string_t".to_string(),
                caption: "UID".to_string(),
                description: "User ID".to_string(),
                requirement: Requirement::Optional,
                observable: None,
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );

        schema.add_object(OCSFObject {
            name: "user".to_string(),
            caption: "User".to_string(),
            description: "User object".to_string(),
            attributes: user_attrs,
            extends: None,
            observables: vec![],
        });

        // Add actor object with nested user
        let mut actor_attrs = HashMap::new();
        actor_attrs.insert(
            "user".to_string(),
            Attribute {
                name: "user".to_string(),
                attr_type: "object_t".to_string(),
                caption: "User".to_string(),
                description: "Actor user".to_string(),
                requirement: Requirement::Optional,
                observable: None,
                is_array: false,
                object_type: Some("user".to_string()),
                enum_values: HashMap::new(),
                default: None,
            },
        );

        schema.add_object(OCSFObject {
            name: "actor".to_string(),
            caption: "Actor".to_string(),
            description: "Actor object".to_string(),
            attributes: actor_attrs,
            extends: None,
            observables: vec![],
        });

        // Add endpoint object
        let mut endpoint_attrs = HashMap::new();
        endpoint_attrs.insert(
            "ip".to_string(),
            Attribute {
                name: "ip".to_string(),
                attr_type: "ip_t".to_string(),
                caption: "IP Address".to_string(),
                description: "IP address".to_string(),
                requirement: Requirement::Optional,
                observable: Some(2),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );
        endpoint_attrs.insert(
            "hostname".to_string(),
            Attribute {
                name: "hostname".to_string(),
                attr_type: "string_t".to_string(),
                caption: "Hostname".to_string(),
                description: "Hostname".to_string(),
                requirement: Requirement::Optional,
                observable: Some(22),
                is_array: false,
                object_type: None,
                enum_values: HashMap::new(),
                default: None,
            },
        );

        schema.add_object(OCSFObject {
            name: "endpoint".to_string(),
            caption: "Endpoint".to_string(),
            description: "Network endpoint".to_string(),
            attributes: endpoint_attrs,
            extends: None,
            observables: vec![],
        });

        // Add event class
        let mut event_attrs = HashMap::new();
        event_attrs.insert(
            "actor".to_string(),
            ocsf_core::AttributeRef {
                name: "actor".to_string(),
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
            attributes: event_attrs,
            observables: vec![],
            extends: None,
            profiles: vec![],
        });

        schema
    }

    #[test]
    fn test_valid_entity_passes_validation() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new("auth_event")
            .with_source_event_classes(vec![3002])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_field_mapping("actor.user.email_addr")
                    .as_dimension(),
            )
            .add_attribute(
                SemanticAttribute::new("user_name")
                    .with_field_mapping("actor.user.name"),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        assert!(result.is_valid(), "Expected valid model, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_invalid_field_reference_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new("auth_event")
            .add_attribute(
                SemanticAttribute::new("invalid_field")
                    .with_field_mapping("nonexistent.field.path"),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidFieldReference { field, .. } if field == "nonexistent.field.path"
        )));
    }

    #[test]
    fn test_invalid_field_in_expression_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new("auth_event")
            .add_attribute(
                SemanticAttribute::new("computed")
                    .with_expression_mapping("COALESCE(invalid.field, actor.user.name)"),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidFieldReference { field, .. } if field == "invalid.field"
        )));
    }

    #[test]
    fn test_valid_expression_passes() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new("auth_event")
            .add_attribute(
                SemanticAttribute::new("source")
                    .with_expression_mapping("COALESCE(src_endpoint.ip, src_endpoint.hostname)"),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        assert!(result.is_valid(), "Expected valid model, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_duplicate_entity_names_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let model = SemanticModel::new("test")
            .add_entity(SemanticEntity::new("duplicate"))
            .add_entity(SemanticEntity::new("duplicate"));

        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::DuplicateEntityName { name } if name == "duplicate"
        )));
    }

    #[test]
    fn test_duplicate_metric_names_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let model = SemanticModel::new("test")
            .add_metric(SemanticMetric::new("duplicate"))
            .add_metric(SemanticMetric::new("duplicate"));

        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::DuplicateMetricName { name } if name == "duplicate"
        )));
    }

    #[test]
    fn test_invalid_relationship_target_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new("auth_event")
            .add_relationship(EntityRelationship::new(
                "performed_by",
                "nonexistent_entity",
                "auth_event.user_id = nonexistent_entity.id",
            ));

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidRelationshipTarget { target, .. } if target == "nonexistent_entity"
        )));
    }

    #[test]
    fn test_valid_relationship_passes() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let model = SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("auth_event")
                    .add_relationship(EntityRelationship::new(
                        "performed_by",
                        "user",
                        "auth_event.user_email = user.email",
                    )),
            )
            .add_entity(SemanticEntity::new("user"));

        let result = validator.validate(&model);

        assert!(result.is_valid(), "Expected valid model, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_non_deterministic_expression_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let metric = SemanticMetric::new("random_metric")
            .with_expression_measure("RANDOM() * severity_id");

        let model = SemanticModel::new("test").add_metric(metric);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::NonDeterministicExpression { metric, .. } if metric == "random_metric"
        )));
    }

    #[test]
    fn test_non_deterministic_now_detected() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let metric = SemanticMetric::new("time_metric")
            .with_expression_measure("NOW() - time");

        let model = SemanticModel::new("test").add_metric(metric);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::NonDeterministicExpression { .. }
        )));
    }

    #[test]
    fn test_deterministic_expression_passes() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let metric = SemanticMetric::new("auth_rate")
            .with_aggregation(Aggregation::Avg)
            .with_expression_measure("CASE WHEN status_id = 1 THEN 1.0 ELSE 0.0 END");

        let model = SemanticModel::new("test").add_metric(metric);
        let result = validator.validate(&model);

        assert!(result.is_valid(), "Expected valid model, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_invalid_metric_field_reference() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let metric = SemanticMetric::new("bad_metric")
            .with_field_measure("nonexistent.field");

        let model = SemanticModel::new("test").add_metric(metric);
        let result = validator.validate(&model);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidMetricFieldReference { field, .. } if field == "nonexistent.field"
        )));
    }

    #[test]
    fn test_valid_metric_field_reference() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        let metric = SemanticMetric::new("auth_count")
            .with_aggregation(Aggregation::Count)
            .with_field_measure("metadata.uid");

        let model = SemanticModel::new("test").add_metric(metric);
        let result = validator.validate(&model);

        assert!(result.is_valid(), "Expected valid model, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_extract_field_references() {
        let expr = "COALESCE(actor.user.name, actor.user.email_addr)";
        let fields = extract_field_references(expr);

        assert!(fields.contains(&"actor.user.name".to_string()));
        assert!(fields.contains(&"actor.user.email_addr".to_string()));
    }

    #[test]
    fn test_extract_field_references_with_case() {
        let expr = "CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END";
        let fields = extract_field_references(expr);

        assert!(fields.contains(&"status_id".to_string()));
        assert!(!fields.contains(&"success".to_string()));
        assert!(!fields.contains(&"failure".to_string()));
    }

    #[test]
    fn test_check_non_deterministic() {
        assert!(check_non_deterministic("RANDOM()").is_some());
        assert!(check_non_deterministic("NOW()").is_some());
        assert!(check_non_deterministic("CURRENT_TIMESTAMP").is_some());
        assert!(check_non_deterministic("UUID()").is_some());
        assert!(check_non_deterministic("CASE WHEN x = 1 THEN 'a' ELSE 'b' END").is_none());
        assert!(check_non_deterministic("COALESCE(a, b)").is_none());
    }

    #[test]
    fn test_is_valid_field_path() {
        let schema = create_test_schema();
        let validator = SemanticModelValidator::new(&schema);

        // Valid paths
        assert!(validator.is_valid_field_path("time"));
        assert!(validator.is_valid_field_path("severity_id"));
        assert!(validator.is_valid_field_path("actor.user.name"));
        assert!(validator.is_valid_field_path("actor.user.email_addr"));
        assert!(validator.is_valid_field_path("src_endpoint.ip"));
        assert!(validator.is_valid_field_path("metadata.uid"));

        // Invalid paths
        assert!(!validator.is_valid_field_path("completely.invalid.path"));
        assert!(!validator.is_valid_field_path("nonexistent"));
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error(ValidationError::DuplicateEntityName {
            name: "test".to_string(),
        });
        result1.add_warning("Warning 1");

        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::DuplicateMetricName {
            name: "metric".to_string(),
        });
        result2.add_warning("Warning 2");

        result1.merge(result2);

        assert_eq!(result1.errors.len(), 2);
        assert_eq!(result1.warnings.len(), 2);
    }
}
