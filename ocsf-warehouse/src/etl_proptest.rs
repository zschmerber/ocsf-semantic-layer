//! Property-based tests for ETL pipeline generation.
//!
//! **Feature: ocsf-semantic-layer, Property 18: Observable Extraction from Events**
//! **Validates: Requirements 11.3**

use proptest::prelude::*;
use ocsf_semantic::WarehouseDialect;

use crate::etl::{ETLConfig, ETLGenerator};

/// Strategy for generating warehouse dialects.
fn dialect_strategy() -> impl Strategy<Value = WarehouseDialect> {
    prop_oneof![
        Just(WarehouseDialect::Snowflake),
        Just(WarehouseDialect::BigQuery),
        Just(WarehouseDialect::Databricks),
        Just(WarehouseDialect::Postgres),
    ]
}

/// Strategy for generating event UIDs.
fn event_uid_strategy() -> impl Strategy<Value = String> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}".prop_map(|s| s)
}

/// Strategy for generating event class UIDs.
fn event_class_uid_strategy() -> impl Strategy<Value = u32> {
    1000u32..9999u32
}

/// Strategy for generating timestamps.
fn timestamp_strategy() -> impl Strategy<Value = String> {
    (2020u32..2025u32, 1u32..13u32, 1u32..29u32, 0u32..24u32, 0u32..60u32)
        .prop_map(|(year, month, day, hour, minute)| {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:00Z",
                year, month, day, hour, minute
            )
        })
}

/// Strategy for generating observable type IDs.
fn observable_type_id_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(2u32),  // IP Address
        Just(5u32),  // Email
        Just(10u32), // Username
        Just(22u32), // Hostname
        Just(30u32), // Hash
    ]
}

/// Strategy for generating observable values.
fn observable_value_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-z0-9]{5,20}".prop_map(Some),
        "\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}".prop_map(Some), // IP-like
        "[a-z]+@[a-z]+\\.[a-z]{2,3}".prop_map(Some), // Email-like
    ]
}

/// Strategy for generating attribute paths.
fn attribute_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("src_endpoint.ip".to_string()),
        Just("dst_endpoint.ip".to_string()),
        Just("actor.user.email_addr".to_string()),
        Just("actor.user.name".to_string()),
        Just("file.hashes.sha256".to_string()),
    ]
}

/// Strategy for generating observable data tuples.
fn observable_data_strategy() -> impl Strategy<Value = (u32, String, String, Option<String>)> {
    (
        observable_type_id_strategy(),
        attribute_path_strategy(),
        observable_value_strategy(),
    )
        .prop_map(|(type_id, path, value)| {
            let type_name = match type_id {
                2 => "IP Address",
                5 => "Email Address",
                10 => "User Name",
                22 => "Hostname",
                30 => "File Hash",
                _ => "Unknown",
            };
            (type_id, type_name.to_string(), path, value)
        })
}

proptest! {
    /// **Property 18: Observable Extraction from Events**
    ///
    /// *For any* OCSF event with observables array, the ETL extraction should
    /// produce one observables table row per observable, with correct reverse-lookup
    /// references to the source event.
    ///
    /// **Validates: Requirements 11.3**
    #[test]
    fn prop_observable_extraction_preserves_reverse_lookup(
        event_uid in event_uid_strategy(),
        event_class_uid in event_class_uid_strategy(),
        event_time in timestamp_strategy(),
        observables_data in prop::collection::vec(observable_data_strategy(), 1..5)
    ) {
        let generator = ETLGenerator::new(WarehouseDialect::Snowflake);
        
        // Convert to the format expected by extract_from_event
        let data: Vec<(u32, &str, &str, Option<&str>)> = observables_data
            .iter()
            .map(|(type_id, type_name, path, value)| {
                (*type_id, type_name.as_str(), path.as_str(), value.as_deref())
            })
            .collect();

        let extracted = generator.extract_from_event(
            &event_uid,
            event_class_uid,
            &event_time,
            &data,
        );

        // Verify one row per observable
        prop_assert_eq!(
            extracted.len(),
            observables_data.len(),
            "Should produce one row per observable"
        );

        // Verify each extracted observable has correct reverse-lookup references
        for observable in &extracted {
            prop_assert!(
                observable.has_valid_reverse_lookup(),
                "Observable should have valid reverse-lookup reference"
            );
            prop_assert_eq!(
                &observable.event_uid,
                &event_uid,
                "event_uid should match source event"
            );
            prop_assert_eq!(
                observable.event_class_uid,
                event_class_uid,
                "event_class_uid should match source event"
            );
            prop_assert_eq!(
                &observable.event_time,
                &event_time,
                "event_time should match source event"
            );
        }
    }

    /// Property: Each extracted observable should have a unique observable_id when paths are unique.
    #[test]
    fn prop_extracted_observables_have_unique_ids(
        event_uid in event_uid_strategy(),
        event_class_uid in event_class_uid_strategy(),
        event_time in timestamp_strategy(),
        observables_data in prop::collection::vec(observable_data_strategy(), 1..5)
    ) {
        let generator = ETLGenerator::new(WarehouseDialect::Snowflake);
        
        let data: Vec<(u32, &str, &str, Option<&str>)> = observables_data
            .iter()
            .map(|(type_id, type_name, path, value)| {
                (*type_id, type_name.as_str(), path.as_str(), value.as_deref())
            })
            .collect();

        let extracted = generator.extract_from_event(
            &event_uid,
            event_class_uid,
            &event_time,
            &data,
        );

        // Count unique paths in input
        let unique_paths: std::collections::HashSet<&str> = data
            .iter()
            .map(|(_, _, path, _)| *path)
            .collect();

        // Collect all observable_ids
        let ids: std::collections::HashSet<&str> = extracted
            .iter()
            .map(|o| o.observable_id.as_str())
            .collect();

        // Number of unique IDs should equal number of unique paths
        // (duplicate paths will produce duplicate IDs, which is expected)
        prop_assert_eq!(
            ids.len(),
            unique_paths.len(),
            "Number of unique observable_ids should equal number of unique paths"
        );
    }

    /// Property: ETL pipeline SQL should contain INSERT INTO target table.
    #[test]
    fn prop_etl_sql_contains_insert(dialect in dialect_strategy()) {
        let generator = ETLGenerator::new(dialect);
        let etl = generator.generate();

        let sql = etl.to_sql();
        prop_assert!(
            sql.contains("INSERT INTO"),
            "ETL SQL should contain INSERT INTO"
        );
    }

    /// Property: ETL pipeline SQL should reference all required columns.
    #[test]
    fn prop_etl_sql_has_required_columns(dialect in dialect_strategy()) {
        let generator = ETLGenerator::new(dialect);
        let etl = generator.generate();

        let sql = etl.to_sql();
        let required_columns = [
            "observable_id",
            "type_id",
            "type_name",
            "value",
            "event_uid",
            "event_class_uid",
            "event_time",
            "attribute_path",
        ];

        for col in required_columns {
            prop_assert!(
                sql.contains(col),
                "ETL SQL should reference column: {}",
                col
            );
        }
    }

    /// Property: ETL pipeline should handle configured observable types.
    #[test]
    fn prop_etl_handles_configured_types(
        dialect in dialect_strategy(),
        types in prop::collection::vec(observable_type_id_strategy(), 1..4)
    ) {
        let config = ETLConfig {
            observable_types: types.clone(),
            ..Default::default()
        };
        let generator = ETLGenerator::new(dialect).with_config(config);
        let etl = generator.generate();

        let sql = etl.to_sql();
        
        // Each configured type should appear in the SQL
        for type_id in &types {
            prop_assert!(
                sql.contains(&type_id.to_string()),
                "ETL SQL should handle type_id: {}",
                type_id
            );
        }
    }
}
