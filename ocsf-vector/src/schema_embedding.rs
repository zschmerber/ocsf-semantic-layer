//! Schema embedding generation for OCSF schema elements.
//!
//! This module provides functionality to generate embeddings for OCSF schema
//! elements (attributes, classes, objects) and semantic entities.

use crate::embedding::{
    Embedding, EmbeddingElementType, EmbeddingError, EmbeddingGenerator, EmbeddingMetadata,
};
use ocsf_core::{Attribute, Category, EventClass, OCSFObject, OCSFSchema};
use ocsf_semantic::{SemanticAttribute, SemanticEntity};
use std::collections::HashMap;

/// Embeddings generated from an OCSF schema.
#[derive(Debug, Clone, Default)]
pub struct SchemaEmbeddings {
    /// Embeddings for attributes.
    pub attributes: Vec<Embedding>,

    /// Embeddings for event classes.
    pub event_classes: Vec<Embedding>,

    /// Embeddings for objects.
    pub objects: Vec<Embedding>,

    /// Embeddings for categories.
    pub categories: Vec<Embedding>,
}

impl SchemaEmbeddings {
    /// Returns the total number of embeddings.
    pub fn total_count(&self) -> usize {
        self.attributes.len() + self.event_classes.len() + self.objects.len() + self.categories.len()
    }

    /// Returns all embeddings as a single vector.
    pub fn all_embeddings(&self) -> Vec<&Embedding> {
        let mut all = Vec::with_capacity(self.total_count());
        all.extend(self.attributes.iter());
        all.extend(self.event_classes.iter());
        all.extend(self.objects.iter());
        all.extend(self.categories.iter());
        all
    }

    /// Finds embeddings by element type.
    pub fn by_type(&self, element_type: EmbeddingElementType) -> &[Embedding] {
        match element_type {
            EmbeddingElementType::OcsfAttribute => &self.attributes,
            EmbeddingElementType::OcsfClass => &self.event_classes,
            EmbeddingElementType::OcsfObject => &self.objects,
            EmbeddingElementType::OcsfCategory => &self.categories,
            _ => &[],
        }
    }
}

/// Embeddings generated from a semantic entity.
#[derive(Debug, Clone)]
pub struct EntityEmbedding {
    /// Embedding for the entity itself.
    pub entity: Embedding,

    /// Embeddings for the entity's attributes.
    pub attributes: Vec<Embedding>,
}

impl EntityEmbedding {
    /// Returns the total number of embeddings.
    pub fn total_count(&self) -> usize {
        1 + self.attributes.len()
    }

    /// Returns all embeddings as a single vector.
    pub fn all_embeddings(&self) -> Vec<&Embedding> {
        let mut all = Vec::with_capacity(self.total_count());
        all.push(&self.entity);
        all.extend(self.attributes.iter());
        all
    }
}

/// Generates embeddings for OCSF schema elements and semantic entities.
pub struct SchemaEmbeddingGenerator<G: EmbeddingGenerator> {
    generator: G,
}

impl<G: EmbeddingGenerator> SchemaEmbeddingGenerator<G> {
    /// Creates a new schema embedding generator.
    pub fn new(generator: G) -> Self {
        Self { generator }
    }

    /// Returns a reference to the underlying embedding generator.
    pub fn generator(&self) -> &G {
        &self.generator
    }

    /// Generates embeddings for an entire OCSF schema.
    pub async fn generate_for_schema(&self, schema: &OCSFSchema) -> Result<SchemaEmbeddings, EmbeddingError> {
        let mut embeddings = SchemaEmbeddings::default();

        // Generate embeddings for categories
        for category in schema.all_categories() {
            let embedding = self.generate_for_category(category).await?;
            embeddings.categories.push(embedding);
        }

        // Generate embeddings for event classes
        for event_class in schema.all_event_classes() {
            let embedding = self.generate_for_event_class(event_class).await?;
            embeddings.event_classes.push(embedding);
        }

        // Generate embeddings for objects
        for object in schema.all_objects() {
            let embedding = self.generate_for_object(object).await?;
            embeddings.objects.push(embedding);
        }

        // Generate embeddings for base attributes
        for attribute in schema.all_attributes() {
            let embedding = self.generate_for_attribute(attribute, None).await?;
            embeddings.attributes.push(embedding);
        }

        Ok(embeddings)
    }

    /// Generates an embedding for a category.
    pub async fn generate_for_category(&self, category: &Category) -> Result<Embedding, EmbeddingError> {
        let text = format_category_text(category);
        let vector = self.generator.embed(&text).await?;

        let metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::OcsfCategory,
            name: category.name.clone(),
            description: category.description.clone(),
            path: None,
            sample_values: Vec::new(),
            properties: {
                let mut props = HashMap::new();
                props.insert("uid".to_string(), category.uid.to_string());
                props.insert("caption".to_string(), category.caption.clone());
                props
            },
        };

        Ok(Embedding::new(
            format!("category:{}", category.uid),
            vector,
            metadata,
        ))
    }

    /// Generates an embedding for an event class.
    pub async fn generate_for_event_class(&self, event_class: &EventClass) -> Result<Embedding, EmbeddingError> {
        let text = format_event_class_text(event_class);
        let vector = self.generator.embed(&text).await?;

        let metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::OcsfClass,
            name: event_class.name.clone(),
            description: event_class.description.clone(),
            path: None,
            sample_values: Vec::new(),
            properties: {
                let mut props = HashMap::new();
                props.insert("class_uid".to_string(), event_class.class_uid.to_string());
                props.insert("category_uid".to_string(), event_class.category_uid.to_string());
                props.insert("caption".to_string(), event_class.caption.clone());
                props
            },
        };

        Ok(Embedding::new(
            format!("class:{}", event_class.class_uid),
            vector,
            metadata,
        ))
    }

    /// Generates an embedding for an object.
    pub async fn generate_for_object(&self, object: &OCSFObject) -> Result<Embedding, EmbeddingError> {
        let text = format_object_text(object);
        let vector = self.generator.embed(&text).await?;

        let metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::OcsfObject,
            name: object.name.clone(),
            description: object.description.clone(),
            path: None,
            sample_values: Vec::new(),
            properties: {
                let mut props = HashMap::new();
                props.insert("caption".to_string(), object.caption.clone());
                if let Some(extends) = &object.extends {
                    props.insert("extends".to_string(), extends.clone());
                }
                props
            },
        };

        Ok(Embedding::new(
            format!("object:{}", object.name),
            vector,
            metadata,
        ))
    }

    /// Generates an embedding for an attribute.
    pub async fn generate_for_attribute(
        &self,
        attribute: &Attribute,
        path: Option<&str>,
    ) -> Result<Embedding, EmbeddingError> {
        let text = format_attribute_text(attribute);
        let vector = self.generator.embed(&text).await?;

        let metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::OcsfAttribute,
            name: attribute.name.clone(),
            description: attribute.description.clone(),
            path: path.map(String::from),
            sample_values: Vec::new(),
            properties: {
                let mut props = HashMap::new();
                props.insert("type".to_string(), attribute.attr_type.clone());
                props.insert("caption".to_string(), attribute.caption.clone());
                props.insert("requirement".to_string(), format!("{:?}", attribute.requirement));
                if let Some(observable) = attribute.observable {
                    props.insert("observable".to_string(), observable.to_string());
                }
                props
            },
        };

        let id = if let Some(p) = path {
            format!("attribute:{}:{}", p, attribute.name)
        } else {
            format!("attribute:{}", attribute.name)
        };

        Ok(Embedding::new(id, vector, metadata))
    }

    /// Generates embeddings for a semantic entity.
    pub async fn generate_for_entity(&self, entity: &SemanticEntity) -> Result<EntityEmbedding, EmbeddingError> {
        // Generate embedding for the entity itself
        let entity_text = format_entity_text(entity);
        let entity_vector = self.generator.embed(&entity_text).await?;

        let entity_metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::SemanticEntity,
            name: entity.name.clone(),
            description: entity.description.clone(),
            path: None,
            sample_values: Vec::new(),
            properties: {
                let mut props = HashMap::new();
                props.insert("caption".to_string(), entity.caption.clone());
                props.insert(
                    "source_event_classes".to_string(),
                    entity.source_event_classes.iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                );
                props
            },
        };

        let entity_embedding = Embedding::new(
            format!("entity:{}", entity.name),
            entity_vector,
            entity_metadata,
        );

        // Generate embeddings for each attribute
        let mut attribute_embeddings = Vec::with_capacity(entity.attributes.len());
        for attr in &entity.attributes {
            let attr_embedding = self.generate_for_semantic_attribute(attr, &entity.name).await?;
            attribute_embeddings.push(attr_embedding);
        }

        Ok(EntityEmbedding {
            entity: entity_embedding,
            attributes: attribute_embeddings,
        })
    }

    /// Generates an embedding for a semantic attribute.
    pub async fn generate_for_semantic_attribute(
        &self,
        attribute: &SemanticAttribute,
        entity_name: &str,
    ) -> Result<Embedding, EmbeddingError> {
        let text = format_semantic_attribute_text(attribute);
        let vector = self.generator.embed(&text).await?;

        let metadata = EmbeddingMetadata {
            element_type: EmbeddingElementType::SemanticAttribute,
            name: attribute.name.clone(),
            description: attribute.description.clone(),
            path: attribute.ocsf_mapping.field.clone(),
            sample_values: attribute.sample_values.clone(),
            properties: {
                let mut props = HashMap::new();
                props.insert("caption".to_string(), attribute.caption.clone());
                props.insert("type".to_string(), format!("{:?}", attribute.attr_type));
                props.insert("is_dimension".to_string(), attribute.is_dimension.to_string());
                if let Some(expr) = &attribute.ocsf_mapping.expression {
                    props.insert("expression".to_string(), expr.clone());
                }
                props
            },
        };

        Ok(Embedding::new(
            format!("semantic_attr:{}:{}", entity_name, attribute.name),
            vector,
            metadata,
        ))
    }
}

/// Formats category information into text for embedding.
fn format_category_text(category: &Category) -> String {
    let mut parts = vec![
        category.name.clone(),
        category.caption.clone(),
    ];
    
    if !category.description.is_empty() {
        parts.push(category.description.clone());
    }
    
    parts.join(" ")
}

/// Formats event class information into text for embedding.
fn format_event_class_text(event_class: &EventClass) -> String {
    let mut parts = vec![
        event_class.name.clone(),
        event_class.caption.clone(),
    ];
    
    if !event_class.description.is_empty() {
        parts.push(event_class.description.clone());
    }
    
    // Include attribute names for context
    let attr_names: Vec<_> = event_class.attributes.keys().cloned().collect();
    if !attr_names.is_empty() {
        parts.push(format!("attributes: {}", attr_names.join(", ")));
    }
    
    parts.join(" ")
}

/// Formats object information into text for embedding.
fn format_object_text(object: &OCSFObject) -> String {
    let mut parts = vec![
        object.name.clone(),
        object.caption.clone(),
    ];
    
    if !object.description.is_empty() {
        parts.push(object.description.clone());
    }
    
    // Include attribute names for context
    let attr_names: Vec<_> = object.attributes.keys().cloned().collect();
    if !attr_names.is_empty() {
        parts.push(format!("attributes: {}", attr_names.join(", ")));
    }
    
    parts.join(" ")
}

/// Formats attribute information into text for embedding.
fn format_attribute_text(attribute: &Attribute) -> String {
    let mut parts = vec![
        attribute.name.clone(),
        attribute.caption.clone(),
        attribute.attr_type.clone(),
    ];
    
    if !attribute.description.is_empty() {
        parts.push(attribute.description.clone());
    }
    
    parts.join(" ")
}

/// Formats semantic entity information into text for embedding.
fn format_entity_text(entity: &SemanticEntity) -> String {
    let mut parts = vec![
        entity.name.clone(),
        entity.caption.clone(),
    ];
    
    if !entity.description.is_empty() {
        parts.push(entity.description.clone());
    }
    
    // Include attribute names for context
    let attr_names: Vec<_> = entity.attributes.iter().map(|a| a.name.clone()).collect();
    if !attr_names.is_empty() {
        parts.push(format!("attributes: {}", attr_names.join(", ")));
    }
    
    parts.join(" ")
}

/// Formats semantic attribute information into text for embedding.
fn format_semantic_attribute_text(attribute: &SemanticAttribute) -> String {
    let mut parts = vec![
        attribute.name.clone(),
        attribute.caption.clone(),
    ];
    
    if !attribute.description.is_empty() {
        parts.push(attribute.description.clone());
    }
    
    // Include OCSF mapping for context
    if let Some(field) = &attribute.ocsf_mapping.field {
        parts.push(format!("maps to: {}", field));
    }
    
    // Include sample values for context
    if !attribute.sample_values.is_empty() {
        parts.push(format!("examples: {}", attribute.sample_values.join(", ")));
    }
    
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::MockEmbeddingGenerator;
    use ocsf_core::Requirement;
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");
        
        schema.add_category(Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "Identity and access management events".to_string(),
            event_classes: vec![],
        });
        
        let mut attrs = HashMap::new();
        attrs.insert("user".to_string(), ocsf_core::AttributeRef {
            name: "user".to_string(),
            requirement: Some(Requirement::Required),
            description: None,
            group: None,
        });
        
        schema.add_event_class(EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Authentication events".to_string(),
            attributes: attrs,
            observables: vec![],
            extends: None,
            profiles: vec![],
        });
        
        schema.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "The name of the user".to_string(),
            requirement: Requirement::Required,
            observable: Some(10),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });
        
        schema
    }

    #[tokio::test]
    async fn test_generate_for_schema() {
        let generator = MockEmbeddingGenerator::new(384);
        let schema_gen = SchemaEmbeddingGenerator::new(generator);
        
        let schema = create_test_schema();
        let embeddings = schema_gen.generate_for_schema(&schema).await.unwrap();
        
        assert_eq!(embeddings.categories.len(), 1);
        assert_eq!(embeddings.event_classes.len(), 1);
        assert_eq!(embeddings.attributes.len(), 1);
    }

    #[tokio::test]
    async fn test_generate_for_category() {
        let generator = MockEmbeddingGenerator::new(384);
        let schema_gen = SchemaEmbeddingGenerator::new(generator);
        
        let category = Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        };
        
        let embedding = schema_gen.generate_for_category(&category).await.unwrap();
        
        assert_eq!(embedding.id, "category:3");
        assert_eq!(embedding.metadata.element_type, EmbeddingElementType::OcsfCategory);
        assert_eq!(embedding.metadata.name, "iam");
        assert_eq!(embedding.dimensions(), 384);
    }

    #[tokio::test]
    async fn test_generate_for_entity() {
        let generator = MockEmbeddingGenerator::new(384);
        let schema_gen = SchemaEmbeddingGenerator::new(generator);
        
        let entity = SemanticEntity::new("authentication_event")
            .with_caption("Authentication Event")
            .with_description("User authentication attempts")
            .with_source_event_classes(vec![3002])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_caption("User Email")
                    .with_field_mapping("actor.user.email_addr")
            );
        
        let embedding = schema_gen.generate_for_entity(&entity).await.unwrap();
        
        assert_eq!(embedding.entity.id, "entity:authentication_event");
        assert_eq!(embedding.entity.metadata.element_type, EmbeddingElementType::SemanticEntity);
        assert_eq!(embedding.attributes.len(), 1);
        assert_eq!(embedding.attributes[0].metadata.element_type, EmbeddingElementType::SemanticAttribute);
    }

    #[tokio::test]
    async fn test_embedding_consistency() {
        let generator = MockEmbeddingGenerator::new(384);
        let schema_gen = SchemaEmbeddingGenerator::new(generator);
        
        let category = Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        };
        
        let embedding1 = schema_gen.generate_for_category(&category).await.unwrap();
        let embedding2 = schema_gen.generate_for_category(&category).await.unwrap();
        
        // Same input should produce same embedding
        assert_eq!(embedding1.vector, embedding2.vector);
    }
}
