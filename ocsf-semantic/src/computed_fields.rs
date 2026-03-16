//! Computed field definitions for derived threat indicators.
//!
//! This module provides types for defining computed fields that derive
//! threat indicators from base attributes using SQL expressions.

use serde::{Deserialize, Serialize};
use crate::entity::SemanticType;

/// A computed field that derives values from base attributes.
///
/// Computed fields enable threat detection by calculating derived
/// indicators like domain entropy, subdomain depth, and query length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedField {
    /// Unique name for the computed field.
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of what this field computes.
    #[serde(default)]
    pub description: String,

    /// SQL expression to compute the field value.
    pub sql_expression: String,

    /// The data type of the computed result.
    #[serde(rename = "type", default)]
    pub data_type: SemanticType,

    /// Base fields required to compute this field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,

    /// Security context explaining threat detection relevance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_context: Option<String>,
}

impl ComputedField {
    /// Creates a new computed field.
    pub fn new(name: impl Into<String>, sql_expression: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            caption: String::new(),
            description: String::new(),
            sql_expression: sql_expression.into(),
            data_type: SemanticType::default(),
            dependencies: Vec::new(),
            security_context: None,
        }
    }

    /// Sets the caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the data type.
    pub fn with_type(mut self, data_type: SemanticType) -> Self {
        self.data_type = data_type;
        self
    }

    /// Sets the dependencies.
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Adds a dependency.
    pub fn add_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Sets the security context.
    pub fn with_security_context(mut self, context: impl Into<String>) -> Self {
        self.security_context = Some(context.into());
        self
    }

    /// Generates a SQL SELECT expression for this computed field.
    pub fn to_select_sql(&self) -> String {
        format!("({}) AS {}", self.sql_expression, self.name)
    }
}

/// Pre-built computed fields for DNS threat detection.
pub mod dns_computed_fields {
    use super::*;

    /// Domain entropy computation - high values (>3.5) may indicate DGA.
    pub fn domain_entropy() -> ComputedField {
        ComputedField::new(
            "domain_entropy",
            "LENGTH(query_hostname) - LENGTH(REPLACE(LOWER(query_hostname), 'a', '')) + \
             LENGTH(query_hostname) - LENGTH(REPLACE(LOWER(query_hostname), 'e', '')) + \
             LENGTH(query_hostname) - LENGTH(REPLACE(LOWER(query_hostname), 'i', '')) + \
             LENGTH(query_hostname) - LENGTH(REPLACE(LOWER(query_hostname), 'o', '')) + \
             LENGTH(query_hostname) - LENGTH(REPLACE(LOWER(query_hostname), 'u', ''))"
        )
        .with_caption("Domain Entropy")
        .with_description("Approximation of Shannon entropy - high values may indicate DGA")
        .with_type(SemanticType::Float)
        .add_dependency("query_hostname".to_string())
        .with_security_context("High entropy (>3.5) in domain names often indicates algorithmically generated domains used by malware for C2 communication.")
    }

    /// Subdomain depth - deep nesting may indicate tunneling.
    pub fn subdomain_depth() -> ComputedField {
        ComputedField::new(
            "subdomain_depth",
            "LENGTH(query_hostname) - LENGTH(REPLACE(query_hostname, '.', '')) - 1"
        )
        .with_caption("Subdomain Depth")
        .with_description("Number of subdomain levels - deep nesting may indicate tunneling")
        .with_type(SemanticType::Integer)
        .add_dependency("query_hostname".to_string())
        .with_security_context("Deep subdomain nesting (>4 levels) can indicate DNS tunneling where data is encoded in subdomain labels.")
    }

    /// Query length - long queries may indicate data encoding.
    pub fn query_length() -> ComputedField {
        ComputedField::new(
            "query_length",
            "LENGTH(query_hostname)"
        )
        .with_caption("Query Length")
        .with_description("Character length of hostname - long queries may indicate data encoding")
        .with_type(SemanticType::Integer)
        .add_dependency("query_hostname".to_string())
        .with_security_context("Unusually long DNS queries (>50 characters) may indicate data exfiltration via DNS tunneling.")
    }

    /// Is numeric heavy - true if hostname contains >40% numeric characters.
    pub fn is_numeric_heavy() -> ComputedField {
        ComputedField::new(
            "is_numeric_heavy",
            "CASE WHEN (LENGTH(REGEXP_REPLACE(query_hostname, '[^0-9]', '')) * 1.0 / NULLIF(LENGTH(query_hostname), 0)) > 0.4 THEN true ELSE false END"
        )
        .with_caption("Is Numeric Heavy")
        .with_description("True if hostname contains >40% numeric characters - DGA indicator")
        .with_type(SemanticType::Boolean)
        .add_dependency("query_hostname".to_string())
        .with_security_context("High numeric content in domain names is a strong indicator of DGA-generated domains.")
    }

    /// Returns all DNS computed fields.
    pub fn all() -> Vec<ComputedField> {
        vec![
            domain_entropy(),
            subdomain_depth(),
            query_length(),
            is_numeric_heavy(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computed_field_builder() {
        let field = ComputedField::new("test_field", "LENGTH(name)")
            .with_caption("Test Field")
            .with_description("A test computed field")
            .with_type(SemanticType::Integer)
            .add_dependency("name".to_string())
            .with_security_context("Test context");

        assert_eq!(field.name, "test_field");
        assert_eq!(field.caption, "Test Field");
        assert_eq!(field.sql_expression, "LENGTH(name)");
        assert_eq!(field.dependencies, vec!["name"]);
        assert!(field.security_context.is_some());
    }

    #[test]
    fn test_computed_field_to_select_sql() {
        let field = ComputedField::new("query_length", "LENGTH(query_hostname)");
        assert_eq!(field.to_select_sql(), "(LENGTH(query_hostname)) AS query_length");
    }

    #[test]
    fn test_dns_computed_fields() {
        let fields = dns_computed_fields::all();
        assert_eq!(fields.len(), 4);
        
        let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"domain_entropy"));
        assert!(names.contains(&"subdomain_depth"));
        assert!(names.contains(&"query_length"));
        assert!(names.contains(&"is_numeric_heavy"));
    }

    #[test]
    fn test_computed_field_serialization_roundtrip() {
        let field = dns_computed_fields::subdomain_depth();
        
        let json = serde_json::to_string(&field).unwrap();
        let deserialized: ComputedField = serde_json::from_str(&json).unwrap();
        assert_eq!(field, deserialized);

        let yaml = serde_yaml::to_string(&field).unwrap();
        let deserialized_yaml: ComputedField = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(field, deserialized_yaml);
    }
}
