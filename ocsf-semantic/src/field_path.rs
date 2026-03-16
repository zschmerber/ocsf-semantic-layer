//! Field path representation and generation for OCSF nested objects.
//!
//! This module provides types for representing and manipulating field paths
//! in OCSF events, supporting nested object traversal and array access.

use ocsf_core::{CompiledAttribute, CompiledSchema};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors during path operations.
#[derive(Debug, Error)]
pub enum PathError {
    #[error("Empty path")]
    EmptyPath,

    #[error("Invalid path segment: {0}")]
    InvalidSegment(String),

    #[error("Unresolved object type: {0}")]
    UnresolvedObjectType(String),

    #[error("Max nesting depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },

    #[error("Circular reference detected: {path}")]
    CircularReference { path: String },
}

/// A segment in a field path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathSegment {
    /// The field name.
    pub name: String,
    /// Whether this segment represents an array access.
    pub is_array: bool,
}

impl PathSegment {
    /// Create a new path segment.
    pub fn new(name: impl Into<String>, is_array: bool) -> Self {
        Self {
            name: name.into(),
            is_array,
        }
    }

    /// Create a scalar (non-array) segment.
    pub fn scalar(name: impl Into<String>) -> Self {
        Self::new(name, false)
    }

    /// Create an array segment.
    pub fn array(name: impl Into<String>) -> Self {
        Self::new(name, true)
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_array {
            write!(f, "{}[]", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// A field path representing a nested attribute location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FieldPath {
    /// The path segments.
    pub segments: Vec<PathSegment>,
}

impl FieldPath {
    /// Create an empty field path.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a field path from segments.
    pub fn from_segments(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }

    /// Parse a field path from dot notation string.
    ///
    /// Supports array notation with `[]` suffix (e.g., "users[].name").
    pub fn parse(path: &str) -> Result<Self, PathError> {
        if path.is_empty() {
            return Err(PathError::EmptyPath);
        }

        let mut segments = Vec::new();
        for part in path.split('.') {
            if part.is_empty() {
                return Err(PathError::InvalidSegment("empty segment".to_string()));
            }

            let (name, is_array) = if let Some(stripped) = part.strip_suffix("[]") {
                (stripped, true)
            } else {
                (part, false)
            };

            if name.is_empty() {
                return Err(PathError::InvalidSegment("empty segment name".to_string()));
            }

            segments.push(PathSegment::new(name, is_array));
        }

        Ok(Self { segments })
    }

    /// Append a segment to the path.
    pub fn push(&mut self, name: impl Into<String>, is_array: bool) {
        self.segments.push(PathSegment::new(name, is_array));
    }

    /// Append a scalar segment.
    pub fn push_scalar(&mut self, name: impl Into<String>) {
        self.push(name, false);
    }

    /// Append an array segment.
    pub fn push_array(&mut self, name: impl Into<String>) {
        self.push(name, true);
    }

    /// Get the depth (number of segments) of the path.
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the last segment name.
    pub fn leaf_name(&self) -> Option<&str> {
        self.segments.last().map(|s| s.name.as_str())
    }

    /// Check if any segment is an array.
    pub fn has_array(&self) -> bool {
        self.segments.iter().any(|s| s.is_array)
    }

    /// Create a child path by appending a segment.
    pub fn child(&self, name: impl Into<String>, is_array: bool) -> Self {
        let mut child = self.clone();
        child.push(name, is_array);
        child
    }

    /// Convert to dot notation string.
    pub fn to_dot_notation(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_dot_notation())
    }
}

impl From<&str> for FieldPath {
    fn from(s: &str) -> Self {
        Self::parse(s).unwrap_or_default()
    }
}

/// Result of path generation containing the path and attribute info.
#[derive(Debug, Clone)]
pub struct GeneratedPath {
    /// The field path.
    pub path: FieldPath,
    /// The attribute at this path.
    pub attribute: CompiledAttribute,
    /// The source object name (for nested objects).
    pub source_object: Option<String>,
}

/// Path generator for resolving nested object references.
pub struct PathGenerator<'a> {
    schema: &'a CompiledSchema,
    max_depth: usize,
}

impl<'a> PathGenerator<'a> {
    /// Create a new path generator.
    pub fn new(schema: &'a CompiledSchema, max_depth: usize) -> Self {
        Self { schema, max_depth }
    }

    /// Generate all field paths for an object, recursively expanding nested objects.
    pub fn generate_paths(&self, object_name: &str) -> Result<Vec<GeneratedPath>, PathError> {
        let mut paths = Vec::new();
        let mut visited = Vec::new();
        self.generate_paths_recursive(object_name, FieldPath::new(), &mut paths, &mut visited)?;
        Ok(paths)
    }

    /// Generate paths for a class, including all nested object attributes.
    pub fn generate_class_paths(&self, class_name: &str) -> Result<Vec<GeneratedPath>, PathError> {
        let class = self
            .schema
            .get_class(class_name)
            .ok_or_else(|| PathError::UnresolvedObjectType(class_name.to_string()))?;

        let mut paths = Vec::new();
        let mut visited = Vec::new();

        for (attr_name, attr) in &class.attributes {
            if attr.attr_type == "object_t" {
                if let Some(ref obj_type) = attr.object_type {
                    let base_path = FieldPath::from_segments(vec![PathSegment::new(
                        attr_name.clone(),
                        attr.is_array,
                    )]);
                    self.generate_paths_recursive(obj_type, base_path, &mut paths, &mut visited)?;
                }
            } else {
                paths.push(GeneratedPath {
                    path: FieldPath::from_segments(vec![PathSegment::new(
                        attr_name.clone(),
                        attr.is_array,
                    )]),
                    attribute: attr.clone(),
                    source_object: None,
                });
            }
        }

        Ok(paths)
    }

    fn generate_paths_recursive(
        &self,
        object_name: &str,
        base_path: FieldPath,
        paths: &mut Vec<GeneratedPath>,
        visited: &mut Vec<String>,
    ) -> Result<(), PathError> {
        // Check depth limit
        if base_path.depth() > self.max_depth {
            return Err(PathError::MaxDepthExceeded {
                depth: base_path.depth(),
            });
        }

        // Check for circular references
        if visited.contains(&object_name.to_string()) {
            return Err(PathError::CircularReference {
                path: format!("{} -> {}", visited.join(" -> "), object_name),
            });
        }

        let object = self
            .schema
            .get_object(object_name)
            .ok_or_else(|| PathError::UnresolvedObjectType(object_name.to_string()))?;

        visited.push(object_name.to_string());

        for (attr_name, attr) in &object.attributes {
            let attr_path = base_path.child(attr_name.clone(), attr.is_array);

            if attr.attr_type == "object_t" {
                if let Some(ref obj_type) = attr.object_type {
                    // Recursively expand nested objects
                    self.generate_paths_recursive(obj_type, attr_path, paths, visited)?;
                }
            } else {
                paths.push(GeneratedPath {
                    path: attr_path,
                    attribute: attr.clone(),
                    source_object: Some(object_name.to_string()),
                });
            }
        }

        visited.pop();
        Ok(())
    }

    /// Resolve a path to its attribute definition.
    pub fn resolve_path(&self, path: &FieldPath) -> Option<CompiledAttribute> {
        if path.is_empty() {
            return None;
        }

        // This is a simplified resolution - in practice you'd need to
        // traverse the schema following the path segments
        None // TODO: Implement full path resolution if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_parse_simple() {
        let path = FieldPath::parse("user.name").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].name, "user");
        assert!(!path.segments[0].is_array);
        assert_eq!(path.segments[1].name, "name");
    }

    #[test]
    fn test_path_parse_with_array() {
        let path = FieldPath::parse("users[].email").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].name, "users");
        assert!(path.segments[0].is_array);
        assert_eq!(path.segments[1].name, "email");
        assert!(!path.segments[1].is_array);
    }

    #[test]
    fn test_path_parse_empty_error() {
        assert!(matches!(FieldPath::parse(""), Err(PathError::EmptyPath)));
    }

    #[test]
    fn test_path_parse_invalid_segment() {
        assert!(matches!(
            FieldPath::parse("user..name"),
            Err(PathError::InvalidSegment(_))
        ));
    }

    #[test]
    fn test_path_to_string_roundtrip() {
        let original = "actor.user.email_addr";
        let path = FieldPath::parse(original).unwrap();
        assert_eq!(path.to_string(), original);
    }

    #[test]
    fn test_path_to_string_with_array_roundtrip() {
        let original = "observables[].value";
        let path = FieldPath::parse(original).unwrap();
        assert_eq!(path.to_string(), original);
    }

    #[test]
    fn test_path_push() {
        let mut path = FieldPath::new();
        path.push_scalar("actor");
        path.push_scalar("user");
        path.push_scalar("name");
        assert_eq!(path.to_string(), "actor.user.name");
    }

    #[test]
    fn test_path_child() {
        let path = FieldPath::parse("actor").unwrap();
        let child = path.child("user", false);
        assert_eq!(child.to_string(), "actor.user");
    }

    #[test]
    fn test_path_depth() {
        let path = FieldPath::parse("a.b.c.d").unwrap();
        assert_eq!(path.depth(), 4);
    }

    #[test]
    fn test_path_has_array() {
        let path1 = FieldPath::parse("user.name").unwrap();
        assert!(!path1.has_array());

        let path2 = FieldPath::parse("users[].name").unwrap();
        assert!(path2.has_array());
    }

    #[test]
    fn test_path_leaf_name() {
        let path = FieldPath::parse("actor.user.email").unwrap();
        assert_eq!(path.leaf_name(), Some("email"));
    }
}
