//! Mapping interpretation API endpoint.
//!
//! This module provides the LLM-powered mapping interpretation endpoint
//! that accepts an OCSF event JSON and a mapping artifact in any format,
//! sends both to the configured LLM provider, and returns structured
//! field-to-field mappings with confidence scores.
//!
//! # Requirements
//! - 15.1: Accept mapping artifacts in any format alongside a reference event
//! - 15.2: Parse LLM response into normalized mapping structure
//! - 15.8: Request transformation logic and source system identification
//! - 15.9: Accept any mapping format (Logstash, Cribl, Python, dbt, YAML, etc.)
//! - 15.12: Proxy LLM calls through the Rust backend

use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::EditorApiError;
use crate::AppState;

// ============================================================================
// Types (Task 8.1)
// ============================================================================

/// Request body for POST /api/llm/interpret-mapping.
///
/// # Requirements
/// - 15.1, 15.2
#[derive(Debug, Serialize, Deserialize)]
pub struct InterpretMappingRequest {
    /// The transformed OCSF event as a JSON string.
    pub event_json: String,
    /// The mapping artifact text in any format.
    pub mapping_text: String,
}

/// A single field-to-field mapping entry extracted by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    /// The source/raw field name from the original log.
    pub raw_field: String,
    /// The target OCSF field path (dot-notation).
    pub ocsf_field: String,
    /// Description of the transformation applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
    /// Plain-language explanation of the mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    /// Confidence level for this mapping entry.
    pub confidence: MappingConfidence,
}

/// Confidence level for an LLM-extracted mapping entry.
///
/// - High: Explicitly defined in the mapping artifact
/// - Medium: Inferred from naming conventions or patterns
/// - Low: LLM-generated guess based on field similarity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingConfidence {
    High,
    Medium,
    Low,
}

/// Source system metadata extracted by the LLM from the mapping artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSystem {
    /// The vendor or product name (e.g., "Palo Alto", "CrowdStrike").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// The type of log (e.g., "firewall", "endpoint", "dns").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,
    /// The format of the mapping artifact (e.g., "logstash", "cribl", "python").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Response from the mapping interpretation endpoint.
///
/// # Requirements
/// - 15.2: Normalized mapping structure with entries, source system, and issues
#[derive(Debug, Serialize, Deserialize)]
pub struct InterpretMappingResponse {
    /// Extracted field-to-field mapping entries.
    pub entries: Vec<MappingEntry>,
    /// Source system metadata identified by the LLM.
    pub source_system: SourceSystem,
    /// Issues or warnings identified during interpretation.
    pub issues: Vec<String>,
}

// ============================================================================
// Handler (Task 8.2)
// ============================================================================

/// Build the LLM prompt for mapping interpretation.
fn build_mapping_prompt(event_json: &str, mapping_text: &str) -> String {
    format!(
        r#"You are a security data engineering expert specializing in OCSF (Open Cybersecurity Schema Framework) field mappings.

You are given two inputs:
1. A transformed OCSF JSON event (the output of a mapping pipeline)
2. A mapping artifact (in any format) that describes how raw log fields were transformed into the OCSF event

Your task: Analyze the mapping artifact alongside the OCSF event and extract structured field-to-field mappings.

## OCSF Event (transformed output):
```json
{event_json}
```

## Mapping Artifact (any format — could be Logstash, Cribl, Python, dbt SQL, YAML, Sigma, JSON, or any other format):
```
{mapping_text}
```

## Instructions:
1. For each field mapping you can identify, extract:
   - `raw_field`: The source/raw field name from the original log
   - `ocsf_field`: The target OCSF field path (use dot-notation, e.g., "src_endpoint.ip")
   - `transformation`: Description of any transformation applied (e.g., "direct copy", "parsed from syslog header", "type cast to integer"). Use null if it's a direct copy with no transformation.
   - `explanation`: A plain-language explanation of this mapping entry. Use null if self-explanatory.
   - `confidence`: "high" if explicitly defined in the artifact, "medium" if inferred from patterns, "low" if guessed from field similarity

2. Identify the source system:
   - `vendor`: The vendor or product name if identifiable (e.g., "Palo Alto", "CrowdStrike")
   - `log_type`: The type of log (e.g., "firewall", "endpoint", "dns")
   - `format`: The format of the mapping artifact (e.g., "logstash", "cribl", "python", "dbt", "yaml")

3. List any issues found:
   - Fields in the mapping that have no OCSF equivalent
   - Lossy transformations or type coercions
   - Unmapped fields that appear in the event but not in the mapping

Respond with ONLY valid JSON in this exact structure (no markdown, no explanation outside the JSON):
{{
  "entries": [
    {{
      "raw_field": "src_ip",
      "ocsf_field": "src_endpoint.ip",
      "transformation": null,
      "explanation": null,
      "confidence": "high"
    }}
  ],
  "source_system": {{
    "vendor": null,
    "log_type": null,
    "format": null
  }},
  "issues": []
}}"#
    )
}

/// POST /api/llm/interpret-mapping — Interpret a mapping artifact via LLM.
///
/// Accepts an OCSF event JSON and a mapping artifact in any format,
/// sends both to the configured LLM provider, and returns structured
/// field-to-field mappings with confidence scores.
///
/// # Requirements
/// - 15.1: Accept mapping artifacts alongside reference events
/// - 15.2: Parse LLM response into normalized mapping structure
/// - 15.8: Request transformation logic and source system identification
/// - 15.9: Accept any mapping format
/// - 15.12: Proxy LLM calls through the Rust backend
///
/// # Errors
/// - 400 Bad Request if no LLM provider is configured
/// - 502 Bad Gateway on LLM communication or parsing errors
pub async fn interpret_mapping(
    State(state): State<AppState>,
    Json(req): Json<InterpretMappingRequest>,
) -> Result<Json<InterpretMappingResponse>, EditorApiError> {
    // Get read lock on LLM service and check if configured
    let llm_guard = state.llm_service.read().await;
    let llm_service = llm_guard.as_ref().ok_or_else(|| {
        EditorApiError::InvalidRequest(
            "LLM service not configured. Configure an LLM provider (Anthropic or OpenAI) in Settings to enable mapping interpretation.".to_string(),
        )
    })?;

    // Build the interpretation prompt
    let prompt = build_mapping_prompt(&req.event_json, &req.mapping_text);

    // Send to LLM
    let response_text = llm_service.send_prompt(&prompt).await.map_err(|e| {
        EditorApiError::LLMError(format!("LLM mapping interpretation failed: {}", e))
    })?;

    // Parse the LLM JSON response
    let json_value = llm_service.parse_json_response(&response_text).map_err(|e| {
        EditorApiError::LLMError(format!(
            "Failed to parse LLM response as JSON: {}",
            e
        ))
    })?;

    // Deserialize into our typed response
    let response: InterpretMappingResponse =
        serde_json::from_value(json_value).map_err(|e| {
            EditorApiError::LLMError(format!(
                "LLM response JSON does not match expected schema: {}",
                e
            ))
        })?;

    Ok(Json(response))
}
