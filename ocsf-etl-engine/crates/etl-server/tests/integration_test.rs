//! Integration test for the full `generate_all` pipeline.
//!
//! Validates that a DNS Activity ETL job (uid=4003) produces correct
//! codegen, semantic model, and warehouse artifacts in a single call.

use std::collections::BTreeMap;

use codegen_adapter::{CompiledAttribute, CompiledClass, CompiledSchema};
use etl_core::{EtlJob, MappingSpec, SourceConfig};
use etl_server::generate_all;
use warehouse_gen::WarehouseDialect;

/// Build a test `CompiledSchema` with DNS Activity class (uid=4003).
fn dns_schema() -> CompiledSchema {
    let mut attributes = BTreeMap::new();
    for (name, caption, required) in [
        ("query.hostname", "Query Hostname", true),
        ("src_endpoint.ip", "Source IP", false),
        ("dst_endpoint.ip", "Destination IP", false),
    ] {
        attributes.insert(
            name.to_string(),
            CompiledAttribute {
                name: name.to_string(),
                caption: caption.to_string(),
                type_name: "String".to_string(),
                is_required: required,
            },
        );
    }

    let mut classes = BTreeMap::new();
    classes.insert(
        4003,
        CompiledClass {
            uid: 4003,
            name: "DNS Activity".to_string(),
            caption: "DNS Activity".to_string(),
            category_uid: 4,
            attributes,
        },
    );

    CompiledSchema {
        version: "1.3.0".to_string(),
        classes,
    }
}

/// Build a test `EtlJob` with 3 DNS field mappings at confidence=1.0.
fn dns_job() -> EtlJob {
    EtlJob {
        job_id: uuid::Uuid::nil(),
        plugin_name: "dns_activity_plugin".to_string(),
        ocsf_class_uid: 4003,
        ocsf_version: "1.3.0".to_string(),
        mappings: vec![
            MappingSpec::new("dns_name".to_string(), "query.hostname".to_string(), None, 1.0),
            MappingSpec::new("src_ip".to_string(), "src_endpoint.ip".to_string(), None, 1.0),
            MappingSpec::new("dst_ip".to_string(), "dst_endpoint.ip".to_string(), None, 1.0),
        ],
        source_config: SourceConfig::File {
            path: "/var/log/dns.log".to_string(),
        },
        delta_table_uri: "s3://bucket/dns_activity".to_string(),
        s3_config: None,
        catalog_ref: None,
    }
}

#[test]
fn generate_all_dns_activity_integration() {
    let schema = dns_schema();
    let job = dns_job();
    let tmp = tempfile::tempdir().unwrap();

    let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path())
        .expect("generate_all should succeed for valid DNS Activity job");

    // ── 1. Codegen output contains DnsActivity struct and ClassUID: 4003 ──
    let go_code = std::fs::read_to_string(result.plugin_dir.join("main.go"))
        .expect("main.go should exist");
    assert!(
        go_code.contains("DnsActivity"),
        "Go code should contain DnsActivity struct name"
    );
    assert!(
        go_code.contains("4003"),
        "Go code should reference ClassUID 4003"
    );

    // ── 2. tangent.yaml contains ocsf_semantic_model block ───────────
    let tangent_yaml = std::fs::read_to_string(result.plugin_dir.join("tangent.yaml"))
        .expect("tangent.yaml should exist");
    assert!(
        tangent_yaml.contains("ocsf_semantic_model"),
        "tangent.yaml should contain ocsf_semantic_model block"
    );

    // ── 3. semantic-model.yaml is valid YAML with at least one entity ─
    let sm_content = std::fs::read_to_string(&result.semantic_model_path)
        .expect("semantic-model.yaml should exist");
    let sm_value: serde_yaml::Value =
        serde_yaml::from_str(&sm_content).expect("semantic-model.yaml should be valid YAML");
    let entities = sm_value
        .get("entities")
        .expect("semantic model should have 'entities' key");
    let entities_seq = entities.as_sequence().expect("entities should be a sequence");
    assert!(
        !entities_seq.is_empty(),
        "semantic model should have at least one entity"
    );

    // ── 4. Warehouse DDL contains CREATE TABLE ───────────────────────
    let table_ddl = std::fs::read_to_string(result.warehouse_dir.join("table.sql"))
        .expect("table.sql should exist");
    assert!(
        table_ddl.contains("CREATE TABLE"),
        "warehouse DDL should contain CREATE TABLE"
    );

    // ── 5. Warehouse views contain CREATE VIEW or CREATE OR REPLACE VIEW ─
    let views_sql = std::fs::read_to_string(result.warehouse_dir.join("views.sql"))
        .expect("views.sql should exist");
    assert!(
        views_sql.contains("CREATE VIEW") || views_sql.contains("CREATE OR REPLACE VIEW"),
        "warehouse views should contain CREATE VIEW or CREATE OR REPLACE VIEW"
    );

    // ── 6. dbt artifacts contain semantic_models YAML key ────────────
    let dbt_manifest =
        std::fs::read_to_string(result.warehouse_dir.join("dbt/semantic_manifest.yaml"))
            .expect("semantic_manifest.yaml should exist");
    let dbt_value: serde_yaml::Value =
        serde_yaml::from_str(&dbt_manifest).expect("dbt manifest should be valid YAML");
    assert!(
        dbt_value.get("semantic_models").is_some(),
        "dbt artifacts should contain 'semantic_models' YAML key"
    );
}
