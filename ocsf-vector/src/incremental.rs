//! Incremental embedding updates.
//!
//! This module provides functionality to detect changes in schema elements
//! and update only the affected embeddings.

use crate::embedding::{Embedding, EmbeddingError, EmbeddingGenerator};
use crate::schema_embedding::SchemaEmbeddingGenerator;
use crate::store::{VectorStore, VectorStoreError};
use ocsf_core::OCSFSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during incremental updates.
#[derive(Debug, Error)]
pub enum IncrementalUpdateError {
    /// Embedding generation error.
    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] EmbeddingError),

    /// Vector store error.
    #[error("Vector store error: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}

/// Type of change detected in a schema element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Element was added.
    Added,
    /// Element was modified.
    Modified,
    /// Element was removed.
    Removed,
}

/// A change detected in a schema element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// The ID of the affected embedding.
    pub embedding_id: String,

    /// The type of change.
    pub change_type: ChangeType,

    /// Description of the change.
    pub description: String,
}

impl SchemaChange {
    /// Creates a new schema change.
    pub fn new(embedding_id: impl Into<String>, change_type: ChangeType, description: impl Into<String>) -> Self {
        Self {
            embedding_id: embedding_id.into(),
            change_type,
            description: description.into(),
        }
    }

    /// Creates an "added" change.
    pub fn added(embedding_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(embedding_id, ChangeType::Added, description)
    }

    /// Creates a "modified" change.
    pub fn modified(embedding_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(embedding_id, ChangeType::Modified, description)
    }

    /// Creates a "removed" change.
    pub fn removed(embedding_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(embedding_id, ChangeType::Removed, description)
    }
}

/// Result of an incremental update operation.
#[derive(Debug, Clone, Default)]
pub struct UpdateResult {
    /// Number of embeddings added.
    pub added: usize,

    /// Number of embeddings updated.
    pub updated: usize,

    /// Number of embeddings removed.
    pub removed: usize,

    /// Details of the changes.
    pub changes: Vec<SchemaChange>,
}

impl UpdateResult {
    /// Returns true if any changes were made.
    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.updated > 0 || self.removed > 0
    }

    /// Returns the total number of changes.
    pub fn total_changes(&self) -> usize {
        self.added + self.updated + self.removed
    }
}

/// Manages incremental embedding updates for schema changes.
pub struct IncrementalEmbeddingUpdater<G: EmbeddingGenerator> {
    schema_generator: SchemaEmbeddingGenerator<G>,
    /// Hashes of previously embedded content for change detection.
    content_hashes: HashMap<String, u64>,
}

impl<G: EmbeddingGenerator> IncrementalEmbeddingUpdater<G> {
    /// Creates a new incremental updater.
    pub fn new(generator: G) -> Self {
        Self {
            schema_generator: SchemaEmbeddingGenerator::new(generator),
            content_hashes: HashMap::new(),
        }
    }

    /// Detects changes between the current schema and previously embedded schema.
    pub fn detect_changes(&self, schema: &OCSFSchema) -> Vec<SchemaChange> {
        let mut changes = Vec::new();
        let mut current_ids = HashSet::new();

        // Check categories
        for category in schema.all_categories() {
            let id = format!("category:{}", category.uid);
            current_ids.insert(id.clone());
            
            let content_hash = Self::hash_content(&format!(
                "{}:{}:{}",
                category.name, category.caption, category.description
            ));

            match self.content_hashes.get(&id) {
                None => {
                    changes.push(SchemaChange::added(&id, format!("Category '{}' added", category.name)));
                }
                Some(&old_hash) if old_hash != content_hash => {
                    changes.push(SchemaChange::modified(&id, format!("Category '{}' modified", category.name)));
                }
                _ => {}
            }
        }

        // Check event classes
        for event_class in schema.all_event_classes() {
            let id = format!("class:{}", event_class.class_uid);
            current_ids.insert(id.clone());
            
            let content_hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                event_class.name,
                event_class.caption,
                event_class.description,
                event_class.attributes.len()
            ));

            match self.content_hashes.get(&id) {
                None => {
                    changes.push(SchemaChange::added(&id, format!("Event class '{}' added", event_class.name)));
                }
                Some(&old_hash) if old_hash != content_hash => {
                    changes.push(SchemaChange::modified(&id, format!("Event class '{}' modified", event_class.name)));
                }
                _ => {}
            }
        }

        // Check objects
        for object in schema.all_objects() {
            let id = format!("object:{}", object.name);
            current_ids.insert(id.clone());
            
            let content_hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                object.name,
                object.caption,
                object.description,
                object.attributes.len()
            ));

            match self.content_hashes.get(&id) {
                None => {
                    changes.push(SchemaChange::added(&id, format!("Object '{}' added", object.name)));
                }
                Some(&old_hash) if old_hash != content_hash => {
                    changes.push(SchemaChange::modified(&id, format!("Object '{}' modified", object.name)));
                }
                _ => {}
            }
        }

        // Check attributes
        for attribute in schema.all_attributes() {
            let id = format!("attribute:{}", attribute.name);
            current_ids.insert(id.clone());
            
            let content_hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                attribute.name,
                attribute.caption,
                attribute.description,
                attribute.attr_type
            ));

            match self.content_hashes.get(&id) {
                None => {
                    changes.push(SchemaChange::added(&id, format!("Attribute '{}' added", attribute.name)));
                }
                Some(&old_hash) if old_hash != content_hash => {
                    changes.push(SchemaChange::modified(&id, format!("Attribute '{}' modified", attribute.name)));
                }
                _ => {}
            }
        }

        // Check for removed elements
        for id in self.content_hashes.keys() {
            if !current_ids.contains(id) {
                changes.push(SchemaChange::removed(id, format!("Element '{}' removed", id)));
            }
        }

        changes
    }

    /// Updates embeddings based on detected changes.
    pub async fn update_embeddings<S: VectorStore>(
        &mut self,
        schema: &OCSFSchema,
        store: &mut S,
    ) -> Result<UpdateResult, IncrementalUpdateError> {
        let changes = self.detect_changes(schema);
        let mut result = UpdateResult::default();

        for change in &changes {
            match change.change_type {
                ChangeType::Added | ChangeType::Modified => {
                    // Generate new embedding
                    if let Some(embedding) = self.generate_embedding_for_id(&change.embedding_id, schema).await? {
                        store.store_one(embedding).await?;
                        
                        if change.change_type == ChangeType::Added {
                            result.added += 1;
                        } else {
                            result.updated += 1;
                        }
                    }
                }
                ChangeType::Removed => {
                    store.delete(std::slice::from_ref(&change.embedding_id)).await?;
                    self.content_hashes.remove(&change.embedding_id);
                    result.removed += 1;
                }
            }
        }

        // Update content hashes for added/modified elements
        self.update_content_hashes(schema);

        result.changes = changes;
        Ok(result)
    }

    /// Generates an embedding for a specific ID.
    async fn generate_embedding_for_id(
        &self,
        id: &str,
        schema: &OCSFSchema,
    ) -> Result<Option<Embedding>, EmbeddingError> {
        let parts: Vec<&str> = id.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        match parts[0] {
            "category" => {
                if let Ok(uid) = parts[1].parse::<u32>() {
                    if let Some(category) = schema.get_category(uid) {
                        return Ok(Some(self.schema_generator.generate_for_category(category).await?));
                    }
                }
            }
            "class" => {
                if let Ok(class_uid) = parts[1].parse::<u32>() {
                    if let Some(event_class) = schema.get_event_class(class_uid) {
                        return Ok(Some(self.schema_generator.generate_for_event_class(event_class).await?));
                    }
                }
            }
            "object" => {
                if let Some(object) = schema.get_object(parts[1]) {
                    return Ok(Some(self.schema_generator.generate_for_object(object).await?));
                }
            }
            "attribute" => {
                if let Some(attribute) = schema.get_attribute(parts[1]) {
                    return Ok(Some(self.schema_generator.generate_for_attribute(attribute, None).await?));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Updates content hashes for all schema elements.
    fn update_content_hashes(&mut self, schema: &OCSFSchema) {
        for category in schema.all_categories() {
            let id = format!("category:{}", category.uid);
            let hash = Self::hash_content(&format!(
                "{}:{}:{}",
                category.name, category.caption, category.description
            ));
            self.content_hashes.insert(id, hash);
        }

        for event_class in schema.all_event_classes() {
            let id = format!("class:{}", event_class.class_uid);
            let hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                event_class.name,
                event_class.caption,
                event_class.description,
                event_class.attributes.len()
            ));
            self.content_hashes.insert(id, hash);
        }

        for object in schema.all_objects() {
            let id = format!("object:{}", object.name);
            let hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                object.name,
                object.caption,
                object.description,
                object.attributes.len()
            ));
            self.content_hashes.insert(id, hash);
        }

        for attribute in schema.all_attributes() {
            let id = format!("attribute:{}", attribute.name);
            let hash = Self::hash_content(&format!(
                "{}:{}:{}:{}",
                attribute.name,
                attribute.caption,
                attribute.description,
                attribute.attr_type
            ));
            self.content_hashes.insert(id, hash);
        }
    }

    /// Computes a hash for content change detection.
    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns the number of tracked elements.
    pub fn tracked_count(&self) -> usize {
        self.content_hashes.len()
    }

    /// Clears all tracked content hashes.
    pub fn clear_tracking(&mut self) {
        self.content_hashes.clear();
    }

    /// Returns a reference to the underlying schema generator.
    pub fn schema_generator(&self) -> &SchemaEmbeddingGenerator<G> {
        &self.schema_generator
    }
}

/// Manages incremental embedding updates for semantic entities.
pub struct IncrementalEntityUpdater<G: EmbeddingGenerator> {
    schema_generator: SchemaEmbeddingGenerator<G>,
    /// Hashes of previously embedded entity content for change detection.
    content_hashes: HashMap<String, u64>,
}

impl<G: EmbeddingGenerator> IncrementalEntityUpdater<G> {
    /// Creates a new incremental entity updater.
    pub fn new(generator: G) -> Self {
        Self {
            schema_generator: SchemaEmbeddingGenerator::new(generator),
            content_hashes: HashMap::new(),
        }
    }

    /// Detects changes in semantic entities.
    pub fn detect_entity_changes(&self, entities: &[ocsf_semantic::SemanticEntity]) -> Vec<SchemaChange> {
        let mut changes = Vec::new();
        let mut current_ids = HashSet::new();

        for entity in entities {
            let id = format!("entity:{}", entity.name);
            current_ids.insert(id.clone());

            let content_hash = Self::hash_entity_content(entity);

            match self.content_hashes.get(&id) {
                None => {
                    changes.push(SchemaChange::added(&id, format!("Entity '{}' added", entity.name)));
                }
                Some(&old_hash) if old_hash != content_hash => {
                    changes.push(SchemaChange::modified(&id, format!("Entity '{}' modified", entity.name)));
                }
                _ => {}
            }

            // Check entity attributes
            for attr in &entity.attributes {
                let attr_id = format!("semantic_attr:{}:{}", entity.name, attr.name);
                current_ids.insert(attr_id.clone());

                let attr_hash = Self::hash_attribute_content(attr);

                match self.content_hashes.get(&attr_id) {
                    None => {
                        changes.push(SchemaChange::added(&attr_id, format!("Attribute '{}' added to entity '{}'", attr.name, entity.name)));
                    }
                    Some(&old_hash) if old_hash != attr_hash => {
                        changes.push(SchemaChange::modified(&attr_id, format!("Attribute '{}' modified in entity '{}'", attr.name, entity.name)));
                    }
                    _ => {}
                }
            }
        }

        // Check for removed elements
        for id in self.content_hashes.keys() {
            if !current_ids.contains(id) {
                changes.push(SchemaChange::removed(id, format!("Element '{}' removed", id)));
            }
        }

        changes
    }

    /// Updates embeddings for semantic entities based on detected changes.
    pub async fn update_entity_embeddings<S: VectorStore>(
        &mut self,
        entities: &[ocsf_semantic::SemanticEntity],
        store: &mut S,
    ) -> Result<UpdateResult, IncrementalUpdateError> {
        let changes = self.detect_entity_changes(entities);
        let mut result = UpdateResult::default();

        // Build a map of entities for lookup
        let entity_map: HashMap<&str, &ocsf_semantic::SemanticEntity> = entities
            .iter()
            .map(|e| (e.name.as_str(), e))
            .collect();

        for change in &changes {
            match change.change_type {
                ChangeType::Added | ChangeType::Modified => {
                    // Generate new embedding
                    if let Some(embedding) = self.generate_entity_embedding_for_id(&change.embedding_id, &entity_map).await? {
                        store.store_one(embedding).await?;

                        if change.change_type == ChangeType::Added {
                            result.added += 1;
                        } else {
                            result.updated += 1;
                        }
                    }
                }
                ChangeType::Removed => {
                    store.delete(std::slice::from_ref(&change.embedding_id)).await?;
                    self.content_hashes.remove(&change.embedding_id);
                    result.removed += 1;
                }
            }
        }

        // Update content hashes
        self.update_entity_content_hashes(entities);

        result.changes = changes;
        Ok(result)
    }

    /// Generates an embedding for a specific entity ID.
    async fn generate_entity_embedding_for_id(
        &self,
        id: &str,
        entity_map: &HashMap<&str, &ocsf_semantic::SemanticEntity>,
    ) -> Result<Option<Embedding>, EmbeddingError> {
        let parts: Vec<&str> = id.splitn(3, ':').collect();

        match parts.as_slice() {
            ["entity", name] => {
                if let Some(entity) = entity_map.get(name) {
                    let embedding = self.schema_generator.generate_for_entity(entity).await?;
                    return Ok(Some(embedding.entity));
                }
            }
            ["semantic_attr", entity_name, attr_name] => {
                if let Some(entity) = entity_map.get(entity_name) {
                    if let Some(attr) = entity.attributes.iter().find(|a| a.name == *attr_name) {
                        return Ok(Some(
                            self.schema_generator.generate_for_semantic_attribute(attr, entity_name).await?
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Updates content hashes for all entities.
    fn update_entity_content_hashes(&mut self, entities: &[ocsf_semantic::SemanticEntity]) {
        for entity in entities {
            let id = format!("entity:{}", entity.name);
            let hash = Self::hash_entity_content(entity);
            self.content_hashes.insert(id, hash);

            for attr in &entity.attributes {
                let attr_id = format!("semantic_attr:{}:{}", entity.name, attr.name);
                let attr_hash = Self::hash_attribute_content(attr);
                self.content_hashes.insert(attr_id, attr_hash);
            }
        }
    }

    /// Computes a hash for entity content.
    fn hash_entity_content(entity: &ocsf_semantic::SemanticEntity) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        entity.name.hash(&mut hasher);
        entity.caption.hash(&mut hasher);
        entity.description.hash(&mut hasher);
        entity.source_event_classes.len().hash(&mut hasher);
        entity.attributes.len().hash(&mut hasher);
        hasher.finish()
    }

    /// Computes a hash for attribute content.
    fn hash_attribute_content(attr: &ocsf_semantic::SemanticAttribute) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        attr.name.hash(&mut hasher);
        attr.caption.hash(&mut hasher);
        attr.description.hash(&mut hasher);
        if let Some(field) = &attr.ocsf_mapping.field {
            field.hash(&mut hasher);
        }
        if let Some(expr) = &attr.ocsf_mapping.expression {
            expr.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Returns the number of tracked elements.
    pub fn tracked_count(&self) -> usize {
        self.content_hashes.len()
    }

    /// Clears all tracked content hashes.
    pub fn clear_tracking(&mut self) {
        self.content_hashes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::MockEmbeddingGenerator;
    use crate::store::InMemoryVectorStore;
    use ocsf_core::{Attribute, Category, Requirement};
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");
        
        schema.add_category(Category {
            uid: 1,
            name: "system".to_string(),
            caption: "System Activity".to_string(),
            description: "System events".to_string(),
            event_classes: vec![],
        });
        
        schema.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "The name of the user".to_string(),
            requirement: Requirement::Required,
            observable: None,
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });
        
        schema
    }

    #[tokio::test]
    async fn test_detect_new_elements() {
        let generator = MockEmbeddingGenerator::new(384);
        let updater = IncrementalEmbeddingUpdater::new(generator);
        
        let schema = create_test_schema();
        let changes = updater.detect_changes(&schema);
        
        // All elements should be detected as added
        assert!(!changes.is_empty());
        assert!(changes.iter().all(|c| c.change_type == ChangeType::Added));
    }

    #[tokio::test]
    async fn test_detect_modified_elements() {
        let generator = MockEmbeddingGenerator::new(384);
        let mut updater = IncrementalEmbeddingUpdater::new(generator);
        
        let mut schema = create_test_schema();
        
        // First update to establish baseline
        let mut store = InMemoryVectorStore::new(384);
        updater.update_embeddings(&schema, &mut store).await.unwrap();
        
        // Modify the category
        schema.categories.get_mut(&1).unwrap().description = "Modified description".to_string();
        
        let changes = updater.detect_changes(&schema);
        
        // Should detect the modification
        let modified = changes.iter().filter(|c| c.change_type == ChangeType::Modified).count();
        assert_eq!(modified, 1);
    }

    #[tokio::test]
    async fn test_detect_removed_elements() {
        let generator = MockEmbeddingGenerator::new(384);
        let mut updater = IncrementalEmbeddingUpdater::new(generator);
        
        let schema = create_test_schema();
        
        // First update to establish baseline
        let mut store = InMemoryVectorStore::new(384);
        updater.update_embeddings(&schema, &mut store).await.unwrap();
        
        // Create a new schema without the attribute
        let mut new_schema = OCSFSchema::new("1.4.0");
        new_schema.add_category(Category {
            uid: 1,
            name: "system".to_string(),
            caption: "System Activity".to_string(),
            description: "System events".to_string(),
            event_classes: vec![],
        });
        
        let changes = updater.detect_changes(&new_schema);
        
        // Should detect the removed attribute
        let removed = changes.iter().filter(|c| c.change_type == ChangeType::Removed).count();
        assert_eq!(removed, 1);
    }

    #[tokio::test]
    async fn test_incremental_update() {
        let generator = MockEmbeddingGenerator::new(384);
        let mut updater = IncrementalEmbeddingUpdater::new(generator);
        let mut store = InMemoryVectorStore::new(384);
        
        let schema = create_test_schema();
        
        // First update
        let result = updater.update_embeddings(&schema, &mut store).await.unwrap();
        
        assert!(result.has_changes());
        assert_eq!(result.added, 2); // 1 category + 1 attribute
        assert_eq!(result.updated, 0);
        assert_eq!(result.removed, 0);
        
        // Second update with no changes
        let result = updater.update_embeddings(&schema, &mut store).await.unwrap();
        
        assert!(!result.has_changes());
    }
}
