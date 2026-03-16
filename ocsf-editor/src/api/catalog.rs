//! Catalog API handlers for the OCSF Semantic Model Editor.
//!
//! Provides REST endpoints for catalog CRUD, plugin sync, and semantic search.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use ocsf_catalog::{CatalogEntry, CatalogEntryId, CatalogError};

use crate::error::EditorApiError;
use crate::AppState;

// ── Error mapping ────────────────────────────────────────────────────────────

fn catalog_err(e: CatalogError) -> EditorApiError {
    match e {
        CatalogError::NotFound(msg) => EditorApiError::NotFound(msg),
        CatalogError::DuplicateEntry { entity_name, ocsf_version } => {
            EditorApiError::InvalidRequest(format!(
                "Duplicate entry: ({}, {})",
                entity_name, ocsf_version
            ))
        }
        CatalogError::Corruption(msg) | CatalogError::Serialization(msg) => {
            EditorApiError::Internal(anyhow::anyhow!(msg))
        }
        CatalogError::Io(e) => {
            EditorApiError::Internal(anyhow::anyhow!(e))
        }
        CatalogError::Plugin { engine, message } => {
            EditorApiError::LLMError(format!("Plugin '{}': {}", engine, message))
        }
    }
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OcsfVersionFilter {
    pub ocsf_version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Debug, Deserialize)]
pub struct SuggestParams {
    pub field: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreateEntryResponse {
    pub entry: CatalogEntry,
}

// ── CRUD handlers ─────────────────────────────────────────────────────────────

/// GET /api/catalog/entries
pub async fn list_entries(
    State(state): State<AppState>,
    Query(params): Query<OcsfVersionFilter>,
) -> Result<Json<Vec<CatalogEntry>>, EditorApiError> {
    let catalog = &state.catalog;
    let entries = catalog
        .list(params.ocsf_version.as_deref())
        .await
        .map_err(catalog_err)?;
    Ok(Json(entries))
}

/// POST /api/catalog/entries
pub async fn create_entry(
    State(state): State<AppState>,
    Json(entry): Json<CatalogEntry>,
) -> Result<impl IntoResponse, EditorApiError> {
    let catalog = &state.catalog;
    let id = catalog.add(entry.clone()).await.map_err(|e| match e {
        CatalogError::DuplicateEntry { entity_name, ocsf_version } => {
            EditorApiError::InvalidRequest(format!(
                "Duplicate entry: ({}, {})",
                entity_name, ocsf_version
            ))
        }
        other => catalog_err(other),
    })?;
    let created = catalog.get(id).await.map_err(catalog_err)?.ok_or_else(|| {
        EditorApiError::Internal(anyhow::anyhow!("Entry disappeared after creation"))
    })?;
    Ok((StatusCode::CREATED, Json(CreateEntryResponse { entry: created })))
}

/// GET /api/catalog/entries/:id
pub async fn get_entry(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<CatalogEntry>, EditorApiError> {
    let catalog = &state.catalog;
    catalog
        .get(CatalogEntryId(id))
        .await
        .map_err(catalog_err)?
        .map(Json)
        .ok_or_else(|| EditorApiError::NotFound(format!("Catalog entry {} not found", id)))
}

/// PUT /api/catalog/entries/:id
pub async fn update_entry(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(entry): Json<CatalogEntry>,
) -> Result<Json<CatalogEntry>, EditorApiError> {
    let catalog = &state.catalog;
    catalog
        .update(CatalogEntryId(id), entry)
        .await
        .map_err(catalog_err)?;
    catalog
        .get(CatalogEntryId(id))
        .await
        .map_err(catalog_err)?
        .map(Json)
        .ok_or_else(|| EditorApiError::NotFound(format!("Catalog entry {} not found", id)))
}

/// DELETE /api/catalog/entries/:id
pub async fn delete_entry(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, EditorApiError> {
    let catalog = &state.catalog;
    catalog
        .remove(CatalogEntryId(id))
        .await
        .map_err(catalog_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/catalog/versions
pub async fn list_versions(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, EditorApiError> {
    let catalog = &state.catalog;
    let versions = catalog.list_versions().await.map_err(catalog_err)?;
    Ok(Json(versions))
}

// ── Plugin sync handlers ──────────────────────────────────────────────────────

/// GET /api/catalog/plugins
pub async fn list_plugins(
    State(state): State<AppState>,
) -> Result<Json<Vec<ocsf_catalog::PluginInfo>>, EditorApiError> {
    let manager = state.plugin_manager.lock().await;
    Ok(Json(manager.list_plugins().await))
}

/// POST /api/catalog/plugins/:engine/push
pub async fn push_sync(
    State(state): State<AppState>,
    Path(engine): Path<String>,
) -> Result<Json<ocsf_catalog::PushResult>, EditorApiError> {
    let manager = state.plugin_manager.lock().await;
    manager
        .push(&engine)
        .await
        .map(Json)
        .map_err(catalog_err)
}

/// POST /api/catalog/plugins/:engine/pull
pub async fn pull_sync(
    State(state): State<AppState>,
    Path(engine): Path<String>,
) -> Result<Json<ocsf_catalog::PullResult>, EditorApiError> {
    let manager = state.plugin_manager.lock().await;
    manager
        .pull(&engine)
        .await
        .map(Json)
        .map_err(catalog_err)
}

/// GET /api/catalog/plugins/:engine/diff
pub async fn get_diff(
    State(state): State<AppState>,
    Path(engine): Path<String>,
) -> Result<Json<ocsf_catalog::SyncDiff>, EditorApiError> {
    let manager = state.plugin_manager.lock().await;
    manager
        .diff(&engine)
        .await
        .map(Json)
        .map_err(catalog_err)
}

/// POST /api/catalog/plugins/:engine/sync
pub async fn full_sync(
    State(state): State<AppState>,
    Path(engine): Path<String>,
) -> Result<Json<ocsf_catalog::SyncResult>, EditorApiError> {
    let manager = state.plugin_manager.lock().await;
    manager
        .full_sync(&engine)
        .await
        .map(Json)
        .map_err(catalog_err)
}

// ── Semantic search handlers ──────────────────────────────────────────────────

/// GET /api/catalog/search?q=...&top_k=...
pub async fn semantic_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<CatalogEntry>>, EditorApiError> {
    let catalog = &state.catalog;
    catalog
        .semantic_search(&params.q, params.top_k)
        .await
        .map(Json)
        .map_err(catalog_err)
}

/// GET /api/catalog/suggest?field=...&top_k=...
pub async fn field_mapping_suggestions(
    State(state): State<AppState>,
    Query(params): Query<SuggestParams>,
) -> Result<Json<Vec<CatalogEntry>>, EditorApiError> {
    let catalog = &state.catalog;
    catalog
        .field_mapping_suggestions(&params.field, params.top_k)
        .await
        .map(Json)
        .map_err(catalog_err)
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn create_catalog_router() -> Router<AppState> {
    Router::new()
        .route("/entries", get(list_entries).post(create_entry))
        .route(
            "/entries/:id",
            get(get_entry).put(update_entry).delete(delete_entry),
        )
        .route("/versions", get(list_versions))
        .route("/plugins", get(list_plugins))
        .route("/plugins/:engine/push", post(push_sync))
        .route("/plugins/:engine/pull", post(pull_sync))
        .route("/plugins/:engine/diff", get(get_diff))
        .route("/plugins/:engine/sync", post(full_sync))
        .route("/search", get(semantic_search))
        .route("/suggest", get(field_mapping_suggestions))
}
