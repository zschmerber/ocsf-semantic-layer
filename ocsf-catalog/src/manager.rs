//! `PluginManager` — registers plugins and orchestrates push/pull/sync operations.

use std::collections::HashMap;
use std::sync::Arc;

use crate::catalog::SemanticCatalog;
use crate::error::{CatalogError, CatalogResult};
use crate::models::{PluginInfo, PullResult, PushResult, SyncDiff, SyncResult};
use crate::plugin::CatalogPlugin;

/// Manages registered plugins and orchestrates sync operations.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn CatalogPlugin>>,
    catalog: Arc<dyn SemanticCatalog>,
}

impl PluginManager {
    /// Create a new plugin manager backed by the given catalog.
    pub fn new(catalog: Arc<dyn SemanticCatalog>) -> Self {
        Self {
            plugins: HashMap::new(),
            catalog,
        }
    }

    /// Register a plugin. Replaces any existing plugin with the same engine name.
    pub fn register(&mut self, plugin: Box<dyn CatalogPlugin>) {
        let name = plugin.engine_name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// List all registered plugins with their connection status.
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut infos = Vec::new();
        for (name, plugin) in &self.plugins {
            infos.push(PluginInfo {
                engine: name.clone(),
                connected: plugin.is_connected().await,
            });
        }
        infos
    }

    /// Push all catalog entries to the specified engine.
    pub async fn push(&self, engine: &str) -> CatalogResult<PushResult> {
        let plugin = self.get_plugin(engine)?;
        let entries = self.catalog.list(None).await?;
        plugin.push(&entries).await
    }

    /// Pull entries from the specified engine and merge into catalog.
    pub async fn pull(&self, engine: &str) -> CatalogResult<PullResult> {
        let plugin = self.get_plugin(engine)?;
        let remote_entries = plugin.pull().await?;
        let pulled = remote_entries.len();
        let mut merged = 0;

        for remote in remote_entries {
            let key = (remote.entity_name.clone(), remote.ocsf_version.clone());
            match self.catalog.get_by_name(&key.0, &key.1).await? {
                None => {
                    // New entry — add it
                    self.catalog.add(remote).await?;
                    merged += 1;
                }
                Some(local) => {
                    // Conflict: last-writer-wins on catalog_version
                    if remote.catalog_version > local.catalog_version {
                        if let Some(id) = local.id {
                            self.catalog.update(id, remote).await?;
                            merged += 1;
                        }
                    }
                }
            }
        }

        Ok(PullResult {
            engine: engine.to_string(),
            entries_pulled: pulled,
            entries_merged: merged,
        })
    }

    /// Compute diff between catalog and the specified engine.
    pub async fn diff(&self, engine: &str) -> CatalogResult<SyncDiff> {
        let plugin = self.get_plugin(engine)?;
        let entries = self.catalog.list(None).await?;
        plugin.diff(&entries).await
    }

    /// Execute a full sync cycle: push local changes → pull remote changes.
    pub async fn full_sync(&self, engine: &str) -> CatalogResult<SyncResult> {
        let push_result = self.push(engine).await?;
        let pull_result = self.pull(engine).await?;
        let conflicts_resolved = pull_result.entries_merged;

        Ok(SyncResult {
            engine: engine.to_string(),
            push: push_result,
            pull: pull_result,
            conflicts_resolved,
        })
    }

    fn get_plugin(&self, engine: &str) -> CatalogResult<&dyn CatalogPlugin> {
        self.plugins
            .get(engine)
            .map(|p| p.as_ref())
            .ok_or_else(|| CatalogError::Plugin {
                engine: engine.to_string(),
                message: format!("Plugin '{}' not registered", engine),
            })
    }
}
