//! Dimension inference from OCSF schema attributes.
//!
//! This module provides types and logic for inferring which attributes
//! should become dimensions in the semantic layer based on schema metadata.

use ocsf_core::{CompiledAttribute, CompiledRequirement};
use serde::{Deserialize, Serialize};

/// Type of dimension inferred from attribute metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DimensionType {
    /// Enum-based categorical dimension (e.g., status_id, activity_id).
    Categorical,
    /// Timestamp-based temporal dimension.
    Temporal,
    /// Identifier dimension (UID, name fields).
    Identifier,
    /// Observable/searchable dimension.
    Searchable,
    /// Numeric dimension for range queries.
    Numeric,
    /// Text dimension for full-text search.
    #[default]
    Text,
}


/// Priority level for dimensions based on requirement and group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DimensionPriority {
    /// Low priority - optional attributes.
    #[default]
    Low = 0,
    /// Medium priority - recommended attributes.
    Medium = 1,
    /// High priority - required or primary group attributes.
    High = 2,
}


/// Result of dimension inference for an attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionInfo {
    /// Whether this attribute should be a dimension.
    pub is_dimension: bool,
    /// The type of dimension.
    pub dimension_type: DimensionType,
    /// Priority level for this dimension.
    pub priority: DimensionPriority,
    /// Sibling attribute name (e.g., activity_id → activity_name).
    pub sibling_attribute: Option<String>,
    /// Reason for the inference decision.
    pub reason: String,
}

impl Default for DimensionInfo {
    fn default() -> Self {
        Self {
            is_dimension: false,
            dimension_type: DimensionType::Text,
            priority: DimensionPriority::Low,
            sibling_attribute: None,
            reason: String::new(),
        }
    }
}

/// Trait for dimension inference implementations.
pub trait DimensionInferrer {
    /// Infer dimension information from an attribute.
    fn infer(&self, attr: &CompiledAttribute) -> DimensionInfo;
    
    /// Infer dimension information with attribute name context.
    fn infer_with_name(&self, name: &str, attr: &CompiledAttribute) -> DimensionInfo;
}

/// Default dimension inference implementation based on OCSF metadata.
#[derive(Debug, Clone, Default)]
pub struct DefaultDimensionInferrer;

impl DefaultDimensionInferrer {
    /// Create a new default dimension inferrer.
    pub fn new() -> Self {
        Self
    }
    
    /// Check if attribute type is a timestamp.
    fn is_timestamp_type(attr_type: &str) -> bool {
        attr_type == "timestamp_t" || attr_type == "datetime_t"
    }
    
    /// Check if attribute type is numeric.
    fn is_numeric_type(attr_type: &str) -> bool {
        matches!(attr_type, "integer_t" | "long_t" | "float_t" | "double_t")
    }
    
    /// Check if attribute name suggests an identifier.
    fn is_identifier_name(name: &str) -> bool {
        name.ends_with("_uid") || 
        name.ends_with("_id") || 
        name == "uid" || 
        name == "id" ||
        name == "name" ||
        name.ends_with("_name")
    }
}

impl DimensionInferrer for DefaultDimensionInferrer {
    fn infer(&self, attr: &CompiledAttribute) -> DimensionInfo {
        self.infer_with_name("", attr)
    }
    
    fn infer_with_name(&self, name: &str, attr: &CompiledAttribute) -> DimensionInfo {
        let is_enum = !attr.enum_values.is_empty();
        let is_required = attr.requirement == CompiledRequirement::Required;
        let is_recommended = attr.requirement == CompiledRequirement::Recommended;
        let is_primary = attr.group.as_deref() == Some("primary");
        let is_classification = attr.group.as_deref() == Some("classification");
        let is_timestamp = Self::is_timestamp_type(&attr.attr_type);
        let has_observable = attr.observable.is_some();
        let is_object = attr.attr_type == "object_t";
        
        // Determine dimension type
        let dimension_type = if is_enum {
            DimensionType::Categorical
        } else if is_timestamp {
            DimensionType::Temporal
        } else if has_observable {
            DimensionType::Searchable
        } else if Self::is_identifier_name(name) {
            DimensionType::Identifier
        } else if Self::is_numeric_type(&attr.attr_type) {
            DimensionType::Numeric
        } else {
            DimensionType::Text
        };
        
        // Determine priority
        let priority = if is_required || is_primary {
            DimensionPriority::High
        } else if is_recommended || is_classification {
            DimensionPriority::Medium
        } else {
            DimensionPriority::Low
        };
        
        // Determine if this should be a dimension
        let (is_dimension, reason) = if is_enum {
            (true, "Enum attribute with categorical values".to_string())
        } else if is_required && !is_object {
            (true, "Required attribute".to_string())
        } else if is_primary && !is_object {
            (true, "Primary group attribute".to_string())
        } else if is_classification {
            (true, "Classification group attribute".to_string())
        } else if has_observable {
            (true, "Observable attribute for threat intelligence".to_string())
        } else if is_timestamp {
            (true, "Timestamp for temporal analysis".to_string())
        } else if Self::is_identifier_name(name) && (is_required || is_recommended) {
            (true, "Identifier attribute".to_string())
        } else {
            (false, "Does not meet dimension criteria".to_string())
        };
        
        DimensionInfo {
            is_dimension,
            dimension_type,
            priority,
            sibling_attribute: attr.sibling.clone(),
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn make_attr(attr_type: &str, requirement: CompiledRequirement) -> CompiledAttribute {
        CompiledAttribute {
            caption: String::new(),
            description: String::new(),
            attr_type: attr_type.to_string(),
            type_name: String::new(),
            requirement,
            group: None,
            object_type: None,
            object_name: None,
            is_array: false,
            enum_values: HashMap::new(),
            observable: None,
            sibling: None,
            profiles: None,
            deprecated: None,
        }
    }
    
    #[test]
    fn test_enum_attribute_is_categorical_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("integer_t", CompiledRequirement::Required);
        attr.enum_values.insert("0".to_string(), ocsf_core::CompiledEnumValue {
            caption: "Unknown".to_string(),
            description: String::new(),
            deprecated: None,
        });
        
        let info = inferrer.infer_with_name("status_id", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.dimension_type, DimensionType::Categorical);
        assert_eq!(info.priority, DimensionPriority::High);
    }
    
    #[test]
    fn test_required_attribute_is_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let attr = make_attr("string_t", CompiledRequirement::Required);
        
        let info = inferrer.infer_with_name("user_name", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.priority, DimensionPriority::High);
    }
    
    #[test]
    fn test_primary_group_is_high_priority() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("string_t", CompiledRequirement::Optional);
        attr.group = Some("primary".to_string());
        
        let info = inferrer.infer_with_name("action", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.priority, DimensionPriority::High);
    }
    
    #[test]
    fn test_timestamp_is_temporal_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let attr = make_attr("timestamp_t", CompiledRequirement::Required);
        
        let info = inferrer.infer_with_name("time", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.dimension_type, DimensionType::Temporal);
    }
    
    #[test]
    fn test_observable_is_searchable_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("string_t", CompiledRequirement::Optional);
        attr.observable = Some(2); // IP address observable
        
        let info = inferrer.infer_with_name("ip", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.dimension_type, DimensionType::Searchable);
    }
    
    #[test]
    fn test_sibling_attribute_preserved() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("integer_t", CompiledRequirement::Required);
        attr.sibling = Some("activity_name".to_string());
        attr.enum_values.insert("0".to_string(), ocsf_core::CompiledEnumValue {
            caption: "Unknown".to_string(),
            description: String::new(),
            deprecated: None,
        });
        
        let info = inferrer.infer_with_name("activity_id", &attr);
        assert_eq!(info.sibling_attribute, Some("activity_name".to_string()));
    }
    
    #[test]
    fn test_optional_non_special_is_not_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let attr = make_attr("string_t", CompiledRequirement::Optional);
        
        let info = inferrer.infer_with_name("some_field", &attr);
        assert!(!info.is_dimension);
    }
    
    #[test]
    fn test_object_type_not_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("object_t", CompiledRequirement::Required);
        attr.object_type = Some("user".to_string());
        
        let info = inferrer.infer_with_name("actor", &attr);
        // Objects themselves are not dimensions, their nested attributes are
        assert!(!info.is_dimension);
    }
    
    #[test]
    fn test_classification_group_is_dimension() {
        let inferrer = DefaultDimensionInferrer::new();
        let mut attr = make_attr("integer_t", CompiledRequirement::Required);
        attr.group = Some("classification".to_string());
        attr.enum_values.insert("0".to_string(), ocsf_core::CompiledEnumValue {
            caption: "Unknown".to_string(),
            description: String::new(),
            deprecated: None,
        });
        
        let info = inferrer.infer_with_name("activity_id", &attr);
        assert!(info.is_dimension);
        assert_eq!(info.priority, DimensionPriority::High); // Required takes precedence
    }
}
