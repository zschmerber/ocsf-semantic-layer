//! `ocsf-catalog` — Universal semantic catalog for OCSF with binary sidecar file and plugin system.

pub mod error;
pub mod models;
pub mod sidecar;
pub mod catalog;
pub mod plugin;
pub mod manager;
pub mod plugins;

pub use catalog::SemanticCatalog;
pub use error::{CatalogError, CatalogResult};
pub use manager::PluginManager;
pub use models::{
    CatalogEntry, CatalogEntryId, CatalogFieldLineage, CatalogLineageRecord,
    DetectionCoverage, PluginInfo, PullResult, PushResult, SyncDiff, SyncResult,
};
pub use plugin::CatalogPlugin;
pub use sidecar::SidecarCatalog;
