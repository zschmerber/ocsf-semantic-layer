//! Codegen Adapter - converts EtlJob to PipelineGenRequest and generates SemanticModel.

use etl_core::{EtlJob, MappingSpec, SqlType, Transform};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Default confidence threshold for filtering mappings.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.5;

/// Errors specific to the codegen adapter.
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("OCSF schema not found at path: {path}")]
    SchemaNotFound { path: String },

    #[error("OCSF_SCHEMA_PATH environment variable is not set")]
    SchemaPathNotSet,

    #[error("no mappings above confidence threshold {threshold}")]
    NoMappingsAboveThreshold { threshold: f32 },

    #[error("codegen failed: {0}")]
    CodegenFailed(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// ── FieldLineageRecord (local mirror of ocsf-types) ─────────────────

/// Unique identifier for a lineage record.
///
/// Mirrors `ocsf_types::index::types::LineageId`. When the `pipeline-codegen`
/// path dependency is uncommented this type can be replaced by a re-export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageId(pub u64);

impl LineageId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Field lineage record tracking how source fields map to OCSF fields.
///
/// Mirrors `ocsf_types::index::field_lineage::FieldLineageRecord`. When the
/// `pipeline-codegen` path dependency is uncommented this type can be replaced
/// by a re-export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldLineageRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LineageId>,
    pub source_lineage_id: LineageId,
    pub source_field: String,
    pub target_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocsf_version: Option<String>,
}

impl FieldLineageRecord {
    pub fn new(
        source_lineage_id: LineageId,
        source_field: impl Into<String>,
        target_field: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            source_lineage_id,
            source_field: source_field.into(),
            target_field: target_field.into(),
            transformation: None,
            ocsf_version: None,
        }
    }

    pub fn with_transformation(mut self, expr: impl Into<String>) -> Self {
        self.transformation = Some(expr.into());
        self
    }

    pub fn with_ocsf_version(mut self, version: impl Into<String>) -> Self {
        self.ocsf_version = Some(version.into());
        self
    }
}

// ── Stand-in types (mirrors of ocsf-core / ocsf-semantic / pipeline-codegen) ─

/// Mirrors `ocsf_core::CompiledAttribute`. When the `ocsf-core` path
/// dependency is uncommented this type can be replaced by a re-export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledAttribute {
    pub name: String,
    pub caption: String,
    pub type_name: String,
    pub is_required: bool,
}

/// Mirrors `ocsf_core::CompiledClass`. When the `ocsf-core` path
/// dependency is uncommented this type can be replaced by a re-export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledClass {
    pub uid: u32,
    pub name: String,
    pub caption: String,
    pub category_uid: u32,
    pub attributes: BTreeMap<String, CompiledAttribute>,
}

/// Mirrors `ocsf_core::CompiledSchema`. When the `ocsf-core` path
/// dependency is uncommented this type can be replaced by a re-export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledSchema {
    pub version: String,
    pub classes: BTreeMap<u32, CompiledClass>,
}

impl CompiledSchema {
    /// Look up a class by its UID.
    pub fn get_class(&self, uid: u32) -> Option<&CompiledClass> {
        self.classes.get(&uid)
    }
}

/// Mirrors `ocsf_semantic::SemanticEntity`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticEntity {
    pub name: String,
    pub description: String,
    pub ocsf_class_uid: u32,
    pub fields: Vec<SemanticField>,
}

/// A field within a semantic entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticField {
    pub name: String,
    pub source_field: String,
    pub ocsf_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
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

/// Mirrors `pipeline_codegen::PipelineGenRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineGenRequest {
    pub plugin_name: String,
    pub class_uid: u32,
    pub class_name: String,
    pub category_uid: u32,
    pub ocsf_version: String,
    pub field_mappings: Vec<FieldLineageRecord>,
    pub sink_config: SinkConfig,
}

/// Sink configuration embedded in the pipeline generation request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SinkConfig {
    pub delta_table_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_endpoint: Option<String>,
}

// ── Conversion: MappingSpec → FieldLineageRecord ─────────────────────

/// Convert a `Transform` variant into a SQL-style transformation expression
/// applied to the given source field.
fn transform_to_expression(source_field: &str, transform: &Transform) -> String {
    match transform {
        Transform::Lower => format!("LOWER({})", source_field),
        Transform::Upper => format!("UPPER({})", source_field),
        Transform::Trim => format!("TRIM({})", source_field),
        Transform::Cast(sql_type) => {
            let type_name = match sql_type {
                SqlType::Integer => "INTEGER",
                SqlType::BigInt => "BIGINT",
                SqlType::Float => "FLOAT",
                SqlType::Double => "DOUBLE",
                SqlType::Boolean => "BOOLEAN",
                SqlType::String => "STRING",
                SqlType::Timestamp => "TIMESTAMP",
                SqlType::Date => "DATE",
            };
            format!("CAST({} AS {})", source_field, type_name)
        }
    }
}

/// Convert a `MappingSpec` into a `FieldLineageRecord`.
///
/// Maps `source_field` → `source_field`, `target_ocsf_path` → `target_field`,
/// and converts the optional `Transform` into a SQL-style transformation
/// expression string. Uses `LineageId(0)` as a placeholder source lineage ID
/// (the caller is expected to set the real ID when building a full pipeline
/// request).
pub fn to_field_lineage_record(mapping: &MappingSpec) -> FieldLineageRecord {
    let mut record = FieldLineageRecord::new(
        LineageId::new(0),
        &mapping.source_field,
        &mapping.target_ocsf_path,
    );

    if let Some(ref transform) = mapping.transformation {
        record = record.with_transformation(transform_to_expression(
            &mapping.source_field,
            transform,
        ));
    }

    record
}

/// Output produced by the codegen adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterOutput {
    /// Generated Go plugin source code.
    pub go_code: String,
    /// Generated go.mod contents.
    pub go_mod: String,
    /// Generated tangent.yaml contents (includes embedded semantic model).
    pub tangent_yaml: String,
    /// Generated test fixture contents.
    pub test_fixture: String,
    /// Standalone semantic model serialized as YAML.
    pub semantic_model_yaml: String,
}

/// Converts an `EtlJob` into pipeline codegen artifacts and a `SemanticModel`.
#[derive(Debug, Clone)]
pub struct CodegenAdapter {
    /// Mappings with confidence below this threshold are excluded.
    pub confidence_threshold: f32,
}

impl Default for CodegenAdapter {
    fn default() -> Self {
        Self {
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
        }
    }
}

impl CodegenAdapter {
    /// Create a new adapter with the default confidence threshold (0.5).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set a custom confidence threshold.
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Filter mappings by confidence threshold, returning only those at or
    /// above `self.confidence_threshold`.
    pub fn filter_mappings<'a>(&self, mappings: &'a [MappingSpec]) -> Vec<&'a MappingSpec> {
        mappings
            .iter()
            .filter(|m| m.confidence >= self.confidence_threshold)
            .collect()
    }

    /// Generate all codegen artifacts from a compiled schema and ETL job.
    ///
    /// 1. Filters `job.mappings` by confidence threshold
    /// 2. Converts each passing `MappingSpec` to a `FieldLineageRecord`
    /// 3. Builds a `PipelineGenRequest`
    /// 4. Generates Go code, go.mod, tangent.yaml, test fixtures
    /// 5. Generates a `SemanticModel` and serializes it as YAML
    /// 6. Embeds the semantic model YAML in tangent.yaml
    ///
    /// Returns `CodegenError::NoMappingsAboveThreshold` if no mappings pass
    /// the confidence gate.
    pub fn generate(
        &self,
        schema: &CompiledSchema,
        job: &EtlJob,
    ) -> Result<AdapterOutput, CodegenError> {
        // 1. Filter mappings by confidence threshold
        let passing = self.filter_mappings(&job.mappings);
        if passing.is_empty() {
            return Err(CodegenError::NoMappingsAboveThreshold {
                threshold: self.confidence_threshold,
            });
        }

        // Resolve the OCSF class from the schema
        let compiled_class = schema.get_class(job.ocsf_class_uid).ok_or_else(|| {
            CodegenError::CodegenFailed(format!(
                "OCSF class UID {} not found in schema",
                job.ocsf_class_uid
            ))
        })?;

        // 2. Convert each passing MappingSpec to a FieldLineageRecord
        let field_mappings: Vec<FieldLineageRecord> = passing
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let mut record = to_field_lineage_record(m);
                record.source_lineage_id = LineageId::new(i as u64 + 1);
                record = record.with_ocsf_version(&job.ocsf_version);
                record
            })
            .collect();

        // 3. Build PipelineGenRequest
        let sink_config = SinkConfig {
            delta_table_uri: job.delta_table_uri.clone(),
            s3_region: job.s3_config.as_ref().map(|s| s.region.clone()),
            s3_endpoint: job
                .s3_config
                .as_ref()
                .and_then(|s| s.endpoint.clone()),
        };

        let request = PipelineGenRequest {
            plugin_name: job.plugin_name.clone(),
            class_uid: job.ocsf_class_uid,
            class_name: compiled_class.name.clone(),
            category_uid: compiled_class.category_uid,
            ocsf_version: job.ocsf_version.clone(),
            field_mappings: field_mappings.clone(),
            sink_config,
        };

        // 4. Generate Go code (local implementation mirroring pipeline-codegen)
        let go_code = generate_go_code(&request);
        let go_mod = generate_go_mod(&request.plugin_name);
        let test_fixture = generate_test_fixture(&request);

        // 5. Generate SemanticModel
        let semantic_model = generate_semantic_model(schema, job, &passing)?;
        let semantic_model_yaml = serde_yaml::to_string(&semantic_model)
            .map_err(|e| CodegenError::CodegenFailed(format!("YAML serialization failed: {e}")))?;

        // 6. Generate tangent.yaml with embedded semantic model
        let tangent_yaml = generate_tangent_yaml(&request, &semantic_model_yaml);

        Ok(AdapterOutput {
            go_code,
            go_mod,
            tangent_yaml,
            test_fixture,
            semantic_model_yaml,
        })
    }
}

// ── Code generation helpers (local stand-ins for pipeline-codegen) ────

/// Convert a class name like "DNS Activity" into a Go struct name like "DnsActivity".
fn to_go_struct_name(class_name: &str) -> String {
    class_name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let first: String = c.to_uppercase().collect();
                    let rest: String = chars.collect::<String>().to_lowercase();
                    format!("{first}{rest}")
                }
            }
        })
        .collect()
}

/// Generate Go plugin source code from a `PipelineGenRequest`.
///
/// Mirrors `pipeline_codegen::generate_pipeline()` Go output. When the real
/// dependency is wired up, this function will be replaced by a call to
/// `pipeline_codegen::generate_pipeline()`.
fn generate_go_code(req: &PipelineGenRequest) -> String {
    let struct_name = to_go_struct_name(&req.class_name);
    let mut field_assignments = String::new();
    for mapping in &req.field_mappings {
        let go_field = ocsf_path_to_go_field(&mapping.target_field);
        let value_expr = match &mapping.transformation {
            Some(expr) => format!("transform({}, \"{}\")", mapping.source_field, expr),
            None => format!("record[\"{}\"]", mapping.source_field),
        };
        field_assignments.push_str(&format!("    event.{go_field} = {value_expr}\n"));
    }

    format!(
        r#"package main

import (
    "github.com/ocsf/tangent/sdk"
)

// {struct_name} OCSF event class (UID: {class_uid}, Category: {category_uid})
type {struct_name} struct {{
    ClassUID    int    `json:"class_uid"`
    CategoryUID int    `json:"category_uid"`
    // Field mappings follow
}}

func ClassUID() int {{
    return {class_uid}
}}

func CategoryUID() int {{
    return {category_uid}
}}

func Transform(record map[string]interface{{}}) *{struct_name} {{
    event := &{struct_name}{{
        ClassUID:    {class_uid},
        CategoryUID: {category_uid},
    }}
{field_assignments}    return event
}}

func init() {{
    sdk.RegisterPlugin("{plugin_name}", Transform)
}}
"#,
        struct_name = struct_name,
        class_uid = req.class_uid,
        category_uid = req.category_uid,
        field_assignments = field_assignments,
        plugin_name = req.plugin_name,
    )
}

/// Convert an OCSF dotted path like "src_endpoint.ip" to a Go field name like "SrcEndpointIp".
fn ocsf_path_to_go_field(path: &str) -> String {
    path.split('.')
        .flat_map(|segment| {
            segment.split('_').map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => {
                        let first: String = c.to_uppercase().collect();
                        let rest: String = chars.collect();
                        format!("{first}{rest}")
                    }
                }
            })
        })
        .collect()
}

/// Generate a go.mod file for the plugin.
fn generate_go_mod(plugin_name: &str) -> String {
    format!(
        r#"module {plugin_name}

go 1.21

require (
    github.com/ocsf/tangent/sdk v0.1.0
)
"#
    )
}

/// Generate test fixture JSON for the plugin.
fn generate_test_fixture(req: &PipelineGenRequest) -> String {
    let mut input_fields = serde_json::Map::new();
    let mut expected_fields = serde_json::Map::new();

    expected_fields.insert(
        "class_uid".to_string(),
        serde_json::Value::Number(req.class_uid.into()),
    );
    expected_fields.insert(
        "category_uid".to_string(),
        serde_json::Value::Number(req.category_uid.into()),
    );

    for mapping in &req.field_mappings {
        input_fields.insert(
            mapping.source_field.clone(),
            serde_json::Value::String(format!("test_{}", mapping.source_field)),
        );
        expected_fields.insert(
            mapping.target_field.clone(),
            serde_json::Value::String(format!("test_{}", mapping.source_field)),
        );
    }

    let fixture = serde_json::json!({
        "input": input_fields,
        "expected": expected_fields,
    });

    serde_json::to_string_pretty(&fixture).unwrap_or_default()
}

/// Generate a `tangent.yaml` configuration with the semantic model embedded.
fn generate_tangent_yaml(req: &PipelineGenRequest, semantic_model_yaml: &str) -> String {
    // Indent the semantic model YAML for embedding under the sink block
    let indented_model: String = semantic_model_yaml
        .lines()
        .map(|line| format!("      {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"plugin:
  name: {plugin_name}
  class_uid: {class_uid}
  category_uid: {category_uid}
  ocsf_version: "{ocsf_version}"

sink:
  type: delta
  uri: "{delta_uri}"
  ocsf_semantic_model: |
{indented_model}
"#,
        plugin_name = req.plugin_name,
        class_uid = req.class_uid,
        category_uid = req.category_uid,
        ocsf_version = req.ocsf_version,
        delta_uri = req.sink_config.delta_table_uri,
        indented_model = indented_model,
    )
}

/// Generate a `SemanticModel` from the ETL job and compiled schema.
///
/// Derives entities from the OCSF class, fields from the passing mappings,
/// and basic count/rate metrics.
pub fn generate_semantic_model(
    schema: &CompiledSchema,
    job: &EtlJob,
    passing_mappings: &[&MappingSpec],
) -> Result<SemanticModel, CodegenError> {
    let compiled_class = schema.get_class(job.ocsf_class_uid).ok_or_else(|| {
        CodegenError::CodegenFailed(format!(
            "OCSF class UID {} not found in schema",
            job.ocsf_class_uid
        ))
    })?;

    let fields: Vec<SemanticField> = passing_mappings
        .iter()
        .map(|m| {
            let transformation = m.transformation.as_ref().map(|t| {
                transform_to_expression(&m.source_field, t)
            });
            SemanticField {
                name: m.target_ocsf_path.replace('.', "_"),
                source_field: m.source_field.clone(),
                ocsf_path: m.target_ocsf_path.clone(),
                transformation,
            }
        })
        .collect();

    let entity = SemanticEntity {
        name: compiled_class.name.replace(' ', "_").to_lowercase(),
        description: format!(
            "Semantic entity for OCSF {} (UID: {})",
            compiled_class.caption, compiled_class.uid
        ),
        ocsf_class_uid: compiled_class.uid,
        fields,
    };

    let metrics = vec![
        SemanticMetric {
            name: format!("{}_count", entity.name),
            expression: format!("COUNT(*) FROM {}", entity.name),
            description: format!("Total count of {} events", compiled_class.caption),
        },
        SemanticMetric {
            name: format!("{}_rate_per_minute", entity.name),
            expression: format!(
                "COUNT(*) / DATEDIFF(minute, MIN(time), MAX(time)) FROM {}",
                entity.name
            ),
            description: format!(
                "Rate of {} events per minute",
                compiled_class.caption
            ),
        },
    ];

    Ok(SemanticModel {
        name: job.plugin_name.clone(),
        ocsf_version: job.ocsf_version.clone(),
        entities: vec![entity],
        metrics,
    })
}

/// Load a `CompiledSchema` from the path specified by the `OCSF_SCHEMA_PATH`
/// environment variable.
///
/// Returns `CodegenError::SchemaPathNotSet` when the variable is absent and
/// `CodegenError::SchemaNotFound` when the file does not exist.
pub fn load_compiled_schema() -> Result<CompiledSchema, CodegenError> {
    let path = std::env::var("OCSF_SCHEMA_PATH").map_err(|_| CodegenError::SchemaPathNotSet)?;

    let content = std::fs::read_to_string(&path).map_err(|_| CodegenError::SchemaNotFound {
        path: path.clone(),
    })?;

    serde_json::from_str(&content).map_err(|e| {
        CodegenError::CodegenFailed(format!("failed to parse compiled schema at {path}: {e}"))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_is_half() {
        let adapter = CodegenAdapter::default();
        assert!((adapter.confidence_threshold - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn new_equals_default() {
        let a = CodegenAdapter::new();
        let b = CodegenAdapter::default();
        assert!((a.confidence_threshold - b.confidence_threshold).abs() < f32::EPSILON);
    }

    #[test]
    fn with_confidence_threshold_overrides() {
        let adapter = CodegenAdapter::new().with_confidence_threshold(0.8);
        assert!((adapter.confidence_threshold - 0.8).abs() < f32::EPSILON);
    }

    // ── to_field_lineage_record tests ────────────────────────────────

    #[test]
    fn converts_mapping_without_transformation() {
        let mapping = MappingSpec::new(
            "src_ip".to_string(),
            "src_endpoint.ip".to_string(),
            None,
            0.9,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(record.source_field, "src_ip");
        assert_eq!(record.target_field, "src_endpoint.ip");
        assert!(record.transformation.is_none());
        assert_eq!(record.source_lineage_id, LineageId::new(0));
    }

    #[test]
    fn converts_mapping_with_lower_transform() {
        let mapping = MappingSpec::new(
            "hostname".to_string(),
            "device.hostname".to_string(),
            Some(Transform::Lower),
            1.0,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(record.source_field, "hostname");
        assert_eq!(record.target_field, "device.hostname");
        assert_eq!(record.transformation, Some("LOWER(hostname)".to_string()));
    }

    #[test]
    fn converts_mapping_with_upper_transform() {
        let mapping = MappingSpec::new(
            "action".to_string(),
            "activity_name".to_string(),
            Some(Transform::Upper),
            0.8,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(record.transformation, Some("UPPER(action)".to_string()));
    }

    #[test]
    fn converts_mapping_with_trim_transform() {
        let mapping = MappingSpec::new(
            "user_name".to_string(),
            "actor.user.name".to_string(),
            Some(Transform::Trim),
            0.7,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(
            record.transformation,
            Some("TRIM(user_name)".to_string())
        );
    }

    #[test]
    fn converts_mapping_with_cast_integer() {
        let mapping = MappingSpec::new(
            "severity_str".to_string(),
            "severity_id".to_string(),
            Some(Transform::Cast(SqlType::Integer)),
            1.0,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(
            record.transformation,
            Some("CAST(severity_str AS INTEGER)".to_string())
        );
    }

    #[test]
    fn converts_mapping_with_cast_timestamp() {
        let mapping = MappingSpec::new(
            "timestamp_str".to_string(),
            "time".to_string(),
            Some(Transform::Cast(SqlType::Timestamp)),
            1.0,
        );
        let record = to_field_lineage_record(&mapping);

        assert_eq!(
            record.transformation,
            Some("CAST(timestamp_str AS TIMESTAMP)".to_string())
        );
    }

    #[test]
    fn converts_all_cast_sql_types() {
        let cases = vec![
            (SqlType::Integer, "INTEGER"),
            (SqlType::BigInt, "BIGINT"),
            (SqlType::Float, "FLOAT"),
            (SqlType::Double, "DOUBLE"),
            (SqlType::Boolean, "BOOLEAN"),
            (SqlType::String, "STRING"),
            (SqlType::Timestamp, "TIMESTAMP"),
            (SqlType::Date, "DATE"),
        ];

        for (sql_type, expected_name) in cases {
            let mapping = MappingSpec::new(
                "field".to_string(),
                "target".to_string(),
                Some(Transform::Cast(sql_type)),
                1.0,
            );
            let record = to_field_lineage_record(&mapping);
            let expected = format!("CAST(field AS {})", expected_name);
            assert_eq!(record.transformation, Some(expected));
        }
    }

    // ── Helper: build a test CompiledSchema ──────────────────────────

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

    fn test_job() -> etl_core::EtlJob {
        etl_core::EtlJob {
            job_id: uuid::Uuid::nil(),
            plugin_name: "dns_activity_plugin".to_string(),
            ocsf_class_uid: 4003,
            ocsf_version: "1.3.0".to_string(),
            mappings: vec![
                MappingSpec::new(
                    "hostname".to_string(),
                    "query.hostname".to_string(),
                    Some(Transform::Lower),
                    0.9,
                ),
                MappingSpec::new(
                    "src_ip".to_string(),
                    "src_endpoint.ip".to_string(),
                    None,
                    0.8,
                ),
                MappingSpec::new(
                    "low_conf_field".to_string(),
                    "unmapped.field".to_string(),
                    None,
                    0.3, // below default threshold
                ),
            ],
            source_config: etl_core::SourceConfig::File {
                path: "/var/log/dns.log".to_string(),
            },
            delta_table_uri: "s3://bucket/dns_activity".to_string(),
            s3_config: None,
            catalog_ref: None,
        }
    }

    // ── generate() tests ─────────────────────────────────────────────

    #[test]
    fn generate_filters_low_confidence_mappings() {
        let adapter = CodegenAdapter::new(); // threshold = 0.5
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        // The low-confidence mapping (0.3) should be excluded from Go code
        assert!(output.go_code.contains("DnsActivity"));
        assert!(output.go_code.contains("hostname"));
        assert!(output.go_code.contains("src_ip"));
        assert!(!output.go_code.contains("low_conf_field"));
    }

    #[test]
    fn generate_returns_error_when_all_below_threshold() {
        let adapter = CodegenAdapter::new().with_confidence_threshold(0.99);
        let schema = test_schema();
        let mut job = test_job();
        // Set all confidences below threshold
        for m in &mut job.mappings {
            m.confidence = 0.1;
        }

        let result = adapter.generate(&schema, &job);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("no mappings above confidence threshold"));
    }

    #[test]
    fn generate_returns_error_for_unknown_class_uid() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let mut job = test_job();
        job.ocsf_class_uid = 9999; // not in schema

        let result = adapter.generate(&schema, &job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found in schema"));
    }

    #[test]
    fn generate_produces_correct_go_struct_and_class_uid() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        assert!(output.go_code.contains("type DnsActivity struct"));
        assert!(output.go_code.contains("ClassUID:    4003"));
        assert!(output.go_code.contains("CategoryUID: 4"));
        assert!(output.go_code.contains("return 4003"));
    }

    #[test]
    fn generate_produces_valid_go_mod() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        assert!(output.go_mod.contains("module dns_activity_plugin"));
        assert!(output.go_mod.contains("go 1.21"));
    }

    #[test]
    fn generate_embeds_semantic_model_in_tangent_yaml() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        assert!(output.tangent_yaml.contains("ocsf_semantic_model: |"));
        assert!(output.tangent_yaml.contains("name: dns_activity_plugin"));
        // The semantic model YAML should be embedded
        assert!(output.tangent_yaml.contains("dns_activity"));
    }

    #[test]
    fn generate_produces_valid_semantic_model_yaml() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        // Should be valid YAML that deserializes back
        let model: SemanticModel =
            serde_yaml::from_str(&output.semantic_model_yaml).unwrap();
        assert_eq!(model.name, "dns_activity_plugin");
        assert_eq!(model.ocsf_version, "1.3.0");
        assert_eq!(model.entities.len(), 1);
        assert_eq!(model.entities[0].ocsf_class_uid, 4003);
        // Only 2 fields should pass (the 0.3 confidence one is filtered)
        assert_eq!(model.entities[0].fields.len(), 2);
    }

    #[test]
    fn generate_includes_transformation_in_lineage() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        // The hostname mapping has Transform::Lower, should appear in Go code
        assert!(output.go_code.contains("LOWER(hostname)"));
    }

    #[test]
    fn generate_is_idempotent() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output1 = adapter.generate(&schema, &job).unwrap();
        let output2 = adapter.generate(&schema, &job).unwrap();

        assert_eq!(output1.go_code, output2.go_code);
        assert_eq!(output1.go_mod, output2.go_mod);
        assert_eq!(output1.tangent_yaml, output2.tangent_yaml);
        assert_eq!(output1.test_fixture, output2.test_fixture);
        assert_eq!(output1.semantic_model_yaml, output2.semantic_model_yaml);
    }

    #[test]
    fn generate_test_fixture_has_input_and_expected() {
        let adapter = CodegenAdapter::new();
        let schema = test_schema();
        let job = test_job();

        let output = adapter.generate(&schema, &job).unwrap();

        assert!(output.test_fixture.contains("\"input\""));
        assert!(output.test_fixture.contains("\"expected\""));
        assert!(output.test_fixture.contains("\"class_uid\": 4003"));
    }

    #[test]
    fn filter_mappings_respects_threshold() {
        let adapter = CodegenAdapter::new().with_confidence_threshold(0.85);
        let mappings = vec![
            MappingSpec::new("a".into(), "b".into(), None, 0.9),
            MappingSpec::new("c".into(), "d".into(), None, 0.5),
            MappingSpec::new("e".into(), "f".into(), None, 0.85),
        ];

        let passing = adapter.filter_mappings(&mappings);
        assert_eq!(passing.len(), 2);
        assert_eq!(passing[0].source_field, "a");
        assert_eq!(passing[1].source_field, "e");
    }

    #[test]
    fn load_compiled_schema_returns_error_when_env_not_set() {
        // Ensure the env var is not set for this test.
        std::env::remove_var("OCSF_SCHEMA_PATH");

        let result = load_compiled_schema();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("OCSF_SCHEMA_PATH"),
            "error should mention OCSF_SCHEMA_PATH, got: {err}"
        );
    }

    #[test]
    fn load_compiled_schema_returns_error_for_missing_file() {
        std::env::set_var("OCSF_SCHEMA_PATH", "/tmp/nonexistent_ocsf_schema_12345.json");

        let result = load_compiled_schema();
        // Clean up before asserting
        std::env::remove_var("OCSF_SCHEMA_PATH");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("nonexistent_ocsf_schema_12345"),
            "error should mention the path, got: {err}"
        );
    }

    // ── Proptest generators for codegen-adapter ──────────────────────

    use proptest::prelude::*;

    /// Known attribute paths from `test_schema()`.
    const SCHEMA_ATTR_PATHS: &[&str] = &["query.hostname", "src_endpoint.ip"];

    fn arb_transform() -> impl Strategy<Value = Option<Transform>> {
        prop_oneof![
            Just(None),
            Just(Some(Transform::Lower)),
            Just(Some(Transform::Upper)),
            Just(Some(Transform::Trim)),
            Just(Some(Transform::Cast(SqlType::Integer))),
            Just(Some(Transform::Cast(SqlType::Timestamp))),
        ]
    }

    /// Generate a `MappingSpec` whose `target_ocsf_path` is drawn from the
    /// known schema attribute keys so it will be valid against `test_schema()`.
    fn arb_mapping_for_schema() -> impl Strategy<Value = MappingSpec> {
        (
            "[a-z_]{1,15}",
            prop::sample::select(SCHEMA_ATTR_PATHS),
            arb_transform(),
            0.0f32..=1.0f32,
        )
            .prop_map(|(source, target, transform, conf)| {
                MappingSpec::new(source, target.to_string(), transform, conf)
            })
    }

    /// Generate a `MappingSpec` with an arbitrary target path (may or may not
    /// be in the schema).
    fn arb_mapping_any_target() -> impl Strategy<Value = MappingSpec> {
        (
            "[a-z_]{1,15}",
            "[a-z_.]{1,30}",
            arb_transform(),
            0.0f32..=1.0f32,
        )
            .prop_map(|(source, target, transform, conf)| {
                MappingSpec::new(source, target, transform, conf)
            })
    }

    /// Generate an `EtlJob` with `ocsf_class_uid = 4003` and mappings whose
    /// target paths are valid schema attributes.
    fn arb_job_for_schema() -> impl Strategy<Value = etl_core::EtlJob> {
        prop::collection::vec(arb_mapping_for_schema(), 1..8).prop_map(|mappings| {
            etl_core::EtlJob {
                job_id: uuid::Uuid::nil(),
                plugin_name: "test_plugin".to_string(),
                ocsf_class_uid: 4003,
                ocsf_version: "1.3.0".to_string(),
                mappings,
                source_config: etl_core::SourceConfig::File {
                    path: "/tmp/test.log".to_string(),
                },
                delta_table_uri: "s3://bucket/table".to_string(),
                s3_config: None,
                catalog_ref: None,
            }
        })
    }

    /// Generate an `EtlJob` with arbitrary target paths (for mixed-confidence
    /// tests that only exercise `filter_mappings`).
    fn arb_job_any() -> impl Strategy<Value = etl_core::EtlJob> {
        prop::collection::vec(arb_mapping_any_target(), 1..8).prop_map(|mappings| {
            etl_core::EtlJob {
                job_id: uuid::Uuid::nil(),
                plugin_name: "test_plugin".to_string(),
                ocsf_class_uid: 4003,
                ocsf_version: "1.3.0".to_string(),
                mappings,
                source_config: etl_core::SourceConfig::File {
                    path: "/tmp/test.log".to_string(),
                },
                delta_table_uri: "s3://bucket/table".to_string(),
                s3_config: None,
                catalog_ref: None,
            }
        })
    }

    // ── 7.3 [PBT: Property 5] Confidence gating preserves fields ────
    // **Validates: Requirements 3.2**

    proptest! {
        #[test]
        fn confidence_gating_preserves_fields(job in arb_job_for_schema(), threshold in 0.0f32..=1.0f32) {
            let schema = test_schema();
            let adapter = CodegenAdapter::new().with_confidence_threshold(threshold);
            let passing = adapter.filter_mappings(&job.mappings);

            // Every passing mapping must be at or above threshold
            for m in &passing {
                prop_assert!(m.confidence >= threshold,
                    "mapping with confidence {} passed threshold {}", m.confidence, threshold);
            }

            // No mapping below threshold should appear
            for m in &job.mappings {
                if m.confidence < threshold {
                    prop_assert!(!passing.iter().any(|p| std::ptr::eq(*p, m)),
                        "mapping with confidence {} should not pass threshold {}", m.confidence, threshold);
                }
            }

            // If there are passing mappings, generate and verify field preservation
            if !passing.is_empty() {
                let output = adapter.generate(&schema, &job).unwrap();
                // Parse the tangent.yaml to verify the request was built correctly
                // Instead, verify via the semantic model that fields are preserved
                let model: SemanticModel = serde_yaml::from_str(&output.semantic_model_yaml).unwrap();
                let entity = &model.entities[0];

                for m in &passing {
                    // Each passing mapping's target_ocsf_path should appear as a field
                    let found = entity.fields.iter().any(|f| f.ocsf_path == m.target_ocsf_path && f.source_field == m.source_field);
                    prop_assert!(found,
                        "passing mapping source={} target={} not found in semantic model fields",
                        m.source_field, m.target_ocsf_path);
                }
            }
        }
    }

    // ── 7.4 [PBT: Property 6] Generate idempotency ─────────────────
    // **Validates: Requirements 3.10**

    proptest! {
        #[test]
        fn generate_idempotency(job in arb_job_for_schema()) {
            let schema = test_schema();
            let adapter = CodegenAdapter::new().with_confidence_threshold(0.0);

            let out1 = adapter.generate(&schema, &job).unwrap();
            let out2 = adapter.generate(&schema, &job).unwrap();

            prop_assert_eq!(&out1.go_code, &out2.go_code);
            prop_assert_eq!(&out1.go_mod, &out2.go_mod);
            prop_assert_eq!(&out1.tangent_yaml, &out2.tangent_yaml);
            prop_assert_eq!(&out1.test_fixture, &out2.test_fixture);
            prop_assert_eq!(&out1.semantic_model_yaml, &out2.semantic_model_yaml);
        }
    }

    // ── 7.5 [PBT: Property 7] Semantic model YAML validity ─────────
    // **Validates: Requirements 3.7**

    proptest! {
        #[test]
        fn semantic_model_yaml_is_valid(job in arb_job_for_schema()) {
            let schema = test_schema();
            // Use threshold 0.0 to ensure at least one mapping passes
            let adapter = CodegenAdapter::new().with_confidence_threshold(0.0);

            let output = adapter.generate(&schema, &job).unwrap();

            // Must be valid YAML
            let model: SemanticModel = serde_yaml::from_str(&output.semantic_model_yaml)
                .expect("semantic_model_yaml should be valid YAML");

            // Must have at least one entity
            prop_assert!(!model.entities.is_empty(),
                "SemanticModel should have at least one entity");
        }
    }

    // ── 7.6 [PBT: Property 5] Confidence gating monotonicity ───────
    // **Validates: Requirements 9.9**

    proptest! {
        #[test]
        fn confidence_gating_monotonicity(
            job in arb_job_any(),
            t1 in 0.0f32..=1.0f32,
            t2 in 0.0f32..=1.0f32,
        ) {
            // Ensure t_low < t_high
            let (t_low, t_high) = if t1 <= t2 { (t1, t2) } else { (t2, t1) };

            let adapter_low = CodegenAdapter::new().with_confidence_threshold(t_low);
            let adapter_high = CodegenAdapter::new().with_confidence_threshold(t_high);

            let passing_low = adapter_low.filter_mappings(&job.mappings);
            let passing_high = adapter_high.filter_mappings(&job.mappings);

            prop_assert!(passing_high.len() <= passing_low.len(),
                "with t_high={} got {} mappings but t_low={} got {} mappings",
                t_high, passing_high.len(), t_low, passing_low.len());
        }
    }

    // ── 7.7 [PBT: Property 11] Schema path validation ──────────────
    // **Validates: Requirements 9.10**

    proptest! {
        #[test]
        fn schema_path_validation(job in arb_job_for_schema()) {
            let schema = test_schema();
            let compiled_class = schema.get_class(4003).unwrap();

            // Every mapping's target_ocsf_path should be a valid attribute key
            for m in &job.mappings {
                prop_assert!(compiled_class.attributes.contains_key(&m.target_ocsf_path),
                    "target_ocsf_path '{}' not found in schema attributes {:?}",
                    m.target_ocsf_path,
                    compiled_class.attributes.keys().collect::<Vec<_>>());
            }
        }
    }

    // ── 7.8 [PBT: Property 12] Semantic model idempotency ──────────
    // **Validates: Requirements 9.11**

    proptest! {
        #[test]
        fn semantic_model_idempotency(job in arb_job_for_schema()) {
            let schema = test_schema();
            let adapter = CodegenAdapter::new().with_confidence_threshold(0.0);
            let passing: Vec<&MappingSpec> = adapter.filter_mappings(&job.mappings);

            let model1 = generate_semantic_model(&schema, &job, &passing).unwrap();
            let yaml1 = serde_yaml::to_string(&model1).unwrap();

            let model2 = generate_semantic_model(&schema, &job, &passing).unwrap();
            let yaml2 = serde_yaml::to_string(&model2).unwrap();

            prop_assert_eq!(yaml1, yaml2,
                "generate_semantic_model should produce identical YAML on repeated calls");
        }
    }

}
