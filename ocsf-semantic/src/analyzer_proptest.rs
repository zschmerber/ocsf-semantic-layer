//! Property-based tests for observable coverage analysis.
//!
//! **Feature: ocsf-semantic-layer, Property 16: Observable Coverage Analysis**
//! **Validates: Requirements 10.2, 10.3, 10.4**
//!
//! For any set of observables and semantic entities, if an entity's attributes
//! fully cover an observable's data path, the coverage report should mark that
//! observable as covered.

use proptest::prelude::*;
use proptest::collection::vec;

use ocsf_core::{extract_observables, Category, OCSFSchema, ObservableDefinition};

use crate::analyzer::{analyze_observable_coverage, CoverageStatus, ObservableAnalyzer};
use crate::entity::{SemanticAttribute, SemanticEntity};
use crate::model::SemanticModel;

// ============================================================================
// Generators for semantic entities and observables
// ============================================================================

/// Generate a valid identifier string.
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,19}".prop_map(|s| s.to_string())
}

/// Generate a caption string.
fn caption_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,29}".prop_map(|s| s.to_string())
}

/// Generate a description string.
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,]{0,50}".prop_map(|s| s.to_string())
}

/// Generate an observable type_id (1-100).
fn observable_type_id_strategy() -> impl Strategy<Value = u32> {
    1u32..100
}

/// Generate a set of observable type_ids.
fn observable_type_ids_strategy() -> impl Strategy<Value = Vec<u32>> {
    vec(observable_type_id_strategy(), 0..10)
        .prop_map(|mut v| {
            v.sort();
            v.dedup();
            v
        })
}

/// Generate a SemanticAttribute.
fn semantic_attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (identifier_strategy(), caption_strategy(), description_strategy())
        .prop_map(|(name, caption, description)| {
            SemanticAttribute::new(name)
                .with_caption(caption)
                .with_description(description)
        })
}

/// Generate a SemanticEntity with optional observable coverage.
fn semantic_entity_strategy() -> impl Strategy<Value = SemanticEntity> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        vec(semantic_attribute_strategy(), 0..5),
        observable_type_ids_strategy(),
    )
        .prop_map(|(name, caption, description, attributes, covers_observables)| {
            let mut entity = SemanticEntity::new(name)
                .with_caption(caption)
                .with_description(description)
                .with_covers_observables(covers_observables);
            for attr in attributes {
                entity = entity.add_attribute(attr);
            }
            entity
        })
}

/// Generate a SemanticModel with entities.
fn semantic_model_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        identifier_strategy(),
        vec(semantic_entity_strategy(), 0..5),
    )
        .prop_map(|(name, entities)| {
            let mut model = SemanticModel::new(name);
            for entity in entities {
                model = model.add_entity(entity);
            }
            model
        })
}

/// Generate an OCSF schema with observables.
fn ocsf_schema_with_observables_strategy() -> impl Strategy<Value = OCSFSchema> {
    vec(
        (observable_type_id_strategy(), caption_strategy(), description_strategy()),
        1..10,
    )
        .prop_map(|observables| {
            let mut schema = OCSFSchema::new("1.0.0");

            // Add a category
            schema.add_category(Category {
                uid: 1,
                name: "test".to_string(),
                caption: "Test".to_string(),
                description: "Test category".to_string(),
                event_classes: vec![],
            });

            // Add observables as schema-level definitions
            for (type_id, type_name, description) in observables {
                schema.add_observable(
                    ObservableDefinition::by_type(type_id, &type_name)
                        .with_description(&description),
                );
            }

            schema
        })
}

/// Generate a schema and model pair where some observables are covered.
fn schema_and_model_strategy() -> impl Strategy<Value = (OCSFSchema, SemanticModel)> {
    ocsf_schema_with_observables_strategy()
        .prop_flat_map(|schema| {
            // Get the observable type_ids from the schema
            let type_ids: Vec<u32> = schema.observables.iter().map(|o| o.type_id).collect();
            
            // Generate a model that may cover some of these observables
            (
                Just(schema),
                semantic_model_with_coverage_strategy(type_ids),
            )
        })
}

/// Generate a semantic model that covers some of the given observable type_ids.
fn semantic_model_with_coverage_strategy(
    available_type_ids: Vec<u32>,
) -> impl Strategy<Value = SemanticModel> {
    let type_ids = available_type_ids.clone();
    (
        identifier_strategy(),
        vec(
            (
                identifier_strategy(),
                caption_strategy(),
                // Select a subset of available type_ids to cover
                proptest::sample::subsequence(type_ids.clone(), 0..=type_ids.len()),
            ),
            0..3,
        ),
    )
        .prop_map(|(model_name, entities)| {
            let mut model = SemanticModel::new(model_name);
            for (entity_name, caption, covers) in entities {
                let entity = SemanticEntity::new(entity_name)
                    .with_caption(caption)
                    .with_covers_observables(covers);
                model = model.add_entity(entity);
            }
            model
        })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 16: Observable Coverage Analysis**
    ///
    /// *For any* set of observables and semantic entities, if an entity explicitly
    /// declares coverage of an observable type_id, the coverage report should mark
    /// that observable as fully covered.
    ///
    /// **Validates: Requirements 10.2, 10.3, 10.4**
    #[test]
    fn prop_explicit_coverage_is_detected(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Collect all type_ids that are explicitly covered by entities
        let covered_type_ids: std::collections::HashSet<u32> = model
            .entities
            .iter()
            .flat_map(|e| e.covers_observables.iter().copied())
            .collect();

        // For each observable in the catalog
        for type_id in catalog.type_ids() {
            let status = analyzer.get_coverage_status(*type_id);
            
            if covered_type_ids.contains(type_id) {
                // If explicitly covered, should be marked as FullyCovered
                prop_assert_eq!(
                    status,
                    CoverageStatus::FullyCovered,
                    "Observable type_id {} is explicitly covered but status is {:?}",
                    type_id,
                    status
                );
            }
        }
    }

    /// *For any* observable not covered by any entity, the coverage report should
    /// mark it as not covered.
    #[test]
    fn prop_uncovered_observables_detected(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Collect all type_ids that are explicitly covered by entities
        let covered_type_ids: std::collections::HashSet<u32> = model
            .entities
            .iter()
            .flat_map(|e| e.covers_observables.iter().copied())
            .collect();

        // For each observable in the catalog
        for type_id in catalog.type_ids() {
            let status = analyzer.get_coverage_status(*type_id);
            
            if !covered_type_ids.contains(type_id) {
                // If not covered, should be marked as NotCovered
                prop_assert_eq!(
                    status,
                    CoverageStatus::NotCovered,
                    "Observable type_id {} is not covered but status is {:?}",
                    type_id,
                    status
                );
            }
        }
    }

    /// *For any* coverage report, the counts should be consistent with the details.
    #[test]
    fn prop_coverage_counts_consistent(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        // Count from details
        let fully_covered = report.details.iter()
            .filter(|d| d.status == CoverageStatus::FullyCovered)
            .count();
        let partially_covered = report.details.iter()
            .filter(|d| d.status == CoverageStatus::PartiallyCovered)
            .count();
        let not_covered = report.details.iter()
            .filter(|d| d.status == CoverageStatus::NotCovered)
            .count();

        prop_assert_eq!(
            report.covered_by_semantic_layer,
            fully_covered,
            "covered_by_semantic_layer count mismatch"
        );
        prop_assert_eq!(
            report.partially_covered,
            partially_covered,
            "partially_covered count mismatch"
        );
        prop_assert_eq!(
            report.not_covered,
            not_covered,
            "not_covered count mismatch"
        );

        // Total should match
        let total_from_counts = fully_covered + partially_covered + not_covered;
        prop_assert_eq!(
            report.total_observables,
            total_from_counts,
            "total_observables should equal sum of status counts"
        );
    }

    /// *For any* coverage report, each detail should have a valid recommendation.
    #[test]
    fn prop_all_details_have_recommendations(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        for detail in &report.details {
            prop_assert!(
                !detail.recommendation.is_empty(),
                "Observable type_id {} should have a recommendation",
                detail.type_id
            );
        }
    }

    /// *For any* fully covered observable, the covering_entities list should not be empty.
    #[test]
    fn prop_covered_observables_have_entities(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        for detail in &report.details {
            if detail.status == CoverageStatus::FullyCovered {
                prop_assert!(
                    !detail.covering_entities.is_empty(),
                    "Fully covered observable type_id {} should have covering entities",
                    detail.type_id
                );
            }
        }
    }

    /// *For any* not covered observable, the covering_entities list should be empty.
    #[test]
    fn prop_uncovered_observables_have_no_entities(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        for detail in &report.details {
            if detail.status == CoverageStatus::NotCovered {
                prop_assert!(
                    detail.covering_entities.is_empty(),
                    "Not covered observable type_id {} should have no covering entities",
                    detail.type_id
                );
            }
        }
    }

    /// *For any* coverage report, the coverage percentage should be between 0 and 1.
    #[test]
    fn prop_coverage_percentage_valid_range(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        let overall = report.overall_coverage_percentage();
        prop_assert!(
            (0.0..=1.0).contains(&overall),
            "Overall coverage percentage {} should be between 0 and 1",
            overall
        );

        for detail in &report.details {
            prop_assert!(
                (0.0..=1.0).contains(&detail.coverage_percentage),
                "Coverage percentage {} for type_id {} should be between 0 and 1",
                detail.coverage_percentage,
                detail.type_id
            );
        }
    }

    /// *For any* redundant observable (fully covered), is_observable_redundant should return true.
    #[test]
    fn prop_redundant_detection_consistent(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        let redundant_from_method: std::collections::HashSet<u32> = 
            analyzer.get_redundant_observables().into_iter().collect();

        let redundant_from_report: std::collections::HashSet<u32> = report
            .redundant_observables()
            .map(|d| d.type_id)
            .collect();

        prop_assert_eq!(
            redundant_from_method,
            redundant_from_report,
            "Redundant observables from method and report should match"
        );
    }

    /// *For any* empty model, all observables should be marked as not covered.
    #[test]
    fn prop_empty_model_no_coverage(schema in ocsf_schema_with_observables_strategy()) {
        let catalog = extract_observables(&schema);
        let model = SemanticModel::new("empty");
        let report = analyze_observable_coverage(&catalog, &model);

        prop_assert_eq!(
            report.covered_by_semantic_layer,
            0,
            "Empty model should have no covered observables"
        );
        prop_assert_eq!(
            report.not_covered,
            report.total_observables,
            "All observables should be not covered for empty model"
        );
    }

    /// *For any* empty catalog, the report should have zero observables.
    #[test]
    fn prop_empty_catalog_zero_observables(model in semantic_model_strategy()) {
        let schema = OCSFSchema::new("1.0.0");
        let catalog = extract_observables(&schema);
        let report = analyze_observable_coverage(&catalog, &model);

        prop_assert_eq!(
            report.total_observables,
            0,
            "Empty catalog should have zero observables"
        );
        prop_assert!(
            report.details.is_empty(),
            "Empty catalog should have no details"
        );
    }

    /// *For any* observable-to-entity mapping, the mapped entities should exist in the model.
    #[test]
    fn prop_mapped_entities_exist(
        (schema, model) in schema_and_model_strategy()
    ) {
        let catalog = extract_observables(&schema);
        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        let entity_names: std::collections::HashSet<&str> = 
            model.entities.iter().map(|e| e.name.as_str()).collect();

        for type_id in catalog.type_ids() {
            let mapped_entities = analyzer.map_observable_to_entities(*type_id);
            for entity in mapped_entities {
                prop_assert!(
                    entity_names.contains(entity.name.as_str()),
                    "Mapped entity '{}' should exist in model",
                    entity.name
                );
            }
        }
    }
}
