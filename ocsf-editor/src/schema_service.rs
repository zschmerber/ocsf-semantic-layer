//! Schema Service for loading and caching OCSF schema.
//!
//! This module provides the SchemaService which loads OCSF compiled schemas,
//! builds tree structures for UI display, and supports search functionality.
//!
//! # Requirements
//! - 1.1: WHEN the Editor loads, THE Schema_Browser SHALL display OCSF categories
//!   as expandable tree nodes
//! - 1.5: THE Schema_Browser SHALL support text search to filter categories,
//!   classes, and attributes by name or description

use std::path::Path;
use std::sync::Arc;

use ocsf_core::compiled_schema::{
    CompiledAttribute, CompiledClass, CompiledObject, CompiledSchema, ParseError,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::EditorApiError;

// ============================================================================
// Schema Tree Types
// ============================================================================

/// A node in the schema tree representing a category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryNode {
    /// Unique category identifier.
    pub uid: u32,
    /// Category name (key in the schema).
    pub name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Detailed description.
    pub description: String,
    /// Event classes within this category.
    pub classes: Vec<ClassNode>,
}

/// A node in the schema tree representing an event class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassNode {
    /// Unique class identifier.
    pub uid: u32,
    /// Class name.
    pub name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Detailed description.
    pub description: String,
    /// Category this class belongs to.
    pub category: String,
    /// Attributes for this class.
    pub attributes: Vec<AttributeNode>,
}

/// A node in the schema tree representing an attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeNode {
    /// Attribute name.
    pub name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Detailed description.
    pub description: String,
    /// The OCSF type (e.g., "string_t", "integer_t", "object_t").
    #[serde(rename = "type")]
    pub attr_type: String,
    /// Human-readable type name.
    pub type_name: String,
    /// Requirement level.
    pub requirement: String,
    /// Whether this is an array attribute.
    pub is_array: bool,
    /// Enum values if this is an enum type.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub enum_values: Vec<EnumValueNode>,
    /// Object type name for object_t types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    /// Child attributes for nested objects.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<AttributeNode>,
}

/// An enum value in an attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValueNode {
    /// The enum key (e.g., "0", "1").
    pub key: String,
    /// Human-readable caption.
    pub caption: String,
    /// Description of this enum value.
    pub description: String,
}

/// An object definition in the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNode {
    /// Object name.
    pub name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Detailed description.
    pub description: String,
    /// Attributes for this object.
    pub attributes: Vec<AttributeNode>,
}

/// The complete schema tree structure for UI display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaTree {
    /// Schema version.
    pub version: String,
    /// Categories with their classes.
    pub categories: Vec<CategoryNode>,
    /// Standalone objects.
    pub objects: Vec<ObjectNode>,
}

// ============================================================================
// Search Result Types
// ============================================================================

/// Type of schema element found in search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchResultType {
    Category,
    Class,
    Object,
    Attribute,
}

/// A search result from the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Type of the result.
    pub result_type: SearchResultType,
    /// Name of the element.
    pub name: String,
    /// Human-readable caption.
    pub caption: String,
    /// Description of the element.
    pub description: String,
    /// Path to the element (e.g., "iam/authentication/actor").
    pub path: String,
    /// Parent category UID (for classes and attributes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_uid: Option<u32>,
    /// Parent class UID (for attributes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_uid: Option<u32>,
}

// ============================================================================
// Schema Service
// ============================================================================

/// Service for loading and caching OCSF schema data.
///
/// The SchemaService provides:
/// - Schema loading from file path
/// - Tree structure building for UI display
/// - Search functionality with name/description matching
/// - Attribute lookup by class UID and path
///
/// # Requirements
/// - 1.1: Display OCSF categories as expandable tree nodes
/// - 1.5: Support text search to filter categories, classes, and attributes
pub struct SchemaService {
    /// The loaded compiled schema.
    schema: Arc<RwLock<Option<CompiledSchema>>>,
    /// Cached tree structure for UI display.
    tree_cache: Arc<RwLock<Option<SchemaTree>>>,
}

impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaService {
    /// Create a new SchemaService.
    pub fn new() -> Self {
        Self {
            schema: Arc::new(RwLock::new(None)),
            tree_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Load a schema from a file path.
    ///
    /// This parses the compiled OCSF schema JSON and builds the tree cache.
    ///
    /// # Arguments
    /// * `path` - Path to the compiled schema JSON file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub async fn load_schema(&self, path: &Path) -> Result<(), EditorApiError> {
        let schema = CompiledSchema::parse_file(path).map_err(|e| match e {
            ParseError::Io(io_err) => {
                EditorApiError::Internal(anyhow::anyhow!("Failed to read schema file: {}", io_err))
            }
            ParseError::InvalidJson(json_err) => EditorApiError::Internal(anyhow::anyhow!(
                "Failed to parse schema JSON: {}",
                json_err
            )),
            other => EditorApiError::Internal(anyhow::anyhow!("Schema parse error: {}", other)),
        })?;

        // Build the tree structure
        let tree = self.build_tree(&schema);

        // Store both the schema and tree
        {
            let mut schema_guard = self.schema.write().await;
            *schema_guard = Some(schema);
        }
        {
            let mut tree_guard = self.tree_cache.write().await;
            *tree_guard = Some(tree);
        }

        Ok(())
    }

    /// Load a schema from a JSON string.
    ///
    /// # Arguments
    /// * `json` - The compiled schema JSON string
    ///
    /// # Errors
    /// Returns an error if the JSON cannot be parsed.
    pub async fn load_schema_from_str(&self, json: &str) -> Result<(), EditorApiError> {
        let schema = CompiledSchema::parse(json).map_err(|e| {
            EditorApiError::Internal(anyhow::anyhow!("Failed to parse schema JSON: {}", e))
        })?;

        // Build the tree structure
        let tree = self.build_tree(&schema);

        // Store both the schema and tree
        {
            let mut schema_guard = self.schema.write().await;
            *schema_guard = Some(schema);
        }
        {
            let mut tree_guard = self.tree_cache.write().await;
            *tree_guard = Some(tree);
        }

        Ok(())
    }

    /// Get the schema version if loaded (async version).
    ///
    /// # Returns
    /// The schema version string, or None if no schema is loaded.
    pub async fn get_version_async(&self) -> Option<String> {
        self.tree_cache
            .read()
            .await
            .as_ref()
            .map(|tree| tree.version.clone())
    }

    /// Check if a schema is loaded.
    pub async fn is_loaded(&self) -> bool {
        self.schema.read().await.is_some()
    }

    /// Get the schema tree for UI display.
    ///
    /// # Returns
    /// The cached schema tree, or an empty tree if no schema is loaded.
    pub async fn get_tree(&self) -> SchemaTree {
        self.tree_cache.read().await.clone().unwrap_or_default()
    }

    /// Get an attribute by class UID and field path.
    ///
    /// # Arguments
    /// * `class_uid` - The class UID to search in
    /// * `path` - The dot-separated field path (e.g., "actor.user.name")
    ///
    /// # Returns
    /// The attribute if found, or None if not found.
    pub async fn get_attribute(&self, class_uid: u32, path: &str) -> Option<CompiledAttribute> {
        let schema_guard = self.schema.read().await;
        let schema = schema_guard.as_ref()?;

        // Find the class by UID
        let class = schema.classes.values().find(|c| c.uid == class_uid)?;

        // Navigate the path
        self.resolve_attribute_path(&class.attributes, path, schema)
    }

    /// Search the schema for elements matching the query.
    ///
    /// Searches categories, classes, objects, and attributes by name or description.
    /// The search is case-insensitive.
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// A vector of search results matching the query.
    ///
    /// # Requirements
    /// - 1.5: THE Schema_Browser SHALL support text search to filter categories,
    ///   classes, and attributes by name or description
    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        let tree_guard = self.tree_cache.read().await;
        let tree = match tree_guard.as_ref() {
            Some(t) => t,
            None => return Vec::new(),
        };

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search categories
        for category in &tree.categories {
            if self.matches_query(
                &category.name,
                &category.caption,
                &category.description,
                &query_lower,
            ) {
                results.push(SearchResult {
                    result_type: SearchResultType::Category,
                    name: category.name.clone(),
                    caption: category.caption.clone(),
                    description: category.description.clone(),
                    path: category.name.clone(),
                    category_uid: Some(category.uid),
                    class_uid: None,
                });
            }

            // Search classes within category
            for class in &category.classes {
                if self.matches_query(
                    &class.name,
                    &class.caption,
                    &class.description,
                    &query_lower,
                ) {
                    results.push(SearchResult {
                        result_type: SearchResultType::Class,
                        name: class.name.clone(),
                        caption: class.caption.clone(),
                        description: class.description.clone(),
                        path: format!("{}/{}", category.name, class.name),
                        category_uid: Some(category.uid),
                        class_uid: Some(class.uid),
                    });
                }

                // Search attributes within class
                self.search_attributes(
                    &class.attributes,
                    &query_lower,
                    &format!("{}/{}", category.name, class.name),
                    Some(category.uid),
                    Some(class.uid),
                    &mut results,
                );
            }
        }

        // Search objects
        for object in &tree.objects {
            if self.matches_query(
                &object.name,
                &object.caption,
                &object.description,
                &query_lower,
            ) {
                results.push(SearchResult {
                    result_type: SearchResultType::Object,
                    name: object.name.clone(),
                    caption: object.caption.clone(),
                    description: object.description.clone(),
                    path: format!("objects/{}", object.name),
                    category_uid: None,
                    class_uid: None,
                });
            }

            // Search attributes within object
            self.search_attributes(
                &object.attributes,
                &query_lower,
                &format!("objects/{}", object.name),
                None,
                None,
                &mut results,
            );
        }

        results
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

    /// Build the tree structure from a compiled schema.
    fn build_tree(&self, schema: &CompiledSchema) -> SchemaTree {
        let mut categories: Vec<CategoryNode> = Vec::new();

        // Group classes by category
        let mut category_classes: std::collections::HashMap<String, Vec<&CompiledClass>> =
            std::collections::HashMap::new();

        for class in schema.classes.values() {
            category_classes
                .entry(class.category.clone())
                .or_default()
                .push(class);
        }

        // Build category nodes
        for (cat_name, cat_info) in &schema.categories.attributes {
            let classes = category_classes
                .get(cat_name)
                .map(|classes| {
                    let mut class_nodes: Vec<ClassNode> = classes
                        .iter()
                        .map(|class| self.build_class_node(class, schema))
                        .collect();
                    // Sort classes by name for consistent ordering
                    class_nodes.sort_by(|a, b| a.name.cmp(&b.name));
                    class_nodes
                })
                .unwrap_or_default();

            categories.push(CategoryNode {
                uid: cat_info.uid,
                name: cat_name.clone(),
                caption: cat_info.caption.clone(),
                description: cat_info.description.clone(),
                classes,
            });
        }

        // Sort categories by name for consistent ordering
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        // Build object nodes
        let mut objects: Vec<ObjectNode> = schema
            .objects
            .iter()
            .map(|(name, obj)| self.build_object_node(name, obj, schema))
            .collect();

        // Sort objects by name for consistent ordering
        objects.sort_by(|a, b| a.name.cmp(&b.name));

        SchemaTree {
            version: schema.version.clone(),
            categories,
            objects,
        }
    }

    /// Maximum depth for nested object expansion to prevent stack overflow.
    const MAX_NESTING_DEPTH: usize = 3;

    /// Build a class node from a compiled class.
    fn build_class_node(&self, class: &CompiledClass, schema: &CompiledSchema) -> ClassNode {
        let mut visited = std::collections::HashSet::new();
        let mut attributes: Vec<AttributeNode> = class
            .attributes
            .iter()
            .map(|(name, attr)| self.build_attribute_node_with_depth(name, attr, schema, 0, &mut visited))
            .collect();

        // Sort attributes by name for consistent ordering
        attributes.sort_by(|a, b| a.name.cmp(&b.name));

        ClassNode {
            uid: class.uid,
            name: class.name.clone(),
            caption: class.caption.clone(),
            description: class.description.clone(),
            category: class.category.clone(),
            attributes,
        }
    }

    /// Build an object node from a compiled object.
    fn build_object_node(
        &self,
        name: &str,
        obj: &CompiledObject,
        schema: &CompiledSchema,
    ) -> ObjectNode {
        let mut visited = std::collections::HashSet::new();
        let mut attributes: Vec<AttributeNode> = obj
            .attributes
            .iter()
            .map(|(attr_name, attr)| self.build_attribute_node_with_depth(attr_name, attr, schema, 0, &mut visited))
            .collect();

        // Sort attributes by name for consistent ordering
        attributes.sort_by(|a, b| a.name.cmp(&b.name));

        ObjectNode {
            name: name.to_string(),
            caption: obj.caption.clone(),
            description: obj.description.clone(),
            attributes,
        }
    }

    /// Build an attribute node from a compiled attribute with depth tracking.
    /// 
    /// This prevents stack overflow from circular references or deeply nested
    /// object types by limiting recursion depth and tracking visited objects.
    fn build_attribute_node_with_depth(
        &self,
        name: &str,
        attr: &CompiledAttribute,
        schema: &CompiledSchema,
        depth: usize,
        visited: &mut std::collections::HashSet<String>,
    ) -> AttributeNode {
        // Convert requirement to string
        let requirement = match attr.requirement {
            ocsf_core::compiled_schema::CompiledRequirement::Required => "required",
            ocsf_core::compiled_schema::CompiledRequirement::Recommended => "recommended",
            ocsf_core::compiled_schema::CompiledRequirement::Optional => "optional",
        }
        .to_string();

        // Build enum values
        let mut enum_values: Vec<EnumValueNode> = attr
            .enum_values
            .iter()
            .map(|(key, val)| EnumValueNode {
                key: key.clone(),
                caption: val.caption.clone(),
                description: val.description.clone(),
            })
            .collect();

        // Sort enum values by key for consistent ordering
        enum_values.sort_by(|a, b| {
            // Try to sort numerically first, then alphabetically
            match (a.key.parse::<i64>(), b.key.parse::<i64>()) {
                (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                _ => a.key.cmp(&b.key),
            }
        });

        // Build children for object types, but limit depth to prevent stack overflow
        let children = if depth < Self::MAX_NESTING_DEPTH {
            if let Some(ref object_type) = attr.object_type {
                // Check for circular references
                if visited.contains(object_type) {
                    Vec::new()
                } else if let Some(obj) = schema.objects.get(object_type) {
                    // Mark this object type as visited
                    visited.insert(object_type.clone());
                    
                    let mut child_attrs: Vec<AttributeNode> = obj
                        .attributes
                        .iter()
                        .map(|(child_name, child_attr)| {
                            self.build_attribute_node_with_depth(
                                child_name, 
                                child_attr, 
                                schema, 
                                depth + 1,
                                visited,
                            )
                        })
                        .collect();
                    child_attrs.sort_by(|a, b| a.name.cmp(&b.name));
                    
                    // Remove from visited after processing (allow same type in different branches)
                    visited.remove(object_type);
                    
                    child_attrs
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            // Max depth reached, don't expand further
            Vec::new()
        };

        AttributeNode {
            name: name.to_string(),
            caption: attr.caption.clone(),
            description: attr.description.clone(),
            attr_type: attr.attr_type.clone(),
            type_name: attr.type_name.clone(),
            requirement,
            is_array: attr.is_array,
            enum_values,
            object_type: attr.object_type.clone(),
            children,
        }
    }

    /// Check if a name, caption, or description matches the query.
    fn matches_query(
        &self,
        name: &str,
        caption: &str,
        description: &str,
        query_lower: &str,
    ) -> bool {
        name.to_lowercase().contains(query_lower)
            || caption.to_lowercase().contains(query_lower)
            || description.to_lowercase().contains(query_lower)
    }

    /// Recursively search attributes for matches.
    fn search_attributes(
        &self,
        attributes: &[AttributeNode],
        query_lower: &str,
        parent_path: &str,
        category_uid: Option<u32>,
        class_uid: Option<u32>,
        results: &mut Vec<SearchResult>,
    ) {
        for attr in attributes {
            let attr_path = format!("{}/{}", parent_path, attr.name);

            if self.matches_query(&attr.name, &attr.caption, &attr.description, query_lower) {
                results.push(SearchResult {
                    result_type: SearchResultType::Attribute,
                    name: attr.name.clone(),
                    caption: attr.caption.clone(),
                    description: attr.description.clone(),
                    path: attr_path.clone(),
                    category_uid,
                    class_uid,
                });
            }

            // Search children recursively
            if !attr.children.is_empty() {
                self.search_attributes(
                    &attr.children,
                    query_lower,
                    &attr_path,
                    category_uid,
                    class_uid,
                    results,
                );
            }
        }
    }

    /// Resolve an attribute path to get the attribute.
    fn resolve_attribute_path(
        &self,
        attributes: &std::collections::HashMap<String, CompiledAttribute>,
        path: &str,
        schema: &CompiledSchema,
    ) -> Option<CompiledAttribute> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let first_attr = attributes.get(parts[0])?;

        if parts.len() == 1 {
            return Some(first_attr.clone());
        }

        // Navigate nested path through object types
        let mut current_attr = first_attr.clone();
        for part in &parts[1..] {
            let object_type = current_attr.object_type.as_ref()?;
            let obj = schema.objects.get(object_type)?;
            current_attr = obj.attributes.get(*part)?.clone();
        }

        Some(current_attr)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal test schema JSON.
    fn test_schema_json() -> &'static str {
        r#"{
            "version": "1.4.0",
            "categories": {
                "attributes": {
                    "iam": {
                        "uid": 3,
                        "caption": "Identity & Access Management",
                        "description": "IAM events for authentication and authorization"
                    },
                    "network": {
                        "uid": 4,
                        "caption": "Network Activity",
                        "description": "Network traffic and connection events"
                    }
                },
                "caption": "Categories",
                "description": "Event categories",
                "name": "categories"
            },
            "classes": {
                "authentication": {
                    "uid": 3002,
                    "name": "authentication",
                    "caption": "Authentication",
                    "description": "Authentication events for user login and logout",
                    "category": "iam",
                    "attributes": {
                        "activity_id": {
                            "caption": "Activity ID",
                            "description": "The normalized identifier of the activity",
                            "type": "integer_t",
                            "type_name": "Integer",
                            "requirement": "required",
                            "enum": {
                                "0": {"caption": "Unknown", "description": "Unknown activity"},
                                "1": {"caption": "Logon", "description": "User logon"}
                            }
                        },
                        "actor": {
                            "caption": "Actor",
                            "description": "The actor that performed the authentication",
                            "type": "object_t",
                            "type_name": "Actor",
                            "object_type": "actor",
                            "requirement": "recommended"
                        }
                    }
                }
            },
            "objects": {
                "actor": {
                    "name": "actor",
                    "caption": "Actor",
                    "description": "The actor object describes the entity that performed the activity",
                    "attributes": {
                        "user": {
                            "caption": "User",
                            "description": "The user that performed the activity",
                            "type": "object_t",
                            "type_name": "User",
                            "object_type": "user",
                            "requirement": "recommended"
                        }
                    }
                },
                "user": {
                    "name": "user",
                    "caption": "User",
                    "description": "The user object describes a person or service account",
                    "attributes": {
                        "name": {
                            "caption": "Name",
                            "description": "The user name",
                            "type": "string_t",
                            "type_name": "String",
                            "requirement": "recommended"
                        },
                        "email_addr": {
                            "caption": "Email Address",
                            "description": "The user email address",
                            "type": "email_t",
                            "type_name": "Email",
                            "requirement": "optional"
                        }
                    }
                }
            },
            "profiles": {},
            "extensions": {}
        }"#
    }

    #[tokio::test]
    async fn test_schema_service_new() {
        let service = SchemaService::new();
        assert!(!service.is_loaded().await);
    }

    #[tokio::test]
    async fn test_load_schema_from_str() {
        let service = SchemaService::new();
        let result = service.load_schema_from_str(test_schema_json()).await;
        assert!(result.is_ok());
        assert!(service.is_loaded().await);
    }

    #[tokio::test]
    async fn test_get_tree_structure() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let tree = service.get_tree().await;
        assert_eq!(tree.version, "1.4.0");
        assert_eq!(tree.categories.len(), 2);
        assert_eq!(tree.objects.len(), 2);
    }

    #[tokio::test]
    async fn test_tree_categories_have_classes() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let tree = service.get_tree().await;

        // Find the IAM category
        let iam_category = tree.categories.iter().find(|c| c.name == "iam");
        assert!(iam_category.is_some());

        let iam = iam_category.unwrap();
        assert_eq!(iam.uid, 3);
        assert_eq!(iam.caption, "Identity & Access Management");
        assert_eq!(iam.classes.len(), 1);
        assert_eq!(iam.classes[0].name, "authentication");
    }

    #[tokio::test]
    async fn test_tree_classes_have_attributes() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let tree = service.get_tree().await;

        let iam = tree.categories.iter().find(|c| c.name == "iam").unwrap();
        let auth_class = &iam.classes[0];

        assert_eq!(auth_class.uid, 3002);
        assert_eq!(auth_class.attributes.len(), 2);

        // Check activity_id attribute
        let activity_id = auth_class
            .attributes
            .iter()
            .find(|a| a.name == "activity_id");
        assert!(activity_id.is_some());
        let activity_id = activity_id.unwrap();
        assert_eq!(activity_id.attr_type, "integer_t");
        assert_eq!(activity_id.requirement, "required");
        assert_eq!(activity_id.enum_values.len(), 2);
    }

    #[tokio::test]
    async fn test_tree_attribute_children() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let tree = service.get_tree().await;

        let iam = tree.categories.iter().find(|c| c.name == "iam").unwrap();
        let auth_class = &iam.classes[0];

        // Check actor attribute has children from the actor object
        let actor = auth_class.attributes.iter().find(|a| a.name == "actor");
        assert!(actor.is_some());
        let actor = actor.unwrap();
        assert_eq!(actor.object_type, Some("actor".to_string()));
        assert!(!actor.children.is_empty());

        // Actor should have user child
        let user_child = actor.children.iter().find(|c| c.name == "user");
        assert!(user_child.is_some());
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("authentication").await;
        assert!(!results.is_empty());

        // Should find the authentication class
        let class_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Class);
        assert!(class_result.is_some());
        assert_eq!(class_result.unwrap().name, "authentication");
    }

    #[tokio::test]
    async fn test_search_by_description() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        // Search for "login" which appears in the authentication class description
        let results = service.search("login").await;
        assert!(!results.is_empty());

        let class_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Class);
        assert!(class_result.is_some());
    }

    #[tokio::test]
    async fn test_search_case_insensitive() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results_lower = service.search("authentication").await;
        let results_upper = service.search("AUTHENTICATION").await;
        let results_mixed = service.search("Authentication").await;

        assert_eq!(results_lower.len(), results_upper.len());
        assert_eq!(results_lower.len(), results_mixed.len());
    }

    #[tokio::test]
    async fn test_search_categories() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("iam").await;

        let category_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Category);
        assert!(category_result.is_some());
        assert_eq!(category_result.unwrap().name, "iam");
    }

    #[tokio::test]
    async fn test_search_objects() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("user").await;

        let object_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Object);
        assert!(object_result.is_some());
        assert_eq!(object_result.unwrap().name, "user");
    }

    #[tokio::test]
    async fn test_search_attributes() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("activity_id").await;

        let attr_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Attribute);
        assert!(attr_result.is_some());
        assert_eq!(attr_result.unwrap().name, "activity_id");
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("nonexistent_xyz_123").await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        // Empty query should match everything (contains "")
        let results = service.search("").await;
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_get_attribute_simple_path() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let attr = service.get_attribute(3002, "activity_id").await;
        assert!(attr.is_some());
        let attr = attr.unwrap();
        assert_eq!(attr.caption, "Activity ID");
        assert_eq!(attr.attr_type, "integer_t");
    }

    #[tokio::test]
    async fn test_get_attribute_nested_path() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        // actor.user.name should resolve through actor -> user -> name
        let attr = service.get_attribute(3002, "actor.user.name").await;
        assert!(attr.is_some());
        let attr = attr.unwrap();
        assert_eq!(attr.caption, "Name");
        assert_eq!(attr.attr_type, "string_t");
    }

    #[tokio::test]
    async fn test_get_attribute_invalid_class() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let attr = service.get_attribute(99999, "activity_id").await;
        assert!(attr.is_none());
    }

    #[tokio::test]
    async fn test_get_attribute_invalid_path() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let attr = service.get_attribute(3002, "nonexistent").await;
        assert!(attr.is_none());
    }

    #[tokio::test]
    async fn test_get_tree_empty_when_not_loaded() {
        let service = SchemaService::new();
        let tree = service.get_tree().await;

        assert!(tree.version.is_empty());
        assert!(tree.categories.is_empty());
        assert!(tree.objects.is_empty());
    }

    #[tokio::test]
    async fn test_search_result_paths() {
        let service = SchemaService::new();
        service
            .load_schema_from_str(test_schema_json())
            .await
            .unwrap();

        let results = service.search("activity_id").await;
        let attr_result = results
            .iter()
            .find(|r| r.result_type == SearchResultType::Attribute)
            .unwrap();

        // Path should be category/class/attribute
        assert!(attr_result.path.contains("iam"));
        assert!(attr_result.path.contains("authentication"));
        assert!(attr_result.path.contains("activity_id"));
    }
}
