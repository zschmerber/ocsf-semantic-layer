//! `CatalogPlugin` trait for bidirectional sync with storage engines.

use async_trait::async_trait;

use crate::error::CatalogResult;
use crate::models::{CatalogEntry, PushResult, SyncDiff};

/// Plugin trait for bidirectional sync with storage engines.
///
/// Each engine (Iceberg, Delta, ClickHouse, DuckDB) implements this trait.
#[async_trait]
pub trait CatalogPlugin: Send + Sync {
    /// Engine name (e.g., "iceberg", "delta", "clickhouse", "duckdb").
    fn engine_name(&self) -> &'static str;

    /// Push catalog entries to the engine's native metadata.
    async fn push(&self, entries: &[CatalogEntry]) -> CatalogResult<PushResult>;

    /// Pull semantic annotations from the engine and return as catalog entries.
    async fn pull(&self) -> CatalogResult<Vec<CatalogEntry>>;

    /// Compute diff between catalog state and engine metadata.
    async fn diff(&self, catalog_entries: &[CatalogEntry]) -> CatalogResult<SyncDiff>;

    /// Check if the plugin is connected/available.
    async fn is_connected(&self) -> bool {
        true
    }
}
