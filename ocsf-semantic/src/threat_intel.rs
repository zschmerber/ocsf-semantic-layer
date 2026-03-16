//! Threat intelligence integration types.
//!
//! This module provides types for defining threat intelligence joins
//! and IOC matching patterns for security analytics.

use serde::{Deserialize, Serialize};
use crate::entity::SemanticType;

/// Schema field definition for threat intel tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field name.
    pub name: String,

    /// Field type.
    #[serde(rename = "type", default)]
    pub field_type: SemanticType,

    /// Field description.
    #[serde(default)]
    pub description: String,
}

impl SchemaField {
    /// Creates a new schema field.
    pub fn new(name: impl Into<String>, field_type: SemanticType) -> Self {
        Self {
            name: name.into(),
            field_type,
            description: String::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// Threat intelligence join definition.
///
/// Defines how to join security event data with threat intelligence
/// IOC (Indicator of Compromise) tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatIntelJoin {
    /// Name of the join definition.
    pub name: String,

    /// Description of the join purpose.
    #[serde(default)]
    pub description: String,

    /// Target threat intel table name.
    pub target_table: String,

    /// Expected schema of the threat intel table.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_schema: Vec<SchemaField>,

    /// Join conditions (SQL expressions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub join_conditions: Vec<String>,
}

impl ThreatIntelJoin {
    /// Creates a new threat intel join.
    pub fn new(name: impl Into<String>, target_table: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            target_table: target_table.into(),
            expected_schema: Vec::new(),
            join_conditions: Vec::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the expected schema.
    pub fn with_schema(mut self, schema: Vec<SchemaField>) -> Self {
        self.expected_schema = schema;
        self
    }

    /// Adds a schema field.
    pub fn add_schema_field(mut self, field: SchemaField) -> Self {
        self.expected_schema.push(field);
        self
    }

    /// Sets the join conditions.
    pub fn with_join_conditions(mut self, conditions: Vec<String>) -> Self {
        self.join_conditions = conditions;
        self
    }

    /// Adds a join condition.
    pub fn add_join_condition(mut self, condition: impl Into<String>) -> Self {
        self.join_conditions.push(condition.into());
        self
    }

    /// Generates a SQL JOIN clause.
    pub fn to_join_sql(&self, source_table: &str) -> String {
        if self.join_conditions.is_empty() {
            return format!("CROSS JOIN {}", self.target_table);
        }

        let conditions = self.join_conditions
            .iter()
            .map(|c| c.replace("{source}", source_table))
            .collect::<Vec<_>>()
            .join(" OR ");

        format!("LEFT JOIN {} ON ({})", self.target_table, conditions)
    }
}

/// Pre-built threat intel join definitions.
pub mod prebuilt {
    use super::*;

    /// Domain IOC lookup join.
    pub fn domain_ioc_lookup() -> ThreatIntelJoin {
        ThreatIntelJoin::new("domain_ioc_lookup", "threat_intel_ioc")
            .with_description("Join DNS queries with domain-based threat intelligence")
            .with_schema(vec![
                SchemaField::new("indicator", SemanticType::String)
                    .with_description("The IOC value (domain, IP, hash)"),
                SchemaField::new("indicator_type", SemanticType::String)
                    .with_description("Type of indicator (domain, ip, hash)"),
                SchemaField::new("threat_type", SemanticType::String)
                    .with_description("Category of threat (malware, phishing, c2)"),
                SchemaField::new("confidence", SemanticType::Integer)
                    .with_description("Confidence score 0-100"),
                SchemaField::new("source", SemanticType::String)
                    .with_description("Threat intel feed source"),
                SchemaField::new("first_seen", SemanticType::Timestamp),
                SchemaField::new("last_seen", SemanticType::Timestamp),
            ])
            .with_join_conditions(vec![
                "{source}.query_hostname = threat_intel_ioc.indicator".to_string(),
                "{source}.query_hostname LIKE '%.' || threat_intel_ioc.indicator".to_string(),
            ])
    }

    /// IP IOC lookup join.
    pub fn ip_ioc_lookup() -> ThreatIntelJoin {
        ThreatIntelJoin::new("ip_ioc_lookup", "threat_intel_ioc")
            .with_description("Join network events with IP-based threat intelligence")
            .with_schema(vec![
                SchemaField::new("indicator", SemanticType::String)
                    .with_description("The IOC value (IP address)"),
                SchemaField::new("indicator_type", SemanticType::String)
                    .with_description("Type of indicator (ip)"),
                SchemaField::new("threat_type", SemanticType::String)
                    .with_description("Category of threat (malware, c2, scanner)"),
                SchemaField::new("confidence", SemanticType::Integer)
                    .with_description("Confidence score 0-100"),
                SchemaField::new("source", SemanticType::String)
                    .with_description("Threat intel feed source"),
            ])
            .with_join_conditions(vec![
                "{source}.source_ip = threat_intel_ioc.indicator".to_string(),
                "{source}.destination_ip = threat_intel_ioc.indicator".to_string(),
            ])
    }

    /// Returns all pre-built threat intel joins.
    pub fn all() -> Vec<ThreatIntelJoin> {
        vec![
            domain_ioc_lookup(),
            ip_ioc_lookup(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_field() {
        let field = SchemaField::new("indicator", SemanticType::String)
            .with_description("The IOC value");

        assert_eq!(field.name, "indicator");
        assert_eq!(field.field_type, SemanticType::String);
        assert_eq!(field.description, "The IOC value");
    }

    #[test]
    fn test_threat_intel_join_builder() {
        let join = ThreatIntelJoin::new("test_join", "ioc_table")
            .with_description("Test join")
            .add_schema_field(SchemaField::new("indicator", SemanticType::String))
            .add_join_condition("events.domain = ioc_table.indicator");

        assert_eq!(join.name, "test_join");
        assert_eq!(join.target_table, "ioc_table");
        assert_eq!(join.expected_schema.len(), 1);
        assert_eq!(join.join_conditions.len(), 1);
    }

    #[test]
    fn test_to_join_sql() {
        let join = ThreatIntelJoin::new("test", "ioc_table")
            .add_join_condition("{source}.domain = ioc_table.indicator")
            .add_join_condition("{source}.domain LIKE '%.' || ioc_table.indicator");

        let sql = join.to_join_sql("dns_event");
        assert!(sql.contains("LEFT JOIN ioc_table"));
        assert!(sql.contains("dns_event.domain = ioc_table.indicator"));
        assert!(sql.contains(" OR "));
    }

    #[test]
    fn test_domain_ioc_lookup() {
        let join = prebuilt::domain_ioc_lookup();
        assert_eq!(join.name, "domain_ioc_lookup");
        assert_eq!(join.target_table, "threat_intel_ioc");
        assert!(!join.expected_schema.is_empty());
        assert!(!join.join_conditions.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let join = prebuilt::domain_ioc_lookup();

        let json = serde_json::to_string(&join).unwrap();
        let deserialized: ThreatIntelJoin = serde_json::from_str(&json).unwrap();
        assert_eq!(join, deserialized);

        let yaml = serde_yaml::to_string(&join).unwrap();
        let deserialized_yaml: ThreatIntelJoin = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(join, deserialized_yaml);
    }
}
