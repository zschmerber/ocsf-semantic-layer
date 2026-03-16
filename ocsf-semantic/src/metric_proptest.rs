//! Property-based tests for semantic metric serialization round-trip.
//!
//! **Feature: ocsf-semantic-layer, Property 4: Metric Definition Round-Trip**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
//!
//! For any valid semantic metric definition (with aggregation, measure, dimensions,
//! and time granularities), saving then loading should produce an equivalent metric.

use proptest::prelude::*;
use proptest::collection::vec;

use crate::entity::OCSFMapping;
use crate::metric::{Aggregation, SemanticMetric, TimeGranularity};
use crate::model::SemanticModel;

// ============================================================================
// Generators for semantic metric types
// ============================================================================

/// Generate a valid identifier string (lowercase letters and underscores).
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,29}".prop_map(|s| s.to_string())
}

/// Generate a human-readable caption.
fn caption_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,49}".prop_map(|s| s.to_string())
}

/// Generate a description string.
fn description_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,;:!?'-]{0,200}".prop_map(|s| s.to_string())
}

/// Generate an Aggregation enum value.
fn aggregation_strategy() -> impl Strategy<Value = Aggregation> {
    prop_oneof![
        Just(Aggregation::Count),
        Just(Aggregation::Sum),
        Just(Aggregation::Avg),
        Just(Aggregation::Min),
        Just(Aggregation::Max),
        Just(Aggregation::CountDistinct),
    ]
}

/// Generate a TimeGranularity enum value.
fn time_granularity_strategy() -> impl Strategy<Value = TimeGranularity> {
    prop_oneof![
        Just(TimeGranularity::Minute),
        Just(TimeGranularity::Hour),
        Just(TimeGranularity::Day),
        Just(TimeGranularity::Week),
        Just(TimeGranularity::Month),
    ]
}

/// Generate an OCSF field path.
fn field_path_strategy() -> impl Strategy<Value = String> {
    "[a-z_]+\\.[a-z_]+".prop_map(|s| s.to_string())
}

/// Generate a SQL expression for metrics.
fn metric_expression_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "CASE WHEN [a-z_]+ != [0-9]+ THEN [0-9]+\\.[0-9]+ ELSE [0-9]+\\.[0-9]+ END"
            .prop_map(|s| s.to_string()),
        "SUM\\([a-z_]+\\) / COUNT\\(\\*\\)".prop_map(|s| s.to_string()),
        "[a-z_]+ \\* [0-9]+".prop_map(|s| s.to_string()),
    ]
}

/// Generate an OCSFMapping for metrics.
fn ocsf_mapping_strategy() -> impl Strategy<Value = OCSFMapping> {
    prop_oneof![
        // Field-based mapping
        field_path_strategy().prop_map(|field| OCSFMapping::from_field(field)),
        // Expression-based mapping
        metric_expression_strategy().prop_map(|expr| OCSFMapping::from_expression(expr)),
    ]
}

/// Generate a list of dimension names.
fn dimensions_strategy() -> impl Strategy<Value = Vec<String>> {
    vec(identifier_strategy(), 0..5)
}

/// Generate a list of time granularities.
fn time_granularities_strategy() -> impl Strategy<Value = Vec<TimeGranularity>> {
    vec(time_granularity_strategy(), 0..5)
}

/// Generate a SemanticMetric.
fn semantic_metric_strategy() -> impl Strategy<Value = SemanticMetric> {
    (
        identifier_strategy(),
        caption_strategy(),
        description_strategy(),
        aggregation_strategy(),
        ocsf_mapping_strategy(),
        dimensions_strategy(),
        time_granularities_strategy(),
        any::<bool>(),
        proptest::option::of(1u32..100),
    )
        .prop_map(
            |(
                name,
                caption,
                description,
                aggregation,
                measure,
                dimensions,
                time_granularities,
                is_hot_path,
                observable_type_id,
            )| {
                SemanticMetric {
                    name,
                    caption,
                    description,
                    aggregation,
                    measure,
                    dimensions,
                    time_granularities,
                    is_hot_path,
                    observable_type_id,
                }
            },
        )
}

/// Generate a SemanticModel with metrics.
fn semantic_model_with_metrics_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        identifier_strategy(),
        description_strategy(),
        "[0-9]+\\.[0-9]+".prop_map(|s| s.to_string()),
        "[0-9]+\\.[0-9]+\\.[0-9]+".prop_map(|s| s.to_string()),
        vec(semantic_metric_strategy(), 1..5),
    )
        .prop_map(|(name, description, version, ocsf_version, metrics)| {
            SemanticModel::new(name)
                .with_description(description)
                .with_version(version)
                .with_ocsf_version(ocsf_version)
                .with_metrics(metrics)
        })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 4: Metric Definition Round-Trip**
    ///
    /// For any valid Aggregation enum, serializing to JSON then deserializing should
    /// produce the same Aggregation value.
    #[test]
    fn aggregation_json_roundtrip(agg in aggregation_strategy()) {
        let json = serde_json::to_string(&agg).expect("Aggregation should serialize to JSON");
        let deserialized: Aggregation = serde_json::from_str(&json).expect("JSON should deserialize to Aggregation");
        prop_assert_eq!(agg, deserialized, "Aggregation round-trip should preserve value");
    }

    /// For any valid TimeGranularity enum, serializing to JSON then deserializing should
    /// produce the same TimeGranularity value.
    #[test]
    fn time_granularity_json_roundtrip(tg in time_granularity_strategy()) {
        let json = serde_json::to_string(&tg).expect("TimeGranularity should serialize to JSON");
        let deserialized: TimeGranularity = serde_json::from_str(&json).expect("JSON should deserialize to TimeGranularity");
        prop_assert_eq!(tg, deserialized, "TimeGranularity round-trip should preserve value");
    }

    /// **Feature: ocsf-semantic-layer, Property 4: Metric Definition Round-Trip**
    ///
    /// For any valid SemanticMetric, serializing to JSON then deserializing should
    /// produce an equivalent metric with aggregation, measure, dimensions, and
    /// time granularities preserved.
    #[test]
    fn semantic_metric_json_roundtrip(metric in semantic_metric_strategy()) {
        let json = serde_json::to_string(&metric).expect("SemanticMetric should serialize to JSON");
        let deserialized: SemanticMetric = serde_json::from_str(&json).expect("JSON should deserialize to SemanticMetric");

        // Verify all fields
        prop_assert_eq!(&metric.name, &deserialized.name, "Name should be preserved");
        prop_assert_eq!(&metric.caption, &deserialized.caption, "Caption should be preserved");
        prop_assert_eq!(&metric.description, &deserialized.description, "Description should be preserved");
        prop_assert_eq!(metric.aggregation, deserialized.aggregation, "Aggregation should be preserved");
        prop_assert_eq!(&metric.measure, &deserialized.measure, "Measure should be preserved");
        prop_assert_eq!(&metric.dimensions, &deserialized.dimensions, "Dimensions should be preserved");
        prop_assert_eq!(&metric.time_granularities, &deserialized.time_granularities, "Time granularities should be preserved");
        prop_assert_eq!(metric.is_hot_path, deserialized.is_hot_path, "Hot path flag should be preserved");
        prop_assert_eq!(metric.observable_type_id, deserialized.observable_type_id, "Observable type_id should be preserved");

        // Full equality check
        prop_assert_eq!(metric, deserialized, "Full metric round-trip should preserve all data");
    }

    /// **Feature: ocsf-semantic-layer, Property 4: Metric Definition Round-Trip**
    ///
    /// For any valid SemanticMetric, serializing to YAML then deserializing should
    /// produce an equivalent metric with all properties preserved.
    #[test]
    fn semantic_metric_yaml_roundtrip(metric in semantic_metric_strategy()) {
        let yaml = serde_yaml::to_string(&metric).expect("SemanticMetric should serialize to YAML");
        let deserialized: SemanticMetric = serde_yaml::from_str(&yaml).expect("YAML should deserialize to SemanticMetric");
        prop_assert_eq!(metric, deserialized, "YAML round-trip should preserve all data");
    }

    /// **Feature: ocsf-semantic-layer, Property 4: Metric Definition Round-Trip**
    ///
    /// For any valid SemanticModel with metrics, serializing to YAML then deserializing
    /// should produce an equivalent model with all metrics and their properties preserved.
    #[test]
    fn semantic_model_with_metrics_yaml_roundtrip(model in semantic_model_with_metrics_strategy()) {
        let yaml = model.to_yaml().expect("SemanticModel should serialize to YAML");
        let deserialized = SemanticModel::from_yaml(&yaml).expect("YAML should deserialize to SemanticModel");

        // Verify model metadata
        prop_assert_eq!(&model.name, &deserialized.name, "Model name should be preserved");
        prop_assert_eq!(&model.version, &deserialized.version, "Model version should be preserved");
        prop_assert_eq!(&model.ocsf_version, &deserialized.ocsf_version, "OCSF version should be preserved");

        // Verify metrics count
        prop_assert_eq!(
            model.metrics.len(),
            deserialized.metrics.len(),
            "Metrics count should be preserved"
        );

        // Verify each metric
        for (original, deser) in model.metrics.iter().zip(deserialized.metrics.iter()) {
            prop_assert_eq!(original, deser, "Metric content should be preserved");
        }

        // Full equality check
        prop_assert_eq!(model, deserialized, "Full model round-trip should preserve all data");
    }

    /// For any valid SemanticMetric with hot path configuration, the hot path
    /// settings should be preserved through serialization round-trip.
    #[test]
    fn hot_path_metric_preserved(
        name in identifier_strategy(),
        observable_type_id in 1u32..100
    ) {
        let metric = SemanticMetric::new(name)
            .with_observable_type_id(observable_type_id);

        let yaml = serde_yaml::to_string(&metric).expect("Should serialize to YAML");
        let deserialized: SemanticMetric = serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

        prop_assert!(deserialized.is_hot_path, "Hot path flag should be preserved");
        prop_assert_eq!(
            Some(observable_type_id),
            deserialized.observable_type_id,
            "Observable type_id should be preserved"
        );
    }

    /// For any valid SemanticMetric with time granularities, all granularities
    /// should be preserved through serialization round-trip.
    #[test]
    fn time_granularities_preserved(
        name in identifier_strategy(),
        granularities in time_granularities_strategy()
    ) {
        let metric = SemanticMetric::new(name)
            .with_time_granularities(granularities.clone());

        let yaml = serde_yaml::to_string(&metric).expect("Should serialize to YAML");
        let deserialized: SemanticMetric = serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

        prop_assert_eq!(
            granularities,
            deserialized.time_granularities,
            "Time granularities should be preserved"
        );
    }

    /// For any valid SemanticMetric with dimensions, all dimensions
    /// should be preserved through serialization round-trip.
    #[test]
    fn dimensions_preserved(
        name in identifier_strategy(),
        dimensions in dimensions_strategy()
    ) {
        let metric = SemanticMetric::new(name)
            .with_dimensions(dimensions.clone());

        let yaml = serde_yaml::to_string(&metric).expect("Should serialize to YAML");
        let deserialized: SemanticMetric = serde_yaml::from_str(&yaml).expect("Should deserialize from YAML");

        prop_assert_eq!(
            dimensions,
            deserialized.dimensions,
            "Dimensions should be preserved"
        );
    }

    /// For any valid aggregation type, the SQL function name should be non-empty.
    #[test]
    fn aggregation_has_sql_function(agg in aggregation_strategy()) {
        let sql_fn = agg.sql_function();
        prop_assert!(!sql_fn.is_empty(), "SQL function should not be empty");
    }

    /// For any valid time granularity, the SQL date_trunc value should be non-empty.
    #[test]
    fn time_granularity_has_sql_date_trunc(tg in time_granularity_strategy()) {
        let date_trunc = tg.sql_date_trunc();
        prop_assert!(!date_trunc.is_empty(), "SQL date_trunc should not be empty");
    }
}
