//! Property-based tests for semantic model validation.
//!
//! **Feature: ocsf-semantic-layer, Property 3: Invalid Field Reference Detection**
//! **Validates: Requirements 2.5**

use proptest::prelude::*;
use std::collections::HashMap;

use ocsf_core::{Attribute, Category, OCSFObject, OCSFSchema, Requirement};

use crate::entity::{SemanticAttribute, SemanticEntity};
use crate::model::SemanticModel;
use crate::validation::{SemanticModelValidator, ValidationError};

/// Strategy for generating invalid field paths that definitely don't exist.
fn invalid_field_path_strategy() -> impl Strategy<Value = String> {
    // Generate paths that are clearly invalid by using a prefix that won't exist
    prop::string::string_regex("__invalid__[a-z0-9_]{1,10}(\\.[a-z0-9_]{1,10}){0,3}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy for generating entity names.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,29}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy for generating attribute names.
fn attribute_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,19}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Creates a minimal test schema with known valid paths.
fn create_minimal_schema() -> OCSFSchema {
    let mut schema = OCSFSchema::new("1.4.0");

    // Add a category
    schema.add_category(Category {
        uid: 1,
        name: "test".to_string(),
        caption: "Test".to_string(),
        description: "Test category".to_string(),
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
        caption: "Severity".to_string(),
        description: "Severity level".to_string(),
        requirement: Requirement::Required,
        observable: None,
        is_array: false,
        object_type: None,
        enum_values: HashMap::new(),
        default: None,
    });

    // Add a simple object
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
        "email".to_string(),
        Attribute {
            name: "email".to_string(),
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

    schema.add_object(OCSFObject {
        name: "user".to_string(),
        caption: "User".to_string(),
        description: "User object".to_string(),
        attributes: user_attrs,
        extends: None,
        observables: vec![],
    });

    schema
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 3: Invalid Field Reference Detection**
    ///
    /// *For any* semantic entity definition that references non-existent OCSF fields,
    /// validation against a schema should return errors identifying the specific
    /// invalid field references.
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn prop_invalid_field_reference_detected(
        entity_name in entity_name_strategy(),
        attr_name in attribute_name_strategy(),
        invalid_path in invalid_field_path_strategy(),
    ) {
        let schema = create_minimal_schema();
        let validator = SemanticModelValidator::new(&schema);

        // Create an entity with an invalid field reference
        let entity = SemanticEntity::new(&entity_name)
            .add_attribute(
                SemanticAttribute::new(&attr_name)
                    .with_field_mapping(&invalid_path),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        // The validation should fail
        prop_assert!(!result.is_valid(),
            "Expected validation to fail for invalid path '{}', but it passed", invalid_path);

        // There should be at least one InvalidFieldReference error
        let has_invalid_field_error = result.errors.iter().any(|e| {
            matches!(e, ValidationError::InvalidFieldReference { field, .. } if field == &invalid_path)
        });

        prop_assert!(has_invalid_field_error,
            "Expected InvalidFieldReference error for path '{}', got errors: {:?}",
            invalid_path, result.errors);
    }

    /// Property: Valid field references should pass validation.
    ///
    /// *For any* semantic entity with valid OCSF field references,
    /// validation should not produce InvalidFieldReference errors.
    #[test]
    fn prop_valid_field_reference_passes(
        entity_name in entity_name_strategy(),
        attr_name in attribute_name_strategy(),
    ) {
        let schema = create_minimal_schema();
        let validator = SemanticModelValidator::new(&schema);

        // Use known valid paths from the schema
        let valid_paths = vec![
            "time",
            "severity_id",
            "user.name",
            "user.email",
            "metadata.uid",
            "actor.user.name",
        ];

        for valid_path in valid_paths {
            let entity = SemanticEntity::new(&entity_name)
                .add_attribute(
                    SemanticAttribute::new(&attr_name)
                        .with_field_mapping(valid_path),
                );

            let model = SemanticModel::new("test").add_entity(entity);
            let result = validator.validate(&model);

            // Should not have InvalidFieldReference errors for this path
            let has_invalid_field_error = result.errors.iter().any(|e| {
                matches!(e, ValidationError::InvalidFieldReference { field, .. } if field == valid_path)
            });

            prop_assert!(!has_invalid_field_error,
                "Valid path '{}' was incorrectly flagged as invalid", valid_path);
        }
    }

    /// Property: Invalid field references in expressions should be detected.
    ///
    /// *For any* semantic entity with an expression containing invalid field references,
    /// validation should identify the specific invalid fields.
    #[test]
    fn prop_invalid_field_in_expression_detected(
        entity_name in entity_name_strategy(),
        attr_name in attribute_name_strategy(),
        invalid_path in invalid_field_path_strategy(),
    ) {
        let schema = create_minimal_schema();
        let validator = SemanticModelValidator::new(&schema);

        // Create an expression that includes the invalid path
        let expression = format!("COALESCE({}, user.name)", invalid_path);

        let entity = SemanticEntity::new(&entity_name)
            .add_attribute(
                SemanticAttribute::new(&attr_name)
                    .with_expression_mapping(&expression),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        // The validation should fail
        prop_assert!(!result.is_valid(),
            "Expected validation to fail for expression with invalid path '{}', but it passed",
            invalid_path);

        // There should be an InvalidFieldReference error for the invalid path
        let has_invalid_field_error = result.errors.iter().any(|e| {
            matches!(e, ValidationError::InvalidFieldReference { field, .. } if field == &invalid_path)
        });

        prop_assert!(has_invalid_field_error,
            "Expected InvalidFieldReference error for path '{}' in expression, got errors: {:?}",
            invalid_path, result.errors);
    }

    /// Property: Multiple invalid field references should all be reported.
    ///
    /// *For any* semantic entity with multiple invalid field references,
    /// validation should report all of them.
    #[test]
    fn prop_multiple_invalid_fields_all_reported(
        entity_name in entity_name_strategy(),
        attr1_name in attribute_name_strategy(),
        attr2_name in attribute_name_strategy().prop_filter("different", |s| s != "attr1"),
        invalid_path1 in invalid_field_path_strategy(),
        invalid_path2 in invalid_field_path_strategy().prop_filter("different", |s| !s.starts_with("__invalid__a")),
    ) {
        // Skip if paths are the same
        prop_assume!(invalid_path1 != invalid_path2);
        prop_assume!(attr1_name != attr2_name);

        let schema = create_minimal_schema();
        let validator = SemanticModelValidator::new(&schema);

        let entity = SemanticEntity::new(&entity_name)
            .add_attribute(
                SemanticAttribute::new(&attr1_name)
                    .with_field_mapping(&invalid_path1),
            )
            .add_attribute(
                SemanticAttribute::new(&attr2_name)
                    .with_field_mapping(&invalid_path2),
            );

        let model = SemanticModel::new("test").add_entity(entity);
        let result = validator.validate(&model);

        // Both invalid paths should be reported
        let has_error1 = result.errors.iter().any(|e| {
            matches!(e, ValidationError::InvalidFieldReference { field, .. } if field == &invalid_path1)
        });
        let has_error2 = result.errors.iter().any(|e| {
            matches!(e, ValidationError::InvalidFieldReference { field, .. } if field == &invalid_path2)
        });

        prop_assert!(has_error1,
            "Expected error for first invalid path '{}', got: {:?}",
            invalid_path1, result.errors);
        prop_assert!(has_error2,
            "Expected error for second invalid path '{}', got: {:?}",
            invalid_path2, result.errors);
    }
}
