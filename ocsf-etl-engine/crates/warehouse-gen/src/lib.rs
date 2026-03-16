//! Warehouse Gen - generates physical DDL, semantic views, dbt artifacts, and materialized views.

use std::collections::HashMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use etl_core::EtlJob;

// ── Stand-in types ───────────────────────────────────────────────────
// These mirror types from `ocsf-semantic` and `ocsf-core`. When the
// path dependencies are uncommented they can be replaced by re-exports.

/// Mirrors `ocsf_semantic::SemanticField`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticField {
    pub name: String,
    pub source_field: String,
    pub ocsf_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
}

/// Mirrors `ocsf_semantic::SemanticEntity`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticEntity {
    pub name: String,
    pub description: String,
    pub ocsf_class_uid: u32,
    pub fields: Vec<SemanticField>,
}

/// Mirrors `ocsf_semantic::SemanticMetric`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticMetric {
    pub name: String,
    pub expression: String,
    pub description: String,
}

/// Mirrors `ocsf_semantic::SemanticModel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticModel {
    pub name: String,
    pub ocsf_version: String,
    pub entities: Vec<SemanticEntity>,
    pub metrics: Vec<SemanticMetric>,
}

/// Mirrors `ocsf_core::CompiledAttribute`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledAttribute {
    pub name: String,
    pub caption: String,
    pub type_name: String,
    pub is_required: bool,
}

/// Mirrors `ocsf_core::CompiledClass`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledClass {
    pub uid: u32,
    pub name: String,
    pub caption: String,
    pub category_uid: u32,
    pub attributes: std::collections::BTreeMap<String, CompiledAttribute>,
}

/// Mirrors `ocsf_core::CompiledSchema`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledSchema {
    pub version: String,
    pub classes: std::collections::BTreeMap<u32, CompiledClass>,
}

impl CompiledSchema {
    pub fn get_class(&self, uid: u32) -> Option<&CompiledClass> {
        self.classes.get(&uid)
    }
}

// ── 8.1 WarehouseDialect & WarehouseGen ──────────────────────────────

/// Supported warehouse SQL dialects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WarehouseDialect {
    Snowflake,
    BigQuery,
    Databricks,
    Postgres,
}

impl Default for WarehouseDialect {
    fn default() -> Self {
        Self::Snowflake
    }
}

/// Top-level warehouse artifact generator.
pub struct WarehouseGen {
    pub dialect: WarehouseDialect,
}

impl Default for WarehouseGen {
    fn default() -> Self {
        Self {
            dialect: WarehouseDialect::default(),
        }
    }
}

impl WarehouseGen {
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self { dialect }
    }
}

// ── 8.3 DBTArtifacts & WarehouseArtifacts ────────────────────────────

/// dbt semantic layer artifacts produced by the generator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DBTArtifacts {
    /// YAML content for the dbt semantic manifest.
    pub semantic_manifest: String,
    /// Model name → SQL content.
    pub models: HashMap<String, String>,
    /// YAML content for dbt sources.
    pub sources: String,
}

/// All warehouse artifacts produced by a single generation run.
#[derive(Debug, Clone, PartialEq)]
pub struct WarehouseArtifacts {
    pub table_ddl: String,
    pub semantic_views: String,
    pub dbt_artifacts: DBTArtifacts,
    pub materialized_views: String,
}

// ── Internal generators ──────────────────────────────────────────────
// Stand-ins for ocsf-warehouse's TableGenerator, ViewGenerator, etc.

struct TableGenerator<'a> {
    dialect: &'a WarehouseDialect,
}

impl<'a> TableGenerator<'a> {
    fn generate(&self, job: &EtlJob, _schema: &CompiledSchema) -> String {
        let json_type = match self.dialect {
            WarehouseDialect::Snowflake => "VARIANT",
            WarehouseDialect::BigQuery => "JSON",
            WarehouseDialect::Databricks => "STRING",
            WarehouseDialect::Postgres => "JSONB",
        };

        let table_name = format!("ocsf_{}", job.ocsf_class_uid);
        let mut ddl = format!("CREATE TABLE {} (\n", table_name);

        for mapping in &job.mappings {
            let col_name = mapping
                .target_ocsf_path
                .replace('.', "_");
            ddl.push_str(&format!("    {} {},\n", col_name, json_type));
        }

        ddl.push_str(&format!("    _raw_event {},\n", json_type));
        ddl.push_str("    _ingested_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n");
        ddl.push_str(");\n");
        ddl
    }
}

struct ViewGenerator<'a> {
    dialect: &'a WarehouseDialect,
}

impl<'a> ViewGenerator<'a> {
    fn generate(&self, model: &SemanticModel) -> String {
        let mut views = String::new();
        for entity in &model.entities {
            let view_name = format!("v_{}", entity.name.to_lowercase().replace(' ', "_"));
            let table_name = format!("ocsf_{}", entity.ocsf_class_uid);

            views.push_str(&format!(
                "CREATE OR REPLACE VIEW {} AS\nSELECT\n",
                view_name
            ));

            let field_lines: Vec<String> = entity
                .fields
                .iter()
                .map(|f| {
                    let col = f.ocsf_path.replace('.', "_");
                    if let Some(ref tx) = f.transformation {
                        format!("    {}({}) AS {}", tx, col, f.name)
                    } else {
                        format!("    {} AS {}", col, f.name)
                    }
                })
                .collect();

            views.push_str(&field_lines.join(",\n"));
            views.push_str(&format!(
                "\nFROM {};\n\n",
                table_name
            ));

            // Add a comment about the dialect for traceability
            let _ = self.dialect; // used for potential dialect-specific syntax
        }
        views
    }
}

struct DBTGenerator;

impl DBTGenerator {
    fn generate(&self, model: &SemanticModel) -> DBTArtifacts {
        // Semantic manifest YAML
        let manifest = serde_yaml::to_string(&serde_yaml::Value::Mapping({
            let mut m = serde_yaml::Mapping::new();
            m.insert(
                serde_yaml::Value::String("semantic_models".into()),
                serde_yaml::Value::Sequence(
                    model
                        .entities
                        .iter()
                        .map(|e| {
                            let mut em = serde_yaml::Mapping::new();
                            em.insert(
                                serde_yaml::Value::String("name".into()),
                                serde_yaml::Value::String(e.name.clone()),
                            );
                            em.insert(
                                serde_yaml::Value::String("description".into()),
                                serde_yaml::Value::String(e.description.clone()),
                            );
                            em.insert(
                                serde_yaml::Value::String("ocsf_class_uid".into()),
                                serde_yaml::Value::Number(e.ocsf_class_uid.into()),
                            );
                            serde_yaml::Value::Mapping(em)
                        })
                        .collect(),
                ),
            );
            m
        }))
        .unwrap_or_default();

        // Model SQL files
        let mut models = HashMap::new();
        for entity in &model.entities {
            let model_name = entity.name.to_lowercase().replace(' ', "_");
            let table_name = format!("ocsf_{}", entity.ocsf_class_uid);

            let field_lines: Vec<String> = entity
                .fields
                .iter()
                .map(|f| {
                    let col = f.ocsf_path.replace('.', "_");
                    format!("    {} AS {}", col, f.name)
                })
                .collect();

            let sql = format!(
                "SELECT\n{}\nFROM {{{{ source('ocsf', '{}') }}}}\n",
                field_lines.join(",\n"),
                table_name,
            );
            models.insert(model_name, sql);
        }

        // Sources YAML
        let sources = serde_yaml::to_string(&serde_yaml::Value::Mapping({
            let mut m = serde_yaml::Mapping::new();
            m.insert(
                serde_yaml::Value::String("sources".into()),
                serde_yaml::Value::Sequence(vec![serde_yaml::Value::Mapping({
                    let mut sm = serde_yaml::Mapping::new();
                    sm.insert(
                        serde_yaml::Value::String("name".into()),
                        serde_yaml::Value::String("ocsf".into()),
                    );
                    sm.insert(
                        serde_yaml::Value::String("tables".into()),
                        serde_yaml::Value::Sequence(
                            model
                                .entities
                                .iter()
                                .map(|e| {
                                    let mut tm = serde_yaml::Mapping::new();
                                    tm.insert(
                                        serde_yaml::Value::String("name".into()),
                                        serde_yaml::Value::String(format!(
                                            "ocsf_{}",
                                            e.ocsf_class_uid
                                        )),
                                    );
                                    serde_yaml::Value::Mapping(tm)
                                })
                                .collect(),
                        ),
                    );
                    sm
                })]),
            );
            m
        }))
        .unwrap_or_default();

        DBTArtifacts {
            semantic_manifest: manifest,
            models,
            sources,
        }
    }
}

struct MaterializedViewGenerator<'a> {
    dialect: &'a WarehouseDialect,
}

impl<'a> MaterializedViewGenerator<'a> {
    fn generate(&self, model: &SemanticModel) -> String {
        let mut views = String::new();
        for entity in &model.entities {
            let mv_name = format!(
                "mv_{}_hot",
                entity.name.to_lowercase().replace(' ', "_")
            );
            let table_name = format!("ocsf_{}", entity.ocsf_class_uid);

            let create_kw = match self.dialect {
                WarehouseDialect::BigQuery => "CREATE MATERIALIZED VIEW",
                _ => "CREATE MATERIALIZED VIEW IF NOT EXISTS",
            };

            views.push_str(&format!("{} {} AS\nSELECT\n", create_kw, mv_name));

            let field_lines: Vec<String> = entity
                .fields
                .iter()
                .map(|f| format!("    {}", f.ocsf_path.replace('.', "_")))
                .collect();

            views.push_str(&field_lines.join(",\n"));
            views.push_str(&format!(
                ",\n    COUNT(*) AS event_count\nFROM {}\nGROUP BY {}\n;\n\n",
                table_name,
                field_lines
                    .iter()
                    .map(|f| f.trim().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
        views
    }
}

// ── 8.2 WarehouseGen::generate ───────────────────────────────────────

impl WarehouseGen {
    /// Generate all warehouse artifacts from an ETL job and its semantic model.
    ///
    /// Returns an error if the job has no mappings.
    pub fn generate(
        &self,
        schema: &CompiledSchema,
        job: &EtlJob,
        semantic_model: &SemanticModel,
    ) -> Result<WarehouseArtifacts> {
        if job.mappings.is_empty() {
            bail!("EtlJob must have at least one mapping to generate warehouse artifacts");
        }

        let table_ddl = TableGenerator { dialect: &self.dialect }.generate(job, schema);
        let semantic_views = ViewGenerator { dialect: &self.dialect }.generate(semantic_model);
        let dbt_artifacts = DBTGenerator.generate(semantic_model);
        let materialized_views =
            MaterializedViewGenerator { dialect: &self.dialect }.generate(semantic_model);

        Ok(WarehouseArtifacts {
            table_ddl,
            semantic_views,
            dbt_artifacts,
            materialized_views,
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use etl_core::{MappingSpec, SourceConfig};
    use proptest::prelude::*;
    use uuid::Uuid;

    fn test_semantic_model() -> SemanticModel {
        SemanticModel {
            name: "dns_activity".to_string(),
            ocsf_version: "1.3.0".to_string(),
            entities: vec![SemanticEntity {
                name: "dns_activity".to_string(),
                description: "DNS Activity events".to_string(),
                ocsf_class_uid: 4003,
                fields: vec![
                    SemanticField {
                        name: "query_hostname".to_string(),
                        source_field: "dns.question.name".to_string(),
                        ocsf_path: "query.hostname".to_string(),
                        transformation: None,
                    },
                    SemanticField {
                        name: "src_ip".to_string(),
                        source_field: "source.ip".to_string(),
                        ocsf_path: "src_endpoint.ip".to_string(),
                        transformation: None,
                    },
                ],
            }],
            metrics: vec![SemanticMetric {
                name: "total_queries".to_string(),
                expression: "COUNT(*)".to_string(),
                description: "Total DNS queries".to_string(),
            }],
        }
    }

    fn test_schema() -> CompiledSchema {
        CompiledSchema {
            version: "1.3.0".to_string(),
            classes: std::collections::BTreeMap::new(),
        }
    }

    fn test_job() -> EtlJob {
        EtlJob {
            job_id: Uuid::new_v4(),
            plugin_name: "dns_plugin".to_string(),
            ocsf_class_uid: 4003,
            ocsf_version: "1.3.0".to_string(),
            mappings: vec![
                MappingSpec::new(
                    "dns.question.name".to_string(),
                    "query.hostname".to_string(),
                    None,
                    1.0,
                ),
                MappingSpec::new(
                    "source.ip".to_string(),
                    "src_endpoint.ip".to_string(),
                    None,
                    1.0,
                ),
            ],
            source_config: SourceConfig::File {
                path: "/var/log/dns.log".to_string(),
            },
            delta_table_uri: "s3://bucket/dns".to_string(),
            s3_config: None,
            catalog_ref: None,
        }
    }

    #[test]
    fn default_dialect_is_snowflake() {
        let gen = WarehouseGen::default();
        assert_eq!(gen.dialect, WarehouseDialect::Snowflake);
    }

    #[test]
    fn generate_returns_error_for_empty_mappings() {
        let gen = WarehouseGen::default();
        let mut job = test_job();
        job.mappings.clear();

        let result = gen.generate(&test_schema(), &job, &test_semantic_model());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least one mapping")
        );
    }

    #[test]
    fn generate_produces_nonempty_artifacts() {
        let gen = WarehouseGen::default();
        let artifacts = gen
            .generate(&test_schema(), &test_job(), &test_semantic_model())
            .unwrap();

        assert!(!artifacts.table_ddl.is_empty());
        assert!(!artifacts.semantic_views.is_empty());
        assert!(!artifacts.dbt_artifacts.semantic_manifest.is_empty());
        assert!(!artifacts.dbt_artifacts.sources.is_empty());
        assert!(!artifacts.dbt_artifacts.models.is_empty());
        assert!(!artifacts.materialized_views.is_empty());
    }

    // ── 9.1 DNS Activity artifacts contain CREATE TABLE and CREATE VIEW ──

    #[test]
    fn dns_activity_artifacts_contain_create_table_and_view() {
        let gen = WarehouseGen::default();
        let artifacts = gen
            .generate(&test_schema(), &test_job(), &test_semantic_model())
            .unwrap();

        assert!(
            artifacts.table_ddl.contains("CREATE TABLE"),
            "table_ddl should contain CREATE TABLE, got: {}",
            artifacts.table_ddl
        );
        assert!(
            artifacts.semantic_views.contains("CREATE OR REPLACE VIEW"),
            "semantic_views should contain CREATE OR REPLACE VIEW, got: {}",
            artifacts.semantic_views
        );
    }

    // ── 9.2 Proptest generators ──────────────────────────────────────

    fn arb_warehouse_dialect() -> impl Strategy<Value = WarehouseDialect> {
        prop_oneof![
            Just(WarehouseDialect::Snowflake),
            Just(WarehouseDialect::BigQuery),
            Just(WarehouseDialect::Databricks),
            Just(WarehouseDialect::Postgres),
        ]
    }

    fn arb_semantic_field() -> impl Strategy<Value = SemanticField> {
        ("[a-z_]{1,15}", "[a-z_.]{1,20}", "[a-z_.]{1,20}").prop_map(
            |(name, source, ocsf_path)| SemanticField {
                name,
                source_field: source,
                ocsf_path,
                transformation: None,
            },
        )
    }

    fn arb_semantic_entity() -> impl Strategy<Value = SemanticEntity> {
        (
            "[a-z_]{1,15}",
            1000u32..10000u32,
            prop::collection::vec(arb_semantic_field(), 1..5),
        )
            .prop_map(|(name, uid, fields)| SemanticEntity {
                name,
                description: "test entity".to_string(),
                ocsf_class_uid: uid,
                fields,
            })
    }

    fn arb_semantic_model() -> impl Strategy<Value = SemanticModel> {
        prop::collection::vec(arb_semantic_entity(), 1..4).prop_map(|entities| SemanticModel {
            name: "test_model".to_string(),
            ocsf_version: "1.3.0".to_string(),
            entities,
            metrics: vec![],
        })
    }

    fn arb_mapping_spec() -> impl Strategy<Value = MappingSpec> {
        ("[a-z_]{1,20}", "[a-z_.]{1,30}", 0.0f32..=1.0f32).prop_map(
            |(source, target, conf)| MappingSpec::new(source, target, None, conf),
        )
    }

    fn arb_etl_job_with_mappings() -> impl Strategy<Value = EtlJob> {
        (
            "[a-z_]{3,15}",
            1000u32..10000u32,
            prop::collection::vec(arb_mapping_spec(), 1..5),
        )
            .prop_map(|(plugin_name, uid, mappings)| EtlJob {
                job_id: Uuid::new_v4(),
                plugin_name,
                ocsf_class_uid: uid,
                ocsf_version: "1.3.0".to_string(),
                mappings,
                source_config: SourceConfig::File {
                    path: "/tmp/test.log".to_string(),
                },
                delta_table_uri: "s3://bucket/table".to_string(),
                s3_config: None,
                catalog_ref: None,
            })
    }

    fn arb_etl_job_no_mappings() -> impl Strategy<Value = EtlJob> {
        ("[a-z_]{3,15}", 1000u32..10000u32).prop_map(|(plugin_name, uid)| EtlJob {
            job_id: Uuid::new_v4(),
            plugin_name,
            ocsf_class_uid: uid,
            ocsf_version: "1.3.0".to_string(),
            mappings: vec![],
            source_config: SourceConfig::File {
                path: "/tmp/test.log".to_string(),
            },
            delta_table_uri: "s3://bucket/table".to_string(),
            s3_config: None,
            catalog_ref: None,
        })
    }

    // ── 9.2 [PBT: Property 8] All artifact fields non-empty ─────────
    // **Validates: Requirements 4.7**

    proptest! {
        #[test]
        fn warehouse_artifacts_all_nonempty(
            job in arb_etl_job_with_mappings(),
            model in arb_semantic_model(),
            dialect in arb_warehouse_dialect(),
        ) {
            let gen = WarehouseGen::new(dialect);
            let schema = test_schema();
            let artifacts = gen.generate(&schema, &job, &model).unwrap();

            prop_assert!(!artifacts.table_ddl.is_empty(), "table_ddl was empty");
            prop_assert!(!artifacts.semantic_views.is_empty(), "semantic_views was empty");
            prop_assert!(
                !artifacts.dbt_artifacts.semantic_manifest.is_empty(),
                "dbt semantic_manifest was empty"
            );
            prop_assert!(!artifacts.materialized_views.is_empty(), "materialized_views was empty");
        }
    }

    // ── 9.3 [PBT: Property 10] Zero mappings → error ────────────────
    // **Validates: Requirements 4.8**

    proptest! {
        #[test]
        fn zero_mappings_returns_error(
            job in arb_etl_job_no_mappings(),
            model in arb_semantic_model(),
            dialect in arb_warehouse_dialect(),
        ) {
            let gen = WarehouseGen::new(dialect);
            let schema = test_schema();
            let result = gen.generate(&schema, &job, &model);

            prop_assert!(result.is_err(), "expected error for zero mappings, got Ok");
            let err_msg = result.unwrap_err().to_string();
            prop_assert!(
                err_msg.contains("at least one mapping"),
                "error message should mention mappings, got: {}",
                err_msg
            );
        }
    }
}
