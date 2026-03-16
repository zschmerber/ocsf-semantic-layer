//! Delta Lake plugin — stores semantic annotations as Delta table properties.

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;

use crate::error::{CatalogError, CatalogResult};
use crate::models::{
    CatalogEntry, CatalogFieldLineage, CatalogLineageRecord, DetectionCoverage, PushResult,
    SyncDiff,
};
use crate::plugin::CatalogPlugin;

const PREFIX: &str = "ocsf.semantic.";

/// Delta Lake plugin using an in-memory HashMap backend for testing.
pub struct DeltaPlugin {
    /// Simulated table properties store: table_key -> property_map
    store: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl DeltaPlugin {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn entry_key(entry: &CatalogEntry) -> String {
        format!("{}::{}", entry.entity_name, entry.ocsf_version)
    }

    fn entry_to_props(entry: &CatalogEntry) -> CatalogResult<HashMap<String, String>> {
        let mut props = HashMap::new();
        props.insert(format!("{}entity_name", PREFIX), entry.entity_name.clone());
        props.insert(format!("{}caption", PREFIX), entry.caption.clone());
        props.insert(format!("{}description", PREFIX), entry.description.clone());
        props.insert(
            format!("{}ocsf_version", PREFIX),
            entry.ocsf_version.clone(),
        );
        props.insert(
            format!("{}source_event_classes", PREFIX),
            serde_json::to_string(&entry.source_event_classes)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?,
        );
        props.insert(
            format!("{}attributes", PREFIX),
            serde_json::to_string(&entry.attributes)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?,
        );
        props.insert(
            format!("{}relationships", PREFIX),
            serde_json::to_string(&entry.relationships)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?,
        );
        if let Some(dc) = &entry.detection_coverage {
            props.insert(
                format!("{}detection_coverage", PREFIX),
                serde_json::to_string(dc)
                    .map_err(|e| CatalogError::Serialization(e.to_string()))?,
            );
        }
        props.insert(
            format!("{}source_lineage", PREFIX),
            serde_json::to_string(&entry.source_lineage)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?,
        );
        props.insert(
            format!("{}field_lineage", PREFIX),
            serde_json::to_string(&entry.field_lineage)
                .map_err(|e| CatalogError::Serialization(e.to_string()))?,
        );
        props.insert(
            format!("{}catalog_version", PREFIX),
            entry.catalog_version.to_string(),
        );
        Ok(props)
    }

    fn props_to_entry(props: &HashMap<String, String>) -> CatalogResult<CatalogEntry> {
        let entity_name = props
            .get(&format!("{}entity_name", PREFIX))
            .cloned()
            .unwrap_or_default();
        let caption = props
            .get(&format!("{}caption", PREFIX))
            .cloned()
            .unwrap_or_default();
        let description = props
            .get(&format!("{}description", PREFIX))
            .cloned()
            .unwrap_or_default();
        let ocsf_version = props
            .get(&format!("{}ocsf_version", PREFIX))
            .cloned()
            .unwrap_or_default();
        let source_event_classes: Vec<u32> = props
            .get(&format!("{}source_event_classes", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let attributes = props
            .get(&format!("{}attributes", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let relationships = props
            .get(&format!("{}relationships", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let detection_coverage: Option<DetectionCoverage> = props
            .get(&format!("{}detection_coverage", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok());
        let source_lineage: Vec<CatalogLineageRecord> = props
            .get(&format!("{}source_lineage", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let field_lineage: Vec<CatalogFieldLineage> = props
            .get(&format!("{}field_lineage", PREFIX))
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let catalog_version: u64 = props
            .get(&format!("{}catalog_version", PREFIX))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(CatalogEntry {
            id: None,
            entity_name,
            caption,
            description,
            ocsf_version,
            source_event_classes,
            attributes,
            relationships,
            covers_observables: vec![],
            detection_coverage,
            source_lineage,
            field_lineage,
            catalog_version,
            updated_at: Utc::now(),
        })
    }
}

impl Default for DeltaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CatalogPlugin for DeltaPlugin {
    fn engine_name(&self) -> &'static str {
        "delta"
    }

    async fn push(&self, entries: &[CatalogEntry]) -> CatalogResult<PushResult> {
        let mut store = self.store.write().unwrap();
        for entry in entries {
            let key = Self::entry_key(entry);
            let props = Self::entry_to_props(entry)?;
            store.insert(key, props);
        }
        Ok(PushResult {
            engine: "delta".to_string(),
            entries_pushed: entries.len(),
        })
    }

    async fn pull(&self) -> CatalogResult<Vec<CatalogEntry>> {
        let store = self.store.read().unwrap();
        let mut entries = Vec::new();
        for props in store.values() {
            entries.push(Self::props_to_entry(props)?);
        }
        Ok(entries)
    }

    async fn diff(&self, catalog_entries: &[CatalogEntry]) -> CatalogResult<SyncDiff> {
        let store = self.store.read().unwrap();
        let engine_keys: std::collections::HashSet<String> = store.keys().cloned().collect();
        let catalog_map: HashMap<String, &CatalogEntry> = catalog_entries
            .iter()
            .map(|e| (format!("{}::{}", e.entity_name, e.ocsf_version), e))
            .collect();

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        for (key, entry) in &catalog_map {
            if !engine_keys.contains(key) {
                added.push((*entry).clone());
            } else {
                let engine_ver: u64 = store[key]
                    .get(&format!("{}catalog_version", PREFIX))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if entry.catalog_version != engine_ver {
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
            engine: "delta".to_string(),
            added,
            modified,
            removed,
        })
    }
}
