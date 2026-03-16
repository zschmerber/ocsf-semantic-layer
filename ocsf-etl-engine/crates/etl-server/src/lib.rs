//! ETL Server - Axum HTTP server for ETL job lifecycle management.
//!
//! This module provides the `generate_all` orchestration function that ties
//! together codegen, semantic model generation, and warehouse artifact
//! generation into a single atomic operation with organized file output.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use codegen_adapter::{CodegenAdapter, CompiledSchema};
use etl_core::EtlJob;
use warehouse_gen::{WarehouseDialect, WarehouseGen};

// ── 10.2 GenerationResult ────────────────────────────────────────────

/// Result of a full generation run containing paths to all produced artifacts.
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Path to the plugin output directory containing Go files.
    pub plugin_dir: PathBuf,
    /// Path to the warehouse output directory containing DDL, views, and dbt.
    pub warehouse_dir: PathBuf,
    /// Path to the standalone semantic-model.yaml file.
    pub semantic_model_path: PathBuf,
    /// The semantic model YAML content.
    pub semantic_model_yaml: String,
}

// ── 10.1 generate_all ────────────────────────────────────────────────

/// Orchestrate full artifact generation: codegen plugin, semantic model, and
/// warehouse artifacts in a single call.
///
/// # Arguments
/// * `job` - The ETL job specification with field mappings
/// * `schema` - The compiled OCSF schema
/// * `dialect` - Target warehouse SQL dialect
/// * `output_dir` - Root output directory for all artifacts
///
/// # Errors
/// Returns an error if:
/// - The job has zero mappings
/// - Codegen generation fails
/// - Warehouse generation fails
/// - File I/O fails
pub fn generate_all(
    job: &EtlJob,
    schema: &CompiledSchema,
    dialect: WarehouseDialect,
    output_dir: &Path,
) -> Result<GenerationResult> {
    // Validate non-empty mappings
    if job.mappings.is_empty() {
        bail!("EtlJob must have at least one mapping to generate artifacts");
    }

    // 1. Run codegen adapter
    let adapter = CodegenAdapter::new();
    let adapter_output = adapter
        .generate(schema, job)
        .context("codegen adapter failed")?;

    // 2. Convert types for warehouse-gen via JSON round-trip
    let wh_schema: warehouse_gen::CompiledSchema = serde_json::from_value(
        serde_json::to_value(schema).context("failed to serialize CompiledSchema")?,
    )
    .context("failed to convert CompiledSchema for warehouse-gen")?;

    let wh_semantic_model: warehouse_gen::SemanticModel = serde_yaml::from_str(
        &adapter_output.semantic_model_yaml,
    )
    .context("failed to parse semantic model YAML for warehouse-gen")?;

    // 3. Run warehouse generation
    let wh_gen = WarehouseGen::new(dialect);
    let wh_job_schema = &wh_schema;
    let wh_artifacts = wh_gen
        .generate(wh_job_schema, job, &wh_semantic_model)
        .context("warehouse generation failed")?;

    // 4. Write all files to output directory
    let result = write_output(
        job,
        &adapter_output,
        &wh_artifacts,
        output_dir,
    )?;

    Ok(result)
}

// ── 10.3 File writing ────────────────────────────────────────────────

/// Write all generated artifacts to the output directory organized by type.
///
/// Layout:
/// ```text
/// {output_dir}/{job_id}/
/// ├── plugin/
/// │   ├── main.go
/// │   ├── go.mod
/// │   ├── tangent.yaml
/// │   └── test_fixture.json
/// ├── warehouse/
/// │   ├── table.sql
/// │   ├── views.sql
/// │   ├── materialized_views.sql
/// │   └── dbt/
/// │       ├── semantic_manifest.yaml
/// │       ├── sources.yaml
/// │       └── models/
/// │           └── *.sql
/// └── semantic-model.yaml
/// ```
fn write_output(
    job: &EtlJob,
    adapter_output: &codegen_adapter::AdapterOutput,
    wh_artifacts: &warehouse_gen::WarehouseArtifacts,
    output_dir: &Path,
) -> Result<GenerationResult> {
    let job_dir = output_dir.join(job.job_id.to_string());

    // ── Plugin directory ─────────────────────────────────────────
    let plugin_dir = job_dir.join("plugin");
    std::fs::create_dir_all(&plugin_dir)
        .with_context(|| format!("failed to create plugin dir: {}", plugin_dir.display()))?;

    std::fs::write(plugin_dir.join("main.go"), &adapter_output.go_code)
        .context("failed to write main.go")?;
    std::fs::write(plugin_dir.join("go.mod"), &adapter_output.go_mod)
        .context("failed to write go.mod")?;
    std::fs::write(plugin_dir.join("tangent.yaml"), &adapter_output.tangent_yaml)
        .context("failed to write tangent.yaml")?;
    std::fs::write(plugin_dir.join("test_fixture.json"), &adapter_output.test_fixture)
        .context("failed to write test_fixture.json")?;

    // ── Warehouse directory ──────────────────────────────────────
    let warehouse_dir = job_dir.join("warehouse");
    std::fs::create_dir_all(&warehouse_dir)
        .with_context(|| format!("failed to create warehouse dir: {}", warehouse_dir.display()))?;

    std::fs::write(warehouse_dir.join("table.sql"), &wh_artifacts.table_ddl)
        .context("failed to write table.sql")?;
    std::fs::write(warehouse_dir.join("views.sql"), &wh_artifacts.semantic_views)
        .context("failed to write views.sql")?;
    std::fs::write(
        warehouse_dir.join("materialized_views.sql"),
        &wh_artifacts.materialized_views,
    )
    .context("failed to write materialized_views.sql")?;

    // dbt subdirectory
    let dbt_dir = warehouse_dir.join("dbt");
    std::fs::create_dir_all(&dbt_dir)
        .with_context(|| format!("failed to create dbt dir: {}", dbt_dir.display()))?;

    std::fs::write(
        dbt_dir.join("semantic_manifest.yaml"),
        &wh_artifacts.dbt_artifacts.semantic_manifest,
    )
    .context("failed to write semantic_manifest.yaml")?;
    std::fs::write(dbt_dir.join("sources.yaml"), &wh_artifacts.dbt_artifacts.sources)
        .context("failed to write sources.yaml")?;

    // dbt models
    let models_dir = dbt_dir.join("models");
    std::fs::create_dir_all(&models_dir)
        .with_context(|| format!("failed to create models dir: {}", models_dir.display()))?;

    for (model_name, sql) in &wh_artifacts.dbt_artifacts.models {
        let filename = format!("{}.sql", model_name);
        std::fs::write(models_dir.join(&filename), sql)
            .with_context(|| format!("failed to write dbt model: {}", filename))?;
    }

    // ── Semantic model ───────────────────────────────────────────
    let semantic_model_path = job_dir.join("semantic-model.yaml");
    std::fs::write(&semantic_model_path, &adapter_output.semantic_model_yaml)
        .context("failed to write semantic-model.yaml")?;

    Ok(GenerationResult {
        plugin_dir,
        warehouse_dir,
        semantic_model_path,
        semantic_model_yaml: adapter_output.semantic_model_yaml.clone(),
    })
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use codegen_adapter::{CompiledAttribute, CompiledClass};
    use etl_core::{MappingSpec, SourceConfig};
    use std::collections::BTreeMap;
    use uuid::Uuid;

    fn test_schema() -> CompiledSchema {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "query.hostname".to_string(),
            CompiledAttribute {
                name: "query.hostname".to_string(),
                caption: "Query Hostname".to_string(),
                type_name: "String".to_string(),
                is_required: true,
            },
        );
        attributes.insert(
            "src_endpoint.ip".to_string(),
            CompiledAttribute {
                name: "src_endpoint.ip".to_string(),
                caption: "Source IP".to_string(),
                type_name: "String".to_string(),
                is_required: false,
            },
        );

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

    fn test_job() -> EtlJob {
        EtlJob {
            job_id: Uuid::nil(),
            plugin_name: "dns_activity_plugin".to_string(),
            ocsf_class_uid: 4003,
            ocsf_version: "1.3.0".to_string(),
            mappings: vec![
                MappingSpec::new(
                    "hostname".to_string(),
                    "query.hostname".to_string(),
                    None,
                    1.0,
                ),
                MappingSpec::new(
                    "src_ip".to_string(),
                    "src_endpoint.ip".to_string(),
                    None,
                    0.9,
                ),
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
    fn generate_all_returns_error_for_empty_mappings() {
        let schema = test_schema();
        let mut job = test_job();
        job.mappings.clear();

        let tmp = tempfile::tempdir().unwrap();
        let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least one mapping")
        );
    }

    #[test]
    fn generation_result_has_correct_paths() {
        let schema = test_schema();
        let job = test_job();

        let tmp = tempfile::tempdir().unwrap();
        let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path()).unwrap();

        let job_dir = tmp.path().join(job.job_id.to_string());
        assert_eq!(result.plugin_dir, job_dir.join("plugin"));
        assert_eq!(result.warehouse_dir, job_dir.join("warehouse"));
        assert_eq!(result.semantic_model_path, job_dir.join("semantic-model.yaml"));
        assert!(!result.semantic_model_yaml.is_empty());
    }

    #[test]
    fn generate_all_writes_plugin_files() {
        let schema = test_schema();
        let job = test_job();

        let tmp = tempfile::tempdir().unwrap();
        let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path()).unwrap();

        assert!(result.plugin_dir.join("main.go").exists());
        assert!(result.plugin_dir.join("go.mod").exists());
        assert!(result.plugin_dir.join("tangent.yaml").exists());
        assert!(result.plugin_dir.join("test_fixture.json").exists());
    }

    #[test]
    fn generate_all_writes_warehouse_files() {
        let schema = test_schema();
        let job = test_job();

        let tmp = tempfile::tempdir().unwrap();
        let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path()).unwrap();

        assert!(result.warehouse_dir.join("table.sql").exists());
        assert!(result.warehouse_dir.join("views.sql").exists());
        assert!(result.warehouse_dir.join("materialized_views.sql").exists());
        assert!(result.warehouse_dir.join("dbt/semantic_manifest.yaml").exists());
        assert!(result.warehouse_dir.join("dbt/sources.yaml").exists());
        assert!(result.warehouse_dir.join("dbt/models").is_dir());
    }

    #[test]
    fn generate_all_writes_semantic_model() {
        let schema = test_schema();
        let job = test_job();

        let tmp = tempfile::tempdir().unwrap();
        let result = generate_all(&job, &schema, WarehouseDialect::Snowflake, tmp.path()).unwrap();

        assert!(result.semantic_model_path.exists());
        let content = std::fs::read_to_string(&result.semantic_model_path).unwrap();
        assert!(!content.is_empty());
        // Verify it's valid YAML
        let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    }
}

pub mod server;
