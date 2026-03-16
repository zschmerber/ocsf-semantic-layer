//! DuckDB plugin — stores semantic annotations in a dedicated catalog table.

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::error::{CatalogError, CatalogResult};
use crate::models::{CatalogEntry, PushResult, SyncDiff};
use crate::plugin::CatalogPlugin;

/// DuckDB plugin using an in-memory HashMap backend for testing.
pub struct DuckDBPlugin {
    /// Simulated ocsf_semantic_catalog table: key -> JSON row
    table: RwLock<HashMap<String, String>>,
}

impl DuckDBPlugin {
    pub fn new() -> Self {
        Self {
            table: RwLock::new(HashMap::new()),
        }
    }

    fn entry_key(entry: &CatalogEntry) -> String {
        format!("{}::{}", entry.entity_name, entry.ocsf_version)
    }
}

impl Default for DuckDBPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CatalogPlugin for DuckDBPlugin {
    fn engine_name(&self) -> &'static str {
        "duckdb"
    }

    async fn push(&self, entries: &[CatalogEntry]) -> CatalogResult<PushResult> {
        let mut table = self.table.write().unwrap();
        for entry in entries {
            let key = Self::entry_key(entry);
            let json = serde_json::to_string(entry)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?;
            table.insert(key, json);
        }
        Ok(PushResult {
            engine: "duckdb".to_string(),
            entries_pushed: entries.len(),
        })
    }

    async fn pull(&self) -> CatalogResult<Vec<CatalogEntry>> {
        let table = self.table.read().unwrap();
        let mut entries = Vec::new();
        for json in table.values() {
            let mut entry: CatalogEntry = serde_json::from_str(json)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?;
            entry.id = None;
            entries.push(entry);
        }
        Ok(entries)
    }

    async fn diff(&self, catalog_entries: &[CatalogEntry]) -> CatalogResult<SyncDiff> {
        let table = self.table.read().unwrap();
        let engine_keys: std::collections::HashSet<String> = table.keys().cloned().collect();
        let catalog_map: HashMap<String, &CatalogEntry> = catalog_entries
            .iter()
            .map(|e| (Self::entry_key(e), e))
            .collect();

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        for (key, entry) in &catalog_map {
            if !engine_keys.contains(key) {
                added.push((*entry).clone());
            } else {
                let engine_entry: CatalogEntry = serde_json::from_str(&table[key])
                    .map_err(|e| CatalogError::Serialization(e.to_string()))?;
                if entry.catalog_version != engine_entry.catalog_version {
                    modified.push((*entry).clone());
                }
            }
        }
        for key in &engine_keys {
            if !catalog_map.contains_key(key) {
                removed.push(key.split("::").next().unwrap_or(key).to_string());
            }
        }

        Ok(SyncDiff {
            engine: "duckdb".to_string(),
            added,
            modified,
            removed,
        })
    }
}
