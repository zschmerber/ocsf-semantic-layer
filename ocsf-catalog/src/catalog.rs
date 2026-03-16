//! `SemanticCatalog` trait and its implementation for `SidecarCatalog`.

use async_trait::async_trait;
use chrono::Utc;

use crate::error::{CatalogError, CatalogResult};
use crate::models::{CatalogEntry, CatalogEntryId};
use crate::sidecar::SidecarCatalog;
use ocsf_vector::VectorStore;

/// Core catalog trait for reading and writing semantic metadata.
///
/// All methods are async and require `Send + Sync` bounds for use in
/// multi-threaded async runtimes.
#[async_trait]
pub trait SemanticCatalog: Send + Sync {
    /// Add a new entry, returns its assigned `CatalogEntryId`.
    async fn add(&self, entry: CatalogEntry) -> CatalogResult<CatalogEntryId>;

    /// Get an entry by ID.
    async fn get(&self, id: CatalogEntryId) -> CatalogResult<Option<CatalogEntry>>;

    /// Get an entry by entity name and OCSF schema version.
    async fn get_by_name(
        &self,
        name: &str,
        ocsf_version: &str,
    ) -> CatalogResult<Option<CatalogEntry>>;

    /// Update an existing entry. Increments catalog version.
    async fn update(&self, id: CatalogEntryId, entry: CatalogEntry) -> CatalogResult<()>;

    /// Remove an entry by ID.
    async fn remove(&self, id: CatalogEntryId) -> CatalogResult<()>;

    /// List entries, optionally filtered by OCSF schema version.
    async fn list(&self, ocsf_version: Option<&str>) -> CatalogResult<Vec<CatalogEntry>>;

    /// List all distinct OCSF schema versions present in the catalog.
    async fn list_versions(&self) -> CatalogResult<Vec<String>>;

    /// Get the current catalog version number.
    async fn catalog_version(&self) -> CatalogResult<u64>;

    /// Semantic search using vector similarity.
    async fn semantic_search(&self, query: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>>;

    /// Field mapping suggestions based on a source field name.
    async fn field_mapping_suggestions(&self, source_field: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>>;
}

#[async_trait]
impl SemanticCatalog for SidecarCatalog {
    async fn add(&self, mut entry: CatalogEntry) -> CatalogResult<CatalogEntryId> {
        // Check for duplicate (entity_name, ocsf_version)
        {
            let index = self.index.read().unwrap();
            if index.contains_key(&(entry.entity_name.clone(), entry.ocsf_version.clone())) {
                return Err(CatalogError::DuplicateEntry {
                    entity_name: entry.entity_name.clone(),
                    ocsf_version: entry.ocsf_version.clone(),
                });
            }
        }

        // Assign next ID and increment catalog version
        let id = {
            let mut next_id = self.next_id.write().unwrap();
            let id = CatalogEntryId(*next_id);
            *next_id += 1;
            id
        };

        let catalog_ver = {
            let mut header = self.header.write().unwrap();
            header.catalog_version += 1;
            header.catalog_version
        };

        entry.id = Some(id);
        entry.catalog_version = catalog_ver;
        entry.updated_at = Utc::now();

        // Insert into in-memory stores
        {
            let mut index = self.index.write().unwrap();
            index.insert((entry.entity_name.clone(), entry.ocsf_version.clone()), id);
        }
        let stored_entry = {
            let mut entries = self.entries.write().unwrap();
            entries.insert(id.0, entry);
            entries.get(&id.0).cloned().unwrap()
        };

        self.flush()?;
        self.index_entry_embedding(&stored_entry).await;
        Ok(id)
    }

    async fn get(&self, id: CatalogEntryId) -> CatalogResult<Option<CatalogEntry>> {
        let entries = self.entries.read().unwrap();
        Ok(entries.get(&id.0).cloned())
    }

    async fn get_by_name(
        &self,
        name: &str,
        ocsf_version: &str,
    ) -> CatalogResult<Option<CatalogEntry>> {
        let index = self.index.read().unwrap();
        let id = index.get(&(name.to_string(), ocsf_version.to_string())).copied();
        drop(index);

        if let Some(id) = id {
            let entries = self.entries.read().unwrap();
            Ok(entries.get(&id.0).cloned())
        } else {
            Ok(None)
        }
    }

    async fn update(&self, id: CatalogEntryId, mut entry: CatalogEntry) -> CatalogResult<()> {
        // Verify entry exists
        {
            let entries = self.entries.read().unwrap();
            if !entries.contains_key(&id.0) {
                return Err(CatalogError::NotFound(format!("Entry with ID {} not found", id.0)));
            }
        }

        let catalog_ver = {
            let mut header = self.header.write().unwrap();
            header.catalog_version += 1;
            header.catalog_version
        };

        entry.id = Some(id);
        entry.catalog_version = catalog_ver;
        entry.updated_at = Utc::now();

        // Update index: remove old entries for this id, insert new key
        {
            let mut index = self.index.write().unwrap();
            index.retain(|_, v| v.0 != id.0);
            index.insert((entry.entity_name.clone(), entry.ocsf_version.clone()), id);
        }
        let stored_entry = {
            let mut entries = self.entries.write().unwrap();
            entries.insert(id.0, entry);
            entries.get(&id.0).cloned().unwrap()
        };

        self.flush()?;
        self.index_entry_embedding(&stored_entry).await;
        Ok(())
    }

    async fn remove(&self, id: CatalogEntryId) -> CatalogResult<()> {
        // Remove from entries
        let removed = {
            let mut entries = self.entries.write().unwrap();
            entries.remove(&id.0)
        };

        if removed.is_none() {
            return Err(CatalogError::NotFound(format!("Entry with ID {} not found", id.0)));
        }

        // Remove from index
        {
            let mut index = self.index.write().unwrap();
            index.retain(|_, v| v.0 != id.0);
        }

        // Increment catalog version
        {
            let mut header = self.header.write().unwrap();
            header.catalog_version += 1;
        }

        self.flush()
    }

    async fn list(&self, ocsf_version: Option<&str>) -> CatalogResult<Vec<CatalogEntry>> {
        let entries = self.entries.read().unwrap();
        let result = entries
            .values()
            .filter(|e| ocsf_version.map(|v| e.ocsf_version == v).unwrap_or(true))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_versions(&self) -> CatalogResult<Vec<String>> {
        let entries = self.entries.read().unwrap();
        let mut versions: Vec<String> = entries
            .values()
            .map(|e| e.ocsf_version.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        versions.sort();
        Ok(versions)
    }

    async fn catalog_version(&self) -> CatalogResult<u64> {
        let header = self.header.read().unwrap();
        Ok(header.catalog_version)
    }

    async fn semantic_search(&self, query: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>> {
        let gen = match &self.embedding_generator {
            Some(g) => g,
            None => return Ok(vec![]),
        };
        let query_vec = gen.embed(query).await
            .map_err(|e| CatalogError::Plugin { engine: "vector".to_string(), message: e.to_string() })?;

        let store = self.vector_store.lock().await;
        let results = store.similarity_search(&query_vec, top_k, None).await
            .map_err(|e: ocsf_vector::VectorStoreError| CatalogError::Plugin { engine: "vector".to_string(), message: e.to_string() })?;
        drop(store);

        let entries_map = self.entries.read().unwrap();
        let mut found = Vec::new();
        for result in results {
            if let Ok(id) = result.id.parse::<u64>() {
                if let Some(entry) = entries_map.get(&id) {
                    found.push(entry.clone());
                }
            }
        }
        Ok(found)
    }

    async fn field_mapping_suggestions(&self, source_field: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>> {
        self.semantic_search(source_field, top_k).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::NamedTempFile;

    fn make_entry(name: &str, version: &str) -> CatalogEntry {
        CatalogEntry {
            id: None,
            entity_name: name.to_string(),
            caption: format!("{} caption", name),
            description: format!("{} description", name),
            ocsf_version: version.to_string(),
            source_event_classes: vec![1001],
            attributes: vec![],
            relationships: vec![],
            covers_observables: vec![],
            detection_coverage: None,
            source_lineage: vec![],
            field_lineage: vec![],
            catalog_version: 0,
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn add_and_get_by_id() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        let entry = make_entry("dns_event", "1.3.0");
        let id = catalog.add(entry.clone()).await.unwrap();

        let retrieved = catalog.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.entity_name, "dns_event");
        assert_eq!(retrieved.ocsf_version, "1.3.0");
        assert_eq!(retrieved.id, Some(id));
    }

    #[tokio::test]
    async fn add_and_get_by_name() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        let entry = make_entry("auth_event", "1.4.0");
        catalog.add(entry).await.unwrap();

        let retrieved = catalog.get_by_name("auth_event", "1.4.0").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().entity_name, "auth_event");
    }

    #[tokio::test]
    async fn duplicate_add_returns_error() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        let entry = make_entry("dns_event", "1.3.0");
        catalog.add(entry.clone()).await.unwrap();

        let result = catalog.add(entry).await;
        assert!(matches!(result, Err(CatalogError::DuplicateEntry { .. })));
    }

    #[tokio::test]
    async fn remove_then_get_returns_none() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        let id = catalog.add(make_entry("net_event", "2.0.0")).await.unwrap();
        catalog.remove(id).await.unwrap();

        let result = catalog.get(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn write_ops_increment_catalog_version() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        let v0 = catalog.catalog_version().await.unwrap();
        let id = catalog.add(make_entry("e1", "1.3.0")).await.unwrap();
        let v1 = catalog.catalog_version().await.unwrap();
        assert!(v1 > v0);

        catalog.update(id, make_entry("e1", "1.3.0")).await.unwrap();
        let v2 = catalog.catalog_version().await.unwrap();
        assert!(v2 > v1);

        catalog.remove(id).await.unwrap();
        let v3 = catalog.catalog_version().await.unwrap();
        assert!(v3 > v2);
    }

    #[tokio::test]
    async fn list_with_version_filter() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        catalog.add(make_entry("e1", "1.3.0")).await.unwrap();
        catalog.add(make_entry("e2", "1.4.0")).await.unwrap();
        catalog.add(make_entry("e3", "1.3.0")).await.unwrap();

        let v13 = catalog.list(Some("1.3.0")).await.unwrap();
        assert_eq!(v13.len(), 2);

        let all = catalog.list(None).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn list_versions_returns_distinct() {
        let tmp = NamedTempFile::new().unwrap();
        let catalog = SidecarCatalog::create(tmp.path()).unwrap();

        catalog.add(make_entry("e1", "1.3.0")).await.unwrap();
        catalog.add(make_entry("e2", "1.4.0")).await.unwrap();
        catalog.add(make_entry("e3", "1.3.0")).await.unwrap();

        let versions = catalog.list_versions().await.unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.3.0".to_string()));
        assert!(versions.contains(&"1.4.0".to_string()));
    }

    #[tokio::test]
    async fn persist_and_reload() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        {
            let catalog = SidecarCatalog::create(&path).unwrap();
            catalog.add(make_entry("dns_event", "1.3.0")).await.unwrap();
            catalog.add(make_entry("auth_event", "1.4.0")).await.unwrap();
        }

        // Reopen and verify entries persisted
        let catalog = SidecarCatalog::open(&path).unwrap();
        let all = catalog.list(None).await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
