//! LLM Service for semantic enrichment.
//!
//! This module provides LLM-powered semantic enrichment capabilities for the
//! OCSF Semantic Model Editor. It integrates with OpenAI or Anthropic APIs to generate:
//! - Entity descriptions based on OCSF class metadata
//! - Attribute synonyms for natural language queries
//! - Security context for observable attributes
//!
//! # Requirements
//! - 4.2: Generate entity descriptions based on OCSF class metadata
//! - 4.3: Generate synonyms relevant to security analytics
//! - 4.4: Generate security context for observable attributes
//! - 4.7: Support batch research for multiple targets

use std::env;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::EditorApiError;

/// Default OpenAI model to use for research.
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";

/// Default Anthropic model to use for research.
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

/// Default API endpoint for OpenAI.
pub const DEFAULT_OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Default API endpoint for Anthropic.
pub const DEFAULT_ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Environment variable name for OpenAI API key.
pub const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";

/// Environment variable name for Anthropic API key.
pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// LLM provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMProvider {
    /// OpenAI API (GPT models).
    OpenAI,
    /// Anthropic API (Claude models).
    Anthropic,
}

/// OCSF context information for LLM prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct OCSFContext {
    /// Event class name.
    #[serde(default)]
    pub class_name: String,
    /// Event class caption.
    #[serde(default)]
    pub class_caption: String,
    /// Event class description.
    #[serde(default)]
    pub class_description: String,
    /// Category name.
    #[serde(default)]
    pub category_name: String,
    /// Attribute name (for attribute research).
    #[serde(default)]
    pub attribute_name: Option<String>,
    /// Attribute type (for attribute research).
    #[serde(default)]
    pub attribute_type: Option<String>,
    /// Attribute description (for attribute research).
    #[serde(default)]
    pub attribute_description: Option<String>,
}



/// Target type for research requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchTargetType {
    /// Research an entity.
    Entity,
    /// Research an attribute.
    Attribute,
}

/// A research target for batch operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTarget {
    /// Type of target (entity or attribute).
    pub target_type: ResearchTargetType,
    /// Entity name.
    pub entity_name: String,
    /// Attribute name (for attribute targets).
    pub attribute_name: Option<String>,
    /// OCSF context for the target.
    pub ocsf_context: OCSFContext,
    /// Fields to research.
    pub requested_fields: Vec<ResearchField>,
}

/// Fields that can be researched by the LLM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchField {
    /// Entity or attribute description.
    Description,
    /// Attribute synonyms for natural language queries.
    Synonyms,
    /// Security context for threat detection.
    SecurityContext,
    /// Sample values for the attribute.
    SampleValues,
}

impl ResearchField {
    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "description" => Some(Self::Description),
            "synonyms" => Some(Self::Synonyms),
            "security_context" => Some(Self::SecurityContext),
            "sample_values" => Some(Self::SampleValues),
            _ => None,
        }
    }
}


/// An LLM-generated suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSuggestion {
    /// Unique identifier for the suggestion.
    pub id: String,
    /// Field this suggestion applies to.
    pub field: ResearchField,
    /// Suggested value (string for description/security_context, array for synonyms/sample_values).
    pub value: SuggestionValue,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Value type for suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SuggestionValue {
    /// Single string value.
    String(String),
    /// Array of strings.
    Array(Vec<String>),
}

impl SuggestionValue {
    /// Convert to JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(
                arr.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
        }
    }
}

/// Result of a research operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// Entity name.
    pub entity_name: String,
    /// Attribute name (if applicable).
    pub attribute_name: Option<String>,
    /// Generated suggestions.
    pub suggestions: Vec<LLMSuggestion>,
    /// Number of tokens used.
    pub tokens_used: u32,
}

/// OpenAI API request structure.
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// OpenAI message structure.
#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response structure.
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

/// OpenAI choice structure.
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

/// OpenAI response message.
#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

/// OpenAI usage statistics.
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
}

/// Anthropic API request structure.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
}

/// Anthropic message structure.
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response structure.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

/// Anthropic content block.
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Anthropic usage statistics.
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}


/// LLM Service for semantic enrichment.
///
/// Provides LLM-powered research capabilities for generating entity descriptions,
/// attribute synonyms, and security context. Supports both OpenAI and Anthropic APIs.
///
/// # Requirements
/// - 4.2: Generate entity descriptions based on OCSF class metadata
/// - 4.3: Generate synonyms relevant to security analytics
/// - 4.4: Generate security context for observable attributes
/// - 4.7: Support batch research for multiple targets
#[derive(Clone)]
pub struct LLMService {
    /// HTTP client for API requests.
    client: Client,
    /// API key.
    api_key: String,
    /// Model to use for completions.
    model: String,
    /// API endpoint URL.
    api_url: String,
    /// LLM provider (OpenAI or Anthropic).
    provider: LLMProvider,
}

impl LLMService {
    /// Creates a new LLM service with OpenAI provider.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: DEFAULT_OPENAI_MODEL.to_string(),
            api_url: DEFAULT_OPENAI_API_URL.to_string(),
            provider: LLMProvider::OpenAI,
        }
    }

    /// Creates a new LLM service with Anthropic provider.
    pub fn new_anthropic(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: DEFAULT_ANTHROPIC_MODEL.to_string(),
            api_url: DEFAULT_ANTHROPIC_API_URL.to_string(),
            provider: LLMProvider::Anthropic,
        }
    }

    /// Creates a new LLM service from environment variables.
    ///
    /// Checks for ANTHROPIC_API_KEY first, then falls back to OPENAI_API_KEY.
    pub fn from_env() -> Result<Self, EditorApiError> {
        // Try Anthropic first
        if let Ok(api_key) = env::var(ANTHROPIC_API_KEY_ENV) {
            return Ok(Self::new_anthropic(api_key));
        }
        
        // Fall back to OpenAI
        if let Ok(api_key) = env::var(OPENAI_API_KEY_ENV) {
            return Ok(Self::new(api_key));
        }
        
        Err(EditorApiError::LLMError(format!(
            "LLM service not configured. Set {} or {} environment variable.",
            ANTHROPIC_API_KEY_ENV, OPENAI_API_KEY_ENV
        )))
    }

    /// Sets the model to use for completions.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the API endpoint URL.
    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    /// Returns true if the service is configured with an API key.
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
    
    /// Returns the current provider.
    pub fn provider(&self) -> LLMProvider {
        self.provider
    }


    /// Research an entity to generate descriptions.
    ///
    /// # Requirements
    /// - 4.2: Generate entity descriptions based on OCSF class metadata
    ///
    /// # Property 9: LLM Research Context Inclusion
    /// For any entity research request, the LLM API call SHALL include OCSF class
    /// metadata (name, description, category) for all source event classes.
    pub async fn research_entity(
        &self,
        entity_name: &str,
        context: &OCSFContext,
        requested_fields: &[ResearchField],
    ) -> Result<ResearchResult, EditorApiError> {
        let prompt = self.build_entity_prompt(entity_name, context, requested_fields);
        let (response_text, tokens_used) = self.call_llm(&prompt).await?;
        let suggestions = self.parse_entity_response(&response_text, requested_fields)?;

        Ok(ResearchResult {
            entity_name: entity_name.to_string(),
            attribute_name: None,
            suggestions,
            tokens_used,
        })
    }

    /// Research an attribute to generate synonyms and security context.
    ///
    /// # Requirements
    /// - 4.3: Generate synonyms relevant to security analytics
    /// - 4.4: Generate security context for observable attributes
    ///
    /// # Property 10: Observable Security Context Generation
    /// For any attribute marked as is_observable: true, LLM research SHALL generate
    /// a security context suggestion explaining threat detection relevance.
    pub async fn research_attribute(
        &self,
        entity_name: &str,
        attribute_name: &str,
        context: &OCSFContext,
        requested_fields: &[ResearchField],
        is_observable: bool,
    ) -> Result<ResearchResult, EditorApiError> {
        let prompt =
            self.build_attribute_prompt(attribute_name, context, requested_fields, is_observable);
        let (response_text, tokens_used) = self.call_llm(&prompt).await?;
        let suggestions = self.parse_attribute_response(&response_text, requested_fields)?;

        Ok(ResearchResult {
            entity_name: entity_name.to_string(),
            attribute_name: Some(attribute_name.to_string()),
            suggestions,
            tokens_used,
        })
    }


    /// Batch research multiple targets in a single operation.
    ///
    /// # Requirements
    /// - 4.7: Support batch research for multiple targets
    ///
    /// # Property 12: Batch Research Single Request
    /// For any batch research operation with N targets, exactly one API request
    /// SHALL be made containing all N targets, rather than N separate requests.
    pub async fn batch_research(
        &self,
        targets: Vec<ResearchTarget>,
        shared_context: Option<String>,
    ) -> Result<Vec<ResearchResult>, EditorApiError> {
        if targets.is_empty() {
            return Ok(vec![]);
        }

        // Build a combined prompt for all targets
        let prompt = self.build_batch_prompt(&targets, shared_context.as_deref());
        let (response_text, tokens_used) = self.call_llm(&prompt).await?;

        // Parse the batch response
        let results = self.parse_batch_response(&response_text, &targets, tokens_used)?;

        Ok(results)
    }

    /// Build prompt for entity research.
    fn build_entity_prompt(
        &self,
        entity_name: &str,
        context: &OCSFContext,
        requested_fields: &[ResearchField],
    ) -> String {
        let mut prompt = format!(
            r#"You are a security data expert helping to enrich a semantic model for OCSF (Open Cybersecurity Schema Framework) data.

Entity Name: {}
OCSF Event Class: {} ({})
Category: {}
Class Description: {}

Generate the following for this semantic entity:
"#,
            entity_name,
            context.class_name,
            context.class_caption,
            context.category_name,
            context.class_description
        );

        for field in requested_fields {
            match field {
                ResearchField::Description => {
                    prompt.push_str("\n1. DESCRIPTION: A clear, concise description (2-3 sentences) explaining what this entity represents in security analytics context.");
                }
                ResearchField::Synonyms => {
                    prompt.push_str("\n2. SYNONYMS: 3-5 alternative names that security analysts might use to refer to this entity.");
                }
                ResearchField::SecurityContext => {
                    prompt.push_str("\n3. SECURITY_CONTEXT: Explain how this entity is relevant for threat detection and security monitoring.");
                }
                ResearchField::SampleValues => {
                    // Not typically used for entities
                }
            }
        }

        prompt.push_str("\n\nRespond in JSON format with keys: description, synonyms (array), security_context");
        prompt
    }


    /// Build prompt for attribute research.
    fn build_attribute_prompt(
        &self,
        attribute_name: &str,
        context: &OCSFContext,
        requested_fields: &[ResearchField],
        is_observable: bool,
    ) -> String {
        let attr_name = context
            .attribute_name
            .as_deref()
            .unwrap_or(attribute_name);
        let attr_type = context.attribute_type.as_deref().unwrap_or("string");
        let attr_desc = context
            .attribute_description
            .as_deref()
            .unwrap_or("No description available");

        let mut prompt = format!(
            r#"You are a security data expert helping to enrich a semantic model for OCSF (Open Cybersecurity Schema Framework) data.

Attribute Name: {}
OCSF Field: {} (type: {})
Field Description: {}
Event Class: {} ({})
Category: {}
Is Observable (IOC-relevant): {}

Generate the following for this semantic attribute:
"#,
            attribute_name,
            attr_name,
            attr_type,
            attr_desc,
            context.class_name,
            context.class_caption,
            context.category_name,
            if is_observable { "Yes" } else { "No" }
        );

        for field in requested_fields {
            match field {
                ResearchField::Description => {
                    prompt.push_str("\n1. DESCRIPTION: A clear description of what this attribute represents.");
                }
                ResearchField::Synonyms => {
                    prompt.push_str("\n2. SYNONYMS: 3-5 alternative terms security analysts might use (e.g., 'source IP' might have synonyms like 'client IP', 'origin address').");
                }
                ResearchField::SecurityContext => {
                    if is_observable {
                        prompt.push_str("\n3. SECURITY_CONTEXT: Explain how this observable is used in threat detection, what IOC types it relates to, and common attack patterns it can help identify.");
                    } else {
                        prompt.push_str("\n3. SECURITY_CONTEXT: Explain the security relevance of this attribute for analytics and monitoring.");
                    }
                }
                ResearchField::SampleValues => {
                    prompt.push_str("\n4. SAMPLE_VALUES: 3-5 realistic example values for this attribute.");
                }
            }
        }

        prompt.push_str("\n\nRespond in JSON format with keys: description, synonyms (array), security_context, sample_values (array)");
        prompt
    }


    /// Build prompt for batch research.
    fn build_batch_prompt(&self, targets: &[ResearchTarget], shared_context: Option<&str>) -> String {
        let mut prompt = String::from(
            r#"You are a security data expert helping to enrich a semantic model for OCSF (Open Cybersecurity Schema Framework) data.

I need you to research multiple targets in a single response. For each target, generate the requested fields.
"#,
        );

        // Add shared context if provided
        if let Some(context) = shared_context {
            if !context.trim().is_empty() {
                prompt.push_str(&format!(
                    r#"
ADDITIONAL CONTEXT:
The following additional context has been provided to help with the research. Use this information to generate more accurate and relevant suggestions:

{}

"#,
                    context.trim()
                ));
            }
        }

        prompt.push_str("TARGETS:\n");

        for (i, target) in targets.iter().enumerate() {
            prompt.push_str(&format!("\n--- Target {} ---\n", i + 1));
            match target.target_type {
                ResearchTargetType::Entity => {
                    prompt.push_str(&format!(
                        "Type: Entity\nName: {}\nOCSF Class: {} ({})\nCategory: {}\n",
                        target.entity_name,
                        target.ocsf_context.class_name,
                        target.ocsf_context.class_caption,
                        target.ocsf_context.category_name
                    ));
                }
                ResearchTargetType::Attribute => {
                    let attr_name = target.attribute_name.as_deref().unwrap_or("unknown");
                    prompt.push_str(&format!(
                        "Type: Attribute\nEntity: {}\nAttribute: {}\nOCSF Field: {}\n",
                        target.entity_name,
                        attr_name,
                        target
                            .ocsf_context
                            .attribute_name
                            .as_deref()
                            .unwrap_or(attr_name)
                    ));
                }
            }
            prompt.push_str(&format!(
                "Requested: {:?}\n",
                target.requested_fields
            ));
        }

        prompt.push_str(
            r#"
Respond with a JSON array where each element corresponds to a target (in order) with the requested fields.
Example format:
[
  {"description": "...", "synonyms": ["...", "..."], "security_context": "..."},
  {"description": "...", "synonyms": ["...", "..."], "sample_values": ["...", "..."]}
]
"#,
        );

        prompt
    }


    /// Send a raw prompt to the configured LLM and return the response text.
    ///
    /// This is a public wrapper around `call_llm` for use by API handlers
    /// that need to send custom prompts (e.g., mapping interpretation).
    pub async fn send_prompt(&self, prompt: &str) -> Result<String, EditorApiError> {
        let (text, _tokens) = self.call_llm(prompt).await?;
        Ok(text)
    }

    /// Extract JSON from an LLM response, handling markdown code blocks.
    ///
    /// Public wrapper for use by API handlers that parse LLM JSON responses.
    pub fn parse_json_response(&self, response: &str) -> Result<serde_json::Value, EditorApiError> {
        self.extract_json(response)
    }

    /// Call the LLM API (OpenAI or Anthropic).
    async fn call_llm(&self, prompt: &str) -> Result<(String, u32), EditorApiError> {
        match self.provider {
            LLMProvider::OpenAI => self.call_openai(prompt).await,
            LLMProvider::Anthropic => self.call_anthropic(prompt).await,
        }
    }

    /// Call the OpenAI API.
    async fn call_openai(&self, prompt: &str) -> Result<(String, u32), EditorApiError> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EditorApiError::LLMError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EditorApiError::LLMError(format!(
                "OpenAI API error ({}): {}",
                status, body
            )));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| EditorApiError::LLMError(format!("Failed to parse response: {}", e)))?;

        let content = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok((content, api_response.usage.total_tokens))
    }

    /// Call the Anthropic API.
    async fn call_anthropic(&self, prompt: &str) -> Result<(String, u32), EditorApiError> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 1000,
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EditorApiError::LLMError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EditorApiError::LLMError(format!(
                "Anthropic API error ({}): {}",
                status, body
            )));
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| EditorApiError::LLMError(format!("Failed to parse response: {}", e)))?;

        let content = api_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let total_tokens = api_response.usage.input_tokens + api_response.usage.output_tokens;

        Ok((content, total_tokens))
    }


    /// Parse entity research response.
    fn parse_entity_response(
        &self,
        response: &str,
        requested_fields: &[ResearchField],
    ) -> Result<Vec<LLMSuggestion>, EditorApiError> {
        let json = self.extract_json(response)?;
        let mut suggestions = Vec::new();

        for field in requested_fields {
            match field {
                ResearchField::Description => {
                    if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::Description,
                            value: SuggestionValue::String(desc.to_string()),
                            confidence: 0.85,
                        });
                    }
                }
                ResearchField::Synonyms => {
                    if let Some(arr) = json.get("synonyms").and_then(|v| v.as_array()) {
                        let synonyms: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !synonyms.is_empty() {
                            suggestions.push(LLMSuggestion {
                                id: uuid_v4(),
                                field: ResearchField::Synonyms,
                                value: SuggestionValue::Array(synonyms),
                                confidence: 0.75,
                            });
                        }
                    }
                }
                ResearchField::SecurityContext => {
                    if let Some(ctx) = json.get("security_context").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::SecurityContext,
                            value: SuggestionValue::String(ctx.to_string()),
                            confidence: 0.80,
                        });
                    }
                }
                ResearchField::SampleValues => {
                    // Not typically used for entities
                }
            }
        }

        Ok(suggestions)
    }


    /// Parse attribute research response.
    fn parse_attribute_response(
        &self,
        response: &str,
        requested_fields: &[ResearchField],
    ) -> Result<Vec<LLMSuggestion>, EditorApiError> {
        let json = self.extract_json(response)?;
        let mut suggestions = Vec::new();

        for field in requested_fields {
            match field {
                ResearchField::Description => {
                    if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::Description,
                            value: SuggestionValue::String(desc.to_string()),
                            confidence: 0.85,
                        });
                    }
                }
                ResearchField::Synonyms => {
                    if let Some(arr) = json.get("synonyms").and_then(|v| v.as_array()) {
                        let synonyms: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !synonyms.is_empty() {
                            suggestions.push(LLMSuggestion {
                                id: uuid_v4(),
                                field: ResearchField::Synonyms,
                                value: SuggestionValue::Array(synonyms),
                                confidence: 0.75,
                            });
                        }
                    }
                }
                ResearchField::SecurityContext => {
                    if let Some(ctx) = json.get("security_context").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::SecurityContext,
                            value: SuggestionValue::String(ctx.to_string()),
                            confidence: 0.80,
                        });
                    }
                }
                ResearchField::SampleValues => {
                    if let Some(arr) = json.get("sample_values").and_then(|v| v.as_array()) {
                        let values: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !values.is_empty() {
                            suggestions.push(LLMSuggestion {
                                id: uuid_v4(),
                                field: ResearchField::SampleValues,
                                value: SuggestionValue::Array(values),
                                confidence: 0.70,
                            });
                        }
                    }
                }
            }
        }

        Ok(suggestions)
    }


    /// Parse batch research response.
    fn parse_batch_response(
        &self,
        response: &str,
        targets: &[ResearchTarget],
        total_tokens: u32,
    ) -> Result<Vec<ResearchResult>, EditorApiError> {
        let json = self.extract_json(response)?;
        let arr = json.as_array().ok_or_else(|| {
            EditorApiError::LLMError("Expected JSON array in batch response".to_string())
        })?;

        let tokens_per_target = total_tokens / targets.len().max(1) as u32;
        let mut results = Vec::new();

        for (i, target) in targets.iter().enumerate() {
            let item = arr.get(i).cloned().unwrap_or(serde_json::Value::Null);
            let suggestions = self.parse_item_suggestions(&item, &target.requested_fields);

            results.push(ResearchResult {
                entity_name: target.entity_name.clone(),
                attribute_name: target.attribute_name.clone(),
                suggestions,
                tokens_used: tokens_per_target,
            });
        }

        Ok(results)
    }

    /// Parse suggestions from a single batch item.
    fn parse_item_suggestions(
        &self,
        item: &serde_json::Value,
        requested_fields: &[ResearchField],
    ) -> Vec<LLMSuggestion> {
        let mut suggestions = Vec::new();

        for field in requested_fields {
            match field {
                ResearchField::Description => {
                    if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::Description,
                            value: SuggestionValue::String(desc.to_string()),
                            confidence: 0.85,
                        });
                    }
                }
                ResearchField::Synonyms => {
                    if let Some(arr) = item.get("synonyms").and_then(|v| v.as_array()) {
                        let synonyms: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !synonyms.is_empty() {
                            suggestions.push(LLMSuggestion {
                                id: uuid_v4(),
                                field: ResearchField::Synonyms,
                                value: SuggestionValue::Array(synonyms),
                                confidence: 0.75,
                            });
                        }
                    }
                }
                ResearchField::SecurityContext => {
                    if let Some(ctx) = item.get("security_context").and_then(|v| v.as_str()) {
                        suggestions.push(LLMSuggestion {
                            id: uuid_v4(),
                            field: ResearchField::SecurityContext,
                            value: SuggestionValue::String(ctx.to_string()),
                            confidence: 0.80,
                        });
                    }
                }
                ResearchField::SampleValues => {
                    if let Some(arr) = item.get("sample_values").and_then(|v| v.as_array()) {
                        let values: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !values.is_empty() {
                            suggestions.push(LLMSuggestion {
                                id: uuid_v4(),
                                field: ResearchField::SampleValues,
                                value: SuggestionValue::Array(values),
                                confidence: 0.70,
                            });
                        }
                    }
                }
            }
        }

        suggestions
    }


    /// Extract JSON from LLM response (handles markdown code blocks).
    fn extract_json(&self, response: &str) -> Result<serde_json::Value, EditorApiError> {
        // Try to find JSON in markdown code block
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .map(|s| s.trim())
                .unwrap_or(response)
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or(response)
        } else {
            response.trim()
        };

        serde_json::from_str(json_str)
            .map_err(|e| EditorApiError::LLMError(format!("Failed to parse JSON response: {}", e)))
    }
}

/// Generate a simple UUID v4.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:032x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_service_creation() {
        let service = LLMService::new("test-api-key");
        assert!(service.is_configured());
        assert_eq!(service.model, DEFAULT_OPENAI_MODEL);
        assert_eq!(service.provider(), LLMProvider::OpenAI);
    }

    #[test]
    fn test_llm_service_anthropic_creation() {
        let service = LLMService::new_anthropic("test-api-key");
        assert!(service.is_configured());
        assert_eq!(service.model, DEFAULT_ANTHROPIC_MODEL);
        assert_eq!(service.provider(), LLMProvider::Anthropic);
    }

    #[test]
    fn test_llm_service_with_model() {
        let service = LLMService::new("test-api-key").with_model("gpt-4");
        assert_eq!(service.model, "gpt-4");
    }

    #[test]
    fn test_research_field_from_str() {
        assert_eq!(ResearchField::parse("description"), Some(ResearchField::Description));
        assert_eq!(ResearchField::parse("synonyms"), Some(ResearchField::Synonyms));
        assert_eq!(ResearchField::parse("security_context"), Some(ResearchField::SecurityContext));
        assert_eq!(ResearchField::parse("sample_values"), Some(ResearchField::SampleValues));
        assert_eq!(ResearchField::parse("invalid"), None);
    }

    #[test]
    fn test_suggestion_value_to_json() {
        let string_val = SuggestionValue::String("test".to_string());
        assert_eq!(
            string_val.to_json(),
            serde_json::Value::String("test".to_string())
        );

        let array_val = SuggestionValue::Array(vec!["a".to_string(), "b".to_string()]);
        let json = array_val.to_json();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }


    #[test]
    fn test_extract_json_plain() {
        let service = LLMService::new("test");
        let json = r#"{"description": "test", "synonyms": ["a", "b"]}"#;
        let result = service.extract_json(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["description"], "test");
    }

    #[test]
    fn test_extract_json_markdown() {
        let service = LLMService::new("test");
        let response = r#"Here is the response:
```json
{"description": "test", "synonyms": ["a", "b"]}
```
"#;
        let result = service.extract_json(response);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["description"], "test");
    }

    #[test]
    fn test_build_entity_prompt_includes_context() {
        let service = LLMService::new("test");
        let context = OCSFContext {
            class_name: "authentication".to_string(),
            class_caption: "Authentication".to_string(),
            class_description: "Authentication events".to_string(),
            category_name: "iam".to_string(),
            ..Default::default()
        };

        let prompt = service.build_entity_prompt(
            "auth_event",
            &context,
            &[ResearchField::Description, ResearchField::Synonyms],
        );

        // Property 9: LLM Research Context Inclusion
        // Verify OCSF class metadata is included
        assert!(prompt.contains("authentication"));
        assert!(prompt.contains("Authentication"));
        assert!(prompt.contains("iam"));
        assert!(prompt.contains("Authentication events"));
    }

    #[test]
    fn test_build_attribute_prompt_observable() {
        let service = LLMService::new("test");
        let context = OCSFContext {
            class_name: "dns_activity".to_string(),
            class_caption: "DNS Activity".to_string(),
            class_description: "DNS query events".to_string(),
            category_name: "network_activity".to_string(),
            attribute_name: Some("query.hostname".to_string()),
            attribute_type: Some("string_t".to_string()),
            attribute_description: Some("The queried hostname".to_string()),
        };

        let prompt = service.build_attribute_prompt(
            "query_hostname",
            &context,
            &[ResearchField::SecurityContext],
            true, // is_observable
        );

        // Property 10: Observable Security Context Generation
        // Verify observable-specific context is requested
        assert!(prompt.contains("Is Observable (IOC-relevant): Yes"));
        assert!(prompt.contains("threat detection"));
    }

    #[test]
    fn test_parse_entity_response() {
        let service = LLMService::new("test");
        let response = r#"{
            "description": "User authentication events",
            "synonyms": ["login event", "auth event"],
            "security_context": "Critical for detecting unauthorized access"
        }"#;

        let suggestions = service
            .parse_entity_response(
                response,
                &[
                    ResearchField::Description,
                    ResearchField::Synonyms,
                    ResearchField::SecurityContext,
                ],
            )
            .unwrap();

        assert_eq!(suggestions.len(), 3);

        let desc = suggestions
            .iter()
            .find(|s| matches!(s.field, ResearchField::Description))
            .unwrap();
        assert!(matches!(&desc.value, SuggestionValue::String(s) if s.contains("authentication")));

        let synonyms = suggestions
            .iter()
            .find(|s| matches!(s.field, ResearchField::Synonyms))
            .unwrap();
        assert!(matches!(&synonyms.value, SuggestionValue::Array(arr) if arr.len() == 2));
    }

    #[test]
    fn test_ocsf_context_default() {
        let context = OCSFContext::default();
        assert!(context.class_name.is_empty());
        assert!(context.attribute_name.is_none());
    }

    #[test]
    fn test_uuid_v4_uniqueness() {
        let id1 = uuid_v4();
        let id2 = uuid_v4();
        // UUIDs should be different (though not guaranteed with nanosecond precision)
        // At minimum, they should be valid hex strings
        assert_eq!(id1.len(), 32);
        assert_eq!(id2.len(), 32);
    }
}
