//! Embedding generation for OCSF schema elements and semantic entities.
//!
//! This module provides traits and implementations for generating vector embeddings
//! from OCSF schema elements and semantic entity definitions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during embedding generation.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    /// The embedding model is not available.
    #[error("Embedding model not available: {0}")]
    ModelNotAvailable(String),

    /// Vector dimension mismatch.
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// API error from external embedding service.
    #[error("API error: {0}")]
    ApiError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Text is empty or invalid.
    #[error("Invalid input text: {0}")]
    InvalidInput(String),
}

/// Configuration for embedding models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    /// The type of embedding model.
    pub model_type: EmbeddingModelType,

    /// The model name or identifier.
    pub model_name: String,

    /// The dimension of the output vectors.
    pub dimensions: usize,

    /// Optional API key for external services.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Optional API base URL for external services.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
}

/// Types of embedding models supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmbeddingModelType {
    /// Sentence transformers models (via candle or ort).
    SentenceTransformers,
    /// OpenAI embedding API.
    OpenAI,
    /// Cohere embedding API.
    Cohere,
    /// Mock embeddings for testing.
    Mock,
}

impl Default for EmbeddingModelConfig {
    fn default() -> Self {
        Self {
            model_type: EmbeddingModelType::Mock,
            model_name: "mock-embedding-model".to_string(),
            dimensions: 384,
            api_key: None,
            api_base_url: None,
        }
    }
}

impl EmbeddingModelConfig {
    /// Creates a new configuration for sentence transformers.
    pub fn sentence_transformers(model_name: impl Into<String>, dimensions: usize) -> Self {
        Self {
            model_type: EmbeddingModelType::SentenceTransformers,
            model_name: model_name.into(),
            dimensions,
            api_key: None,
            api_base_url: None,
        }
    }

    /// Creates a new configuration for OpenAI embeddings.
    pub fn openai(model_name: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model_type: EmbeddingModelType::OpenAI,
            model_name: model_name.into(),
            dimensions: 1536, // Default for text-embedding-ada-002
            api_key: Some(api_key.into()),
            api_base_url: None,
        }
    }

    /// Creates a new configuration for Cohere embeddings.
    pub fn cohere(model_name: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model_type: EmbeddingModelType::Cohere,
            model_name: model_name.into(),
            dimensions: 1024, // Default for embed-english-v3.0
            api_key: Some(api_key.into()),
            api_base_url: None,
        }
    }

    /// Creates a mock configuration for testing.
    pub fn mock(dimensions: usize) -> Self {
        Self {
            model_type: EmbeddingModelType::Mock,
            model_name: "mock".to_string(),
            dimensions,
            api_key: None,
            api_base_url: None,
        }
    }

    /// Sets the API base URL.
    pub fn with_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.api_base_url = Some(url.into());
        self
    }
}

/// Metadata associated with an embedding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// The type of element this embedding represents.
    pub element_type: EmbeddingElementType,

    /// The name of the element.
    pub name: String,

    /// Description of the element.
    #[serde(default)]
    pub description: String,

    /// Path for nested elements (e.g., "actor.user.name").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Sample values for the element.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_values: Vec<String>,

    /// Additional properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
}

/// Types of elements that can be embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingElementType {
    /// An OCSF attribute.
    OcsfAttribute,
    /// An OCSF event class.
    OcsfClass,
    /// An OCSF object.
    OcsfObject,
    /// An OCSF category.
    OcsfCategory,
    /// A semantic entity.
    SemanticEntity,
    /// A semantic attribute.
    SemanticAttribute,
}

impl EmbeddingMetadata {
    /// Creates new metadata for an OCSF attribute.
    pub fn ocsf_attribute(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            element_type: EmbeddingElementType::OcsfAttribute,
            name: name.into(),
            description: description.into(),
            path: None,
            sample_values: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Creates new metadata for an OCSF class.
    pub fn ocsf_class(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            element_type: EmbeddingElementType::OcsfClass,
            name: name.into(),
            description: description.into(),
            path: None,
            sample_values: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Creates new metadata for a semantic entity.
    pub fn semantic_entity(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            element_type: EmbeddingElementType::SemanticEntity,
            name: name.into(),
            description: description.into(),
            path: None,
            sample_values: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Creates new metadata for a semantic attribute.
    pub fn semantic_attribute(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            element_type: EmbeddingElementType::SemanticAttribute,
            name: name.into(),
            description: description.into(),
            path: None,
            sample_values: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Sets the path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets sample values.
    pub fn with_sample_values(mut self, values: Vec<String>) -> Self {
        self.sample_values = values;
        self
    }

    /// Adds a property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// A single embedding with its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedding {
    /// Unique identifier for this embedding.
    pub id: String,

    /// The embedding vector.
    pub vector: Vec<f32>,

    /// Metadata about the embedded element.
    pub metadata: EmbeddingMetadata,
}

impl Embedding {
    /// Creates a new embedding.
    pub fn new(id: impl Into<String>, vector: Vec<f32>, metadata: EmbeddingMetadata) -> Self {
        Self {
            id: id.into(),
            vector,
            metadata,
        }
    }

    /// Returns the dimension of the embedding vector.
    pub fn dimensions(&self) -> usize {
        self.vector.len()
    }
}

/// Trait for embedding generation.
///
/// Implementations of this trait generate vector embeddings from text input.
/// Different implementations can use different embedding models (sentence-transformers,
/// OpenAI, Cohere, etc.).
#[allow(async_fn_in_trait)]
pub trait EmbeddingGenerator: Send + Sync {
    /// Returns the configuration for this generator.
    fn config(&self) -> &EmbeddingModelConfig;

    /// Generates an embedding for a single text input.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generates embeddings for multiple text inputs.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Returns the dimension of the output vectors.
    fn dimensions(&self) -> usize {
        self.config().dimensions
    }

    /// Returns the model name.
    fn model_name(&self) -> &str {
        &self.config().model_name
    }
}


/// A mock embedding generator for testing.
///
/// This generator produces deterministic embeddings based on the hash of the input text,
/// which is useful for testing without requiring an actual embedding model.
#[derive(Debug, Clone)]
pub struct MockEmbeddingGenerator {
    config: EmbeddingModelConfig,
}

impl MockEmbeddingGenerator {
    /// Creates a new mock embedding generator.
    pub fn new(dimensions: usize) -> Self {
        Self {
            config: EmbeddingModelConfig::mock(dimensions),
        }
    }

    /// Creates a mock generator with the default dimension (384).
    pub fn default_dimensions() -> Self {
        Self::new(384)
    }

    /// Generates a deterministic embedding from text using a simple hash-based approach.
    fn generate_deterministic_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut result = Vec::with_capacity(self.config.dimensions);
        
        // Use multiple hash seeds to generate different values for each dimension
        for i in 0..self.config.dimensions {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();
            
            // Convert hash to a float in range [-1, 1]
            let value = ((hash as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32;
            result.push(value);
        }

        // Normalize the vector to unit length
        let magnitude: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut result {
                *v /= magnitude;
            }
        }

        result
    }
}

impl EmbeddingGenerator for MockEmbeddingGenerator {
    fn config(&self) -> &EmbeddingModelConfig {
        &self.config
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Text cannot be empty".to_string()));
        }
        Ok(self.generate_deterministic_embedding(text))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}

/// OpenAI embedding generator.
///
/// This generator uses the OpenAI API to generate embeddings.
/// Requires an API key to be configured.
#[derive(Debug, Clone)]
pub struct OpenAIEmbeddingGenerator {
    config: EmbeddingModelConfig,
    client: reqwest::Client,
}

impl OpenAIEmbeddingGenerator {
    /// Creates a new OpenAI embedding generator.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_model("text-embedding-ada-002", api_key)
    }

    /// Creates a new OpenAI embedding generator with a specific model.
    pub fn with_model(model_name: impl Into<String>, api_key: impl Into<String>) -> Self {
        let model_name = model_name.into();
        let dimensions = match model_name.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536,
        };

        Self {
            config: EmbeddingModelConfig {
                model_type: EmbeddingModelType::OpenAI,
                model_name,
                dimensions,
                api_key: Some(api_key.into()),
                api_base_url: Some("https://api.openai.com/v1".to_string()),
            },
            client: reqwest::Client::new(),
        }
    }

    /// Sets a custom API base URL (for Azure OpenAI or proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.api_base_url = Some(base_url.into());
        self
    }
}

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

impl EmbeddingGenerator for OpenAIEmbeddingGenerator {
    fn config(&self) -> &EmbeddingModelConfig {
        &self.config
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let results = self.embed_batch(&[text]).await?;
        results.into_iter().next().ok_or_else(|| {
            EmbeddingError::ApiError("No embedding returned".to_string())
        })
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        for text in texts {
            if text.is_empty() {
                return Err(EmbeddingError::InvalidInput("Text cannot be empty".to_string()));
            }
        }

        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            EmbeddingError::ConfigError("API key not configured".to_string())
        })?;

        let base_url = self.config.api_base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1");

        let request = OpenAIEmbeddingRequest {
            model: self.config.model_name.clone(),
            input: texts.iter().map(|s| s.to_string()).collect(),
        };

        let response = self.client
            .post(format!("{}/embeddings", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let response: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;

        Ok(response.data.into_iter().map(|d| d.embedding).collect())
    }
}

/// An enum-based embedding generator that wraps different implementations.
///
/// This is used instead of `Box<dyn EmbeddingGenerator>` because async traits
/// are not dyn-compatible.
#[derive(Debug, Clone)]
pub enum AnyEmbeddingGenerator {
    /// Mock embedding generator.
    Mock(MockEmbeddingGenerator),
    /// OpenAI embedding generator.
    OpenAI(OpenAIEmbeddingGenerator),
}

impl AnyEmbeddingGenerator {
    /// Returns the configuration for this generator.
    pub fn config(&self) -> &EmbeddingModelConfig {
        match self {
            Self::Mock(g) => g.config(),
            Self::OpenAI(g) => g.config(),
        }
    }

    /// Generates an embedding for a single text input.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        match self {
            Self::Mock(g) => g.embed(text).await,
            Self::OpenAI(g) => g.embed(text).await,
        }
    }

    /// Generates embeddings for multiple text inputs.
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        match self {
            Self::Mock(g) => g.embed_batch(texts).await,
            Self::OpenAI(g) => g.embed_batch(texts).await,
        }
    }

    /// Returns the dimension of the output vectors.
    pub fn dimensions(&self) -> usize {
        self.config().dimensions
    }

    /// Returns the model name.
    pub fn model_name(&self) -> &str {
        &self.config().model_name
    }
}

/// Creates an embedding generator based on the configuration.
pub fn create_embedding_generator(config: &EmbeddingModelConfig) -> Result<AnyEmbeddingGenerator, EmbeddingError> {
    match config.model_type {
        EmbeddingModelType::Mock => {
            Ok(AnyEmbeddingGenerator::Mock(MockEmbeddingGenerator::new(config.dimensions)))
        }
        EmbeddingModelType::OpenAI => {
            let api_key = config.api_key.as_ref().ok_or_else(|| {
                EmbeddingError::ConfigError("OpenAI API key required".to_string())
            })?;
            let mut generator = OpenAIEmbeddingGenerator::with_model(&config.model_name, api_key);
            if let Some(base_url) = &config.api_base_url {
                generator = generator.with_base_url(base_url);
            }
            Ok(AnyEmbeddingGenerator::OpenAI(generator))
        }
        EmbeddingModelType::SentenceTransformers => {
            // For now, fall back to mock - sentence transformers would require
            // candle or ort integration which is more complex
            Err(EmbeddingError::ModelNotAvailable(
                "Sentence transformers not yet implemented - use Mock or OpenAI".to_string()
            ))
        }
        EmbeddingModelType::Cohere => {
            // Cohere implementation would be similar to OpenAI
            Err(EmbeddingError::ModelNotAvailable(
                "Cohere embeddings not yet implemented - use Mock or OpenAI".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_generator() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let embedding = generator.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Verify normalization (unit vector)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_embedding_consistency() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let embedding1 = generator.embed("test text").await.unwrap();
        let embedding2 = generator.embed("test text").await.unwrap();
        
        // Same input should produce same output
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_embedding_different_inputs() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let embedding1 = generator.embed("test text 1").await.unwrap();
        let embedding2 = generator.embed("test text 2").await.unwrap();
        
        // Different inputs should produce different outputs
        assert_ne!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_embedding_batch() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let embeddings = generator.embed_batch(&["text1", "text2", "text3"]).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        
        // Each embedding should have correct dimensions
        for embedding in &embeddings {
            assert_eq!(embedding.len(), 384);
        }
    }

    #[tokio::test]
    async fn test_mock_embedding_empty_input() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let result = generator.embed("").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_embedding_metadata() {
        let metadata = EmbeddingMetadata::ocsf_attribute("user_name", "The name of the user")
            .with_path("actor.user.name")
            .with_sample_values(vec!["john_doe".to_string()])
            .with_property("type", "string_t");

        assert_eq!(metadata.element_type, EmbeddingElementType::OcsfAttribute);
        assert_eq!(metadata.name, "user_name");
        assert_eq!(metadata.path, Some("actor.user.name".to_string()));
        assert_eq!(metadata.sample_values, vec!["john_doe"]);
        assert_eq!(metadata.properties.get("type"), Some(&"string_t".to_string()));
    }

    #[test]
    fn test_embedding_config_defaults() {
        let config = EmbeddingModelConfig::default();
        assert_eq!(config.model_type, EmbeddingModelType::Mock);
        assert_eq!(config.dimensions, 384);
    }

    #[test]
    fn test_embedding_config_openai() {
        let config = EmbeddingModelConfig::openai("text-embedding-ada-002", "test-key");
        assert_eq!(config.model_type, EmbeddingModelType::OpenAI);
        assert_eq!(config.dimensions, 1536);
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_create_mock_generator() {
        let config = EmbeddingModelConfig::mock(256);
        let generator = create_embedding_generator(&config).unwrap();
        assert_eq!(generator.dimensions(), 256);
    }
}
