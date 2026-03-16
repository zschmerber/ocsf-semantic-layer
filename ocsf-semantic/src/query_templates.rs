//! Query template definitions for pre-built security queries.
//!
//! This module provides types for defining parameterized SQL query templates
//! with MITRE ATT&CK mappings for common security analytics use cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category of security query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum QueryCategory {
    /// Threat detection queries (e.g., DGA, tunneling, C2).
    ThreatDetection,
    /// Anomaly detection queries.
    Anomaly,
    /// Investigation and forensics queries.
    Investigation,
    /// Compliance and audit queries.
    Compliance,
    /// Operational monitoring queries.
    #[default]
    Operational,
}


/// Severity level for query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Informational - no action required.
    Informational,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity - requires attention.
    High,
    /// Critical severity - immediate action required.
    Critical,
}

/// Parameter type for query templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ParameterType {
    /// String parameter.
    #[default]
    String,
    /// Integer parameter.
    Integer,
    /// Float parameter.
    Float,
    /// Boolean parameter.
    Boolean,
}


/// A parameter for a query template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParameter {
    /// Parameter name (used in template as {{name}}).
    pub name: String,

    /// Parameter type for validation.
    #[serde(rename = "type", default)]
    pub param_type: ParameterType,

    /// Default value if not provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Description of the parameter.
    #[serde(default)]
    pub description: String,
}

impl QueryParameter {
    /// Creates a new query parameter.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::default(),
            default: None,
            description: String::new(),
        }
    }

    /// Sets the parameter type.
    pub fn with_type(mut self, param_type: ParameterType) -> Self {
        self.param_type = param_type;
        self
    }

    /// Sets the default value.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// MITRE ATT&CK mapping for a query template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MitreMapping {
    /// MITRE technique ID (e.g., "T1071.004").
    pub technique_id: String,

    /// Technique name (e.g., "Application Layer Protocol: DNS").
    pub technique_name: String,

    /// Tactic (e.g., "Command and Control").
    pub tactic: String,
}

impl MitreMapping {
    /// Creates a new MITRE mapping.
    pub fn new(
        technique_id: impl Into<String>,
        technique_name: impl Into<String>,
        tactic: impl Into<String>,
    ) -> Self {
        Self {
            technique_id: technique_id.into(),
            technique_name: technique_name.into(),
            tactic: tactic.into(),
        }
    }
}

/// A pre-built query template for security analytics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryTemplate {
    /// Unique name for the template.
    pub name: String,

    /// Natural language description of the query intent.
    pub intent: String,

    /// Example natural language patterns that match this template.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub natural_language_patterns: Vec<String>,

    /// Parameterized SQL template (uses {{param}} syntax).
    pub sql_template: String,

    /// Parameters for the template.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<QueryParameter>,

    /// MITRE ATT&CK mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mitre_attack: Option<MitreMapping>,

    /// Query category.
    #[serde(default)]
    pub category: QueryCategory,

    /// Severity level of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
}

impl QueryTemplate {
    /// Creates a new query template.
    pub fn new(name: impl Into<String>, intent: impl Into<String>, sql_template: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            intent: intent.into(),
            natural_language_patterns: Vec::new(),
            sql_template: sql_template.into(),
            parameters: Vec::new(),
            mitre_attack: None,
            category: QueryCategory::default(),
            severity: None,
        }
    }

    /// Adds natural language patterns.
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.natural_language_patterns = patterns;
        self
    }

    /// Adds a natural language pattern.
    pub fn add_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.natural_language_patterns.push(pattern.into());
        self
    }

    /// Sets the parameters.
    pub fn with_parameters(mut self, params: Vec<QueryParameter>) -> Self {
        self.parameters = params;
        self
    }

    /// Adds a parameter.
    pub fn add_parameter(mut self, param: QueryParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Sets the MITRE ATT&CK mapping.
    pub fn with_mitre_attack(mut self, mapping: MitreMapping) -> Self {
        self.mitre_attack = Some(mapping);
        self
    }

    /// Sets the category.
    pub fn with_category(mut self, category: QueryCategory) -> Self {
        self.category = category;
        self
    }

    /// Sets the severity.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Substitutes parameters in the SQL template.
    ///
    /// Parameters are specified as {{name}} in the template.
    /// Missing parameters use their default values if available.
    pub fn substitute(&self, params: &HashMap<String, String>) -> Result<String, String> {
        let mut sql = self.sql_template.clone();
        
        for param in &self.parameters {
            let placeholder = format!("{{{{{}}}}}", param.name);
            let value = params.get(&param.name)
                .or(param.default.as_ref())
                .ok_or_else(|| format!("Missing required parameter: {}", param.name))?;
            
            // Validate parameter type
            self.validate_param_value(&param.name, value, param.param_type)?;
            
            sql = sql.replace(&placeholder, value);
        }
        
        Ok(sql)
    }

    /// Validates a parameter value against its expected type.
    fn validate_param_value(&self, name: &str, value: &str, param_type: ParameterType) -> Result<(), String> {
        match param_type {
            ParameterType::Integer => {
                value.parse::<i64>()
                    .map_err(|_| format!("Parameter '{}' must be an integer, got: {}", name, value))?;
            }
            ParameterType::Float => {
                value.parse::<f64>()
                    .map_err(|_| format!("Parameter '{}' must be a float, got: {}", name, value))?;
            }
            ParameterType::Boolean => {
                if value != "true" && value != "false" {
                    return Err(format!("Parameter '{}' must be 'true' or 'false', got: {}", name, value));
                }
            }
            ParameterType::String => {
                // Any string is valid
            }
        }
        Ok(())
    }

    /// Returns all parameter names.
    pub fn parameter_names(&self) -> Vec<&str> {
        self.parameters.iter().map(|p| p.name.as_str()).collect()
    }

    /// Returns parameters with their default values.
    pub fn defaults(&self) -> HashMap<String, String> {
        self.parameters
            .iter()
            .filter_map(|p| p.default.as_ref().map(|d| (p.name.clone(), d.clone())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parameter_builder() {
        let param = QueryParameter::new("hours")
            .with_type(ParameterType::Integer)
            .with_default("24")
            .with_description("Time window in hours");

        assert_eq!(param.name, "hours");
        assert_eq!(param.param_type, ParameterType::Integer);
        assert_eq!(param.default, Some("24".to_string()));
    }

    #[test]
    fn test_mitre_mapping() {
        let mapping = MitreMapping::new(
            "T1071.004",
            "Application Layer Protocol: DNS",
            "Command and Control"
        );

        assert_eq!(mapping.technique_id, "T1071.004");
        assert_eq!(mapping.tactic, "Command and Control");
    }

    #[test]
    fn test_query_template_builder() {
        let template = QueryTemplate::new(
            "dns_tunneling",
            "Detect DNS tunneling",
            "SELECT * FROM dns WHERE length > {{min_length}}"
        )
        .add_pattern("Find DNS tunneling".to_string())
        .add_parameter(
            QueryParameter::new("min_length")
                .with_type(ParameterType::Integer)
                .with_default("50")
        )
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
        .with_mitre_attack(MitreMapping::new("T1071.004", "DNS", "C2"));

        assert_eq!(template.name, "dns_tunneling");
        assert_eq!(template.category, QueryCategory::ThreatDetection);
        assert_eq!(template.severity, Some(Severity::High));
        assert!(template.mitre_attack.is_some());
    }

    #[test]
    fn test_parameter_substitution() {
        let template = QueryTemplate::new(
            "test",
            "Test query",
            "SELECT * FROM events WHERE hours = {{hours}} AND limit = {{limit}}"
        )
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"));

        // Test with defaults
        let sql = template.substitute(&HashMap::new()).unwrap();
        assert_eq!(sql, "SELECT * FROM events WHERE hours = 24 AND limit = 100");

        // Test with custom values
        let mut params = HashMap::new();
        params.insert("hours".to_string(), "48".to_string());
        params.insert("limit".to_string(), "50".to_string());
        let sql = template.substitute(&params).unwrap();
        assert_eq!(sql, "SELECT * FROM events WHERE hours = 48 AND limit = 50");
    }

    #[test]
    fn test_parameter_validation() {
        let template = QueryTemplate::new("test", "Test", "SELECT {{num}}")
            .add_parameter(QueryParameter::new("num").with_type(ParameterType::Integer));

        let mut params = HashMap::new();
        params.insert("num".to_string(), "not_a_number".to_string());
        
        let result = template.substitute(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be an integer"));
    }

    #[test]
    fn test_missing_required_parameter() {
        let template = QueryTemplate::new("test", "Test", "SELECT {{required}}")
            .add_parameter(QueryParameter::new("required").with_type(ParameterType::String));

        let result = template.substitute(&HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required parameter"));
    }

    #[test]
    fn test_query_template_serialization_roundtrip() {
        let template = QueryTemplate::new(
            "test_template",
            "Test intent",
            "SELECT * FROM test WHERE x = {{x}}"
        )
        .add_pattern("test pattern".to_string())
        .add_parameter(QueryParameter::new("x").with_type(ParameterType::Integer).with_default("10"))
        .with_category(QueryCategory::Investigation)
        .with_severity(Severity::Medium)
        .with_mitre_attack(MitreMapping::new("T1234", "Test Technique", "Test Tactic"));

        let json = serde_json::to_string(&template).unwrap();
        let deserialized: QueryTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(template, deserialized);

        let yaml = serde_yaml::to_string(&template).unwrap();
        let deserialized_yaml: QueryTemplate = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(template, deserialized_yaml);
    }
}
