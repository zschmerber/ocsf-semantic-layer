//! Warehouse artifact generation.
//!
//! This crate generates warehouse-specific artifacts including:
//! - OCSF table schemas
//! - Observables table schemas
//! - dbt semantic layer YAML
//! - Cube.js schemas
//! - SQL views for semantic entities
//! - ETL pipelines for observable extraction
//! - Migration scripts

pub mod table;
pub mod observables;
pub mod dbt;
pub mod cubejs;
pub mod views;
pub mod etl;
pub mod migration;

#[cfg(test)]
mod table_proptest;

#[cfg(test)]
mod observables_proptest;

#[cfg(test)]
mod dbt_proptest;

#[cfg(test)]
mod cubejs_proptest;

#[cfg(test)]
mod views_proptest;

#[cfg(test)]
mod etl_proptest;

pub use table::*;
pub use observables::*;
pub use dbt::*;
pub use cubejs::*;
pub use views::*;
pub use etl::*;
pub use migration::*;
