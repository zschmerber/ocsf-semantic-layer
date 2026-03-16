//! HTTP server for ETL job lifecycle management.
//!
//! Provides REST endpoints for submitting, generating, compiling, testing,
//! running, and stopping ETL jobs via Axum.

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream;
use serde::Serialize;
use tokio::io::AsyncBufReadExt;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use codegen_adapter::CompiledSchema;
use etl_core::{EtlJob, EtlJobStatus};
use semantic_bridge::SemanticBridge;

use crate::GenerationResult;

// ── 11.1 AppState ────────────────────────────────────────────────────

/// Shared application state for the Axum server.
pub struct AppState {
    /// In-memory job store keyed by job ID.
    pub jobs: Arc<RwLock<HashMap<Uuid, JobRecord>>>,
    /// Semantic bridge for catalog entry resolution.
    pub bridge: Arc<dyn SemanticBridge>,
    /// Compiled OCSF schema used for codegen and warehouse generation.
    pub schema: CompiledSchema,
    /// Codegen adapter for generating Go WASM plugins.
    pub codegen: codegen_adapter::CodegenAdapter,
    /// Target warehouse SQL dialect.
    pub warehouse_dialect: warehouse_gen::WarehouseDialect,
    /// Path to the Tangent binary for compile/test/run operations.
    pub tangent_bin: PathBuf,
    /// Root output directory for generated artifacts.
    pub output_dir: PathBuf,
}

// ── 11.2 JobRecord ───────────────────────────────────────────────────

/// Record tracking a single ETL job and its lifecycle state.
pub struct JobRecord {
    /// The ETL job specification.
    pub job: EtlJob,
    /// Current lifecycle status.
    pub status: EtlJobStatus,
    /// Result from `generate_all`, populated after generation.
    pub generation_result: Option<GenerationResult>,
    /// Log lines captured from Tangent process stdout/stderr.
    pub log_buffer: Vec<String>,
    /// Handle to a running Tangent child process (run phase).
    pub process: Option<tokio::process::Child>,
}

// ── Response types ───────────────────────────────────────────────────

// (JobCreatedResponse removed — we use inline serde_json::json! instead)

#[derive(Serialize)]
struct JobSummary {
    job_id: Uuid,
    plugin_name: String,
    status: String,
}

#[derive(Serialize)]
struct JobDetail {
    job_id: Uuid,
    plugin_name: String,
    ocsf_class_uid: u32,
    status: String,
    artifact_files: Vec<String>,
    log_lines: usize,
}

#[derive(Serialize)]
struct StatusResponse {
    job_id: Uuid,
    status: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn status_label(s: &EtlJobStatus) -> String {
    match s {
        EtlJobStatus::Pending => "Pending".into(),
        EtlJobStatus::Generating => "Generating".into(),
        EtlJobStatus::Compiling => "Compiling".into(),
        EtlJobStatus::Testing => "Testing".into(),
        EtlJobStatus::Running => "Running".into(),
        EtlJobStatus::Completed => "Completed".into(),
        EtlJobStatus::Failed(msg) => format!("Failed: {msg}"),
    }
}

/// Collect artifact file paths from a `GenerationResult`.
fn collect_artifact_files(result: &GenerationResult) -> Vec<String> {
    let mut files = Vec::new();
    // Plugin files
    for name in &["main.go", "go.mod", "tangent.yaml", "test_fixture.json"] {
        let p = result.plugin_dir.join(name);
        if p.exists() {
            files.push(p.display().to_string());
        }
    }
    // Warehouse files
    for name in &["table.sql", "views.sql", "materialized_views.sql"] {
        let p = result.warehouse_dir.join(name);
        if p.exists() {
            files.push(p.display().to_string());
        }
    }
    // Semantic model
    if result.semantic_model_path.exists() {
        files.push(result.semantic_model_path.display().to_string());
    }
    files
}

// ── Router ───────────────────────────────────────────────────────────

/// Build the Axum router with all job lifecycle endpoints.
pub fn job_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/jobs", post(create_job).get(list_jobs))
        .route("/api/jobs/:id", get(get_job))
        .route("/api/jobs/:id/generate", post(generate_job))
        .route("/api/jobs/:id/compile", post(compile_job))
        .route("/api/jobs/:id/test", post(test_job))
        .route("/api/jobs/:id/run", post(run_job))
        .route("/api/jobs/:id/stop", post(stop_job))
        .route("/api/jobs/:id/logs", get(get_job_logs))
        .route("/api/jobs/from-catalog/:entry_id", post(create_job_from_catalog))
        .route("/api/catalog/entries", get(list_catalog_entries))
        .route("/api/health", get(health_check))
}

// ── 11.3 POST /api/jobs ──────────────────────────────────────────────

/// Accept a JSON `EtlJob`, store it as `Pending`, and return the job ID.
async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(job): Json<EtlJob>,
) -> (StatusCode, Json<serde_json::Value>) {
    let job_id = job.job_id;
    let record = JobRecord {
        job,
        status: EtlJobStatus::Pending,
        generation_result: None,
        log_buffer: Vec::new(),
        process: None,
    };

    state.jobs.write().await.insert(job_id, record);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "job_id": job_id })),
    )
}

// ── 11.4 GET /api/jobs ───────────────────────────────────────────────

/// List all jobs with their ID, plugin name, and status.
async fn list_jobs(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<JobSummary>> {
    let jobs = state.jobs.read().await;
    let summaries: Vec<JobSummary> = jobs
        .iter()
        .map(|(id, rec)| JobSummary {
            job_id: *id,
            plugin_name: rec.job.plugin_name.clone(),
            status: status_label(&rec.status),
        })
        .collect();
    Json(summaries)
}

// ── 11.5 GET /api/jobs/:id ───────────────────────────────────────────

/// Return full job detail including status and artifact file list.
async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobDetail>, (StatusCode, Json<ErrorResponse>)> {
    let jobs = state.jobs.read().await;
    let rec = jobs.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    let artifact_files = rec
        .generation_result
        .as_ref()
        .map(collect_artifact_files)
        .unwrap_or_default();

    Ok(Json(JobDetail {
        job_id: id,
        plugin_name: rec.job.plugin_name.clone(),
        ocsf_class_uid: rec.job.ocsf_class_uid,
        status: status_label(&rec.status),
        artifact_files,
        log_lines: rec.log_buffer.len(),
    }))
}

// ── 11.6 POST /api/jobs/:id/generate ─────────────────────────────────

/// Call `generate_all`, store the result, and transition to `Compiling`.
async fn generate_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Read job data while holding a short-lived read lock.
    let job = {
        let jobs = state.jobs.read().await;
        let rec = jobs.get(&id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("job {id} not found"),
                }),
            )
        })?;
        rec.job.clone()
    };

    // Run generation (potentially slow, outside the lock).
    let result = crate::generate_all(
        &job,
        &state.schema,
        state.warehouse_dialect.clone(),
        &state.output_dir,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("generation failed: {e}"),
            }),
        )
    })?;

    // Update the job record.
    let mut jobs = state.jobs.write().await;
    let rec = jobs.get_mut(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    rec.status = rec
        .status
        .transition(&EtlJobStatus::Generating)
        .and_then(|s| s.transition(&EtlJobStatus::Compiling))
        .map_err(|e| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: format!("state transition failed: {e}"),
                }),
            )
        })?;
    rec.generation_result = Some(result);

    Ok(Json(StatusResponse {
        job_id: id,
        status: status_label(&rec.status),
    }))
}

// ── 11.7 POST /api/jobs/:id/compile ──────────────────────────────────

/// Spawn `tangent plugin compile` in the plugin directory, capture output
/// to the log buffer, and transition to `Testing` on success.
async fn compile_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (plugin_dir, tangent_bin) = {
        let jobs = state.jobs.read().await;
        let rec = jobs.get(&id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("job {id} not found"),
                }),
            )
        })?;
        let gen = rec.generation_result.as_ref().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "must run generate before compile".into(),
                }),
            )
        })?;
        (gen.plugin_dir.clone(), state.tangent_bin.clone())
    };

    // Spawn the compile process in a background task.
    let jobs = Arc::clone(&state.jobs);
    tokio::spawn(async move {
        let result = tokio::process::Command::new(&tangent_bin)
            .args(["plugin", "compile"])
            .current_dir(&plugin_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                // Read stdout in background.
                if let Some(stdout) = child.stdout.take() {
                    let jobs_stdout = Arc::clone(&jobs);
                    let id_stdout = id;
                    tokio::spawn(async move {
                        let reader = tokio::io::BufReader::new(stdout);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if let Some(rec) = jobs_stdout.write().await.get_mut(&id_stdout) {
                                rec.log_buffer.push(format!("[compile:stdout] {line}"));
                            }
                        }
                    });
                }
                // Read stderr in background.
                if let Some(stderr) = child.stderr.take() {
                    let jobs_stderr = Arc::clone(&jobs);
                    let id_stderr = id;
                    tokio::spawn(async move {
                        let reader = tokio::io::BufReader::new(stderr);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if let Some(rec) = jobs_stderr.write().await.get_mut(&id_stderr) {
                                rec.log_buffer.push(format!("[compile:stderr] {line}"));
                            }
                        }
                    });
                }

                // Wait for the process to finish.
                match child.wait().await {
                    Ok(exit) if exit.success() => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            if let Ok(next) = rec.status.transition(&EtlJobStatus::Testing) {
                                rec.status = next;
                            }
                        }
                    }
                    Ok(exit) => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            rec.status = EtlJobStatus::Failed(format!(
                                "compile exited with code {}",
                                exit.code().unwrap_or(-1)
                            ));
                        }
                    }
                    Err(e) => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            rec.status =
                                EtlJobStatus::Failed(format!("compile wait error: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(rec) = jobs.write().await.get_mut(&id) {
                    rec.status = EtlJobStatus::Failed(format!("failed to spawn compile: {e}"));
                }
            }
        }
    });

    // Transition to Compiling (already there from generate, but confirm).
    let mut jobs_lock = state.jobs.write().await;
    let rec = jobs_lock.get_mut(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    Ok(Json(StatusResponse {
        job_id: id,
        status: status_label(&rec.status),
    }))
}

// ── 11.8 POST /api/jobs/:id/test ─────────────────────────────────────

/// Spawn `tangent plugin test` in the plugin directory, capture output
/// to the log buffer, and transition to `Running` on success.
async fn test_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (plugin_dir, tangent_bin) = {
        let jobs = state.jobs.read().await;
        let rec = jobs.get(&id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("job {id} not found"),
                }),
            )
        })?;
        let gen = rec.generation_result.as_ref().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "must run generate before test".into(),
                }),
            )
        })?;
        (gen.plugin_dir.clone(), state.tangent_bin.clone())
    };

    let jobs = Arc::clone(&state.jobs);
    tokio::spawn(async move {
        let result = tokio::process::Command::new(&tangent_bin)
            .args(["plugin", "test"])
            .current_dir(&plugin_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let jobs_stdout = Arc::clone(&jobs);
                    let id_stdout = id;
                    tokio::spawn(async move {
                        let reader = tokio::io::BufReader::new(stdout);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if let Some(rec) = jobs_stdout.write().await.get_mut(&id_stdout) {
                                rec.log_buffer.push(format!("[test:stdout] {line}"));
                            }
                        }
                    });
                }
                if let Some(stderr) = child.stderr.take() {
                    let jobs_stderr = Arc::clone(&jobs);
                    let id_stderr = id;
                    tokio::spawn(async move {
                        let reader = tokio::io::BufReader::new(stderr);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if let Some(rec) = jobs_stderr.write().await.get_mut(&id_stderr) {
                                rec.log_buffer.push(format!("[test:stderr] {line}"));
                            }
                        }
                    });
                }

                match child.wait().await {
                    Ok(exit) if exit.success() => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            if let Ok(next) = rec.status.transition(&EtlJobStatus::Running) {
                                rec.status = next;
                            }
                        }
                    }
                    Ok(exit) => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            rec.status = EtlJobStatus::Failed(format!(
                                "test exited with code {}",
                                exit.code().unwrap_or(-1)
                            ));
                        }
                    }
                    Err(e) => {
                        if let Some(rec) = jobs.write().await.get_mut(&id) {
                            rec.status = EtlJobStatus::Failed(format!("test wait error: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(rec) = jobs.write().await.get_mut(&id) {
                    rec.status = EtlJobStatus::Failed(format!("failed to spawn test: {e}"));
                }
            }
        }
    });

    let jobs_lock = state.jobs.read().await;
    let rec = jobs_lock.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    Ok(Json(StatusResponse {
        job_id: id,
        status: status_label(&rec.status),
    }))
}

// ── 11.9 POST /api/jobs/:id/run ──────────────────────────────────────

/// Spawn `tangent run` with the tangent.yaml, keep the child handle in
/// the job record, and transition to `Running`.
async fn run_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (plugin_dir, tangent_bin) = {
        let jobs = state.jobs.read().await;
        let rec = jobs.get(&id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("job {id} not found"),
                }),
            )
        })?;
        let gen = rec.generation_result.as_ref().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "must run generate before run".into(),
                }),
            )
        })?;
        (gen.plugin_dir.clone(), state.tangent_bin.clone())
    };

    let tangent_yaml = plugin_dir.join("tangent.yaml");

    let mut child = tokio::process::Command::new(&tangent_bin)
        .args(["run", "--config"])
        .arg(&tangent_yaml)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("failed to spawn tangent run: {e}"),
                }),
            )
        })?;

    // Stream stdout/stderr to log buffer in background.
    if let Some(stdout) = child.stdout.take() {
        let jobs_stdout = Arc::clone(&state.jobs);
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(rec) = jobs_stdout.write().await.get_mut(&id) {
                    rec.log_buffer.push(format!("[run:stdout] {line}"));
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let jobs_stderr = Arc::clone(&state.jobs);
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(rec) = jobs_stderr.write().await.get_mut(&id) {
                    rec.log_buffer.push(format!("[run:stderr] {line}"));
                }
            }
        });
    }

    // Store the child handle and transition status.
    let mut jobs = state.jobs.write().await;
    let rec = jobs.get_mut(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    // Transition to Running (may already be Running from test phase).
    match rec.status.transition(&EtlJobStatus::Running) {
        Ok(next) => rec.status = next,
        Err(_) => {
            // Already Running is acceptable for the run endpoint.
            if !matches!(rec.status, EtlJobStatus::Running) {
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!(
                            "cannot run from status: {}",
                            status_label(&rec.status)
                        ),
                    }),
                ));
            }
        }
    }
    rec.process = Some(child);

    Ok(Json(StatusResponse {
        job_id: id,
        status: status_label(&rec.status),
    }))
}

// ── 11.10 POST /api/jobs/:id/stop ────────────────────────────────────

/// Kill the running child process (if any) and set status to `Completed`.
async fn stop_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut jobs = state.jobs.write().await;
    let rec = jobs.get_mut(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    // Kill the child process if one is running.
    if let Some(ref mut child) = rec.process {
        let _ = child.kill().await;
    }
    rec.process = None;

    // Transition to Completed.
    match rec.status.transition(&EtlJobStatus::Completed) {
        Ok(next) => rec.status = next,
        Err(_) => {
            // Force to Completed if we're stopping.
            rec.status = EtlJobStatus::Completed;
        }
    }

    Ok(Json(StatusResponse {
        job_id: id,
        status: status_label(&rec.status),
    }))
}

// ── 12.1 GET /api/catalog/entries ────────────────────────────────────

/// Proxy catalog entry listing via the configured `SemanticBridge`.
async fn list_catalog_entries(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<semantic_bridge::CatalogEntrySummary>>, (StatusCode, Json<ErrorResponse>)> {
    let entries = state.bridge.list_entries().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to list catalog entries: {e}"),
            }),
        )
    })?;
    Ok(Json(entries))
}

// ── 12.2 POST /api/jobs/from-catalog/:entry_id ──────────────────────

/// Resolve a catalog entry into an `EtlJob` and submit it as a new job.
async fn create_job_from_catalog(
    State(state): State<Arc<AppState>>,
    Path(entry_id): Path<u64>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    let job = state.bridge.resolve_job(entry_id).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("failed to resolve catalog entry {entry_id}: {e}"),
            }),
        )
    })?;

    let job_id = job.job_id;
    let record = JobRecord {
        job,
        status: EtlJobStatus::Pending,
        generation_result: None,
        log_buffer: Vec::new(),
        process: None,
    };

    state.jobs.write().await.insert(job_id, record);

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "job_id": job_id })),
    ))
}

// ── 12.3 GET /api/jobs/:id/logs ─────────────────────────────────────

/// SSE endpoint streaming the current `log_buffer` lines for a job.
async fn get_job_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<ErrorResponse>)>
{
    let jobs = state.jobs.read().await;
    let rec = jobs.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("job {id} not found"),
            }),
        )
    })?;

    let lines: Vec<String> = rec.log_buffer.clone();
    drop(jobs);

    let event_stream = stream::iter(lines.into_iter().map(|line| {
        Ok(Event::default().data(line))
    }));

    Ok(Sse::new(event_stream))
}

// ── 12.4 GET /api/health ────────────────────────────────────────────

/// Return server health status and Tangent binary availability.
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let tangent_available = state.tangent_bin.exists();
    Json(serde_json::json!({
        "status": "ok",
        "tangent_available": tangent_available,
    }))
}

// ── 12.5 create_router ──────────────────────────────────────────────

/// Build the full Axum router with all routes and middleware.
///
/// Merges job lifecycle routes, catalog proxy, SSE logs, and health check,
/// then applies a permissive CORS layer for local development.
pub fn create_router(state: Arc<AppState>) -> Router {
    job_routes()
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ── 12.6 CompiledSchema loading ─────────────────────────────────────

/// Load a `CompiledSchema` from the path specified by the `OCSF_SCHEMA_PATH`
/// environment variable.
///
/// # Errors
/// Returns an error if the env var is not set, the file cannot be read,
/// or the JSON content cannot be parsed.
pub fn load_schema_from_env() -> anyhow::Result<CompiledSchema> {
    let path = std::env::var("OCSF_SCHEMA_PATH")
        .map_err(|_| anyhow::anyhow!("OCSF_SCHEMA_PATH environment variable is not set"))?;

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read schema from {path}: {e}"))?;

    let schema: CompiledSchema = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse compiled schema at {path}: {e}"))?;

    Ok(schema)
}
