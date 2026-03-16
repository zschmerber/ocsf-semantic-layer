//! Configuration management for the OCSF CLI.
//!
//! This module provides configuration loading and management for the CLI,
//! supporting both file-based configuration and environment variables.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// CLI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default schema cache directory.
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Default OCSF schema version.
    #[serde(default = "default_ocsf_version")]
    pub default_ocsf_version: String,

    /// Default warehouse dialect.
    #[serde(default = "default_dialect")]
    pub default_dialect: String,

    /// dbt configuration.
    #[serde(default)]
    pub dbt: DbtConfig,

    /// Cube.js configuration.
    #[serde(default)]
    pub cubejs: CubeJsConfig,

    /// Visualization configuration.
    #[serde(default)]
    pub visualization: VisualizationConfig,
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ocsf-semantic")
}

fn default_ocsf_version() -> String {
    "1.4.0".to_string()
}

fn default_dialect() -> String {
    "snowflake".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
            default_ocsf_version: default_ocsf_version(),
            default_dialect: default_dialect(),
            dbt: DbtConfig::default(),
            cubejs: CubeJsConfig::default(),
            visualization: VisualizationConfig::default(),
        }
    }
}

#[allow(dead_code)]
impl Config {
    /// Loads configuration from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;

        Ok(config)
    }

    /// Saves configuration to a file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;

        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Returns the schema cache path for a given version.
    pub fn schema_cache_path(&self, version: &str) -> PathBuf {
        self.cache_dir.join("schemas").join(version)
    }

    /// Returns the default schema cache path.
    pub fn default_schema_path(&self) -> PathBuf {
        self.schema_cache_path(&self.default_ocsf_version)
    }
}

/// dbt-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbtConfig {
    /// Source name for OCSF tables.
    #[serde(default = "default_source_name")]
    pub source_name: String,

    /// Database name.
    pub database: Option<String>,

    /// Schema name.
    pub schema: Option<String>,

    /// Table prefix for OCSF tables.
    #[serde(default = "default_table_prefix")]
    pub table_prefix: String,
}

fn default_source_name() -> String {
    "ocsf".to_string()
}

fn default_table_prefix() -> String {
    "ocsf_".to_string()
}

impl Default for DbtConfig {
    fn default() -> Self {
        Self {
            source_name: default_source_name(),
            database: None,
            schema: None,
            table_prefix: default_table_prefix(),
        }
    }
}

/// Cube.js-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeJsConfig {
    /// Data source name.
    #[serde(default = "default_data_source")]
    pub data_source: String,

    /// Schema path for generated files.
    #[serde(default = "default_schema_path")]
    pub schema_path: String,
}

fn default_data_source() -> String {
    "default".to_string()
}

fn default_schema_path() -> String {
    "schema".to_string()
}

impl Default for CubeJsConfig {
    fn default() -> Self {
        Self {
            data_source: default_data_source(),
            schema_path: default_schema_path(),
        }
    }
}

/// Visualization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Default layout algorithm.
    #[serde(default = "default_layout")]
    pub default_layout: String,

    /// Default view mode.
    #[serde(default = "default_view_mode")]
    pub default_view_mode: String,

    /// Whether to show observables by default.
    #[serde(default = "default_true")]
    pub show_observables: bool,

    /// Whether to show relationships by default.
    #[serde(default = "default_true")]
    pub show_relationships: bool,
}

fn default_layout() -> String {
    "hierarchical".to_string()
}

fn default_view_mode() -> String {
    "interchange".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            default_layout: default_layout(),
            default_view_mode: default_view_mode(),
            show_observables: true,
            show_relationships: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_ocsf_version, "1.4.0");
        assert_eq!(config.default_dialect, "snowflake");
    }

    #[test]
    fn test_config_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let config = Config::default();
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(config.default_ocsf_version, loaded.default_ocsf_version);
        assert_eq!(config.default_dialect, loaded.default_dialect);
    }

    #[test]
    fn test_schema_cache_path() {
        let config = Config::default();
        let path = config.schema_cache_path("1.4.0");
        assert!(path.ends_with("schemas/1.4.0"));
    }
}
