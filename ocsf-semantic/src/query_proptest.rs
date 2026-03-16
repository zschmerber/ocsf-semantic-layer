//! Property-based tests for query translation.
//!
//! These tests verify the correctness properties of the query translator.

use proptest::prelude::*;

use crate::entity::{SemanticAttribute, SemanticEntity, SemanticType};
use crate::metric::{Aggregation, SemanticMetric, TimeGranularity};
use crate::model::SemanticModel;
use crate::query::{
    QueryValidator, SemanticQuery, SqlGenerator, WarehouseDialect,
};

/// Strategy for generating valid attribute names.
fn attr_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,15}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid entity names.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid metric names.
fn metric_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating OCSF field paths.
fn field_path_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}(\\.[a-z][a-z0-9_]{1,10}){0,3}".prop_map(|s| s.to_string())
}

/// Strategy for generating semantic types.
fn semantic_type_strategy() -> impl Strategy<Value = SemanticType> {
    prop_oneof![
        Just(SemanticType::String),
        Just(SemanticType::Integer),
        Just(SemanticType::Float),
        Just(SemanticType::Boolean),
        Just(SemanticType::Timestamp),
    ]
}

/// Strategy for generating aggregation types.
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

/// Strategy for generating time granularities.
fn time_granularity_strategy() -> impl Strategy<Value = TimeGranularity> {
    prop_oneof![
        Just(TimeGranularity::Minute),
        Just(TimeGranularity::Hour),
        Just(TimeGranularity::Day),
        Just(TimeGranularity::Week),
        Just(TimeGranularity::Month),
    ]
}

/// Strategy for generating warehouse dialects.
fn dialect_strategy() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Postgres),
    ]
}

/// Strategy for generating a semantic attribute.
fn semantic_attribute_strategy() -> impl Strategy<Value = SemanticAttribute> {
    (attr_name_strategy(), field_path_strategy(), semantic_type_strategy(), any::<bool>())
        .prop_map(|(name, field, attr_type, is_dimension)| {
            let mut attr = SemanticAttribute::new(&name)
                .with_type(attr_type)
                .with_field_mapping(&field);
            if is_dimension {
                attr = attr.as_dimension();
            }
            attr
        })
}

/// Strategy for generating a semantic entity with attributes.
fn semantic_entity_strategy() -> impl Strategy<Value = SemanticEntity> {
    (
        entity_name_strategy(),
        prop::collection::vec(semantic_attribute_strategy(), 1..5),
        prop::collection::vec(1000u32..9999u32, 1..3),
    )
        .prop_map(|(name, attributes, event_classes)| {
            let mut entity = SemanticEntity::new(&name)
                .with_source_event_classes(event_classes);
            for attr in attributes {
                entity = entity.add_attribute(attr);
            }
            entity
        })
}

/// Strategy for generating a semantic metric.
fn semantic_metric_strategy(dimension_names: Vec<String>) -> impl Strategy<Value = SemanticMetric> {
    (
        metric_name_strategy(),
        aggregation_strategy(),
        field_path_strategy(),
        prop::collection::vec(time_granularity_strategy(), 0..4),
    )
        .prop_map(move |(name, aggregation, measure_field, granularities)| {
            SemanticMetric::new(&name)
                .with_aggregation(aggregation)
                .with_field_measure(&measure_field)
                .with_dimensions(dimension_names.clone())
                .with_time_granularities(granularities)
        })
}

/// Strategy for generating a complete semantic model.
fn semantic_model_strategy() -> impl Strategy<Value = SemanticModel> {
    semantic_entity_strategy().prop_flat_map(|entity| {
        let dimension_names: Vec<String> = entity
            .attributes
            .iter()
            .filter(|a| a.is_dimension)
            .map(|a| a.name.clone())
            .collect();
        
        let entity_clone = entity.clone();
        prop::collection::vec(semantic_metric_strategy(dimension_names), 1..3)
            .prop_map(move |metrics| {
                let mut model = SemanticModel::new("test_model")
                    .add_entity(entity_clone.clone());
                for metric in metrics {
                    model = model.add_metric(metric);
                }
                model
            })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 13: Query Translation Correctness**
    /// 
    /// *For any* valid semantic query, the translated SQL should:
    /// - Contain SELECT clauses for all requested attributes
    /// - Contain JOIN clauses for all referenced entities
    /// - Contain aggregations for all requested metrics
    /// 
    /// **Validates: Requirements 8.1, 8.2, 8.3**
    #[test]
    fn prop_query_translation_contains_select_for_attributes(
        model in semantic_model_strategy(),
        dialect in dialect_strategy(),
    ) {
        // Get the entity from the model
        let entity = model.entities.first().unwrap();
        
        // Select some attributes from the entity
        let attr_names: Vec<String> = entity.attributes.iter()
            .take(2)
            .map(|a| a.name.clone())
            .collect();
        
        if attr_names.is_empty() {
            return Ok(());
        }
        
        // Build a query selecting those attributes
        let mut query = SemanticQuery::new(&entity.name);
        for attr_name in &attr_names {
            query = query.add_select(attr_name.clone());
        }
        
        // Validate and translate
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        // Property: SQL should contain SELECT clause with all requested attributes
        prop_assert!(result.sql.contains("SELECT"), "SQL should contain SELECT");
        
        for attr_name in &attr_names {
            prop_assert!(
                result.sql.contains(&format!("AS {}", attr_name)),
                "SQL should contain alias for attribute: {}",
                attr_name
            );
        }
    }

    /// **Property 13 (continued): Query Translation Contains Aggregations for Metrics**
    /// 
    /// *For any* valid semantic query with metrics, the translated SQL should
    /// contain the appropriate aggregation functions.
    /// 
    /// **Validates: Requirements 8.1, 8.2, 8.3**
    #[test]
    fn prop_query_translation_contains_aggregations_for_metrics(
        model in semantic_model_strategy(),
        dialect in dialect_strategy(),
    ) {
        // Get the entity and metrics from the model
        let entity = model.entities.first().unwrap();
        
        if model.metrics.is_empty() {
            return Ok(());
        }
        
        let metric = model.metrics.first().unwrap();
        
        // Build a query with the metric
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&metric.name);
        
        // Validate and translate
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        // Property: SQL should contain the aggregation function
        let expected_agg = match metric.aggregation {
            Aggregation::Count => "COUNT(",
            Aggregation::Sum => "SUM(",
            Aggregation::Avg => "AVG(",
            Aggregation::Min => "MIN(",
            Aggregation::Max => "MAX(",
            Aggregation::CountDistinct => "COUNT(DISTINCT",
        };
        
        prop_assert!(
            result.sql.contains(expected_agg),
            "SQL should contain aggregation function {} for metric {}. SQL: {}",
            expected_agg,
            metric.name,
            result.sql
        );
        
        // Property: SQL should contain the metric alias
        prop_assert!(
            result.sql.contains(&format!("AS {}", metric.name)),
            "SQL should contain alias for metric: {}",
            metric.name
        );
    }

    /// **Property 13 (continued): Query Translation Contains FROM Clause**
    /// 
    /// *For any* valid semantic query, the translated SQL should contain
    /// a FROM clause referencing the appropriate table.
    /// 
    /// **Validates: Requirements 8.1, 8.2**
    #[test]
    fn prop_query_translation_contains_from_clause(
        model in semantic_model_strategy(),
        dialect in dialect_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        
        // Select at least one attribute
        let attr_name = entity.attributes.first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "id".to_string());
        
        let query = SemanticQuery::new(&entity.name)
            .add_select(&attr_name);
        
        let validator = QueryValidator::new(&model);
        
        // If validation fails due to missing attribute, that's fine
        if let Ok(validated) = validator.validate(&query) {
            let generator = SqlGenerator::new(dialect);
            let result = generator.generate(&validated);
            
            // Property: SQL should contain FROM clause
            prop_assert!(
                result.sql.contains("FROM"),
                "SQL should contain FROM clause"
            );
            
            // Property: FROM clause should reference the entity table
            prop_assert!(
                result.sql.contains(&format!("FROM ocsf_{}", entity.name)),
                "SQL should reference table for entity: {}",
                entity.name
            );
        }
    }

    /// **Property 13 (continued): Query Translation with Time Granularity**
    /// 
    /// *For any* valid semantic query with time granularity, the translated SQL
    /// should contain the appropriate date truncation function.
    /// 
    /// **Validates: Requirements 8.3**
    #[test]
    fn prop_query_translation_with_time_granularity(
        model in semantic_model_strategy(),
        dialect in dialect_strategy(),
        granularity in time_granularity_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        
        // Find a metric that supports this granularity
        let metric = model.metrics.iter()
            .find(|m| m.time_granularities.is_empty() || m.supports_granularity(granularity));
        
        if let Some(metric) = metric {
            let query = SemanticQuery::new(&entity.name)
                .add_metric(&metric.name)
                .with_time_granularity(granularity);
            
            let validator = QueryValidator::new(&model);
            
            if let Ok(validated) = validator.validate(&query) {
                let generator = SqlGenerator::new(dialect);
                let result = generator.generate(&validated);
                
                // Property: SQL should contain time bucket
                prop_assert!(
                    result.sql.contains("time_bucket"),
                    "SQL should contain time_bucket for time granularity"
                );
                
                // Property: SQL should contain GROUP BY
                prop_assert!(
                    result.sql.contains("GROUP BY"),
                    "SQL should contain GROUP BY for time-based aggregation"
                );
            }
        }
    }
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 14: Invalid Query Error Handling**
    /// 
    /// *For any* semantic query referencing undefined entities or metrics,
    /// the translator should return an error (not throw an exception) with
    /// a descriptive message.
    /// 
    /// **Validates: Requirements 8.4**
    #[test]
    fn prop_invalid_entity_returns_error(
        model in semantic_model_strategy(),
        invalid_entity_name in "[a-z][a-z0-9_]{5,15}_invalid".prop_map(|s| s.to_string()),
    ) {
        let validator = QueryValidator::new(&model);
        
        // Create a query with an invalid entity name
        let query = SemanticQuery::new(&invalid_entity_name)
            .add_select("some_attribute");
        
        // Property: Validation should return an error, not panic
        let result = validator.validate(&query);
        
        prop_assert!(
            result.is_err(),
            "Query with invalid entity should return error"
        );
        
        // Property: Error should be UnknownEntity variant
        if let Err(err) = result {
            let err_string = err.to_string();
            prop_assert!(
                err_string.contains("Unknown entity") || err_string.contains(&invalid_entity_name),
                "Error message should mention unknown entity or the invalid name. Got: {}",
                err_string
            );
        }
    }

    /// **Property 14 (continued): Invalid Metric Returns Error**
    /// 
    /// *For any* semantic query referencing an undefined metric,
    /// the translator should return an error with a descriptive message.
    /// 
    /// **Validates: Requirements 8.4**
    #[test]
    fn prop_invalid_metric_returns_error(
        model in semantic_model_strategy(),
        invalid_metric_name in "[a-z][a-z0-9_]{5,15}_invalid".prop_map(|s| s.to_string()),
    ) {
        let validator = QueryValidator::new(&model);
        let entity = model.entities.first().unwrap();
        
        // Create a query with an invalid metric name
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&invalid_metric_name);
        
        // Property: Validation should return an error, not panic
        let result = validator.validate(&query);
        
        prop_assert!(
            result.is_err(),
            "Query with invalid metric should return error"
        );
        
        // Property: Error should mention the invalid metric
        if let Err(err) = result {
            let err_string = err.to_string();
            prop_assert!(
                err_string.contains("Unknown metric") || err_string.contains(&invalid_metric_name),
                "Error message should mention unknown metric or the invalid name. Got: {}",
                err_string
            );
        }
    }

    /// **Property 14 (continued): Invalid Attribute Returns Error**
    /// 
    /// *For any* semantic query referencing an undefined attribute,
    /// the translator should return an error with a descriptive message.
    /// 
    /// **Validates: Requirements 8.4**
    #[test]
    fn prop_invalid_attribute_returns_error(
        model in semantic_model_strategy(),
        invalid_attr_name in "[a-z][a-z0-9_]{5,15}_invalid".prop_map(|s| s.to_string()),
    ) {
        let validator = QueryValidator::new(&model);
        let entity = model.entities.first().unwrap();
        
        // Create a query with an invalid attribute name
        let query = SemanticQuery::new(&entity.name)
            .add_select(&invalid_attr_name);
        
        // Property: Validation should return an error, not panic
        let result = validator.validate(&query);
        
        prop_assert!(
            result.is_err(),
            "Query with invalid attribute should return error"
        );
        
        // Property: Error should mention the invalid attribute
        if let Err(err) = result {
            let err_string = err.to_string();
            prop_assert!(
                err_string.contains("Unknown attribute") || err_string.contains(&invalid_attr_name),
                "Error message should mention unknown attribute or the invalid name. Got: {}",
                err_string
            );
        }
    }

    /// **Property 14 (continued): Empty Query Returns Error**
    /// 
    /// *For any* semantic query with no selections (no attributes or metrics),
    /// the translator should return an error.
    /// 
    /// **Validates: Requirements 8.4**
    #[test]
    fn prop_empty_query_returns_error(
        model in semantic_model_strategy(),
    ) {
        let validator = QueryValidator::new(&model);
        let entity = model.entities.first().unwrap();
        
        // Create an empty query (no selections)
        let query = SemanticQuery::new(&entity.name);
        
        // Property: Validation should return an error for empty query
        let result = validator.validate(&query);
        
        prop_assert!(
            result.is_err(),
            "Empty query should return error"
        );
        
        // Property: Error should indicate the query is empty
        if let Err(err) = result {
            let err_string = err.to_string();
            prop_assert!(
                err_string.contains("empty") || err_string.contains("select"),
                "Error message should indicate empty query. Got: {}",
                err_string
            );
        }
    }

    /// **Property 14 (continued): Error Messages Are Descriptive**
    /// 
    /// *For any* invalid query, the error message should provide helpful
    /// information about what went wrong and available options.
    /// 
    /// **Validates: Requirements 8.4**
    #[test]
    fn prop_error_messages_are_descriptive(
        model in semantic_model_strategy(),
    ) {
        let validator = QueryValidator::new(&model);
        
        // Test with invalid entity - error should list available entities
        let query = SemanticQuery::new("nonexistent_entity_xyz")
            .add_select("some_attr");
        
        if let Err(err) = validator.validate(&query) {
            let err_string = err.to_string();
            
            // Property: Error should list available entities
            let entity_name = &model.entities.first().unwrap().name;
            prop_assert!(
                err_string.contains(entity_name) || err_string.contains("Available"),
                "Error should list available entities. Got: {}",
                err_string
            );
        }
        
        // Test with invalid metric - error should list available metrics
        let entity = model.entities.first().unwrap();
        let query = SemanticQuery::new(&entity.name)
            .add_metric("nonexistent_metric_xyz");
        
        if let Err(err) = validator.validate(&query) {
            let err_string = err.to_string();
            
            // Property: Error should list available metrics
            if !model.metrics.is_empty() {
                let metric_name = &model.metrics.first().unwrap().name;
                prop_assert!(
                    err_string.contains(metric_name) || err_string.contains("Available"),
                    "Error should list available metrics. Got: {}",
                    err_string
                );
            }
        }
    }
}


/// Strategy for generating a hot path metric.
fn hot_path_metric_strategy() -> impl Strategy<Value = SemanticMetric> {
    (
        metric_name_strategy(),
        aggregation_strategy(),
        1u32..100u32, // observable type_id
    )
        .prop_map(|(name, aggregation, type_id)| {
            SemanticMetric::new(&name)
                .with_aggregation(aggregation)
                .with_field_measure("observable_value")
                .with_observable_type_id(type_id)
                .as_hot_path()
        })
}

/// Strategy for generating a model with hot path metrics.
fn model_with_hot_path_strategy() -> impl Strategy<Value = SemanticModel> {
    (
        semantic_entity_strategy(),
        prop::collection::vec(hot_path_metric_strategy(), 1..3),
    )
        .prop_map(|(entity, hot_metrics)| {
            let mut model = SemanticModel::new("test_model")
                .add_entity(entity);
            for metric in hot_metrics {
                model = model.add_metric(metric);
            }
            model
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 20: Hot Path Query Routing**
    /// 
    /// *For any* semantic query with pathPreference='hot' against a hot-path-enabled
    /// metric, the translated SQL should query the observables table, not the full
    /// events table.
    /// 
    /// **Validates: Requirements 11.4, 11.6**
    #[test]
    fn prop_hot_path_queries_observables_table(
        model in model_with_hot_path_strategy(),
        dialect in dialect_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        let hot_metric = model.metrics.iter()
            .find(|m| m.is_hot_path)
            .unwrap();
        
        // Create a query with hot path preference
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&hot_metric.name)
            .with_path_preference(crate::query::PathPreference::Hot);
        
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        // Property: Validated query should use hot path
        prop_assert!(
            validated.uses_hot_path,
            "Query with hot path preference should use hot path"
        );
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        // Property: SQL should query observables table
        prop_assert!(
            result.uses_hot_path,
            "Generated SQL should indicate hot path usage"
        );
        
        prop_assert!(
            result.sql.contains("ocsf_observables"),
            "SQL should query observables table. SQL: {}",
            result.sql
        );
        
        // Property: SQL should NOT query the entity table
        let entity_table = format!("ocsf_{}", entity.name);
        prop_assert!(
            !result.sql.contains(&entity_table),
            "Hot path SQL should not query entity table {}. SQL: {}",
            entity_table,
            result.sql
        );
    }

    /// **Property 20 (continued): Hot Path Filters by Type ID**
    /// 
    /// *For any* hot path query, the SQL should filter by the observable type_id.
    /// 
    /// **Validates: Requirements 11.4, 11.6**
    #[test]
    fn prop_hot_path_filters_by_type_id(
        model in model_with_hot_path_strategy(),
        dialect in dialect_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        let hot_metric = model.metrics.iter()
            .find(|m| m.is_hot_path && m.observable_type_id.is_some())
            .unwrap();
        
        let type_id = hot_metric.observable_type_id.unwrap();
        
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&hot_metric.name)
            .with_path_preference(crate::query::PathPreference::Hot);
        
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        // Property: SQL should filter by type_id
        prop_assert!(
            result.sql.contains("type_id"),
            "Hot path SQL should filter by type_id. SQL: {}",
            result.sql
        );
        
        prop_assert!(
            result.sql.contains(&type_id.to_string()),
            "Hot path SQL should contain the type_id value {}. SQL: {}",
            type_id,
            result.sql
        );
    }

    /// **Property 20 (continued): Auto Path Preference Detects Hot Path**
    /// 
    /// *For any* query with auto path preference against a hot-path-enabled metric,
    /// the system should automatically route to the hot path.
    /// 
    /// **Validates: Requirements 11.4, 11.6**
    #[test]
    fn prop_auto_path_detects_hot_path(
        model in model_with_hot_path_strategy(),
        dialect in dialect_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        let hot_metric = model.metrics.iter()
            .find(|m| m.is_hot_path)
            .unwrap();
        
        // Use auto path preference
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&hot_metric.name)
            .with_path_preference(crate::query::PathPreference::Auto);
        
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        // Property: Auto should detect hot path metric
        prop_assert!(
            validated.uses_hot_path,
            "Auto path preference should detect hot path metric"
        );
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        prop_assert!(
            result.uses_hot_path,
            "Auto-detected hot path should generate hot path SQL"
        );
    }

    /// **Property 20 (continued): Cold Path Preference Overrides Hot Path**
    /// 
    /// *For any* query with cold path preference, even against a hot-path-enabled
    /// metric, the system should route to the cold path (full events table).
    /// 
    /// **Validates: Requirements 11.4, 11.6**
    #[test]
    fn prop_cold_path_overrides_hot_path(
        model in model_with_hot_path_strategy(),
        dialect in dialect_strategy(),
    ) {
        let entity = model.entities.first().unwrap();
        let hot_metric = model.metrics.iter()
            .find(|m| m.is_hot_path)
            .unwrap();
        
        // Force cold path
        let query = SemanticQuery::new(&entity.name)
            .add_metric(&hot_metric.name)
            .with_path_preference(crate::query::PathPreference::Cold);
        
        let validator = QueryValidator::new(&model);
        let validated = validator.validate(&query)?;
        
        // Property: Cold preference should override hot path
        prop_assert!(
            !validated.uses_hot_path,
            "Cold path preference should override hot path"
        );
        
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate(&validated);
        
        // Property: SQL should query entity table, not observables
        prop_assert!(
            !result.uses_hot_path,
            "Cold path should not use hot path"
        );
        
        let entity_table = format!("ocsf_{}", entity.name);
        prop_assert!(
            result.sql.contains(&entity_table),
            "Cold path SQL should query entity table. SQL: {}",
            result.sql
        );
    }
}


use crate::query::ObservableMatch;

/// Strategy for generating observable matches.
fn observable_match_strategy() -> impl Strategy<Value = ObservableMatch> {
    (
        1u32..100u32,                                    // type_id
        "[a-zA-Z0-9.@_-]{5,50}",                        // value
        "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}", // event_uid (UUID-like)
        1000u32..9999u32,                               // event_class_uid
        "2024-0[1-9]-[0-2][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9]Z", // event_time
    )
        .prop_map(|(type_id, value, event_uid, event_class_uid, event_time)| {
            ObservableMatch::new(type_id, value, event_uid, event_class_uid, event_time)
        })
}

/// Strategy for generating a list of observable matches.
fn observable_matches_strategy() -> impl Strategy<Value = Vec<ObservableMatch>> {
    prop::collection::vec(observable_match_strategy(), 1..10)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 19: Reverse Lookup Query Generation**
    /// 
    /// *For any* set of observable matches (with event_uid values), the generated
    /// reverse-lookup query should retrieve all matching full events.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_contains_all_event_uids(
        matches in observable_matches_strategy(),
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: SQL should contain all event UIDs
        for m in &matches {
            prop_assert!(
                result.sql.contains(&m.event_uid) || result.sql.contains(&m.event_uid.replace('\'', "''")),
                "SQL should contain event_uid: {}. SQL: {}",
                m.event_uid,
                result.sql
            );
        }
    }

    /// **Property 19 (continued): Reverse Lookup Uses IN Clause for Multiple Matches**
    /// 
    /// *For any* set of multiple observable matches, the query should use an IN clause
    /// for efficient batch lookup.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_uses_in_clause_for_multiple(
        matches in prop::collection::vec(observable_match_strategy(), 2..10),
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: SQL should use IN clause for multiple matches
        prop_assert!(
            result.sql.contains("IN ("),
            "SQL should use IN clause for multiple matches. SQL: {}",
            result.sql
        );
    }

    /// **Property 19 (continued): Reverse Lookup Uses Equality for Single Match**
    /// 
    /// *For any* single observable match, the query should use equality comparison
    /// for optimal performance.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_uses_equality_for_single(
        single_match in observable_match_strategy(),
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let matches = vec![single_match.clone()];
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: SQL should use equality for single match
        prop_assert!(
            result.sql.contains("metadata.uid ="),
            "SQL should use equality for single match. SQL: {}",
            result.sql
        );
        
        // Property: SQL should NOT use IN clause for single match
        prop_assert!(
            !result.sql.contains("IN ("),
            "SQL should not use IN clause for single match. SQL: {}",
            result.sql
        );
    }

    /// **Property 19 (continued): Reverse Lookup Queries Target Table**
    /// 
    /// *For any* reverse lookup query, the SQL should query the specified target table.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_queries_target_table(
        matches in observable_matches_strategy(),
        dialect in dialect_strategy(),
        table_name in "[a-z][a-z0-9_]{5,20}",
    ) {
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate_reverse_lookup(&matches, &table_name);
        
        // Property: SQL should query the target table
        prop_assert!(
            result.sql.contains(&format!("FROM {}", table_name)),
            "SQL should query target table {}. SQL: {}",
            table_name,
            result.sql
        );
    }

    /// **Property 19 (continued): Reverse Lookup Is Not Hot Path**
    /// 
    /// *For any* reverse lookup query, it should be marked as cold path
    /// (not hot path) since it queries full events.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_is_cold_path(
        matches in observable_matches_strategy(),
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: Reverse lookup should not be hot path
        prop_assert!(
            !result.uses_hot_path,
            "Reverse lookup should be cold path (not hot path)"
        );
    }

    /// **Property 19 (continued): Reverse Lookup Orders by Event Time**
    /// 
    /// *For any* reverse lookup query, results should be ordered by event time
    /// for consistent results.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_orders_by_time(
        matches in observable_matches_strategy(),
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: SQL should order by event_time
        prop_assert!(
            result.sql.contains("ORDER BY event_time"),
            "SQL should order by event_time. SQL: {}",
            result.sql
        );
    }

    /// **Property 19 (continued): Empty Matches Returns No Results**
    /// 
    /// *For any* empty set of matches, the query should return no results.
    /// 
    /// **Validates: Requirements 11.5**
    #[test]
    fn prop_reverse_lookup_empty_returns_no_results(
        dialect in dialect_strategy(),
    ) {
        let generator = SqlGenerator::new(dialect);
        let matches: Vec<ObservableMatch> = vec![];
        let result = generator.generate_reverse_lookup(&matches, "ocsf_events");
        
        // Property: Empty matches should return query that yields no results
        prop_assert!(
            result.sql.contains("1=0") || result.sql.contains("WHERE FALSE"),
            "Empty matches should generate query returning no results. SQL: {}",
            result.sql
        );
    }
}
