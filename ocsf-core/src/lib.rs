//! Core OCSF schema data structures and parsing.
//!
//! This crate provides the foundational types for representing OCSF schema elements
//! including categories, event classes, objects, attributes, and observables.

pub mod schema;
pub mod compiled_schema;
pub mod ingester;
pub mod github;
pub mod observable;
pub mod version;

#[cfg(test)]
mod schema_proptest;

#[cfg(test)]
mod observable_proptest;

pub use schema::*;
pub use compiled_schema::{
    CompiledSchema, CompiledClass, CompiledObject, CompiledAttribute, CompiledCategory,
    CompiledProfile, CompiledRequirement, CompiledEnumValue, ParseError, SchemaStats,
};
pub use ingester::{SchemaIngester, SchemaSource};
pub use github::{GitHubConfig, GitHubFetcher};
pub use observable::{extract_observables, ObservableCatalog, ObservableEntry, ObservableExtractor, ObservableSource};
pub use version::{compare_schemas, detect_version, ChangeType, SchemaChange, SchemaDiff, SchemaVersion, VersionError};
