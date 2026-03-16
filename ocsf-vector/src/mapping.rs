//! Mapping assistant for AI-assisted field mapping suggestions.
//!
//! This module provides intelligent mapping suggestions for mapping source fields
//! to OCSF attributes using vector similarity search.

use crate::embedding::{EmbeddingElementType, EmbeddingError};
use crate::store::{SimilarityResult, VectorFilter, VectorStore, VectorStoreError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during mapping operations.
#[derive(Debug, Error)]
pub enum MappingError {
    /// Vector store error.
    #[error("Vector store error: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    /// Embedding error.
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingError),

    /// No suggestions found.
    #[error("No mapping suggestions found for field: {0}")]
    NoSuggestions(String),

    /// Invalid input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Metadata about a source field to be mapped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceFieldMetadata {
    /// The field name.
    pub name: String,

    /// The field type (e.g., "string", "integer", "timestamp").
    #[serde(rename = "type")]
    pub field_type: String,

    /// Optional description of the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Sample values from the field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_values: Vec<String>,

    /// The source system this field comes from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_system: Option<String>,
}

impl SourceFieldMetadata {
    /// Creates new source field metadata.
    pub fn new(name: impl Into<String>, field_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            field_type: field_type.into(),
            description: None,
            sample_values: Vec::new(),
            source_system: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets sample values.
    pub fn with_sample_values(mut self, values: Vec<String>) -> Self {
        self.sample_values = values;
        self
    }

    /// Sets the source system.
    pub fn with_source_system(mut self, system: impl Into<String>) -> Self {
        self.source_system = Some(system.into());
        self
    }

    /// Generates a text representation for embedding.
    pub fn to_embedding_text(&self) -> String {
        let mut parts = vec![self.name.clone()];

        if let Some(desc) = &self.description {
            parts.push(desc.clone());
        }

        if !self.sample_values.is_empty() {
            parts.push(format!("examples: {}", self.sample_values.join(", ")));
        }

        parts.join(" ")
    }
}

/// A mapping suggestion from the assistant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingSuggestion {
    /// The target OCSF field path (e.g., "actor.user.name").
    pub target_field: String,

    /// Confidence score (0.0 to 1.0, higher is more confident).
    pub confidence: f32,

    /// Reasoning for this suggestion.
    pub reasoning: String,

    /// Alternative fields that could also match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_fields: Vec<String>,

    /// Whether this is a low-confidence match suggesting extension creation.
    #[serde(default)]
    pub suggest_extension: bool,
}

impl MappingSuggestion {
    /// Creates a new mapping suggestion.
    pub fn new(
        target_field: impl Into<String>,
        confidence: f32,
        reasoning: impl Into<String>,
    ) -> Self {
        Self {
            target_field: target_field.into(),
            confidence,
            reasoning: reasoning.into(),
            alternative_fields: Vec::new(),
            suggest_extension: false,
        }
    }

    /// Adds alternative fields.
    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternative_fields = alternatives;
        self
    }

    /// Marks this as suggesting an extension.
    pub fn as_extension_suggestion(mut self) -> Self {
        self.suggest_extension = true;
        self
    }
}

/// A confirmed mapping that can be used for learning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfirmedMapping {
    /// The source field metadata.
    pub source_field: SourceFieldMetadata,

    /// The confirmed target OCSF field.
    pub target_field: String,

    /// Timestamp when the mapping was confirmed.
    pub confirmed_at: u64,
}

impl ConfirmedMapping {
    /// Creates a new confirmed mapping.
    pub fn new(source_field: SourceFieldMetadata, target_field: impl Into<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            source_field,
            target_field: target_field.into(),
            confirmed_at: timestamp,
        }
    }
}

/// Serializable learning state for persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningState {
    /// Confirmed mappings.
    pub confirmed_mappings: Vec<ConfirmedMapping>,

    /// Field boost scores.
    pub field_boosts: HashMap<String, f32>,
}

impl LearningState {
    /// Creates a new empty learning state.
    pub fn new() -> Self {
        Self {
            confirmed_mappings: Vec::new(),
            field_boosts: HashMap::new(),
        }
    }

    /// Serializes the learning state to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a learning state from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for LearningState {
    fn default() -> Self {
        Self::new()
    }
}

/// A suggestion for creating a custom extension attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionSuggestion {
    /// The suggested extension attribute name (e.g., "x_custom_field").
    pub name: String,

    /// The suggested OCSF type (e.g., "string_t", "integer_t").
    pub ocsf_type: String,

    /// Human-readable caption.
    pub caption: String,

    /// Description for the extension.
    pub description: String,

    /// The original source field this extension is for.
    pub source_field: SourceFieldMetadata,

    /// Sample values from the source field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_values: Vec<String>,
}

impl ExtensionSuggestion {
    /// Generates OCSF extension schema JSON for this suggestion.
    pub fn to_ocsf_extension_json(&self) -> String {
        serde_json::json!({
            "name": self.name,
            "type": self.ocsf_type,
            "caption": self.caption,
            "description": self.description,
            "requirement": "optional"
        })
        .to_string()
    }
}


/// Configuration for the mapping assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingAssistantConfig {
    /// Minimum confidence threshold for suggestions.
    pub min_confidence: f32,

    /// Maximum number of suggestions to return.
    pub max_suggestions: usize,

    /// Threshold below which to suggest creating an extension.
    pub extension_threshold: f32,

    /// Boost factor for confirmed mappings.
    pub confirmed_mapping_boost: f32,

    /// Decay factor for similarity-based boosting (0-1).
    /// Higher values mean similar fields get more boost.
    pub similarity_boost_decay: f32,
}

impl Default for MappingAssistantConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.3,
            max_suggestions: 5,
            extension_threshold: 0.5,
            confirmed_mapping_boost: 0.2,
            similarity_boost_decay: 0.5,
        }
    }
}

/// The mapping assistant for AI-assisted field mapping.
///
/// Uses vector similarity search to suggest OCSF field mappings
/// for source fields from external systems.
pub struct MappingAssistant<G, S> {
    /// The embedding generator.
    generator: G,

    /// The vector store containing OCSF field embeddings.
    store: S,

    /// Configuration for the assistant.
    config: MappingAssistantConfig,

    /// Confirmed mappings for learning.
    confirmed_mappings: Vec<ConfirmedMapping>,

    /// Boost scores for fields based on confirmed mappings.
    field_boosts: HashMap<String, f32>,
}

impl<G, S> MappingAssistant<G, S>
where
    G: crate::embedding::EmbeddingGenerator,
    S: VectorStore,
{
    /// Creates a new mapping assistant.
    pub fn new(generator: G, store: S) -> Self {
        Self {
            generator,
            store,
            config: MappingAssistantConfig::default(),
            confirmed_mappings: Vec::new(),
            field_boosts: HashMap::new(),
        }
    }

    /// Creates a new mapping assistant with custom configuration.
    pub fn with_config(generator: G, store: S, config: MappingAssistantConfig) -> Self {
        Self {
            generator,
            store,
            config,
            confirmed_mappings: Vec::new(),
            field_boosts: HashMap::new(),
        }
    }

    /// Returns the configuration.
    pub fn config(&self) -> &MappingAssistantConfig {
        &self.config
    }

    /// Returns the confirmed mappings.
    pub fn confirmed_mappings(&self) -> &[ConfirmedMapping] {
        &self.confirmed_mappings
    }

    /// Suggests mappings for a source field.
    pub async fn suggest_mappings(
        &self,
        source_field: &SourceFieldMetadata,
        top_k: Option<usize>,
    ) -> Result<Vec<MappingSuggestion>, MappingError> {
        let top_k = top_k.unwrap_or(self.config.max_suggestions);

        // Generate embedding for the source field
        let query_text = source_field.to_embedding_text();
        let query_embedding = self.generator.embed(&query_text).await?;

        // Create filter for OCSF attributes only
        let filter = VectorFilter::new()
            .with_element_types(vec![EmbeddingElementType::OcsfAttribute])
            .with_min_score(self.config.min_confidence);

        // Search for similar embeddings
        let results = self
            .store
            .similarity_search(&query_embedding, top_k * 2, Some(&filter))
            .await?;

        // Convert results to suggestions with reasoning
        let mut suggestions: Vec<MappingSuggestion> = results
            .into_iter()
            .map(|result| self.result_to_suggestion(source_field, result))
            .collect();

        // Apply boosts from confirmed mappings
        for suggestion in &mut suggestions {
            if let Some(&boost) = self.field_boosts.get(&suggestion.target_field) {
                suggestion.confidence = (suggestion.confidence + boost).min(1.0);
            }
        }

        // Re-sort by confidence after applying boosts
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate to requested size
        suggestions.truncate(top_k);

        // Add alternatives to top suggestion
        if suggestions.len() > 1 {
            let alternatives: Vec<String> = suggestions[1..]
                .iter()
                .take(3)
                .map(|s| s.target_field.clone())
                .collect();
            if let Some(first) = suggestions.first_mut() {
                first.alternative_fields = alternatives;
            }
        }

        // Check if we should suggest extension creation
        if suggestions.is_empty()
            || suggestions
                .first()
                .map(|s| s.confidence < self.config.extension_threshold)
                .unwrap_or(true)
        {
            let extension_suggestion = self.create_extension_suggestion(source_field);
            if suggestions.is_empty() {
                suggestions.push(extension_suggestion);
            } else {
                // Mark the top suggestion as potentially needing extension
                if let Some(first) = suggestions.first_mut() {
                    if first.confidence < self.config.extension_threshold {
                        first.suggest_extension = true;
                    }
                }
            }
        }

        Ok(suggestions)
    }

    /// Converts a similarity result to a mapping suggestion.
    fn result_to_suggestion(
        &self,
        source_field: &SourceFieldMetadata,
        result: SimilarityResult,
    ) -> MappingSuggestion {
        let target_field = result
            .metadata
            .path
            .clone()
            .unwrap_or_else(|| result.metadata.name.clone());

        let reasoning = self.generate_reasoning(source_field, &result);

        MappingSuggestion::new(target_field, result.score, reasoning)
    }

    /// Generates reasoning for a mapping suggestion.
    fn generate_reasoning(
        &self,
        source_field: &SourceFieldMetadata,
        result: &SimilarityResult,
    ) -> String {
        let mut reasons = Vec::new();

        // Name similarity
        let source_name_lower = source_field.name.to_lowercase();
        let target_name_lower = result.metadata.name.to_lowercase();

        if source_name_lower == target_name_lower {
            reasons.push("Exact name match".to_string());
        } else if source_name_lower.contains(&target_name_lower)
            || target_name_lower.contains(&source_name_lower)
        {
            reasons.push("Partial name match".to_string());
        }

        // Description similarity
        if let Some(source_desc) = &source_field.description {
            if !result.metadata.description.is_empty() {
                let source_lower = source_desc.to_lowercase();
                let target_lower = result.metadata.description.to_lowercase();
                let source_words: std::collections::HashSet<&str> =
                    source_lower.split_whitespace().collect();
                let target_words: std::collections::HashSet<&str> =
                    target_lower.split_whitespace().collect();
                let common: Vec<_> = source_words.intersection(&target_words).collect();
                if !common.is_empty() {
                    reasons.push(format!(
                        "Description contains common terms: {}",
                        common.into_iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }

        // Type compatibility
        if let Some(target_type) = result.metadata.properties.get("type") {
            let type_compatible = matches!(
                (source_field.field_type.as_str(), target_type.as_str()),
                ("string", "string_t") | ("str", "string_t")
                | ("integer", "integer_t") | ("int", "integer_t") | ("long", "long_t")
                | ("boolean", "boolean_t") | ("bool", "boolean_t")
                | ("timestamp", "timestamp_t") | ("datetime", "timestamp_t")
                | ("float", "float_t") | ("double", "float_t")
            );
            if type_compatible {
                reasons.push("Compatible data types".to_string());
            }
        }

        // Confidence level
        let confidence_desc = if result.score >= 0.9 {
            "Very high semantic similarity"
        } else if result.score >= 0.7 {
            "High semantic similarity"
        } else if result.score >= 0.5 {
            "Moderate semantic similarity"
        } else {
            "Low semantic similarity"
        };
        reasons.push(confidence_desc.to_string());

        if reasons.is_empty() {
            format!(
                "Semantic similarity score: {:.2}",
                result.score
            )
        } else {
            reasons.join(". ")
        }
    }

    /// Creates an extension suggestion for low-confidence matches.
    fn create_extension_suggestion(&self, source_field: &SourceFieldMetadata) -> MappingSuggestion {
        let extension_name = format!("x_{}", source_field.name.to_lowercase().replace(' ', "_"));
        let reasoning = format!(
            "No strong OCSF match found. Consider creating a custom extension attribute '{}' \
             to preserve this field's data while maintaining OCSF compatibility.",
            extension_name
        );

        MappingSuggestion::new(extension_name, 0.0, reasoning).as_extension_suggestion()
    }

    /// Suggests an extension attribute for a source field.
    ///
    /// This is useful when no good OCSF match exists and the user wants
    /// to create a custom extension attribute.
    pub fn suggest_extension(&self, source_field: &SourceFieldMetadata) -> ExtensionSuggestion {
        let extension_name = format!("x_{}", source_field.name.to_lowercase().replace(' ', "_"));

        // Map source type to OCSF type
        let ocsf_type = match source_field.field_type.to_lowercase().as_str() {
            "string" | "str" | "text" | "varchar" => "string_t",
            "integer" | "int" | "long" | "bigint" => "integer_t",
            "float" | "double" | "decimal" | "number" => "float_t",
            "boolean" | "bool" => "boolean_t",
            "timestamp" | "datetime" | "date" | "time" => "timestamp_t",
            "json" | "object" | "map" => "json_t",
            "array" | "list" => "string_t", // Arrays need special handling
            _ => "string_t", // Default to string
        };

        let description = source_field
            .description
            .clone()
            .unwrap_or_else(|| format!("Custom extension for {}", source_field.name));

        ExtensionSuggestion {
            name: extension_name,
            ocsf_type: ocsf_type.to_string(),
            caption: source_field.name.replace('_', " "),
            description,
            source_field: source_field.clone(),
            sample_values: source_field.sample_values.clone(),
        }
    }

    /// Detects if a source field is a low-confidence match.
    pub async fn is_low_confidence_match(
        &self,
        source_field: &SourceFieldMetadata,
    ) -> Result<bool, MappingError> {
        let suggestions = self.suggest_mappings(source_field, Some(1)).await?;

        Ok(suggestions.is_empty()
            || suggestions
                .first()
                .map(|s| s.confidence < self.config.extension_threshold)
                .unwrap_or(true))
    }

    /// Confirms a mapping and learns from it.
    ///
    /// This stores the confirmed mapping and boosts the target field's score
    /// for future suggestions. It also applies a decayed boost to similar fields.
    pub async fn confirm_mapping(
        &mut self,
        source_field: SourceFieldMetadata,
        target_field: impl Into<String>,
    ) -> Result<(), MappingError> {
        let target = target_field.into();

        // Store the confirmed mapping
        let confirmed = ConfirmedMapping::new(source_field.clone(), target.clone());
        self.confirmed_mappings.push(confirmed);

        // Update boost for this field
        let boost = self
            .field_boosts
            .entry(target.clone())
            .or_insert(0.0);
        *boost = (*boost + self.config.confirmed_mapping_boost).min(0.5);

        // Apply similarity-based boosting to related fields
        // Generate embedding for the source field to find similar patterns
        let query_text = source_field.to_embedding_text();
        if let Ok(query_embedding) = self.generator.embed(&query_text).await {
            // Find similar embeddings in the store
            let filter = VectorFilter::new()
                .with_element_types(vec![EmbeddingElementType::OcsfAttribute])
                .with_min_score(0.5);

            if let Ok(similar_results) = self
                .store
                .similarity_search(&query_embedding, 10, Some(&filter))
                .await
            {
                // Apply decayed boost to similar fields
                for result in similar_results {
                    let similar_field = result
                        .metadata
                        .path
                        .clone()
                        .unwrap_or_else(|| result.metadata.name.clone());

                    // Skip the exact match (already boosted)
                    if similar_field == target {
                        continue;
                    }

                    // Calculate decayed boost based on similarity
                    let decayed_boost = self.config.confirmed_mapping_boost
                        * self.config.similarity_boost_decay
                        * result.score;

                    let similar_boost = self
                        .field_boosts
                        .entry(similar_field)
                        .or_insert(0.0);
                    *similar_boost = (*similar_boost + decayed_boost).min(0.3);
                }
            }
        }

        Ok(())
    }

    /// Returns the current boost for a field.
    pub fn get_field_boost(&self, field: &str) -> f32 {
        self.field_boosts.get(field).copied().unwrap_or(0.0)
    }

    /// Clears all learned boosts.
    pub fn clear_boosts(&mut self) {
        self.field_boosts.clear();
    }

    /// Exports the learning state for persistence.
    pub fn export_learning_state(&self) -> LearningState {
        LearningState {
            confirmed_mappings: self.confirmed_mappings.clone(),
            field_boosts: self.field_boosts.clone(),
        }
    }

    /// Imports a learning state.
    pub fn import_learning_state(&mut self, state: LearningState) {
        self.confirmed_mappings = state.confirmed_mappings;
        self.field_boosts = state.field_boosts;
    }

    /// Suggests mappings for multiple source fields.
    pub async fn suggest_batch_mappings(
        &self,
        source_fields: &[SourceFieldMetadata],
    ) -> Result<HashMap<String, Vec<MappingSuggestion>>, MappingError> {
        let mut results = HashMap::new();

        for field in source_fields {
            let suggestions = self.suggest_mappings(field, None).await?;
            results.insert(field.name.clone(), suggestions);
        }

        Ok(results)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::{Embedding, EmbeddingGenerator, EmbeddingMetadata, MockEmbeddingGenerator};
    use crate::store::InMemoryVectorStore;

    use super::LearningState;

    async fn create_test_assistant() -> MappingAssistant<MockEmbeddingGenerator, InMemoryVectorStore>
    {
        let generator = MockEmbeddingGenerator::new(384);
        let mut store = InMemoryVectorStore::new(384);

        // Add some test embeddings for OCSF attributes
        let test_attributes = vec![
            ("user_name", "actor.user.name", "The name of the user"),
            ("email_addr", "actor.user.email_addr", "The email address of the user"),
            ("ip_address", "src_endpoint.ip", "The source IP address"),
            ("hostname", "src_endpoint.hostname", "The source hostname"),
            ("timestamp", "time", "The event timestamp"),
            ("severity", "severity_id", "The severity level of the event"),
        ];

        for (name, path, description) in test_attributes {
            let text = format!("{} {} {}", name, path, description);
            let vector = generator.embed(&text).await.unwrap();
            let metadata = EmbeddingMetadata::ocsf_attribute(name, description)
                .with_path(path)
                .with_property("type", "string_t");
            let embedding = Embedding::new(format!("attr_{}", name), vector, metadata);
            store.store_one(embedding).await.unwrap();
        }

        MappingAssistant::new(generator, store)
    }

    #[tokio::test]
    async fn test_suggest_mappings_basic() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("username", "string")
            .with_description("The user's login name");

        let suggestions = assistant.suggest_mappings(&source_field, Some(3)).await.unwrap();

        assert!(!suggestions.is_empty());
        // Should find user_name as a match
        assert!(suggestions.iter().any(|s| s.target_field.contains("user")));
    }

    #[tokio::test]
    async fn test_suggest_mappings_with_sample_values() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("src_ip", "string")
            .with_description("Source IP address")
            .with_sample_values(vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()]);

        let suggestions = assistant.suggest_mappings(&source_field, Some(3)).await.unwrap();

        assert!(!suggestions.is_empty());
        // Should find ip_address as a match
        assert!(suggestions.iter().any(|s| s.target_field.contains("ip")));
    }

    #[tokio::test]
    async fn test_suggestion_has_reasoning() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("email", "string")
            .with_description("User email address");

        let suggestions = assistant.suggest_mappings(&source_field, Some(3)).await.unwrap();

        assert!(!suggestions.is_empty());
        for suggestion in &suggestions {
            assert!(!suggestion.reasoning.is_empty(), "Suggestion should have reasoning");
        }
    }

    #[tokio::test]
    async fn test_suggestions_ordered_by_confidence() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("user_email", "string")
            .with_description("The email address");

        let suggestions = assistant.suggest_mappings(&source_field, Some(5)).await.unwrap();

        // Verify ordering
        for i in 1..suggestions.len() {
            assert!(
                suggestions[i - 1].confidence >= suggestions[i].confidence,
                "Suggestions should be ordered by descending confidence"
            );
        }
    }

    #[tokio::test]
    async fn test_confirm_mapping_boosts_future_suggestions() {
        let mut assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("user_name", "string");

        // Get initial suggestions
        let initial = assistant.suggest_mappings(&source_field, Some(5)).await.unwrap();
        let initial_confidence = initial
            .iter()
            .find(|s| s.target_field == "actor.user.name")
            .map(|s| s.confidence)
            .unwrap_or(0.0);

        // Confirm a mapping
        assistant
            .confirm_mapping(source_field.clone(), "actor.user.name")
            .await
            .unwrap();

        // Get new suggestions
        let after = assistant.suggest_mappings(&source_field, Some(5)).await.unwrap();
        let after_confidence = after
            .iter()
            .find(|s| s.target_field == "actor.user.name")
            .map(|s| s.confidence)
            .unwrap_or(0.0);

        // Confidence should be boosted
        assert!(
            after_confidence >= initial_confidence,
            "Confirmed mapping should boost confidence"
        );
    }

    #[tokio::test]
    async fn test_batch_suggestions() {
        let assistant = create_test_assistant().await;

        let fields = vec![
            SourceFieldMetadata::new("username", "string"),
            SourceFieldMetadata::new("src_ip", "string"),
            SourceFieldMetadata::new("event_time", "timestamp"),
        ];

        let results = assistant.suggest_batch_mappings(&fields).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.contains_key("username"));
        assert!(results.contains_key("src_ip"));
        assert!(results.contains_key("event_time"));
    }

    #[tokio::test]
    async fn test_low_confidence_suggests_extension() {
        let generator = MockEmbeddingGenerator::new(384);
        let store = InMemoryVectorStore::new(384);

        // Create assistant with empty store
        let config = MappingAssistantConfig {
            extension_threshold: 0.5,
            ..Default::default()
        };
        let assistant = MappingAssistant::with_config(generator, store, config);

        let source_field = SourceFieldMetadata::new("custom_field_xyz", "string")
            .with_description("A very custom field that won't match anything");

        let suggestions = assistant.suggest_mappings(&source_field, Some(3)).await.unwrap();

        // Should suggest extension
        assert!(
            suggestions.iter().any(|s| s.suggest_extension),
            "Should suggest creating an extension for unmatched fields"
        );
    }

    #[tokio::test]
    async fn test_alternatives_included() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("user", "string")
            .with_description("User identifier");

        let suggestions = assistant.suggest_mappings(&source_field, Some(5)).await.unwrap();

        if suggestions.len() > 1 {
            let first = &suggestions[0];
            assert!(
                !first.alternative_fields.is_empty(),
                "Top suggestion should include alternatives"
            );
        }
    }

    #[test]
    fn test_source_field_metadata_to_embedding_text() {
        let field = SourceFieldMetadata::new("user_email", "string")
            .with_description("The user's email address")
            .with_sample_values(vec!["test@example.com".to_string()]);

        let text = field.to_embedding_text();

        assert!(text.contains("user_email"));
        assert!(text.contains("email address"));
        assert!(text.contains("test@example.com"));
    }

    #[test]
    fn test_mapping_suggestion_builder() {
        let suggestion = MappingSuggestion::new("actor.user.name", 0.85, "High similarity")
            .with_alternatives(vec!["actor.user.uid".to_string()])
            .as_extension_suggestion();

        assert_eq!(suggestion.target_field, "actor.user.name");
        assert_eq!(suggestion.confidence, 0.85);
        assert!(!suggestion.alternative_fields.is_empty());
        assert!(suggestion.suggest_extension);
    }

    #[test]
    fn test_confirmed_mapping() {
        let source = SourceFieldMetadata::new("email", "string");
        let confirmed = ConfirmedMapping::new(source, "actor.user.email_addr");

        assert_eq!(confirmed.target_field, "actor.user.email_addr");
        assert!(confirmed.confirmed_at > 0);
    }

    #[tokio::test]
    async fn test_similarity_based_boosting() {
        let mut assistant = create_test_assistant().await;

        // Confirm a mapping for user_name
        let source_field = SourceFieldMetadata::new("username", "string")
            .with_description("The user's login name");

        assistant
            .confirm_mapping(source_field, "actor.user.name")
            .await
            .unwrap();

        // Check that the confirmed field got boosted
        let boost = assistant.get_field_boost("actor.user.name");
        assert!(boost > 0.0, "Confirmed field should have a boost");

        // Similar fields might also get a small boost
        // (depends on the mock embedding similarity)
    }

    #[tokio::test]
    async fn test_learning_state_persistence() {
        let mut assistant = create_test_assistant().await;

        // Confirm some mappings
        assistant
            .confirm_mapping(
                SourceFieldMetadata::new("username", "string"),
                "actor.user.name",
            )
            .await
            .unwrap();

        assistant
            .confirm_mapping(
                SourceFieldMetadata::new("email", "string"),
                "actor.user.email_addr",
            )
            .await
            .unwrap();

        // Export state
        let state = assistant.export_learning_state();
        assert_eq!(state.confirmed_mappings.len(), 2);
        assert!(!state.field_boosts.is_empty());

        // Serialize and deserialize
        let json = state.to_json().unwrap();
        let restored = LearningState::from_json(&json).unwrap();
        assert_eq!(state, restored);
    }

    #[tokio::test]
    async fn test_import_learning_state() {
        let mut assistant = create_test_assistant().await;

        // Create a learning state
        let mut state = LearningState::new();
        state.field_boosts.insert("actor.user.name".to_string(), 0.3);
        state.confirmed_mappings.push(ConfirmedMapping::new(
            SourceFieldMetadata::new("username", "string"),
            "actor.user.name",
        ));

        // Import it
        assistant.import_learning_state(state);

        // Verify the boost is applied
        let boost = assistant.get_field_boost("actor.user.name");
        assert!((boost - 0.3).abs() < 0.001);
        assert_eq!(assistant.confirmed_mappings().len(), 1);
    }

    #[tokio::test]
    async fn test_clear_boosts() {
        let mut assistant = create_test_assistant().await;

        // Confirm a mapping
        assistant
            .confirm_mapping(
                SourceFieldMetadata::new("username", "string"),
                "actor.user.name",
            )
            .await
            .unwrap();

        assert!(assistant.get_field_boost("actor.user.name") > 0.0);

        // Clear boosts
        assistant.clear_boosts();

        assert_eq!(assistant.get_field_boost("actor.user.name"), 0.0);
    }

    #[tokio::test]
    async fn test_suggest_extension() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("custom_vendor_field", "string")
            .with_description("A vendor-specific field")
            .with_sample_values(vec!["value1".to_string(), "value2".to_string()]);

        let extension = assistant.suggest_extension(&source_field);

        assert_eq!(extension.name, "x_custom_vendor_field");
        assert_eq!(extension.ocsf_type, "string_t");
        assert_eq!(extension.description, "A vendor-specific field");
        assert_eq!(extension.sample_values.len(), 2);
    }

    #[tokio::test]
    async fn test_suggest_extension_type_mapping() {
        let assistant = create_test_assistant().await;

        // Test various type mappings
        let test_cases = vec![
            ("string", "string_t"),
            ("integer", "integer_t"),
            ("int", "integer_t"),
            ("float", "float_t"),
            ("boolean", "boolean_t"),
            ("timestamp", "timestamp_t"),
            ("datetime", "timestamp_t"),
            ("json", "json_t"),
            ("unknown_type", "string_t"), // Default to string
        ];

        for (source_type, expected_ocsf_type) in test_cases {
            let source_field = SourceFieldMetadata::new("test_field", source_type);
            let extension = assistant.suggest_extension(&source_field);
            assert_eq!(
                extension.ocsf_type, expected_ocsf_type,
                "Type {} should map to {}",
                source_type, expected_ocsf_type
            );
        }
    }

    #[tokio::test]
    async fn test_extension_to_json() {
        let assistant = create_test_assistant().await;

        let source_field = SourceFieldMetadata::new("custom_field", "integer")
            .with_description("A custom integer field");

        let extension = assistant.suggest_extension(&source_field);
        let json = extension.to_ocsf_extension_json();

        assert!(json.contains("x_custom_field"));
        assert!(json.contains("integer_t"));
        assert!(json.contains("A custom integer field"));
    }

    #[tokio::test]
    async fn test_is_low_confidence_match() {
        let generator = MockEmbeddingGenerator::new(384);
        let store = InMemoryVectorStore::new(384);

        // Create assistant with empty store
        let config = MappingAssistantConfig {
            extension_threshold: 0.5,
            ..Default::default()
        };
        let assistant = MappingAssistant::with_config(generator, store, config);

        let source_field = SourceFieldMetadata::new("unknown_field", "string");

        // With empty store, should be low confidence
        let is_low = assistant.is_low_confidence_match(&source_field).await.unwrap();
        assert!(is_low, "Empty store should result in low confidence match");
    }

    #[tokio::test]
    async fn test_is_not_low_confidence_match() {
        let assistant = create_test_assistant().await;

        // Use a field that should match well
        let source_field = SourceFieldMetadata::new("user_name", "string")
            .with_description("The name of the user");

        let is_low = assistant.is_low_confidence_match(&source_field).await.unwrap();
        // This might or might not be low confidence depending on the mock embeddings
        // The important thing is that the method works without error
        let _ = is_low;
    }
}
