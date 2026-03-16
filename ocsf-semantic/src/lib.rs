//! Semantic model definitions and validation.
//!
//! This crate provides semantic entity definitions, metrics, and validation
//! for building a semantic layer on top of OCSF physical schema.

pub mod analyzer;
pub mod computed_fields;
pub mod dimension_inference;
pub mod dns_templates;
pub mod entity;
pub mod field_path;
pub mod llm_export;
pub mod metric;
pub mod model;
pub mod query;
pub mod query_templates;
pub mod schema_generator;
pub mod synonym_resolver;
pub mod threat_intel;
pub mod validation;
pub mod versioning;

#[cfg(test)]
mod entity_proptest;

#[cfg(test)]
mod metric_proptest;

#[cfg(test)]
mod validation_proptest;

#[cfg(test)]
mod analyzer_proptest;

#[cfg(test)]
mod query_proptest;

pub use analyzer::*;
pub use computed_fields::*;
pub use dimension_inference::*;
pub use dns_templates::*;
pub use entity::*;
pub use field_path::*;
pub use llm_export::*;
pub use metric::*;
pub use model::*;
pub use query::*;
pub use query_templates::*;
pub use schema_generator::*;
pub use synonym_resolver::*;
pub use threat_intel::*;
pub use validation::*;
pub use versioning::*;
