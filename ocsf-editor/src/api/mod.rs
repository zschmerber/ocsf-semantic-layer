//! API endpoint handlers for the OCSF Semantic Model Editor.
//!
//! This module contains the HTTP handlers for all API endpoints.
//! Each handler corresponds to a requirement from the specification.
//!
//! # Submodules
//!
//! - [`index`]: Index API endpoints for table registry, lineage, coverage, and statistics

pub mod catalog;
pub mod index;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use ocsf_semantic::model::SemanticModel;
use serde::{Deserialize, Serialize};

use crate::error::EditorApiError;
use crate::schema_service::{CategoryNode, ObjectNode};
use crate::validation_service::ValidationReport;
use crate::AppState;

// ============================================================================
// Health Check
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check endpoint.
///
/// Returns the server status and version information.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============================================================================
// Schema Endpoints (Requirement 7.1)
// ============================================================================

/// Response for GET /api/schema endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface SchemaResponse {
///   version: string;
///   categories: Category[];
///   objects: OCSFObject[];
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaResponse {
    /// Schema version (e.g., "1.4.0").
    pub version: String,
    /// Categories with their event classes and attributes.
    pub categories: Vec<CategoryNode>,
    /// Standalone objects with their attributes.
    pub objects: Vec<ObjectNode>,
}

/// Cache duration for schema responses (1 hour in seconds).
const SCHEMA_CACHE_MAX_AGE: u32 = 3600;

/// GET /api/schema - Returns the loaded OCSF schema as JSON.
///
/// Returns the schema tree structure including categories, classes, objects,
/// and attributes. The response includes caching headers for browser caching.
///
/// # Requirements
/// - 7.1: THE API_Server SHALL expose a GET /api/schema endpoint that returns
///   the loaded OCSF schema as JSON
///
/// # Response Headers
/// - `Cache-Control: public, max-age=3600` - Allow browser caching for 1 hour
/// - `ETag` - Based on schema version for conditional requests
///
/// # Returns
/// - 200 OK with SchemaResponse JSON body
/// - 503 Service Unavailable if no schema is loaded
pub async fn get_schema(State(state): State<AppState>) -> impl IntoResponse {
    // Check if schema is loaded
    if !state.schema_service.is_loaded().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            HeaderMap::new(),
            Json(SchemaResponse {
                version: String::new(),
                categories: vec![],
                objects: vec![],
            }),
        );
    }

    // Get the schema tree from the service
    let tree = state.schema_service.get_tree().await;

    // Build the response
    let response = SchemaResponse {
        version: tree.version.clone(),
        categories: tree.categories,
        objects: tree.objects,
    };

    // Build caching headers
    let mut headers = HeaderMap::new();

    // Cache-Control: Allow browser caching for 1 hour
    // Using public since schema data is not user-specific
    headers.insert(
        header::CACHE_CONTROL,
        format!("public, max-age={}", SCHEMA_CACHE_MAX_AGE)
            .parse()
            .unwrap(),
    );

    // ETag: Based on schema version for conditional requests
    // This allows browsers to validate cached responses
    let etag = format!("\"ocsf-schema-{}\"", response.version);
    headers.insert(header::ETAG, etag.parse().unwrap());

    (StatusCode::OK, headers, Json(response))
}

// ============================================================================
// Validation Endpoints (Requirement 7.2)
// ============================================================================

/// Request body for POST /api/validate endpoint.
///
/// The model field accepts a SemanticModel JSON structure.
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    /// The semantic model to validate.
    pub model: SemanticModel,
}

/// Validation error details for API response.
///
/// This structure matches the design specification:
/// ```typescript
/// interface ValidationError {
///   path: string;
///   message: string;
///   code: string;
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ValidationError {
    /// Path to the error location (e.g., "entities[0].attributes[2].ocsf_mapping.field").
    pub path: String,
    /// Human-readable error message.
    pub message: String,
    /// Error code for programmatic handling.
    pub code: String,
}

/// Validation warning details for API response.
#[derive(Debug, Serialize)]
pub struct ValidationWarning {
    /// Path to the warning location.
    pub path: String,
    /// Human-readable warning message.
    pub message: String,
    /// Warning code for programmatic handling.
    pub code: String,
}

/// Response for POST /api/validate endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface ValidateResponse {
///   valid: boolean;
///   errors: ValidationError[];
///   warnings: ValidationWarning[];
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    /// Whether the model is valid (no errors).
    pub valid: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationWarning>,
}

impl From<ValidationReport> for ValidateResponse {
    fn from(report: ValidationReport) -> Self {
        Self {
            valid: report.valid,
            errors: report
                .errors
                .into_iter()
                .map(|e| ValidationError {
                    path: e.path,
                    message: e.message,
                    code: format!("{:?}", e.code),
                })
                .collect(),
            warnings: report
                .warnings
                .into_iter()
                .map(|w| ValidationWarning {
                    path: w.path,
                    message: w.message,
                    code: w.code,
                })
                .collect(),
        }
    }
}

/// POST /api/validate - Validates a semantic model against the schema.
///
/// Accepts a SemanticModel JSON body and validates it against the loaded
/// OCSF schema. Returns validation errors and warnings with path information
/// for error navigation.
///
/// # Requirements
/// - 7.2: THE API_Server SHALL expose a POST /api/validate endpoint that
///   validates a semantic model against the schema
///
/// # Request Body
/// ```json
/// {
///   "model": {
///     "name": "my-model",
///     "entities": [...],
///     "metrics": [...]
///   }
/// }
/// ```
///
/// # Response
/// - 200 OK with ValidateResponse JSON body containing validation results
/// - 503 Service Unavailable if no schema is loaded
///
/// # Property 19: Validation API Response Format
/// For any semantic model submitted to POST /api/validate, the response SHALL
/// contain a `valid` boolean and arrays of `errors` and `warnings` with path
/// and message fields.
pub async fn validate_model(
    State(state): State<AppState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, EditorApiError> {
    // Check if schema is loaded
    if !state.schema_service.is_loaded().await {
        return Err(EditorApiError::SchemaNotLoaded);
    }

    // Validate the model using the ValidationService
    let report = state.validation_service.validate(&request.model).await;

    // Convert the report to API response format
    Ok(Json(ValidateResponse::from(report)))
}

// ============================================================================
// Generation Endpoints (Requirement 7.3)
// ============================================================================

/// Request body for POST /api/generate endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface GenerateRequest {
///   model: SemanticModel;
///   dialect: 'snowflake' | 'databricks' | 'bigquery';
///   artifacts: ('dbt' | 'cubejs' | 'views' | 'etl')[];
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    /// The semantic model to generate artifacts from.
    pub model: SemanticModel,
    /// The warehouse dialect to generate for.
    pub dialect: String,
    /// The artifact types to generate.
    pub artifacts: Vec<String>,
}

/// Generated file information.
///
/// This structure matches the design specification:
/// ```typescript
/// interface GeneratedFile {
///   path: string;
///   content: string;
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct GeneratedFile {
    /// The file path (relative to output directory).
    pub path: String,
    /// The file content.
    pub content: String,
}

/// Response for POST /api/generate endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface GenerateResponse {
///   files: GeneratedFile[];
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    /// Generated files.
    pub files: Vec<GeneratedFile>,
}

/// Supported warehouse dialects.
fn parse_dialect(dialect: &str) -> Result<ocsf_semantic::WarehouseDialect, EditorApiError> {
    match dialect.to_lowercase().as_str() {
        "snowflake" => Ok(ocsf_semantic::WarehouseDialect::Snowflake),
        "databricks" => Ok(ocsf_semantic::WarehouseDialect::Databricks),
        "bigquery" => Ok(ocsf_semantic::WarehouseDialect::BigQuery),
        "postgres" => Ok(ocsf_semantic::WarehouseDialect::Postgres),
        _ => Err(EditorApiError::InvalidRequest(format!(
            "Invalid dialect: {}. Must be one of: snowflake, databricks, bigquery, postgres",
            dialect
        ))),
    }
}

/// POST /api/generate - Generates warehouse artifacts from a model.
///
/// Accepts a SemanticModel, dialect, and artifact types, then generates
/// the requested warehouse artifacts using the ocsf-warehouse crate.
///
/// # Requirements
/// - 7.3: THE API_Server SHALL expose a POST /api/generate endpoint that
///   generates warehouse artifacts from a model
///
/// # Request Body
/// ```json
/// {
///   "model": { ... },
///   "dialect": "snowflake",
///   "artifacts": ["dbt", "cubejs", "views", "etl"]
/// }
/// ```
///
/// # Response
/// - 200 OK with GenerateResponse JSON body containing generated files
/// - 400 Bad Request if dialect or artifacts are invalid
///
/// # Property 20: Generation API Artifact Output
/// For any valid semantic model submitted to POST /api/generate with specified
/// dialect and artifacts, the response SHALL contain generated files for each
/// requested artifact type.
pub async fn generate_artifacts(
    State(_state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, EditorApiError> {
    // Parse the dialect
    let dialect = parse_dialect(&request.dialect)?;

    // Validate artifact types
    let valid_artifacts = ["dbt", "cubejs", "views", "etl"];
    for artifact in &request.artifacts {
        if !valid_artifacts.contains(&artifact.to_lowercase().as_str()) {
            return Err(EditorApiError::InvalidRequest(format!(
                "Invalid artifact type: {}. Must be one of: dbt, cubejs, views, etl",
                artifact
            )));
        }
    }

    if request.artifacts.is_empty() {
        return Err(EditorApiError::InvalidRequest(
            "At least one artifact type must be specified".to_string(),
        ));
    }

    let mut files = Vec::new();

    // Generate requested artifacts
    for artifact in &request.artifacts {
        match artifact.to_lowercase().as_str() {
            "dbt" => {
                let dbt_files = generate_dbt_artifacts(&request.model, dialect);
                files.extend(dbt_files);
            }
            "cubejs" => {
                let cube_files = generate_cubejs_artifacts(&request.model);
                files.extend(cube_files);
            }
            "views" => {
                let view_files = generate_view_artifacts(&request.model, dialect);
                files.extend(view_files);
            }
            "etl" => {
                let etl_files = generate_etl_artifacts(&request.model, dialect);
                files.extend(etl_files);
            }
            _ => {} // Already validated above
        }
    }

    Ok(Json(GenerateResponse { files }))
}

/// Generate dbt artifacts from a semantic model.
fn generate_dbt_artifacts(
    model: &SemanticModel,
    dialect: ocsf_semantic::WarehouseDialect,
) -> Vec<GeneratedFile> {
    use ocsf_warehouse::DBTGenerator;

    let generator = DBTGenerator::new(dialect);
    let artifacts = generator.generate(model);

    let mut files = Vec::new();

    // Add semantic manifest
    if !artifacts.semantic_manifest.is_empty() {
        files.push(GeneratedFile {
            path: "models/semantic_manifest.yml".to_string(),
            content: artifacts.semantic_manifest,
        });
    }

    // Add sources
    if !artifacts.sources.is_empty() {
        files.push(GeneratedFile {
            path: "models/sources.yml".to_string(),
            content: artifacts.sources,
        });
    }

    // Add model SQL files
    for (model_name, sql) in artifacts.models {
        files.push(GeneratedFile {
            path: format!("models/{}.sql", model_name),
            content: sql,
        });
    }

    files
}

/// Generate Cube.js artifacts from a semantic model.
fn generate_cubejs_artifacts(model: &SemanticModel) -> Vec<GeneratedFile> {
    use ocsf_warehouse::CubeGenerator;

    let generator = CubeGenerator::new();
    let artifacts = generator.generate(model);

    let mut files = Vec::new();

    // Add cube definition files
    for (cube_name, content) in artifacts.cubes {
        files.push(GeneratedFile {
            path: format!("schema/{}.js", cube_name),
            content,
        });
    }

    files
}

/// Generate SQL view artifacts from a semantic model.
fn generate_view_artifacts(
    model: &SemanticModel,
    dialect: ocsf_semantic::WarehouseDialect,
) -> Vec<GeneratedFile> {
    use ocsf_warehouse::ViewGenerator;

    let generator = ViewGenerator::new(dialect);
    let views = generator.generate(model);

    let mut files = Vec::new();

    // Add view SQL files
    for (view_name, sql) in views.views {
        files.push(GeneratedFile {
            path: format!("views/{}.sql", view_name),
            content: sql,
        });
    }

    files
}

/// Generate ETL artifacts from a semantic model.
fn generate_etl_artifacts(
    model: &SemanticModel,
    dialect: ocsf_semantic::WarehouseDialect,
) -> Vec<GeneratedFile> {
    use ocsf_warehouse::{ETLConfig, ETLGenerator};

    // Configure ETL based on model's observable config
    let config = ETLConfig {
        source_table: "ocsf_events".to_string(),
        target_table: model.observable_config.table_name.clone(),
        observable_types: if model.observable_config.include_types.is_empty() {
            vec![2, 5, 10, 22, 30] // Default observable types
        } else {
            model.observable_config.include_types.clone()
        },
        batch_mode: false,
        batch_size: 10000,
    };

    let generator = ETLGenerator::new(dialect).with_config(config);
    let etl = generator.generate();

    vec![GeneratedFile {
        path: "etl/observable_extraction.sql".to_string(),
        content: etl.to_sql(),
    }]
}

// ============================================================================
// LLM Research Endpoints (Requirement 7.4)
// ============================================================================

/// Request body for POST /api/llm/research endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface ResearchRequest {
///   target_type: 'entity' | 'attribute';
///   entity_name: string;
///   attribute_name?: string;
///   ocsf_context: OCSFContext;
///   requested_fields: ('description' | 'synonyms' | 'security_context')[];
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct ResearchRequest {
    /// Type of target: "entity" or "attribute".
    pub target_type: String,
    /// Name of the entity.
    pub entity_name: String,
    /// Name of the attribute (for attribute research).
    pub attribute_name: Option<String>,
    /// OCSF context for the research.
    pub ocsf_context: crate::llm_service::OCSFContext,
    /// Fields to research.
    pub requested_fields: Vec<String>,
    /// Whether the attribute is an observable (for attribute research).
    #[serde(default)]
    pub is_observable: bool,
}

/// Batch research request for multiple targets.
#[derive(Debug, Deserialize)]
pub struct BatchResearchRequest {
    /// List of research targets.
    pub targets: Vec<ResearchTargetRequest>,
    /// Shared context to include in all research prompts.
    /// Can include detection rules, documentation, or other relevant information.
    #[serde(default)]
    pub shared_context: Option<String>,
}

/// A single target in a batch research request.
#[derive(Debug, Deserialize)]
pub struct ResearchTargetRequest {
    /// Type of target: "entity" or "attribute".
    pub target_type: String,
    /// Name of the entity.
    pub entity_name: String,
    /// Name of the attribute (for attribute research).
    pub attribute_name: Option<String>,
    /// OCSF context for the research.
    pub ocsf_context: crate::llm_service::OCSFContext,
    /// Fields to research.
    pub requested_fields: Vec<String>,
    /// Whether the attribute is an observable (for attribute research).
    #[serde(default)]
    pub is_observable: bool,
}

/// LLM suggestion in API response.
#[derive(Debug, Serialize)]
pub struct LLMSuggestion {
    /// Unique identifier for the suggestion.
    pub id: String,
    /// Field this suggestion applies to.
    pub field: String,
    /// Suggested value.
    pub value: serde_json::Value,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Response for POST /api/llm/research endpoint.
///
/// This structure matches the design specification:
/// ```typescript
/// interface ResearchResponse {
///   suggestions: LLMSuggestion[];
///   tokens_used: number;
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ResearchResponse {
    /// Generated suggestions.
    pub suggestions: Vec<LLMSuggestion>,
    /// Number of tokens used.
    pub tokens_used: u32,
}

impl From<crate::llm_service::LLMSuggestion> for LLMSuggestion {
    fn from(s: crate::llm_service::LLMSuggestion) -> Self {
        Self {
            id: s.id,
            field: format!("{:?}", s.field).to_lowercase(),
            value: s.value.to_json(),
            confidence: s.confidence,
        }
    }
}

/// POST /api/llm/research - Queries the LLM for semantic enrichments.
///
/// Accepts a ResearchRequest with target type and context, and returns
/// suggestions with confidence scores. Supports both single target and
/// batch research operations.
///
/// # Requirements
/// - 7.4: THE API_Server SHALL expose a POST /api/llm/research endpoint that
///   queries the LLM for semantic enrichments
/// - 4.7: Support batch research for multiple targets
///
/// # Property 9: LLM Research Context Inclusion
/// For any entity research request, the LLM API call SHALL include OCSF class
/// metadata (name, description, category) for all source event classes.
///
/// # Property 12: Batch Research Single Request
/// For any batch research operation with N targets, exactly one API request
/// SHALL be made containing all N targets, rather than N separate requests.
pub async fn llm_research(
    State(state): State<AppState>,
    Json(request): Json<ResearchRequest>,
) -> Result<Json<ResearchResponse>, EditorApiError> {
    // Check if LLM service is configured
    let llm_guard = state.llm_service.read().await;
    let llm_service = llm_guard.as_ref().ok_or_else(|| {
        EditorApiError::LLMError("LLM service not configured. Go to Settings to add your API key.".to_string())
    })?;

    // Parse requested fields
    let requested_fields: Vec<crate::llm_service::ResearchField> = request
        .requested_fields
        .iter()
        .filter_map(|f| crate::llm_service::ResearchField::parse(f))
        .collect();

    if requested_fields.is_empty() {
        return Err(EditorApiError::InvalidRequest(
            "No valid requested_fields provided".to_string(),
        ));
    }

    // Perform research based on target type
    let result = match request.target_type.to_lowercase().as_str() {
        "entity" => {
            llm_service
                .research_entity(&request.entity_name, &request.ocsf_context, &requested_fields)
                .await?
        }
        "attribute" => {
            let attr_name = request.attribute_name.as_deref().ok_or_else(|| {
                EditorApiError::InvalidRequest(
                    "attribute_name required for attribute research".to_string(),
                )
            })?;
            llm_service
                .research_attribute(
                    &request.entity_name,
                    attr_name,
                    &request.ocsf_context,
                    &requested_fields,
                    request.is_observable,
                )
                .await?
        }
        _ => {
            return Err(EditorApiError::InvalidRequest(format!(
                "Invalid target_type: {}. Must be 'entity' or 'attribute'",
                request.target_type
            )));
        }
    };

    // Convert to API response format
    let response = ResearchResponse {
        suggestions: result.suggestions.into_iter().map(Into::into).collect(),
        tokens_used: result.tokens_used,
    };

    Ok(Json(response))
}

/// POST /api/llm/research/batch - Batch research for multiple targets.
///
/// # Requirements
/// - 4.7: Support batch research for multiple targets
///
/// # Property 12: Batch Research Single Request
/// For any batch research operation with N targets, exactly one API request
/// SHALL be made containing all N targets, rather than N separate requests.
pub async fn llm_research_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchResearchRequest>,
) -> Result<Json<Vec<ResearchResponse>>, EditorApiError> {
    // Check if LLM service is configured
    let llm_guard = state.llm_service.read().await;
    let llm_service = llm_guard.as_ref().ok_or_else(|| {
        EditorApiError::LLMError("LLM service not configured. Go to Settings to add your API key.".to_string())
    })?;

    if request.targets.is_empty() {
        return Ok(Json(vec![]));
    }

    // Convert request targets to internal format
    let targets: Vec<crate::llm_service::ResearchTarget> = request
        .targets
        .into_iter()
        .filter_map(|t| {
            let target_type = match t.target_type.to_lowercase().as_str() {
                "entity" => crate::llm_service::ResearchTargetType::Entity,
                "attribute" => crate::llm_service::ResearchTargetType::Attribute,
                _ => return None,
            };

            let requested_fields: Vec<crate::llm_service::ResearchField> = t
                .requested_fields
                .iter()
                .filter_map(|f| crate::llm_service::ResearchField::parse(f))
                .collect();

            if requested_fields.is_empty() {
                return None;
            }

            Some(crate::llm_service::ResearchTarget {
                target_type,
                entity_name: t.entity_name,
                attribute_name: t.attribute_name,
                ocsf_context: t.ocsf_context,
                requested_fields,
            })
        })
        .collect();

    if targets.is_empty() {
        return Err(EditorApiError::InvalidRequest(
            "No valid targets provided".to_string(),
        ));
    }

    // Perform batch research (single API call for all targets)
    let results = llm_service.batch_research(targets, request.shared_context).await?;

    // Convert to API response format
    let responses: Vec<ResearchResponse> = results
        .into_iter()
        .map(|r| ResearchResponse {
            suggestions: r.suggestions.into_iter().map(Into::into).collect(),
            tokens_used: r.tokens_used,
        })
        .collect();

    Ok(Json(responses))
}

// ============================================================================
// LLM Configuration Endpoints
// ============================================================================

/// Response for GET /api/llm/config endpoint.
#[derive(Debug, Serialize)]
pub struct LLMConfigResponse {
    /// Whether the LLM service is configured.
    pub configured: bool,
    /// The current provider (if configured).
    pub provider: Option<String>,
}

/// Request body for POST /api/llm/config endpoint.
#[derive(Debug, Deserialize)]
pub struct LLMConfigRequest {
    /// The LLM provider: "anthropic" or "openai".
    pub provider: String,
    /// The API key for the provider.
    pub api_key: String,
}

/// GET /api/llm/config - Get the current LLM configuration status.
pub async fn get_llm_config(
    State(state): State<AppState>,
) -> Json<LLMConfigResponse> {
    let configured = state.is_llm_configured().await;
    let provider = state.llm_provider_name().await;
    
    Json(LLMConfigResponse { configured, provider })
}

/// POST /api/llm/config - Configure the LLM service with an API key.
pub async fn set_llm_config(
    State(state): State<AppState>,
    Json(request): Json<LLMConfigRequest>,
) -> Result<Json<LLMConfigResponse>, EditorApiError> {
    // Validate the API key is not empty
    if request.api_key.trim().is_empty() {
        return Err(EditorApiError::InvalidRequest("API key cannot be empty".to_string()));
    }
    
    // Configure the LLM service
    state.configure_llm(&request.provider, request.api_key).await
        .map_err(EditorApiError::InvalidRequest)?;
    
    // Return the new configuration status
    let configured = state.is_llm_configured().await;
    let provider = state.llm_provider_name().await;
    
    Ok(Json(LLMConfigResponse { configured, provider }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SchemaService;
    use axum::body::Body;
    use axum::response::Response;
    use http_body_util::BodyExt;
    use std::sync::Arc;

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
                            "type": "string_t",
                            "type_name": "String",
                            "requirement": "recommended"
                        }
                    }
                }
            },
            "profiles": {},
            "extensions": {}
        }"#
    }

    /// Helper to convert IntoResponse to Response for testing.
    async fn to_response(resp: impl IntoResponse) -> Response<Body> {
        resp.into_response()
    }

    /// Helper to extract body as string from response.
    async fn body_string(response: Response<Body>) -> String {
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(body_bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.status, "ok");
        assert!(!response.version.is_empty());
    }

    #[tokio::test]
    async fn test_get_schema_returns_503_when_no_schema_loaded() {
        // Create state without loading a schema
        let state = AppState::new().await;

        let response = to_response(get_schema(State(state)).await).await;

        // Should return 503 Service Unavailable when no schema is loaded
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_schema_returns_schema_tree() {
        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        let response = to_response(get_schema(State(state)).await).await;

        // Verify status is OK
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response body
        let body_str = body_string(response).await;
        let body: SchemaResponse = serde_json::from_str(&body_str).unwrap();

        // Verify response body
        assert_eq!(body.version, "1.4.0");
        assert_eq!(body.categories.len(), 1);
        assert_eq!(body.objects.len(), 1);

        // Verify category structure
        let iam = &body.categories[0];
        assert_eq!(iam.name, "iam");
        assert_eq!(iam.uid, 3);
        assert_eq!(iam.classes.len(), 1);

        // Verify class structure
        let auth_class = &iam.classes[0];
        assert_eq!(auth_class.name, "authentication");
        assert_eq!(auth_class.uid, 3002);
        assert!(!auth_class.attributes.is_empty());

        // Verify object structure
        let actor = &body.objects[0];
        assert_eq!(actor.name, "actor");
        assert!(!actor.attributes.is_empty());
    }

    #[tokio::test]
    async fn test_get_schema_includes_cache_headers() {
        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        let response = to_response(get_schema(State(state)).await).await;
        let headers = response.headers();

        // Verify Cache-Control header
        let cache_control = headers.get(header::CACHE_CONTROL);
        assert!(
            cache_control.is_some(),
            "Cache-Control header should be present"
        );
        let cache_value = cache_control.unwrap().to_str().unwrap();
        assert!(
            cache_value.contains("public"),
            "Cache-Control should include 'public'"
        );
        assert!(
            cache_value.contains("max-age=3600"),
            "Cache-Control should include 'max-age=3600'"
        );

        // Verify ETag header
        let etag = headers.get(header::ETAG);
        assert!(etag.is_some(), "ETag header should be present");
        let etag_value = etag.unwrap().to_str().unwrap();
        assert!(
            etag_value.contains("ocsf-schema-1.4.0"),
            "ETag should contain schema version"
        );
    }

    #[tokio::test]
    async fn test_get_schema_includes_attributes_with_metadata() {
        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        let response = to_response(get_schema(State(state)).await).await;
        let body_str = body_string(response).await;
        let body: SchemaResponse = serde_json::from_str(&body_str).unwrap();

        // Find the authentication class
        let iam = &body.categories[0];
        let auth_class = &iam.classes[0];

        // Find activity_id attribute
        let activity_id = auth_class
            .attributes
            .iter()
            .find(|a| a.name == "activity_id")
            .expect("activity_id attribute should exist");

        // Verify attribute metadata
        assert_eq!(activity_id.attr_type, "integer_t");
        assert_eq!(activity_id.type_name, "Integer");
        assert_eq!(activity_id.requirement, "required");
        assert!(!activity_id.enum_values.is_empty());

        // Verify enum values
        let unknown_enum = activity_id
            .enum_values
            .iter()
            .find(|e| e.key == "0")
            .expect("enum value 0 should exist");
        assert_eq!(unknown_enum.caption, "Unknown");
    }

    // ========================================================================
    // Validation Endpoint Tests (Requirement 7.2)
    // ========================================================================

    #[tokio::test]
    async fn test_validate_model_returns_503_when_no_schema_loaded() {
        // Create state without loading a schema
        let state = AppState::new().await;

        let request = ValidateRequest {
            model: SemanticModel::new("test-model"),
        };

        let result = validate_model(State(state), Json(request)).await;

        // Should return error when no schema is loaded
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_model_valid_model() {
        use ocsf_semantic::entity::{SemanticAttribute, SemanticEntity, SemanticType};

        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        // Create a valid model with valid field mapping
        let model = SemanticModel::new("test-model").add_entity(
            SemanticEntity::new("auth_event")
                .with_source_event_classes(vec![3002])
                .add_attribute(
                    SemanticAttribute::new("activity")
                        .with_type(SemanticType::Integer)
                        .with_field_mapping("activity_id"),
                ),
        );

        let request = ValidateRequest { model };

        let result = validate_model(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.valid);
        assert!(response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_model_invalid_field_path() {
        use ocsf_semantic::entity::{SemanticAttribute, SemanticEntity, SemanticType};

        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        // Create a model with invalid field mapping
        let model = SemanticModel::new("test-model").add_entity(
            SemanticEntity::new("auth_event")
                .with_source_event_classes(vec![3002])
                .add_attribute(
                    SemanticAttribute::new("bad_attr")
                        .with_type(SemanticType::String)
                        .with_field_mapping("nonexistent.path"),
                ),
        );

        let request = ValidateRequest { model };


        let result = validate_model(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.valid);
        assert!(!response.errors.is_empty());

        // Verify error has path information for navigation
        let error = &response.errors[0];
        assert!(error.path.contains("entities[0]"));
        assert!(error.path.contains("ocsf_mapping.field"));
        assert!(!error.message.is_empty());
        assert!(!error.code.is_empty());
    }

    #[tokio::test]
    async fn test_validate_model_invalid_event_class() {
        use ocsf_semantic::entity::SemanticEntity;

        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        // Create a model with invalid event class
        let model = SemanticModel::new("test-model").add_entity(
            SemanticEntity::new("auth_event").with_source_event_classes(vec![99999]), // Invalid class UID
        );

        let request = ValidateRequest { model };

        let result = validate_model(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.valid);
        assert!(!response.errors.is_empty());

        // Verify error indicates event class not found
        let error = &response.errors[0];
        assert!(error.path.contains("source_event_classes"));
        assert!(error.code.contains("EventClassNotFound"));
    }

    #[tokio::test]
    async fn test_validate_model_invalid_dimension_reference() {
        use ocsf_semantic::entity::{SemanticAttribute, SemanticEntity, SemanticType};
        use ocsf_semantic::metric::{Aggregation, SemanticMetric};

        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        // Create a model with metric referencing non-existent dimension
        let model = SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("auth_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("activity")
                            .with_type(SemanticType::Integer)
                            .with_field_mapping("activity_id")
                            .as_dimension(),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_count")
                    .with_aggregation(Aggregation::Count)
                    .with_dimensions(vec!["nonexistent_dimension".to_string()]),
            );

        let request = ValidateRequest { model };

        let result = validate_model(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.valid);
        assert!(!response.errors.is_empty());

        // Verify error indicates dimension not found
        let error = &response.errors[0];
        assert!(error.path.contains("dimensions"));
        assert!(error.code.contains("DimensionNotFound"));
    }

    #[tokio::test]
    async fn test_validate_response_format() {
        // Property 19: Validation API Response Format
        // For any semantic model submitted to POST /api/validate, the response SHALL
        // contain a `valid` boolean and arrays of `errors` and `warnings` with path
        // and message fields.

        // Create schema service and load test schema
        let schema_service = Arc::new(SchemaService::new());
        schema_service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let state = AppState::with_schema_service(schema_service).await;

        let model = SemanticModel::new("test-model");
        let request = ValidateRequest { model };

        let result = validate_model(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify response has required fields
        // valid is a boolean
        let _ = response.valid;
        // errors is an array
        let _ = response.errors.len();
        // warnings is an array
        let _ = response.warnings.len();

        // Serialize to JSON to verify format
        let json = serde_json::to_value(&*response).unwrap();
        assert!(json.get("valid").is_some());
        assert!(json.get("errors").is_some());
        assert!(json.get("warnings").is_some());
        assert!(json["errors"].is_array());
        assert!(json["warnings"].is_array());
    }

    // ========================================================================
    // LLM Research Endpoint Tests (Requirement 7.4)
    // ========================================================================

    #[tokio::test]
    async fn test_llm_research_returns_error_when_not_configured() {
        // Create state without LLM service configured
        let schema_service = Arc::new(SchemaService::new());
        let state = AppState::with_services(schema_service, None).await;

        let request = ResearchRequest {
            target_type: "entity".to_string(),
            entity_name: "auth_event".to_string(),
            attribute_name: None,
            ocsf_context: crate::llm_service::OCSFContext::default(),
            requested_fields: vec!["description".to_string()],
            is_observable: false,
        };

        let result = llm_research(State(state), Json(request)).await;

        // Should return error when LLM service is not configured
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::LLMError(_)));
    }

    #[tokio::test]
    async fn test_llm_research_invalid_target_type() {
        // Create state with mock LLM service
        let schema_service = Arc::new(SchemaService::new());
        let llm_service = crate::LLMService::new("test-key");
        let state = AppState::with_services(schema_service, Some(llm_service)).await;

        let request = ResearchRequest {
            target_type: "invalid".to_string(),
            entity_name: "auth_event".to_string(),
            attribute_name: None,
            ocsf_context: crate::llm_service::OCSFContext::default(),
            requested_fields: vec!["description".to_string()],
            is_observable: false,
        };

        let result = llm_research(State(state), Json(request)).await;

        // Should return error for invalid target type
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_llm_research_attribute_missing_name() {
        // Create state with mock LLM service
        let schema_service = Arc::new(SchemaService::new());
        let llm_service = crate::LLMService::new("test-key");
        let state = AppState::with_services(schema_service, Some(llm_service)).await;

        let request = ResearchRequest {
            target_type: "attribute".to_string(),
            entity_name: "auth_event".to_string(),
            attribute_name: None, // Missing attribute name
            ocsf_context: crate::llm_service::OCSFContext::default(),
            requested_fields: vec!["description".to_string()],
            is_observable: false,
        };

        let result = llm_research(State(state), Json(request)).await;

        // Should return error when attribute_name is missing for attribute research
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_llm_research_no_valid_fields() {
        // Create state with mock LLM service
        let schema_service = Arc::new(SchemaService::new());
        let llm_service = crate::LLMService::new("test-key");
        let state = AppState::with_services(schema_service, Some(llm_service)).await;

        let request = ResearchRequest {
            target_type: "entity".to_string(),
            entity_name: "auth_event".to_string(),
            attribute_name: None,
            ocsf_context: crate::llm_service::OCSFContext::default(),
            requested_fields: vec!["invalid_field".to_string()],
            is_observable: false,
        };

        let result = llm_research(State(state), Json(request)).await;

        // Should return error when no valid fields are provided
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_llm_research_batch_empty_targets() {
        // Create state with mock LLM service
        let schema_service = Arc::new(SchemaService::new());
        let llm_service = crate::LLMService::new("test-key");
        let state = AppState::with_services(schema_service, Some(llm_service)).await;

        let request = BatchResearchRequest { 
            targets: vec![],
            shared_context: None,
        };

        let result = llm_research_batch(State(state), Json(request)).await;

        // Should return empty array for empty targets
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_research_batch_no_valid_targets() {
        // Create state with mock LLM service
        let schema_service = Arc::new(SchemaService::new());
        let llm_service = crate::LLMService::new("test-key");
        let state = AppState::with_services(schema_service, Some(llm_service)).await;

        let request = BatchResearchRequest {
            targets: vec![ResearchTargetRequest {
                target_type: "invalid".to_string(),
                entity_name: "auth_event".to_string(),
                attribute_name: None,
                ocsf_context: crate::llm_service::OCSFContext::default(),
                requested_fields: vec!["description".to_string()],
                is_observable: false,
            }],
            shared_context: None,
        };

        let result = llm_research_batch(State(state), Json(request)).await;

        // Should return error when no valid targets are provided
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[test]
    fn test_research_response_serialization() {
        let response = ResearchResponse {
            suggestions: vec![LLMSuggestion {
                id: "test-id".to_string(),
                field: "description".to_string(),
                value: serde_json::Value::String("Test description".to_string()),
                confidence: 0.85,
            }],
            tokens_used: 100,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("suggestions").is_some());
        assert!(json.get("tokens_used").is_some());
        assert_eq!(json["tokens_used"], 100);
        assert!(json["suggestions"].is_array());
        assert_eq!(json["suggestions"].as_array().unwrap().len(), 1);
    }

    // ========================================================================
    // Generation Endpoint Tests (Requirement 7.3)
    // ========================================================================

    #[tokio::test]
    async fn test_generate_artifacts_dbt() {
        use ocsf_semantic::entity::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .with_ocsf_version("1.4.0")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_description("Authentication events")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr"))
                            .as_dimension(),
                    ),
            );

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec!["dbt".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.files.is_empty());

        // Verify dbt files are generated
        let has_manifest = response.files.iter().any(|f| f.path.contains("semantic_manifest"));
        let has_sources = response.files.iter().any(|f| f.path.contains("sources"));
        let has_model = response.files.iter().any(|f| f.path.ends_with(".sql"));

        assert!(has_manifest, "Should generate semantic_manifest.yml");
        assert!(has_sources, "Should generate sources.yml");
        assert!(has_model, "Should generate model SQL files");
    }

    #[tokio::test]
    async fn test_generate_artifacts_cubejs() {
        use ocsf_semantic::entity::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_caption("Authentication Event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr")),
                    ),
            );

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec!["cubejs".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.files.is_empty());

        // Verify cube.js files are generated
        let has_cube = response.files.iter().any(|f| f.path.contains("schema/") && f.path.ends_with(".js"));
        assert!(has_cube, "Should generate Cube.js schema files");

        // Verify cube content
        let cube_file = response.files.iter().find(|f| f.path.ends_with(".js")).unwrap();
        assert!(cube_file.content.contains("cube("));
        assert!(cube_file.content.contains("sql:"));
    }

    #[tokio::test]
    async fn test_generate_artifacts_views() {
        use ocsf_semantic::entity::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr")),
                    ),
            );

        let request = GenerateRequest {
            model,
            dialect: "postgres".to_string(),
            artifacts: vec!["views".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.files.is_empty());

        // Verify view files are generated
        let has_view = response.files.iter().any(|f| f.path.contains("views/") && f.path.ends_with(".sql"));
        assert!(has_view, "Should generate SQL view files");

        // Verify view content
        let view_file = response.files.iter().find(|f| f.path.ends_with(".sql")).unwrap();
        assert!(view_file.content.contains("CREATE"));
        assert!(view_file.content.contains("VIEW"));
        assert!(view_file.content.contains("SELECT"));
    }


    #[tokio::test]
    async fn test_generate_artifacts_etl() {
        use ocsf_semantic::model::ObservableConfig;

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5],
            });

        let request = GenerateRequest {
            model,
            dialect: "bigquery".to_string(),
            artifacts: vec!["etl".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.files.is_empty());

        // Verify ETL files are generated
        let has_etl = response.files.iter().any(|f| f.path.contains("etl/"));
        assert!(has_etl, "Should generate ETL files");

        // Verify ETL content
        let etl_file = response.files.iter().find(|f| f.path.contains("etl/")).unwrap();
        assert!(etl_file.content.contains("INSERT INTO"));
        assert!(etl_file.content.contains("observable"));
    }

    #[tokio::test]
    async fn test_generate_artifacts_multiple() {
        use ocsf_semantic::entity::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr")),
                    ),
            );

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec!["dbt".to_string(), "cubejs".to_string(), "views".to_string(), "etl".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify files from all artifact types are generated
        let has_dbt = response.files.iter().any(|f| f.path.contains("models/"));
        let has_cube = response.files.iter().any(|f| f.path.contains("schema/"));
        let has_views = response.files.iter().any(|f| f.path.contains("views/"));
        let has_etl = response.files.iter().any(|f| f.path.contains("etl/"));

        assert!(has_dbt, "Should generate dbt files");
        assert!(has_cube, "Should generate Cube.js files");
        assert!(has_views, "Should generate view files");
        assert!(has_etl, "Should generate ETL files");
    }

    #[tokio::test]
    async fn test_generate_artifacts_invalid_dialect() {
        let state = AppState::new().await;

        let model = SemanticModel::new("test-model");

        let request = GenerateRequest {
            model,
            dialect: "invalid_dialect".to_string(),
            artifacts: vec!["dbt".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_generate_artifacts_invalid_artifact_type() {
        let state = AppState::new().await;

        let model = SemanticModel::new("test-model");

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec!["invalid_artifact".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_generate_artifacts_empty_artifacts() {
        let state = AppState::new().await;

        let model = SemanticModel::new("test-model");

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec![],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EditorApiError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_generate_artifacts_all_dialects() {
        use ocsf_semantic::entity::SemanticEntity;

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .add_entity(SemanticEntity::new("test_entity").with_source_event_classes(vec![3002]));

        // Test all supported dialects
        for dialect in &["snowflake", "databricks", "bigquery", "postgres"] {
            let request = GenerateRequest {
                model: model.clone(),
                dialect: dialect.to_string(),
                artifacts: vec!["views".to_string()],
            };

            let result = generate_artifacts(State(state.clone()), Json(request)).await;
            assert!(result.is_ok(), "Should support {} dialect", dialect);
        }
    }

    #[tokio::test]
    async fn test_generate_response_format() {
        // Property 20: Generation API Artifact Output
        // For any valid semantic model submitted to POST /api/generate with specified
        // dialect and artifacts, the response SHALL contain generated files for each
        // requested artifact type.

        use ocsf_semantic::entity::{OCSFMapping, SemanticAttribute, SemanticEntity, SemanticType};

        let state = AppState::new().await;

        let model = SemanticModel::new("test-model")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr")),
                    ),
            );

        let request = GenerateRequest {
            model,
            dialect: "snowflake".to_string(),
            artifacts: vec!["dbt".to_string()],
        };

        let result = generate_artifacts(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Serialize to JSON to verify format
        let json = serde_json::to_value(&*response).unwrap();
        assert!(json.get("files").is_some());
        assert!(json["files"].is_array());

        // Verify each file has path and content
        for file in json["files"].as_array().unwrap() {
            assert!(file.get("path").is_some());
            assert!(file.get("content").is_some());
            assert!(file["path"].is_string());
            assert!(file["content"].is_string());
        }
    }
}
