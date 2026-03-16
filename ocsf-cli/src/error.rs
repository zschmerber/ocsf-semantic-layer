//! Error types for the OCSF CLI.
//!
//! This module provides comprehensive error types for all CLI operations,
//! with actionable error messages to help users resolve issues.

use std::path::PathBuf;

use thiserror::Error;

/// CLI error types.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CliError {
    /// Schema ingestion error.
    #[error("Schema ingestion failed: {message}")]
    SchemaIngestion {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Model validation error.
    #[error("Model validation failed: {message}")]
    ModelValidation {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Artifact generation error.
    #[error("Artifact generation failed: {message}")]
    ArtifactGeneration {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Visualization error.
    #[error("Visualization failed: {message}")]
    Visualization {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Query translation error.
    #[error("Query translation failed: {message}")]
    QueryTranslation {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// File not found error.
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Invalid file format error.
    #[error("Invalid file format for {path}: {reason}")]
    InvalidFileFormat { path: PathBuf, reason: String },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Network error.
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// IO error.
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },
}

#[allow(dead_code)]
impl CliError {
    /// Creates a schema ingestion error.
    pub fn schema_ingestion(message: impl Into<String>) -> Self {
        Self::SchemaIngestion {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a schema ingestion error with a source.
    pub fn schema_ingestion_with_source(message: impl Into<String>, source: anyhow::Error) -> Self {
        Self::SchemaIngestion {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates a model validation error.
    pub fn model_validation(message: impl Into<String>) -> Self {
        Self::ModelValidation {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a model validation error with a source.
    pub fn model_validation_with_source(message: impl Into<String>, source: anyhow::Error) -> Self {
        Self::ModelValidation {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates an artifact generation error.
    pub fn artifact_generation(message: impl Into<String>) -> Self {
        Self::ArtifactGeneration {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a visualization error.
    pub fn visualization(message: impl Into<String>) -> Self {
        Self::Visualization {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a query translation error.
    pub fn query_translation(message: impl Into<String>) -> Self {
        Self::QueryTranslation {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a file not found error.
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Creates an invalid file format error.
    pub fn invalid_file_format(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::InvalidFileFormat {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Creates a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Creates a network error.
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// Returns a suggestion for how to fix this error.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::SchemaIngestion { .. } => Some(
                "Try running 'ocsf ingest --source github' to download the latest schema".to_string(),
            ),
            Self::ModelValidation { .. } => Some(
                "Check your semantic model YAML for syntax errors and invalid field references"
                    .to_string(),
            ),
            Self::FileNotFound { path } => Some(format!(
                "Ensure the file exists at: {}",
                path.display()
            )),
            Self::InvalidFileFormat { path, .. } => Some(format!(
                "Check the file format of: {}",
                path.display()
            )),
            Self::Configuration { .. } => Some(
                "Check your configuration file or use default settings".to_string(),
            ),
            Self::Network { .. } => Some(
                "Check your internet connection and try again".to_string(),
            ),
            _ => None,
        }
    }
}

/// Prints an error with optional suggestion.
pub fn print_error(error: &CliError) {
    eprintln!("Error: {}", error);
    if let Some(suggestion) = error.suggestion() {
        eprintln!("\nSuggestion: {}", suggestion);
    }
}

/// Prints a validation error summary.
pub fn print_validation_summary(errors: &[ocsf_semantic::ValidationError], warnings: &[String]) {
    if !errors.is_empty() {
        eprintln!("\nValidation Errors ({}):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, error);
        }
    }

    if !warnings.is_empty() {
        eprintln!("\nWarnings ({}):", warnings.len());
        for (i, warning) in warnings.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, warning);
        }
    }
}
