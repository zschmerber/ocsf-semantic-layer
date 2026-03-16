//! Validation Service for semantic model validation.
//!
//! This module provides the ValidationService which validates semantic models
//! against the loaded OCSF schema, checking field paths, event class existence,
//! and dimension references.
//!
//! # Requirements
//! - 5.1: WHEN a user modifies a field mapping, THE Validation_Engine SHALL
//!   validate the path against the OCSF schema within 500ms
//! - 5.2: IF a field path is invalid, THEN THE Validation_Engine SHALL display
//!   an inline error with the invalid path highlighted
//! - 5.3: THE Validation_Engine SHALL validate that referenced event classes
//!   exist in the loaded schema
//! - 5.4: THE Validation_Engine SHALL validate that metric dimensions reference
//!   existing entity attributes

use std::collections::HashSet;
use std::sync::Arc;

use ocsf_semantic::entity::SemanticEntity;
use ocsf_semantic::model::SemanticModel;
use serde::{Deserialize, Serialize};

use crate::schema_service::SchemaService;

// ============================================================================
// Validation Error Types
// ============================================================================

/// Error code for validation errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationErrorCode {
    /// Invalid field path in OCSF mapping.
    InvalidFieldPath,
    /// Event class does not exist in schema.
    EventClassNotFound,
    /// Dimension references non-existent attribute.
    DimensionNotFound,
    /// Entity not found for metric dimension validation.
    EntityNotFound,
    /// Empty field mapping.
    EmptyMapping,
}

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    /// Error - must be fixed before export.
    Error,
    /// Warning - should be reviewed but not blocking.
    Warning,
}

/// A validation error found during model validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Path to the error location (e.g., "entities[0].attributes[2].ocsf_mapping.field").
    pub path: String,
    /// Human-readable error message.
    pub message: String,
    /// Error code for programmatic handling.
    pub code: ValidationErrorCode,
    /// Severity of the error.
    pub severity: ValidationSeverity,
    /// The invalid value that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_value: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(
        path: impl Into<String>,
        message: impl Into<String>,
        code: ValidationErrorCode,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            code,
            severity: ValidationSeverity::Error,
            invalid_value: None,
        }
    }

    /// Sets the severity to warning.
    pub fn as_warning(mut self) -> Self {
        self.severity = ValidationSeverity::Warning;
        self
    }

    /// Sets the invalid value.
    pub fn with_invalid_value(mut self, value: impl Into<String>) -> Self {
        self.invalid_value = Some(value.into());
        self
    }
}

/// A validation warning found during model validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Path to the warning location.
    pub path: String,
    /// Human-readable warning message.
    pub message: String,
    /// Warning code for programmatic handling.
    pub code: String,
}

impl ValidationWarning {
    /// Creates a new validation warning.
    pub fn new(
        path: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

/// Result of validating a semantic model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the model is valid (no errors).
    pub valid: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Creates a new empty validation report.
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Adds an error to the report.
    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Adds a warning to the report.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Merges another report into this one.
    pub fn merge(&mut self, other: ValidationReport) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

// ============================================================================
// Path Error Type
// ============================================================================

/// Error returned when validating a field path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathError {
    /// The invalid path.
    pub path: String,
    /// The class UID where the path was validated.
    pub class_uid: u32,
    /// Description of what went wrong.
    pub reason: String,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid path '{}' in class {}: {}",
            self.path, self.class_uid, self.reason
        )
    }
}

impl std::error::Error for PathError {}

// ============================================================================
// Validation Service
// ============================================================================

/// Service for validating semantic models against the OCSF schema.
///
/// The ValidationService provides:
/// - Field path validation against the schema
/// - Event class existence validation
/// - Dimension reference validation for metrics
///
/// # Requirements
/// - 5.1: Validate field paths within 500ms
/// - 5.2: Report invalid paths with highlighting
/// - 5.3: Validate event class existence
/// - 5.4: Validate metric dimension references
pub struct ValidationService {
    /// Reference to the schema service for lookups.
    schema: Arc<SchemaService>,
}

impl ValidationService {
    /// Creates a new ValidationService with the given schema service.
    pub fn new(schema: Arc<SchemaService>) -> Self {
        Self { schema }
    }

    /// Validates a complete semantic model against the schema.
    ///
    /// This performs all validation checks:
    /// - Field path validation for all entity attributes
    /// - Event class existence validation
    /// - Dimension reference validation for metrics
    ///
    /// # Arguments
    /// * `model` - The semantic model to validate
    ///
    /// # Returns
    /// A ValidationReport containing all errors and warnings.
    pub async fn validate(&self, model: &SemanticModel) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Validate each entity
        for (entity_idx, entity) in model.entities.iter().enumerate() {
            let entity_errors = self.validate_entity(entity, entity_idx).await;
            for error in entity_errors {
                report.add_error(error);
            }
        }

        // Validate each metric's dimension references
        for (metric_idx, metric) in model.metrics.iter().enumerate() {
            let metric_errors =
                self.validate_metric_dimensions(metric, &model.entities, metric_idx);
            for error in metric_errors {
                report.add_error(error);
            }
        }

        report
    }

    /// Validates a field path against the OCSF schema for a specific class.
    ///
    /// # Arguments
    /// * `class_uid` - The class UID to validate the path against
    /// * `path` - The dot-separated field path (e.g., "actor.user.name")
    ///
    /// # Returns
    /// Ok(()) if the path is valid, or Err(PathError) if invalid.
    ///
    /// # Requirements
    /// - 5.1: Validate the path against the OCSF schema within 500ms
    /// - 5.2: Return error with the invalid path for highlighting
    pub async fn validate_field_path(&self, class_uid: u32, path: &str) -> Result<(), PathError> {
        // Empty paths are invalid
        if path.is_empty() {
            return Err(PathError {
                path: path.to_string(),
                class_uid,
                reason: "Field path cannot be empty".to_string(),
            });
        }

        // Check if the schema is loaded
        if !self.schema.is_loaded().await {
            return Err(PathError {
                path: path.to_string(),
                class_uid,
                reason: "Schema not loaded".to_string(),
            });
        }

        // Try to resolve the attribute path
        match self.schema.get_attribute(class_uid, path).await {
            Some(_) => Ok(()),
            None => Err(PathError {
                path: path.to_string(),
                class_uid,
                reason: format!("Path '{}' does not exist in class {}", path, class_uid),
            }),
        }
    }

    /// Validates a semantic entity against the schema.
    ///
    /// Checks:
    /// - All source event classes exist in the schema
    /// - All field mappings are valid paths in the source classes
    ///
    /// # Arguments
    /// * `entity` - The entity to validate
    /// * `entity_idx` - Index of the entity in the model (for error paths)
    ///
    /// # Returns
    /// A vector of validation errors found.
    ///
    /// # Requirements
    /// - 5.3: Validate that referenced event classes exist in the loaded schema
    pub async fn validate_entity(
        &self,
        entity: &SemanticEntity,
        entity_idx: usize,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Get valid class UIDs from the schema
        let valid_class_uids = self.get_valid_class_uids().await;

        // Validate source event classes exist (Requirement 5.3)
        for (class_idx, &class_uid) in entity.source_event_classes.iter().enumerate() {
            if !valid_class_uids.contains(&class_uid) {
                errors.push(
                    ValidationError::new(
                        format!(
                            "entities[{}].source_event_classes[{}]",
                            entity_idx, class_idx
                        ),
                        format!(
                            "Event class {} does not exist in the loaded schema",
                            class_uid
                        ),
                        ValidationErrorCode::EventClassNotFound,
                    )
                    .with_invalid_value(class_uid.to_string()),
                );
            }
        }

        // Validate field mappings for each attribute
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            // Only validate field mappings (not expressions)
            if let Some(ref field) = attr.ocsf_mapping.field {
                // Validate against each source event class
                for &class_uid in &entity.source_event_classes {
                    // Skip validation for classes that don't exist
                    if !valid_class_uids.contains(&class_uid) {
                        continue;
                    }

                    if let Err(path_error) = self.validate_field_path(class_uid, field).await {
                        errors.push(
                            ValidationError::new(
                                format!(
                                    "entities[{}].attributes[{}].ocsf_mapping.field",
                                    entity_idx, attr_idx
                                ),
                                format!(
                                    "Invalid field path '{}' for attribute '{}' in class {}: {}",
                                    field, attr.name, class_uid, path_error.reason
                                ),
                                ValidationErrorCode::InvalidFieldPath,
                            )
                            .with_invalid_value(field.clone()),
                        );
                    }
                }
            }
        }

        errors
    }

    /// Validates metric dimension references against entity attributes.
    ///
    /// # Arguments
    /// * `metric` - The metric to validate
    /// * `entities` - All entities in the model
    /// * `metric_idx` - Index of the metric in the model (for error paths)
    ///
    /// # Returns
    /// A vector of validation errors found.
    ///
    /// # Requirements
    /// - 5.4: Validate that metric dimensions reference existing entity attributes
    pub fn validate_metric_dimensions(
        &self,
        metric: &ocsf_semantic::metric::SemanticMetric,
        entities: &[SemanticEntity],
        metric_idx: usize,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Build a set of all available dimension attribute names from all entities
        let available_dimensions: HashSet<&str> = entities
            .iter()
            .flat_map(|e| e.attributes.iter())
            .filter(|a| a.is_dimension)
            .map(|a| a.name.as_str())
            .collect();

        // Also include all attribute names (not just dimensions) for more flexible validation
        let all_attributes: HashSet<&str> = entities
            .iter()
            .flat_map(|e| e.attributes.iter())
            .map(|a| a.name.as_str())
            .collect();

        // Validate each dimension reference
        for (dim_idx, dimension) in metric.dimensions.iter().enumerate() {
            if !all_attributes.contains(dimension.as_str()) {
                errors.push(
                    ValidationError::new(
                        format!("metrics[{}].dimensions[{}]", metric_idx, dim_idx),
                        format!(
                            "Dimension '{}' for metric '{}' does not reference an existing entity attribute",
                            dimension, metric.name
                        ),
                        ValidationErrorCode::DimensionNotFound,
                    )
                    .with_invalid_value(dimension.clone()),
                );
            } else if !available_dimensions.contains(dimension.as_str()) {
                // Attribute exists but is not marked as a dimension - add as warning
                // (This is a softer check - the attribute exists but isn't flagged as dimension)
            }
        }

        errors
    }

    /// Gets all valid class UIDs from the loaded schema.
    async fn get_valid_class_uids(&self) -> HashSet<u32> {
        let tree = self.schema.get_tree().await;
        tree.categories
            .iter()
            .flat_map(|cat| cat.classes.iter())
            .map(|class| class.uid)
            .collect()
    }

    /// Checks if a specific event class exists in the schema.
    ///
    /// # Arguments
    /// * `class_uid` - The class UID to check
    ///
    /// # Returns
    /// true if the class exists, false otherwise.
    pub async fn class_exists(&self, class_uid: u32) -> bool {
        let valid_uids = self.get_valid_class_uids().await;
        valid_uids.contains(&class_uid)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::entity::{SemanticAttribute, SemanticType};
    use ocsf_semantic::metric::{Aggregation, SemanticMetric, TimeGranularity};

    /// Create a minimal test schema JSON.
    fn test_schema_json() -> &'static str {
        r#"{
            "version": "1.4.0",
            "categories": {
                "attributes": {
                    "iam": {
                        "uid": 3,
                        "caption": "Identity & Access Management",
                        "description": "IAM events for authentication and authorization"
                    },
                    "network": {
                        "uid": 4,
                        "caption": "Network Activity",
                        "description": "Network traffic and connection events"
                    }
                },
                "caption": "Categories",
                "description": "Event categories",
                "name": "categories"
            },
            "classes": {
                "authentication": {
                    "uid": 3002,
                    "name": "authentication",
                    "caption": "Authentication",
                    "description": "Authentication events for user login and logout",
                    "category": "iam",
                    "attributes": {
                        "activity_id": {
                            "caption": "Activity ID",
                            "description": "The normalized identifier of the activity",
                            "type": "integer_t",
                            "type_name": "Integer",
                            "requirement": "required",
                            "enum": {
                                "0": {"caption": "Unknown", "description": "Unknown activity"},
                                "1": {"caption": "Logon", "description": "User logon"}
                            }
                        },
                        "actor": {
                            "caption": "Actor",
                            "description": "The actor that performed the authentication",
                            "type": "object_t",
                            "type_name": "Actor",
                            "object_type": "actor",
                            "requirement": "recommended"
                        },
                        "status_id": {
                            "caption": "Status ID",
                            "description": "The status of the authentication",
                            "type": "integer_t",
                            "type_name": "Integer",
                            "requirement": "required"
                        }
                    }
                }
            },
            "objects": {
                "actor": {
                    "name": "actor",
                    "caption": "Actor",
                    "description": "The actor object describes the entity that performed the activity",
                    "attributes": {
                        "user": {
                            "caption": "User",
                            "description": "The user that performed the activity",
                            "type": "object_t",
                            "type_name": "User",
                            "object_type": "user",
                            "requirement": "recommended"
                        }
                    }
                },
                "user": {
                    "name": "user",
                    "caption": "User",
                    "description": "The user object describes a person or service account",
                    "attributes": {
                        "name": {
                            "caption": "Name",
                            "description": "The user name",
                            "type": "string_t",
                            "type_name": "String",
                            "requirement": "recommended"
                        },
                        "email_addr": {
                            "caption": "Email Address",
                            "description": "The user email address",
                            "type": "email_t",
                            "type_name": "Email",
                            "requirement": "optional"
                        }
                    }
                }
            },
            "profiles": {},
            "extensions": {}
        }"#
    }

    /// Helper to create a validation service with test schema loaded.
    async fn create_test_service() -> ValidationService {
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();
        ValidationService::new(schema_service)
    }

    /// Helper to create a test entity.
    fn create_test_entity() -> SemanticEntity {
        SemanticEntity::new("authentication_event")
            .with_caption("Authentication Event")
            .with_description("User authentication attempts")
            .with_source_event_classes(vec![3002])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_caption("User Email")
                    .with_type(SemanticType::String)
                    .with_field_mapping("actor.user.email_addr")
                    .as_dimension(),
            )
            .add_attribute(
                SemanticAttribute::new("user_name")
                    .with_caption("User Name")
                    .with_type(SemanticType::String)
                    .with_field_mapping("actor.user.name")
                    .as_dimension(),
            )
            .add_attribute(
                SemanticAttribute::new("status")
                    .with_caption("Status")
                    .with_type(SemanticType::Integer)
                    .with_field_mapping("status_id"),
            )
    }

    /// Helper to create a test metric.
    fn create_test_metric() -> SemanticMetric {
        SemanticMetric::new("auth_attempts")
            .with_caption("Authentication Attempts")
            .with_description("Count of authentication attempts")
            .with_aggregation(Aggregation::Count)
            .with_field_measure("activity_id")
            .with_dimensions(vec!["user_email".to_string(), "user_name".to_string()])
            .with_time_granularities(vec![TimeGranularity::Hour, TimeGranularity::Day])
    }

    // ========================================================================
    // Field Path Validation Tests (Requirement 5.1, 5.2)
    // ========================================================================

    #[tokio::test]
    async fn test_validate_field_path_simple_valid() {
        let service = create_test_service().await;
        let result = service.validate_field_path(3002, "activity_id").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_field_path_nested_valid() {
        let service = create_test_service().await;
        let result = service.validate_field_path(3002, "actor.user.name").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_field_path_deeply_nested_valid() {
        let service = create_test_service().await;
        let result = service
            .validate_field_path(3002, "actor.user.email_addr")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_field_path_invalid_simple() {
        let service = create_test_service().await;
        let result = service.validate_field_path(3002, "nonexistent_field").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "nonexistent_field");
        assert_eq!(err.class_uid, 3002);
    }

    #[tokio::test]
    async fn test_validate_field_path_invalid_nested() {
        let service = create_test_service().await;
        let result = service
            .validate_field_path(3002, "actor.nonexistent.field")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_field_path_empty() {
        let service = create_test_service().await;
        let result = service.validate_field_path(3002, "").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.reason.contains("empty"));
    }

    #[tokio::test]
    async fn test_validate_field_path_invalid_class() {
        let service = create_test_service().await;
        let result = service.validate_field_path(99999, "activity_id").await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Event Class Existence Validation Tests (Requirement 5.3)
    // ========================================================================

    #[tokio::test]
    async fn test_class_exists_valid() {
        let service = create_test_service().await;
        assert!(service.class_exists(3002).await);
    }

    #[tokio::test]
    async fn test_class_exists_invalid() {
        let service = create_test_service().await;
        assert!(!service.class_exists(99999).await);
    }

    #[tokio::test]
    async fn test_validate_entity_valid_class() {
        let service = create_test_service().await;
        let entity = create_test_entity();
        let errors = service.validate_entity(&entity, 0).await;

        // Should have no event class errors
        let class_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::EventClassNotFound)
            .collect();
        assert!(class_errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_entity_invalid_class() {
        let service = create_test_service().await;
        let entity = SemanticEntity::new("test").with_source_event_classes(vec![99999, 88888]);

        let errors = service.validate_entity(&entity, 0).await;

        // Should have errors for both invalid classes
        let class_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::EventClassNotFound)
            .collect();
        assert_eq!(class_errors.len(), 2);

        // Check error paths
        assert!(class_errors[0].path.contains("source_event_classes[0]"));
        assert!(class_errors[1].path.contains("source_event_classes[1]"));
    }

    #[tokio::test]
    async fn test_validate_entity_mixed_valid_invalid_classes() {
        let service = create_test_service().await;
        let entity = SemanticEntity::new("test").with_source_event_classes(vec![3002, 99999]); // 3002 is valid, 99999 is not

        let errors = service.validate_entity(&entity, 0).await;

        let class_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::EventClassNotFound)
            .collect();
        assert_eq!(class_errors.len(), 1);
        assert!(class_errors[0]
            .invalid_value
            .as_ref()
            .unwrap()
            .contains("99999"));
    }

    // ========================================================================
    // Dimension Reference Validation Tests (Requirement 5.4)
    // ========================================================================

    #[tokio::test]
    async fn test_validate_metric_dimensions_valid() {
        let service = create_test_service().await;
        let entity = create_test_entity();
        let metric = create_test_metric();

        let errors = service.validate_metric_dimensions(&metric, &[entity], 0);
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_metric_dimensions_invalid() {
        let service = create_test_service().await;
        let entity = create_test_entity();
        let metric = SemanticMetric::new("test_metric")
            .with_dimensions(vec!["nonexistent_dimension".to_string()]);

        let errors = service.validate_metric_dimensions(&metric, &[entity], 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::DimensionNotFound);
        assert!(errors[0].path.contains("dimensions[0]"));
        assert!(errors[0]
            .invalid_value
            .as_ref()
            .unwrap()
            .contains("nonexistent_dimension"));
    }

    #[tokio::test]
    async fn test_validate_metric_dimensions_mixed() {
        let service = create_test_service().await;
        let entity = create_test_entity();
        let metric = SemanticMetric::new("test_metric").with_dimensions(vec![
            "user_email".to_string(),  // valid
            "nonexistent".to_string(), // invalid
            "user_name".to_string(),   // valid
        ]);

        let errors = service.validate_metric_dimensions(&metric, &[entity], 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .invalid_value
            .as_ref()
            .unwrap()
            .contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_validate_metric_dimensions_empty() {
        let service = create_test_service().await;
        let entity = create_test_entity();
        let metric = SemanticMetric::new("test_metric"); // no dimensions

        let errors = service.validate_metric_dimensions(&metric, &[entity], 0);
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_metric_dimensions_multiple_entities() {
        let service = create_test_service().await;

        let entity1 = SemanticEntity::new("entity1")
            .add_attribute(SemanticAttribute::new("attr1").as_dimension());
        let entity2 = SemanticEntity::new("entity2")
            .add_attribute(SemanticAttribute::new("attr2").as_dimension());

        let metric = SemanticMetric::new("test_metric")
            .with_dimensions(vec!["attr1".to_string(), "attr2".to_string()]);

        let errors = service.validate_metric_dimensions(&metric, &[entity1, entity2], 0);
        assert!(errors.is_empty());
    }

    // ========================================================================
    // Full Model Validation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_validate_model_valid() {
        let service = create_test_service().await;
        let model = SemanticModel::new("test-model")
            .add_entity(create_test_entity())
            .add_metric(create_test_metric());

        let report = service.validate(&model).await;
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_model_invalid_field_path() {
        let service = create_test_service().await;

        let entity = SemanticEntity::new("test")
            .with_source_event_classes(vec![3002])
            .add_attribute(
                SemanticAttribute::new("bad_attr").with_field_mapping("nonexistent.path"),
            );

        let model = SemanticModel::new("test-model").add_entity(entity);

        let report = service.validate(&model).await;
        assert!(!report.valid);

        let path_errors: Vec<_> = report
            .errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::InvalidFieldPath)
            .collect();
        assert!(!path_errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_model_invalid_class_and_dimension() {
        let service = create_test_service().await;

        let entity = SemanticEntity::new("test")
            .with_source_event_classes(vec![99999]) // invalid class
            .add_attribute(SemanticAttribute::new("attr1").as_dimension());

        let metric =
            SemanticMetric::new("test_metric").with_dimensions(vec!["nonexistent".to_string()]); // invalid dimension

        let model = SemanticModel::new("test-model")
            .add_entity(entity)
            .add_metric(metric);

        let report = service.validate(&model).await;
        assert!(!report.valid);

        // Should have both class and dimension errors
        let class_errors: Vec<_> = report
            .errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::EventClassNotFound)
            .collect();
        let dim_errors: Vec<_> = report
            .errors
            .iter()
            .filter(|e| e.code == ValidationErrorCode::DimensionNotFound)
            .collect();

        assert!(!class_errors.is_empty());
        assert!(!dim_errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_model_empty() {
        let service = create_test_service().await;
        let model = SemanticModel::new("empty-model");

        let report = service.validate(&model).await;
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    // ========================================================================
    // Validation Report Tests
    // ========================================================================

    #[test]
    fn test_validation_report_new() {
        let report = ValidationReport::new();
        assert!(report.valid);
        assert!(report.errors.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_validation_report_add_error() {
        let mut report = ValidationReport::new();
        report.add_error(ValidationError::new(
            "test.path",
            "Test error",
            ValidationErrorCode::InvalidFieldPath,
        ));

        assert!(!report.valid);
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_validation_report_add_warning() {
        let mut report = ValidationReport::new();
        report.add_warning(ValidationWarning::new(
            "test.path",
            "Test warning",
            "TEST_WARNING",
        ));

        assert!(report.valid); // Warnings don't affect validity
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn test_validation_report_merge() {
        let mut report1 = ValidationReport::new();
        report1.add_error(ValidationError::new(
            "path1",
            "Error 1",
            ValidationErrorCode::InvalidFieldPath,
        ));

        let mut report2 = ValidationReport::new();
        report2.add_error(ValidationError::new(
            "path2",
            "Error 2",
            ValidationErrorCode::EventClassNotFound,
        ));
        report2.add_warning(ValidationWarning::new("path3", "Warning 1", "WARN"));

        report1.merge(report2);

        assert!(!report1.valid);
        assert_eq!(report1.errors.len(), 2);
        assert_eq!(report1.warnings.len(), 1);
    }

    #[test]
    fn test_validation_error_with_invalid_value() {
        let error = ValidationError::new(
            "test.path",
            "Test error",
            ValidationErrorCode::InvalidFieldPath,
        )
        .with_invalid_value("bad_value");

        assert_eq!(error.invalid_value, Some("bad_value".to_string()));
    }

    #[test]
    fn test_validation_error_as_warning() {
        let error = ValidationError::new(
            "test.path",
            "Test error",
            ValidationErrorCode::InvalidFieldPath,
        )
        .as_warning();

        assert_eq!(error.severity, ValidationSeverity::Warning);
    }
}
