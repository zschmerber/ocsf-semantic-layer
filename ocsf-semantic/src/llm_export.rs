//! LLM-optimized export format for semantic models.
//!
//! This module provides functionality to export semantic models in a compact
//! format optimized for LLM context windows, including synonyms, descriptions,
//! query templates, and other metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::computed_fields::ComputedField;
use crate::entity::{SemanticAttribute, SemanticEntity};
use crate::model::SemanticModel;
use crate::query_templates::QueryTemplate;
use crate::threat_intel::ThreatIntelJoin;

/// Sections that can be included in the LLM export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportSection {
    /// Entity and attribute definitions with synonyms.
    Entities,
    /// Synonym lookup table.
    Synonyms,
    /// Query templates with natural language patterns.
    QueryTemplates,
    /// Computed field definitions.
    ComputedFields,
    /// Threat intelligence join definitions.
    ThreatIntel,
    /// Sample values for attributes.
    SampleValues,
    /// Security context descriptions.
    SecurityContext,
}

impl ExportSection {
    /// Returns all available sections.
    pub fn all() -> Vec<ExportSection> {
        vec![
            ExportSection::Entities,
            ExportSection::Synonyms,
            ExportSection::QueryTemplates,
            ExportSection::ComputedFields,
            ExportSection::ThreatIntel,
            ExportSection::SampleValues,
            ExportSection::SecurityContext,
        ]
    }
}

/// Configuration for LLM export.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Sections to include in the export.
    pub sections: Vec<ExportSection>,
    /// Whether to include detailed descriptions.
    pub include_descriptions: bool,
    /// Maximum number of sample values per attribute.
    pub max_sample_values: usize,
    /// Maximum number of natural language patterns per template.
    pub max_patterns: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            sections: ExportSection::all(),
            include_descriptions: true,
            max_sample_values: 5,
            max_patterns: 5,
        }
    }
}

impl ExportConfig {
    /// Creates a new export config with all sections.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a minimal export config for smaller context windows.
    pub fn minimal() -> Self {
        Self {
            sections: vec![
                ExportSection::Entities,
                ExportSection::Synonyms,
                ExportSection::QueryTemplates,
            ],
            include_descriptions: false,
            max_sample_values: 2,
            max_patterns: 3,
        }
    }

    /// Sets the sections to include.
    pub fn with_sections(mut self, sections: Vec<ExportSection>) -> Self {
        self.sections = sections;
        self
    }

    /// Sets whether to include descriptions.
    pub fn with_descriptions(mut self, include: bool) -> Self {
        self.include_descriptions = include;
        self
    }
}

/// Compact attribute representation for LLM context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactAttribute {
    /// Attribute name.
    pub name: String,
    /// Human-readable caption.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub caption: String,
    /// Data type.
    #[serde(rename = "type")]
    pub data_type: String,
    /// Alternative names (synonyms).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
    /// Description (optional based on config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Security context (optional based on config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_context: Option<String>,
    /// Sample values (limited by config).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sample_values: Vec<String>,
    /// Whether this is a dimension for grouping.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_dimension: bool,
    /// Whether this is an observable (IOC-relevant).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_observable: bool,
}

/// Compact entity representation for LLM context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactEntity {
    /// Entity name.
    pub name: String,
    /// Human-readable caption.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub caption: String,
    /// Description (optional based on config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Attributes.
    pub attributes: Vec<CompactAttribute>,
}

/// Compact query template for LLM context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactQueryTemplate {
    /// Template name.
    pub name: String,
    /// Natural language intent.
    pub intent: String,
    /// Example questions/patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,
    /// SQL template.
    pub sql: String,
    /// Parameter names with defaults.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, String>,
    /// MITRE ATT&CK technique (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitre_technique: Option<String>,
    /// Category.
    pub category: String,
}

/// Compact computed field for LLM context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactComputedField {
    /// Field name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// SQL expression.
    pub sql: String,
    /// Dependencies.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

/// The complete LLM export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMExport {
    /// Model name.
    pub model_name: String,
    /// Model version.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// Entities with attributes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<CompactEntity>,
    /// Synonym lookup table (synonym -> canonical name).
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub synonyms: HashMap<String, String>,
    /// Query templates.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query_templates: Vec<CompactQueryTemplate>,
    /// Computed fields.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub computed_fields: Vec<CompactComputedField>,
    /// Threat intel join definitions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub threat_intel_joins: Vec<String>,
}

impl LLMExport {
    /// Creates a new LLM export from a semantic model.
    pub fn from_model(model: &SemanticModel, config: &ExportConfig) -> Self {
        let mut export = Self {
            model_name: model.name.clone(),
            version: model.version.clone(),
            entities: Vec::new(),
            synonyms: HashMap::new(),
            query_templates: Vec::new(),
            computed_fields: Vec::new(),
            threat_intel_joins: Vec::new(),
        };

        if config.sections.contains(&ExportSection::Entities) {
            export.entities = model.entities.iter()
                .map(|e| Self::compact_entity(e, config))
                .collect();
        }

        if config.sections.contains(&ExportSection::Synonyms) {
            export.synonyms = Self::build_synonym_map(model);
        }

        export
    }

    /// Adds query templates to the export.
    pub fn with_query_templates(mut self, templates: &[QueryTemplate], config: &ExportConfig) -> Self {
        if config.sections.contains(&ExportSection::QueryTemplates) {
            self.query_templates = templates.iter()
                .map(|t| Self::compact_template(t, config))
                .collect();
        }
        self
    }

    /// Adds computed fields to the export.
    pub fn with_computed_fields(mut self, fields: &[ComputedField], config: &ExportConfig) -> Self {
        if config.sections.contains(&ExportSection::ComputedFields) {
            self.computed_fields = fields.iter()
                .map(Self::compact_computed_field)
                .collect();
        }
        self
    }

    /// Adds threat intel joins to the export.
    pub fn with_threat_intel(mut self, joins: &[ThreatIntelJoin], config: &ExportConfig) -> Self {
        if config.sections.contains(&ExportSection::ThreatIntel) {
            self.threat_intel_joins = joins.iter()
                .map(|j| format!("{}: {}", j.name, j.description))
                .collect();
        }
        self
    }

    /// Exports to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Exports to compact JSON string (no pretty printing).
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Exports to YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    fn compact_entity(entity: &SemanticEntity, config: &ExportConfig) -> CompactEntity {
        CompactEntity {
            name: entity.name.clone(),
            caption: entity.caption.clone(),
            description: if config.include_descriptions && !entity.description.is_empty() {
                Some(entity.description.clone())
            } else {
                None
            },
            attributes: entity.attributes.iter()
                .map(|a| Self::compact_attribute(a, config))
                .collect(),
        }
    }

    fn compact_attribute(attr: &SemanticAttribute, config: &ExportConfig) -> CompactAttribute {
        CompactAttribute {
            name: attr.name.clone(),
            caption: attr.caption.clone(),
            data_type: format!("{:?}", attr.attr_type).to_lowercase(),
            synonyms: attr.synonyms.clone(),
            description: if config.include_descriptions 
                && config.sections.contains(&ExportSection::SecurityContext)
                && !attr.description.is_empty() 
            {
                Some(attr.description.clone())
            } else {
                None
            },
            security_context: if config.sections.contains(&ExportSection::SecurityContext) {
                attr.security_context.clone()
            } else {
                None
            },
            sample_values: if config.sections.contains(&ExportSection::SampleValues) {
                attr.sample_values.iter()
                    .take(config.max_sample_values)
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            },
            is_dimension: attr.is_dimension,
            is_observable: attr.is_observable,
        }
    }

    fn compact_template(template: &QueryTemplate, config: &ExportConfig) -> CompactQueryTemplate {
        CompactQueryTemplate {
            name: template.name.clone(),
            intent: template.intent.clone(),
            patterns: template.natural_language_patterns.iter()
                .take(config.max_patterns)
                .cloned()
                .collect(),
            sql: template.sql_template.clone(),
            parameters: template.parameters.iter()
                .filter_map(|p| p.default.as_ref().map(|d| (p.name.clone(), d.clone())))
                .collect(),
            mitre_technique: template.mitre_attack.as_ref()
                .map(|m| format!("{} - {}", m.technique_id, m.technique_name)),
            category: format!("{:?}", template.category),
        }
    }

    fn compact_computed_field(field: &ComputedField) -> CompactComputedField {
        CompactComputedField {
            name: field.name.clone(),
            description: field.description.clone(),
            sql: field.sql_expression.clone(),
            dependencies: field.dependencies.clone(),
        }
    }

    fn build_synonym_map(model: &SemanticModel) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for entity in &model.entities {
            for attr in &entity.attributes {
                for synonym in &attr.synonyms {
                    map.insert(synonym.to_lowercase(), attr.name.clone());
                }
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{SemanticAttribute, SemanticEntity, SemanticType};

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test_model")
            .with_version("1.0.0".to_string())
            .add_entity(
                SemanticEntity::new("dns_event")
                    .with_caption("DNS Event")
                    .with_description("DNS query events")
                    .add_attribute(
                        SemanticAttribute::new("query_hostname")
                            .with_caption("Query Hostname")
                            .with_description("The domain being queried")
                            .with_type(SemanticType::String)
                            .with_synonyms(vec!["domain".to_string(), "fqdn".to_string()])
                            .with_security_context("Check for DGA patterns")
                            .with_sample_values(vec!["example.com".to_string()])
                            .as_dimension()
                            .as_observable()
                    )
            )
    }

    #[test]
    fn test_export_with_default_config() {
        let model = create_test_model();
        let config = ExportConfig::default();
        let export = LLMExport::from_model(&model, &config);

        assert_eq!(export.model_name, "test_model");
        assert_eq!(export.version, "1.0.0");
        assert_eq!(export.entities.len(), 1);
        assert!(!export.synonyms.is_empty());
    }

    #[test]
    fn test_export_with_minimal_config() {
        let model = create_test_model();
        let config = ExportConfig::minimal();
        let export = LLMExport::from_model(&model, &config);

        assert_eq!(export.entities.len(), 1);
        // Minimal config should not include descriptions
        assert!(export.entities[0].description.is_none());
    }

    #[test]
    fn test_synonym_map() {
        let model = create_test_model();
        let config = ExportConfig::default();
        let export = LLMExport::from_model(&model, &config);

        assert_eq!(export.synonyms.get("domain"), Some(&"query_hostname".to_string()));
        assert_eq!(export.synonyms.get("fqdn"), Some(&"query_hostname".to_string()));
    }

    #[test]
    fn test_export_to_json() {
        let model = create_test_model();
        let config = ExportConfig::default();
        let export = LLMExport::from_model(&model, &config);

        let json = export.to_json().unwrap();
        assert!(json.contains("test_model"));
        assert!(json.contains("query_hostname"));
    }

    #[test]
    fn test_export_to_yaml() {
        let model = create_test_model();
        let config = ExportConfig::default();
        let export = LLMExport::from_model(&model, &config);

        let yaml = export.to_yaml().unwrap();
        assert!(yaml.contains("test_model"));
        assert!(yaml.contains("query_hostname"));
    }

    #[test]
    fn test_export_with_query_templates() {
        use crate::dns_templates;
        
        let model = create_test_model();
        let config = ExportConfig::default();
        let templates = dns_templates::all_templates();
        
        let export = LLMExport::from_model(&model, &config)
            .with_query_templates(&templates, &config);

        assert!(!export.query_templates.is_empty());
        assert!(export.query_templates.iter().any(|t| t.name == "dns_tunneling_high_volume"));
    }

    #[test]
    fn test_export_with_computed_fields() {
        use crate::computed_fields::dns_computed_fields;
        
        let model = create_test_model();
        let config = ExportConfig::default();
        let fields = dns_computed_fields::all();
        
        let export = LLMExport::from_model(&model, &config)
            .with_computed_fields(&fields, &config);

        assert!(!export.computed_fields.is_empty());
        assert!(export.computed_fields.iter().any(|f| f.name == "domain_entropy"));
    }

    #[test]
    fn test_section_filtering() {
        let model = create_test_model();
        let config = ExportConfig::new()
            .with_sections(vec![ExportSection::Entities]);
        
        let export = LLMExport::from_model(&model, &config);

        assert!(!export.entities.is_empty());
        assert!(export.synonyms.is_empty()); // Not included
    }
}
