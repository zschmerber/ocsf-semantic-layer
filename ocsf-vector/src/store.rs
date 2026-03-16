//! Vector store for embedding storage and similarity search.
//!
//! This module provides an in-memory vector store with cosine similarity search
//! for finding semantically similar schema elements.

use crate::embedding::{Embedding, EmbeddingElementType, EmbeddingError, EmbeddingMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during vector store operations.
#[derive(Debug, Error)]
pub enum VectorStoreError {
    /// The embedding was not found.
    #[error("Embedding not found: {0}")]
    NotFound(String),

    /// Vector dimension mismatch.
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// Storage capacity exceeded.
    #[error("Storage capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Invalid query.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// Embedding error.
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingError),
}

/// Filter for vector similarity search.
#[derive(Debug, Clone, Default)]
pub struct VectorFilter {
    /// Filter by element types.
    pub element_types: Option<Vec<EmbeddingElementType>>,

    /// Filter by metadata properties.
    pub properties: HashMap<String, String>,

    /// Minimum similarity score threshold.
    pub min_score: Option<f32>,
}

impl VectorFilter {
    /// Creates a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by element types.
    pub fn with_element_types(mut self, types: Vec<EmbeddingElementType>) -> Self {
        self.element_types = Some(types);
        self
    }

    /// Adds a property filter.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Sets the minimum similarity score.
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Checks if an embedding matches this filter.
    pub fn matches(&self, embedding: &Embedding) -> bool {
        // Check element type filter
        if let Some(types) = &self.element_types {
            if !types.contains(&embedding.metadata.element_type) {
                return false;
            }
        }

        // Check property filters
        for (key, value) in &self.properties {
            match embedding.metadata.properties.get(key) {
                Some(v) if v == value => {}
                _ => return false,
            }
        }

        true
    }
}

/// Result of a similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    /// The ID of the matching embedding.
    pub id: String,

    /// The similarity score (0-1, higher is more similar).
    pub score: f32,

    /// Metadata of the matching embedding.
    pub metadata: EmbeddingMetadata,
}

impl SimilarityResult {
    /// Creates a new similarity result.
    pub fn new(id: String, score: f32, metadata: EmbeddingMetadata) -> Self {
        Self { id, score, metadata }
    }
}

/// Trait for vector storage and similarity search.
#[allow(async_fn_in_trait)]
pub trait VectorStore: Send + Sync {
    /// Stores embeddings in the vector store.
    async fn store(&mut self, embeddings: Vec<Embedding>) -> Result<(), VectorStoreError>;

    /// Stores a single embedding.
    async fn store_one(&mut self, embedding: Embedding) -> Result<(), VectorStoreError> {
        self.store(vec![embedding]).await
    }

    /// Performs similarity search with a query vector.
    async fn similarity_search(
        &self,
        query: &[f32],
        top_k: usize,
        filter: Option<&VectorFilter>,
    ) -> Result<Vec<SimilarityResult>, VectorStoreError>;

    /// Gets an embedding by ID.
    async fn get(&self, id: &str) -> Result<Option<Embedding>, VectorStoreError>;

    /// Deletes embeddings by ID.
    async fn delete(&mut self, ids: &[String]) -> Result<(), VectorStoreError>;

    /// Clears all embeddings from the store.
    async fn clear(&mut self) -> Result<(), VectorStoreError>;

    /// Returns the number of embeddings in the store.
    fn len(&self) -> usize;

    /// Returns true if the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the expected vector dimensions.
    fn dimensions(&self) -> usize;
}

/// In-memory vector store implementation.
///
/// This store keeps all embeddings in memory and performs brute-force
/// cosine similarity search. Suitable for small to medium datasets.
#[derive(Debug, Clone)]
pub struct InMemoryVectorStore {
    /// Stored embeddings indexed by ID.
    embeddings: HashMap<String, Embedding>,

    /// Expected vector dimensions.
    dimensions: usize,

    /// Maximum capacity (0 = unlimited).
    max_capacity: usize,
}

impl InMemoryVectorStore {
    /// Creates a new in-memory vector store.
    pub fn new(dimensions: usize) -> Self {
        Self {
            embeddings: HashMap::new(),
            dimensions,
            max_capacity: 0,
        }
    }

    /// Creates a new in-memory vector store with a maximum capacity.
    pub fn with_capacity(dimensions: usize, max_capacity: usize) -> Self {
        Self {
            embeddings: HashMap::with_capacity(max_capacity),
            dimensions,
            max_capacity,
        }
    }

    /// Computes cosine similarity between two vectors.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// Returns all embeddings in the store.
    pub fn all_embeddings(&self) -> impl Iterator<Item = &Embedding> {
        self.embeddings.values()
    }

    /// Returns embeddings filtered by element type.
    pub fn embeddings_by_type(&self, element_type: EmbeddingElementType) -> Vec<&Embedding> {
        self.embeddings
            .values()
            .filter(|e| e.metadata.element_type == element_type)
            .collect()
    }
}

impl VectorStore for InMemoryVectorStore {
    async fn store(&mut self, embeddings: Vec<Embedding>) -> Result<(), VectorStoreError> {
        // Check capacity
        if self.max_capacity > 0 && self.embeddings.len() + embeddings.len() > self.max_capacity {
            return Err(VectorStoreError::CapacityExceeded(format!(
                "Cannot store {} embeddings, would exceed capacity of {}",
                embeddings.len(),
                self.max_capacity
            )));
        }

        // Validate and store embeddings
        for embedding in embeddings {
            if embedding.vector.len() != self.dimensions {
                return Err(VectorStoreError::DimensionMismatch {
                    expected: self.dimensions,
                    actual: embedding.vector.len(),
                });
            }
            self.embeddings.insert(embedding.id.clone(), embedding);
        }

        Ok(())
    }

    async fn similarity_search(
        &self,
        query: &[f32],
        top_k: usize,
        filter: Option<&VectorFilter>,
    ) -> Result<Vec<SimilarityResult>, VectorStoreError> {
        if query.len() != self.dimensions {
            return Err(VectorStoreError::DimensionMismatch {
                expected: self.dimensions,
                actual: query.len(),
            });
        }

        if top_k == 0 {
            return Ok(Vec::new());
        }

        // Calculate similarities for all embeddings
        let mut results: Vec<SimilarityResult> = self
            .embeddings
            .values()
            .filter(|e| filter.map_or(true, |f| f.matches(e)))
            .map(|e| {
                let score = Self::cosine_similarity(query, &e.vector);
                SimilarityResult::new(e.id.clone(), score, e.metadata.clone())
            })
            .filter(|r| {
                filter
                    .and_then(|f| f.min_score)
                    .map_or(true, |min| r.score >= min)
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k results
        results.truncate(top_k);

        Ok(results)
    }

    async fn get(&self, id: &str) -> Result<Option<Embedding>, VectorStoreError> {
        Ok(self.embeddings.get(id).cloned())
    }

    async fn delete(&mut self, ids: &[String]) -> Result<(), VectorStoreError> {
        for id in ids {
            self.embeddings.remove(id);
        }
        Ok(())
    }

    async fn clear(&mut self) -> Result<(), VectorStoreError> {
        self.embeddings.clear();
        Ok(())
    }

    fn len(&self) -> usize {
        self.embeddings.len()
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Serializable state for the in-memory vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreState {
    /// The embeddings in the store.
    pub embeddings: Vec<Embedding>,

    /// The expected vector dimensions.
    pub dimensions: usize,
}

impl InMemoryVectorStore {
    /// Exports the store state for persistence.
    pub fn export_state(&self) -> VectorStoreState {
        VectorStoreState {
            embeddings: self.embeddings.values().cloned().collect(),
            dimensions: self.dimensions,
        }
    }

    /// Imports state from a persisted store.
    pub fn import_state(state: VectorStoreState) -> Result<Self, VectorStoreError> {
        let mut store = Self::new(state.dimensions);
        for embedding in state.embeddings {
            if embedding.vector.len() != state.dimensions {
                return Err(VectorStoreError::DimensionMismatch {
                    expected: state.dimensions,
                    actual: embedding.vector.len(),
                });
            }
            store.embeddings.insert(embedding.id.clone(), embedding);
        }
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding(id: &str, vector: Vec<f32>) -> Embedding {
        Embedding::new(
            id,
            vector,
            EmbeddingMetadata::ocsf_attribute("test", "test description"),
        )
    }

    fn create_normalized_vector(seed: u64, dimensions: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut result = Vec::with_capacity(dimensions);
        for i in 0..dimensions {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();
            let value = ((hash as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32;
            result.push(value);
        }

        // Normalize
        let magnitude: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut result {
                *v /= magnitude;
            }
        }

        result
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let mut store = InMemoryVectorStore::new(3);
        
        let embedding = create_test_embedding("test1", vec![1.0, 0.0, 0.0]);
        store.store_one(embedding.clone()).await.unwrap();
        
        let retrieved = store.get("test1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test1");
    }

    #[tokio::test]
    async fn test_dimension_mismatch() {
        let mut store = InMemoryVectorStore::new(3);
        
        let embedding = create_test_embedding("test1", vec![1.0, 0.0]); // Wrong dimensions
        let result = store.store_one(embedding).await;
        
        assert!(matches!(result, Err(VectorStoreError::DimensionMismatch { .. })));
    }

    #[tokio::test]
    async fn test_similarity_search_ordering() {
        let mut store = InMemoryVectorStore::new(3);
        
        // Store embeddings with known vectors
        store.store_one(create_test_embedding("exact", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("similar", vec![0.9, 0.1, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("different", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        // Search with query similar to "exact"
        let query = vec![1.0, 0.0, 0.0];
        let results = store.similarity_search(&query, 3, None).await.unwrap();
        
        assert_eq!(results.len(), 3);
        // Results should be ordered by descending similarity
        assert!(results[0].score >= results[1].score);
        assert!(results[1].score >= results[2].score);
        // "exact" should be first (highest similarity)
        assert_eq!(results[0].id, "exact");
    }

    #[tokio::test]
    async fn test_similarity_search_with_filter() {
        let mut store = InMemoryVectorStore::new(3);
        
        let mut attr_embedding = create_test_embedding("attr1", vec![1.0, 0.0, 0.0]);
        attr_embedding.metadata.element_type = EmbeddingElementType::OcsfAttribute;
        
        let mut class_embedding = create_test_embedding("class1", vec![0.9, 0.1, 0.0]);
        class_embedding.metadata.element_type = EmbeddingElementType::OcsfClass;
        
        store.store_one(attr_embedding).await.unwrap();
        store.store_one(class_embedding).await.unwrap();
        
        // Filter by element type
        let filter = VectorFilter::new()
            .with_element_types(vec![EmbeddingElementType::OcsfAttribute]);
        
        let query = vec![1.0, 0.0, 0.0];
        let results = store.similarity_search(&query, 10, Some(&filter)).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "attr1");
    }

    #[tokio::test]
    async fn test_similarity_search_min_score() {
        let mut store = InMemoryVectorStore::new(3);
        
        store.store_one(create_test_embedding("high", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("low", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        let filter = VectorFilter::new().with_min_score(0.5);
        
        let query = vec![1.0, 0.0, 0.0];
        let results = store.similarity_search(&query, 10, Some(&filter)).await.unwrap();
        
        // Only "high" should pass the min_score filter
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "high");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut store = InMemoryVectorStore::new(3);
        
        store.store_one(create_test_embedding("test1", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("test2", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        assert_eq!(store.len(), 2);
        
        store.delete(&["test1".to_string()]).await.unwrap();
        
        assert_eq!(store.len(), 1);
        assert!(store.get("test1").await.unwrap().is_none());
        assert!(store.get("test2").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_clear() {
        let mut store = InMemoryVectorStore::new(3);
        
        store.store_one(create_test_embedding("test1", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("test2", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        store.clear().await.unwrap();
        
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn test_capacity_limit() {
        let mut store = InMemoryVectorStore::with_capacity(3, 2);
        
        store.store_one(create_test_embedding("test1", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("test2", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        let result = store.store_one(create_test_embedding("test3", vec![0.0, 0.0, 1.0])).await;
        
        assert!(matches!(result, Err(VectorStoreError::CapacityExceeded(_))));
    }

    #[tokio::test]
    async fn test_export_import_state() {
        let mut store = InMemoryVectorStore::new(3);
        
        store.store_one(create_test_embedding("test1", vec![1.0, 0.0, 0.0])).await.unwrap();
        store.store_one(create_test_embedding("test2", vec![0.0, 1.0, 0.0])).await.unwrap();
        
        let state = store.export_state();
        let imported = InMemoryVectorStore::import_state(state).unwrap();
        
        assert_eq!(imported.len(), 2);
        assert!(imported.get("test1").await.unwrap().is_some());
        assert!(imported.get("test2").await.unwrap().is_some());
    }

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors should have similarity 1.0
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((InMemoryVectorStore::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        // Orthogonal vectors should have similarity 0.0
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(InMemoryVectorStore::cosine_similarity(&a, &b).abs() < 0.001);

        // Opposite vectors should have similarity -1.0
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((InMemoryVectorStore::cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_top_k_limit() {
        let mut store = InMemoryVectorStore::new(384);
        
        // Store many embeddings
        for i in 0..10 {
            let vector = create_normalized_vector(i, 384);
            store.store_one(create_test_embedding(&format!("test{}", i), vector)).await.unwrap();
        }
        
        let query = create_normalized_vector(0, 384);
        let results = store.similarity_search(&query, 3, None).await.unwrap();
        
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_vector_filter() {
        let filter = VectorFilter::new()
            .with_element_types(vec![EmbeddingElementType::OcsfAttribute])
            .with_property("type", "string_t")
            .with_min_score(0.5);

        let mut matching = Embedding::new(
            "test",
            vec![1.0],
            EmbeddingMetadata::ocsf_attribute("test", "desc")
                .with_property("type", "string_t"),
        );
        matching.metadata.element_type = EmbeddingElementType::OcsfAttribute;

        assert!(filter.matches(&matching));

        // Wrong element type
        let mut wrong_type = matching.clone();
        wrong_type.metadata.element_type = EmbeddingElementType::OcsfClass;
        assert!(!filter.matches(&wrong_type));

        // Wrong property
        let mut wrong_prop = matching.clone();
        wrong_prop.metadata.properties.insert("type".to_string(), "integer_t".to_string());
        assert!(!filter.matches(&wrong_prop));
    }
}
