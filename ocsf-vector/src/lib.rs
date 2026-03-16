//! Embeddings and vector store for mapping assistance.
//!
//! This crate provides vector embedding generation and similarity search
//! for AI-assisted field mapping in the OCSF semantic layer.
//!
//! # Features
//!
//! - **Embedding Generation**: Generate vector embeddings for OCSF schema elements
//!   and semantic entities using configurable embedding models (Mock, OpenAI, etc.)
//! - **Vector Store**: In-memory vector store with cosine similarity search
//! - **Schema Embeddings**: Generate embeddings for entire OCSF schemas
//! - **Incremental Updates**: Detect and update only changed schema elements
//!
//! # Example
//!
//! ```rust,no_run
//! use ocsf_vector::{
//!     MockEmbeddingGenerator, InMemoryVectorStore, SchemaEmbeddingGenerator,
//!     VectorStore, EmbeddingGenerator,
//! };
//! use ocsf_core::OCSFSchema;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create an embedding generator
//! let generator = MockEmbeddingGenerator::new(384);
//!
//! // Create a vector store
//! let mut store = InMemoryVectorStore::new(384);
//!
//! // Generate embeddings for a schema
//! let schema = OCSFSchema::new("1.4.0");
//! let schema_gen = SchemaEmbeddingGenerator::new(generator);
//! let embeddings = schema_gen.generate_for_schema(&schema).await?;
//!
//! // Store embeddings
//! for embedding in embeddings.all_embeddings() {
//!     store.store_one(embedding.clone()).await?;
//! }
//!
//! // Search for similar embeddings
//! let query = vec![0.0f32; 384]; // Your query vector
//! let results = store.similarity_search(&query, 5, None).await?;
//! # Ok(())
//! # }
//! ```

pub mod embedding;
pub mod mapping;
pub mod schema_embedding;
pub mod store;
pub mod incremental;

#[cfg(test)]
mod embedding_proptest;

#[cfg(test)]
mod mapping_proptest;

// Re-export main types
pub use embedding::{
    create_embedding_generator, AnyEmbeddingGenerator, Embedding, EmbeddingElementType, EmbeddingError,
    EmbeddingGenerator, EmbeddingMetadata, EmbeddingModelConfig, EmbeddingModelType,
    MockEmbeddingGenerator, OpenAIEmbeddingGenerator,
};

pub use schema_embedding::{EntityEmbedding, SchemaEmbeddingGenerator, SchemaEmbeddings};

pub use store::{
    InMemoryVectorStore, SimilarityResult, VectorFilter, VectorStore, VectorStoreError,
    VectorStoreState,
};

pub use incremental::{IncrementalEmbeddingUpdater, IncrementalEntityUpdater, SchemaChange, ChangeType, UpdateResult, IncrementalUpdateError};

pub use mapping::{
    ConfirmedMapping, ExtensionSuggestion, LearningState, MappingAssistant, MappingAssistantConfig,
    MappingError, MappingSuggestion, SourceFieldMetadata,
};
