use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A catalog manifest tracking multiple semantic models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticCatalog {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub models: Vec<CatalogEntry>,
}

/// An entry in the semantic catalog referencing a single model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub model_name: String,
    pub path: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

impl SemanticCatalog {
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize semantic catalog to YAML")
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse semantic catalog from YAML")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_yaml_roundtrip() {
        let catalog = SemanticCatalog {
            name: "test_catalog".to_string(),
            version: "1.0".to_string(),
            description: Some("A test catalog".to_string()),
            models: vec![
                CatalogEntry {
                    model_name: "model_a".to_string(),
                    path: "models/a.yaml".to_string(),
                    version: "1.0".to_string(),
                    dependencies: vec![],
                },
                CatalogEntry {
                    model_name: "model_b".to_string(),
                    path: "models/b.yaml".to_string(),
                    version: "2.0".to_string(),
                    dependencies: vec!["model_a".to_string()],
                },
            ],
        };

        let yaml = catalog.to_yaml().unwrap();
        let deserialized = SemanticCatalog::from_yaml(&yaml).unwrap();
        assert_eq!(catalog, deserialized);
    }

    #[test]
    fn test_catalog_without_optional_fields() {
        let catalog = SemanticCatalog {
            name: "minimal".to_string(),
            version: "0.1".to_string(),
            description: None,
            models: vec![],
        };

        let yaml = catalog.to_yaml().unwrap();
        assert!(!yaml.contains("description"));
        let deserialized = SemanticCatalog::from_yaml(&yaml).unwrap();
        assert_eq!(catalog, deserialized);
    }

    #[test]
    fn test_catalog_entry_empty_dependencies_omitted() {
        let entry = CatalogEntry {
            model_name: "test".to_string(),
            path: "test.yaml".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
        };

        let yaml = serde_yaml::to_string(&entry).unwrap();
        assert!(!yaml.contains("dependencies"));
    }
}
