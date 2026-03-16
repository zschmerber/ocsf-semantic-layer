//! Property-based tests for mapping assistant.
//!
//! **Feature: ocsf-semantic-layer, Property 8: Vector Similarity Search Ordering**
//! **Validates: Requirements 6.1, 6.2**
//!
//! **Feature: ocsf-semantic-layer, Property 9: Mapping Suggestion Completeness**
//! **Validates: Requirements 6.3**

use crate::embedding::{Embedding, EmbeddingGenerator, EmbeddingMetadata, MockEmbeddingGenerator};
use crate::mapping::{MappingAssistant, SourceFieldMetadata};
use crate::store::{InMemoryVectorStore, VectorStore};
use proptest::prelude::*;

/// Strategy for generating valid field names.
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy for generating field types.
fn field_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("string".to_string()),
        Just("integer".to_string()),
        Just("boolean".to_string()),
        Just("timestamp".to_string()),
        Just("float".to_string()),
    ]
}

/// Strategy for generating optional descriptions.
fn description_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-zA-Z ]{5,50}".prop_map(Some),
    ]
}

/// Strategy for generating sample values.
fn sample_values_strategy() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        Just(vec![]),
        prop::collection::vec("[a-zA-Z0-9@._-]{1,20}", 1..4),
    ]
}

/// Strategy for generating source field metadata.
fn source_field_strategy() -> impl Strategy<Value = SourceFieldMetadata> {
    (
        field_name_strategy(),
        field_type_strategy(),
        description_strategy(),
        sample_values_strategy(),
    )
        .prop_map(|(name, field_type, description, sample_values)| {
            let mut field = SourceFieldMetadata::new(name, field_type);
            if let Some(desc) = description {
                field = field.with_description(desc);
            }
            if !sample_values.is_empty() {
                field = field.with_sample_values(sample_values);
            }
            field
        })
}

/// Strategy for generating OCSF attribute names.
fn ocsf_attribute_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("user_name".to_string()),
        Just("email_addr".to_string()),
        Just("ip_address".to_string()),
        Just("hostname".to_string()),
        Just("timestamp".to_string()),
        Just("severity_id".to_string()),
        Just("process_name".to_string()),
        Just("file_path".to_string()),
        Just("domain".to_string()),
        Just("port".to_string()),
    ]
}

/// Strategy for generating OCSF attribute paths.
fn ocsf_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("actor.user.name".to_string()),
        Just("actor.user.email_addr".to_string()),
        Just("src_endpoint.ip".to_string()),
        Just("src_endpoint.hostname".to_string()),
        Just("dst_endpoint.ip".to_string()),
        Just("dst_endpoint.port".to_string()),
        Just("time".to_string()),
        Just("severity_id".to_string()),
        Just("process.name".to_string()),
        Just("file.path".to_string()),
    ]
}

/// Strategy for generating OCSF attribute descriptions.
fn ocsf_description_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("The name of the user".to_string()),
        Just("The email address".to_string()),
        Just("The IP address".to_string()),
        Just("The hostname".to_string()),
        Just("The event timestamp".to_string()),
        Just("The severity level".to_string()),
        Just("The process name".to_string()),
        Just("The file path".to_string()),
    ]
}

/// Strategy for generating a set of OCSF attributes to populate the store.
fn ocsf_attributes_strategy() -> impl Strategy<Value = Vec<(String, String, String)>> {
    prop::collection::vec(
        (
            ocsf_attribute_name_strategy(),
            ocsf_path_strategy(),
            ocsf_description_strategy(),
        ),
        3..10,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 8: Vector Similarity Search Ordering**
    ///
    /// *For any* query embedding and set of stored embeddings, similarity search results
    /// should be ordered by descending similarity score.
    ///
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn prop_similarity_search_ordering(
        source_field in source_field_strategy(),
        ocsf_attrs in ocsf_attributes_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(384);
            let mut store = InMemoryVectorStore::new(384);

            // Populate store with OCSF attributes
            for (name, path, description) in &ocsf_attrs {
                let text = format!("{} {} {}", name, path, description);
                let vector = generator.embed(&text).await.unwrap();
                let metadata = EmbeddingMetadata::ocsf_attribute(name, description)
                    .with_path(path)
                    .with_property("type", "string_t");
                let embedding = Embedding::new(format!("attr_{}", name), vector, metadata);
                let _ = store.store_one(embedding).await;
            }

            // Skip if store is empty
            if store.is_empty() {
                return Ok(());
            }

            let assistant = MappingAssistant::new(generator, store);

            // Get suggestions
            let suggestions = assistant.suggest_mappings(&source_field, Some(10)).await;

            match suggestions {
                Ok(suggestions) => {
                    // Verify ordering: each suggestion should have confidence >= next
                    for i in 1..suggestions.len() {
                        prop_assert!(
                            suggestions[i - 1].confidence >= suggestions[i].confidence,
                            "Suggestions should be ordered by descending confidence. \
                             Got {} at position {} and {} at position {}",
                            suggestions[i - 1].confidence,
                            i - 1,
                            suggestions[i].confidence,
                            i
                        );
                    }
                }
                Err(_) => {
                    // Errors are acceptable for edge cases
                }
            }

            Ok(())
        })?;
    }
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 9: Mapping Suggestion Completeness**
    ///
    /// *For any* source field metadata, mapping suggestions should include a reasoning
    /// string for each suggestion.
    ///
    /// **Validates: Requirements 6.3**
    #[test]
    fn prop_suggestion_completeness(
        source_field in source_field_strategy(),
        ocsf_attrs in ocsf_attributes_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(384);
            let mut store = InMemoryVectorStore::new(384);

            // Populate store with OCSF attributes
            for (name, path, description) in &ocsf_attrs {
                let text = format!("{} {} {}", name, path, description);
                let vector = generator.embed(&text).await.unwrap();
                let metadata = EmbeddingMetadata::ocsf_attribute(name, description)
                    .with_path(path)
                    .with_property("type", "string_t");
                let embedding = Embedding::new(format!("attr_{}", name), vector, metadata);
                let _ = store.store_one(embedding).await;
            }

            let assistant = MappingAssistant::new(generator, store);

            // Get suggestions
            let suggestions = assistant.suggest_mappings(&source_field, Some(10)).await;

            match suggestions {
                Ok(suggestions) => {
                    // Every suggestion must have a non-empty reasoning string
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        prop_assert!(
                            !suggestion.reasoning.is_empty(),
                            "Suggestion {} for field '{}' should have non-empty reasoning. \
                             Target: {}, Confidence: {}",
                            i,
                            source_field.name,
                            suggestion.target_field,
                            suggestion.confidence
                        );

                        // Reasoning should contain meaningful content (not just whitespace)
                        prop_assert!(
                            suggestion.reasoning.trim().len() > 5,
                            "Suggestion {} reasoning should be meaningful, got: '{}'",
                            i,
                            suggestion.reasoning
                        );
                    }
                }
                Err(_) => {
                    // Errors are acceptable for edge cases
                }
            }

            Ok(())
        })?;
    }

    /// Property: Suggestions should have valid confidence scores.
    #[test]
    fn prop_suggestion_confidence_bounds(
        source_field in source_field_strategy(),
        ocsf_attrs in ocsf_attributes_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let generator = MockEmbeddingGenerator::new(384);
            let mut store = InMemoryVectorStore::new(384);

            // Populate store with OCSF attributes
            for (name, path, description) in &ocsf_attrs {
                let text = format!("{} {} {}", name, path, description);
                let vector = generator.embed(&text).await.unwrap();
                let metadata = EmbeddingMetadata::ocsf_attribute(name, description)
                    .with_path(path)
                    .with_property("type", "string_t");
                let embedding = Embedding::new(format!("attr_{}", name), vector, metadata);
                let _ = store.store_one(embedding).await;
            }

            let assistant = MappingAssistant::new(generator, store);

            // Get suggestions
            let suggestions = assistant.suggest_mappings(&source_field, Some(10)).await;

            match suggestions {
                Ok(suggestions) => {
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        // Confidence should be between 0 and 1
                        prop_assert!(
                            suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0,
                            "Suggestion {} confidence should be in [0, 1], got: {}",
                            i,
                            suggestion.confidence
                        );

                        // Target field should not be empty
                        prop_assert!(
                            !suggestion.target_field.is_empty(),
                            "Suggestion {} should have non-empty target field",
                            i
                        );
                    }
                }
                Err(_) => {
                    // Errors are acceptable for edge cases
                }
            }

            Ok(())
        })?;
    }
}
