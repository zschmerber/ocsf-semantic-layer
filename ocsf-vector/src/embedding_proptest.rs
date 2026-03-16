//! Property-based tests for embedding generation.
//!
//! **Feature: ocsf-semantic-layer, Property 7: Embedding Generation Consistency**
//! **Validates: Requirements 5.1, 5.2**

use crate::embedding::{EmbeddingGenerator, MockEmbeddingGenerator};
use proptest::prelude::*;

/// Strategy for generating non-empty text strings suitable for embedding.
fn text_strategy() -> impl Strategy<Value = String> {
    // Generate non-empty strings with printable ASCII characters
    "[a-zA-Z0-9 _-]{1,100}".prop_filter("non-empty", |s| !s.trim().is_empty())
}

/// Strategy for generating embedding dimensions.
fn dimensions_strategy() -> impl Strategy<Value = usize> {
    prop_oneof![
        Just(128),
        Just(256),
        Just(384),
        Just(512),
        Just(768),
        Just(1024),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 7: Embedding Generation Consistency**
    ///
    /// *For any* OCSF attribute or semantic entity, generating embeddings multiple times
    /// with the same model should produce identical vectors.
    ///
    /// **Validates: Requirements 5.1, 5.2**
    #[test]
    fn prop_embedding_consistency(
        text in text_strategy(),
        dimensions in dimensions_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(dimensions);

            // Generate embedding twice for the same input
            let embedding1 = generator.embed(&text).await.unwrap();
            let embedding2 = generator.embed(&text).await.unwrap();

            // Embeddings should be identical
            prop_assert_eq!(embedding1.len(), dimensions);
            prop_assert_eq!(embedding2.len(), dimensions);
            prop_assert_eq!(embedding1, embedding2, "Same input should produce same embedding");

            Ok(())
        })?;
    }

    /// Property: Embeddings should be normalized (unit vectors).
    #[test]
    fn prop_embedding_normalized(
        text in text_strategy(),
        dimensions in dimensions_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(dimensions);
            let embedding = generator.embed(&text).await.unwrap();

            // Calculate magnitude
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            // Should be approximately 1.0 (unit vector)
            prop_assert!(
                (magnitude - 1.0).abs() < 0.001,
                "Embedding should be normalized, got magnitude {}",
                magnitude
            );

            Ok(())
        })?;
    }

    /// Property: Different inputs should produce different embeddings.
    #[test]
    fn prop_different_inputs_different_embeddings(
        text1 in text_strategy(),
        text2 in text_strategy(),
        dimensions in dimensions_strategy()
    ) {
        // Skip if texts are the same
        prop_assume!(text1 != text2);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(dimensions);

            let embedding1 = generator.embed(&text1).await.unwrap();
            let embedding2 = generator.embed(&text2).await.unwrap();

            // Different inputs should produce different embeddings
            prop_assert_ne!(
                embedding1, embedding2,
                "Different inputs should produce different embeddings"
            );

            Ok(())
        })?;
    }

    /// Property: Batch embedding should produce same results as individual embedding.
    #[test]
    fn prop_batch_embedding_consistency(
        texts in prop::collection::vec(text_strategy(), 1..5),
        dimensions in dimensions_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(dimensions);

            // Generate embeddings individually
            let mut individual_embeddings = Vec::new();
            for text in &texts {
                individual_embeddings.push(generator.embed(text).await.unwrap());
            }

            // Generate embeddings in batch
            let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            let batch_embeddings = generator.embed_batch(&text_refs).await.unwrap();

            // Results should be identical
            prop_assert_eq!(
                individual_embeddings.len(),
                batch_embeddings.len(),
                "Batch should return same number of embeddings"
            );

            for (i, (individual, batch)) in individual_embeddings.iter().zip(batch_embeddings.iter()).enumerate() {
                prop_assert_eq!(
                    individual, batch,
                    "Embedding {} should be identical in batch and individual generation",
                    i
                );
            }

            Ok(())
        })?;
    }

    /// Property: Embedding dimensions should match configuration.
    #[test]
    fn prop_embedding_dimensions_match_config(
        text in text_strategy(),
        dimensions in dimensions_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(dimensions);
            let embedding = generator.embed(&text).await.unwrap();

            prop_assert_eq!(
                embedding.len(),
                dimensions,
                "Embedding dimensions should match configured dimensions"
            );

            prop_assert_eq!(
                generator.dimensions(),
                dimensions,
                "Generator dimensions() should return configured dimensions"
            );

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_consistency_example() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let text = "user authentication event";
        let embedding1 = generator.embed(text).await.unwrap();
        let embedding2 = generator.embed(text).await.unwrap();
        
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_embedding_normalized_example() {
        let generator = MockEmbeddingGenerator::new(384);
        
        let embedding = generator.embed("test text").await.unwrap();
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        assert!((magnitude - 1.0).abs() < 0.001);
    }
}
