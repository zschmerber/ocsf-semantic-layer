//! Error types for the semantic catalog.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Catalog file corrupted: {0}")]
    Corruption(String),

    #[error("Duplicate entry: entity '{entity_name}' version '{ocsf_version}'")]
    DuplicateEntry {
        entity_name: String,
        ocsf_version: String,
    },

    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Plugin '{engine}' error: {message}")]
    Plugin {
        engine: String,
        message: String,
    },
}

pub type CatalogResult<T> = Result<T, CatalogError>;
