//! OCSF Semantic Model Editor Library
//!
//! This crate provides a browser-based GUI server for editing OCSF semantic
//! models with LLM-assisted research capabilities.
//!
//! # Features
//!
//! - REST API for schema access, validation, and artifact generation
//! - LLM integration for semantic enrichment suggestions
//! - Semantic index integration for lineage and coverage tracking
//! - CORS support for local development
//! - Static file serving for the frontend application
//!
//! # Example
//!
//! ```no_run
//! use std::net::SocketAddr;
//! use ocsf_editor::{create_app, run_server};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!     run_server(addr).await
//! }
//! ```

pub mod api;
pub mod error;
pub mod llm_service;
pub mod schema_service;
pub mod validation_service;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use ocsf_catalog::{PluginManager, SemanticCatalog, SidecarCatalog};
use ocsf_index::backend::{BackendResult, IndexBackend, IndexRecord, InMemoryBackend, RecordFilter, SqliteBackend};
use ocsf_index::types::RecordId;
use ocsf_index::{IndexConfig, SemanticIndex};

/// Enum wrapper for index backends to support runtime backend selection.
///
/// Since `IndexBackend` trait is not dyn-compatible (due to async methods with generics),
/// we use an enum to support both SQLite and in-memory backends at runtime.
/// The backends are wrapped in Arc to allow cloning.
#[derive(Clone)]
pub enum AnyBackend {
    /// In-memory backend for development and testing.
    InMemory(Arc<InMemoryBackend>),
    /// SQLite backend for persistent storage.
    Sqlite(Arc<SqliteBackend>),
}

impl IndexBackend for AnyBackend {
    async fn create<T: IndexRecord>(&self, record: &T) -> BackendResult<RecordId> {
        match self {
            AnyBackend::InMemory(b) => b.create(record).await,
            AnyBackend::Sqlite(b) => b.create(record).await,
        }
    }

    async fn read<T: IndexRecord>(&self, id: RecordId) -> BackendResult<Option<T>> {
        match self {
            AnyBackend::InMemory(b) => b.read(id).await,
            AnyBackend::Sqlite(b) => b.read(id).await,
        }
    }

    async fn update<T: IndexRecord>(&self, id: RecordId, record: &T) -> BackendResult<()> {
        match self {
            AnyBackend::InMemory(b) => b.update(id, record).await,
            AnyBackend::Sqlite(b) => b.update(id, record).await,
        }
    }

    async fn delete(&self, record_type: &str, id: RecordId) -> BackendResult<()> {
        match self {
            AnyBackend::InMemory(b) => b.delete(record_type, id).await,
            AnyBackend::Sqlite(b) => b.delete(record_type, id).await,
        }
    }

    async fn list<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<Vec<T>> {
        match self {
            AnyBackend::InMemory(b) => b.list(filter).await,
            AnyBackend::Sqlite(b) => b.list(filter).await,
        }
    }

    async fn batch_create<T: IndexRecord>(&self, records: &[T]) -> BackendResult<Vec<RecordId>> {
        match self {
            AnyBackend::InMemory(b) => b.batch_create(records).await,
            AnyBackend::Sqlite(b) => b.batch_create(records).await,
        }
    }

    async fn batch_delete(&self, record_type: &str, ids: &[RecordId]) -> BackendResult<u64> {
        match self {
            AnyBackend::InMemory(b) => b.batch_delete(record_type, ids).await,
            AnyBackend::Sqlite(b) => b.batch_delete(record_type, ids).await,
        }
    }

    async fn count<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<u64> {
        match self {
            AnyBackend::InMemory(b) => b.count::<T>(filter).await,
            AnyBackend::Sqlite(b) => b.count::<T>(filter).await,
        }
    }
}

pub use llm_service::{
    LLMService, LLMSuggestion, OCSFContext, ResearchField, ResearchResult, ResearchTarget,
    ResearchTargetType, SuggestionValue, LLMProvider,
};
pub use schema_service::{SchemaService, SchemaTree, SearchResult};
pub use validation_service::{
    ValidationError, ValidationReport, ValidationService, ValidationWarning,
};

// Re-export ocsf-index types for API handlers
pub use ocsf_index::backend::IndexBackend as IndexBackendTrait;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Schema service for loading and caching OCSF schema.
    pub schema_service: Arc<SchemaService>,
    /// Validation service for model validation.
    pub validation_service: Arc<ValidationService>,
    /// LLM service for semantic enrichment (configurable at runtime).
    pub llm_service: Arc<RwLock<Option<LLMService>>>,
    /// Semantic index for lineage and coverage tracking.
    pub semantic_index: Arc<SemanticIndex<AnyBackend>>,
    /// Semantic catalog for entity metadata.
    pub catalog: Arc<dyn SemanticCatalog>,
    /// Plugin manager for engine sync.
    pub plugin_manager: Arc<tokio::sync::Mutex<PluginManager>>,
}

impl Default for AppState {
    fn default() -> Self {
        // Use blocking runtime to call async new() from sync context
        // This is only used for tests and simple initialization
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::new())
        })
    }
}

impl AppState {
    /// Create a new application state.
    ///
    /// This initializes all services including the semantic index.
    /// The index backend is selected based on the `OCSF_INDEX_PATH` environment variable:
    /// - If set: Uses SQLite backend at the specified path
    /// - If not set: Uses in-memory backend for development
    ///
    /// The schema is auto-loaded from `OCSF_SCHEMA_PATH` environment variable if set,
    /// otherwise it tries to load from `demo/schema/ocsf-compiled-v1.6.0.json`.
    pub async fn new() -> Self {
        let schema_service = Arc::new(SchemaService::new());
        
        // Try to auto-load schema from environment or default path
        if let Err(e) = Self::try_load_schema(&schema_service).await {
            eprintln!("Warning: Could not auto-load schema: {}", e);
        }
        
        let validation_service = Arc::new(ValidationService::new(schema_service.clone()));
        // Try to create LLM service from environment, but don't fail if not configured
        let llm_service = LLMService::from_env().ok();
        
        // Initialize index backend based on environment
        let index_backend = Self::create_index_backend();
        let index_config = IndexConfig::default();
        let semantic_index = SemanticIndex::new(index_backend, index_config)
            .await
            .expect("Failed to initialize semantic index");
        
        let catalog_path = std::env::temp_dir().join("ocsf_catalog.bin");
        let catalog: Arc<dyn SemanticCatalog> = Arc::new(
            if catalog_path.exists() {
                SidecarCatalog::open(&catalog_path).unwrap_or_else(|_| {
                    SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
                })
            } else {
                SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
            }
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(PluginManager::new(catalog.clone())));
        
        Self {
            schema_service,
            validation_service,
            llm_service: Arc::new(RwLock::new(llm_service)),
            semantic_index: Arc::new(semantic_index),
            catalog,
            plugin_manager,
        }
    }
    
    /// Try to load schema from environment variable or default path.
    async fn try_load_schema(schema_service: &Arc<SchemaService>) -> Result<(), String> {
        use std::path::Path;
        
        // Check environment variable first
        let schema_path = std::env::var("OCSF_SCHEMA_PATH")
            .unwrap_or_else(|_| "demo/schema/ocsf-compiled-v1.6.0.json".to_string());
        
        let path = Path::new(&schema_path);
        if path.exists() {
            schema_service
                .load_schema(path)
                .await
                .map_err(|e| format!("Failed to load schema from {}: {}", schema_path, e))?;
            println!("   Schema: Loaded from {}", schema_path);
            Ok(())
        } else {
            Err(format!("Schema file not found: {}", schema_path))
        }
    }
    
    /// Create the index backend based on environment configuration.
    ///
    /// Checks the `OCSF_INDEX_PATH` environment variable:
    /// - If set: Uses SQLite backend at the specified path (Requirement 1.4)
    /// - If not set: Uses in-memory backend for development (Requirement 1.5)
    fn create_index_backend() -> AnyBackend {
        match std::env::var("OCSF_INDEX_PATH") {
            Ok(path) => {
                // Use SQLite backend when path is configured (Requirement 1.4)
                AnyBackend::Sqlite(Arc::new(
                    SqliteBackend::new_file(&path)
                        .expect("Failed to open SQLite index database")
                ))
            }
            Err(_) => {
                // Use in-memory backend for development (Requirement 1.5)
                AnyBackend::InMemory(Arc::new(InMemoryBackend::new()))
            }
        }
    }

    /// Create a new application state with a pre-configured schema service.
    pub async fn with_schema_service(schema_service: Arc<SchemaService>) -> Self {
        let validation_service = Arc::new(ValidationService::new(schema_service.clone()));
        let llm_service = LLMService::from_env().ok();
        
        let index_backend = Self::create_index_backend();
        let index_config = IndexConfig::default();
        let semantic_index = SemanticIndex::new(index_backend, index_config)
            .await
            .expect("Failed to initialize semantic index");
        
        let catalog_path = std::env::temp_dir().join("ocsf_catalog.bin");
        let catalog: Arc<dyn SemanticCatalog> = Arc::new(
            if catalog_path.exists() {
                SidecarCatalog::open(&catalog_path).unwrap_or_else(|_| {
                    SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
                })
            } else {
                SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
            }
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(PluginManager::new(catalog.clone())));
        
        Self {
            schema_service,
            validation_service,
            llm_service: Arc::new(RwLock::new(llm_service)),
            semantic_index: Arc::new(semantic_index),
            catalog,
            plugin_manager,
        }
    }

    /// Create a new application state with all services configured.
    pub async fn with_services(
        schema_service: Arc<SchemaService>,
        llm_service: Option<LLMService>,
    ) -> Self {
        let validation_service = Arc::new(ValidationService::new(schema_service.clone()));
        
        let index_backend = Self::create_index_backend();
        let index_config = IndexConfig::default();
        let semantic_index = SemanticIndex::new(index_backend, index_config)
            .await
            .expect("Failed to initialize semantic index");
        
        let catalog_path = std::env::temp_dir().join("ocsf_catalog.bin");
        let catalog: Arc<dyn SemanticCatalog> = Arc::new(
            if catalog_path.exists() {
                SidecarCatalog::open(&catalog_path).unwrap_or_else(|_| {
                    SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
                })
            } else {
                SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
            }
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(PluginManager::new(catalog.clone())));
        
        Self {
            schema_service,
            validation_service,
            llm_service: Arc::new(RwLock::new(llm_service)),
            semantic_index: Arc::new(semantic_index),
            catalog,
            plugin_manager,
        }
    }
    
    /// Create a new application state with a custom index backend.
    ///
    /// This is useful for testing with specific backend configurations.
    pub async fn with_index_backend(
        schema_service: Arc<SchemaService>,
        llm_service: Option<LLMService>,
        index_backend: AnyBackend,
    ) -> Self {
        let validation_service = Arc::new(ValidationService::new(schema_service.clone()));
        
        let index_config = IndexConfig::default();
        let semantic_index = SemanticIndex::new(index_backend, index_config)
            .await
            .expect("Failed to initialize semantic index");
        
        let catalog_path = std::env::temp_dir().join("ocsf_catalog.bin");
        let catalog: Arc<dyn SemanticCatalog> = Arc::new(
            if catalog_path.exists() {
                SidecarCatalog::open(&catalog_path).unwrap_or_else(|_| {
                    SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
                })
            } else {
                SidecarCatalog::create(&catalog_path).expect("Failed to create catalog")
            }
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(PluginManager::new(catalog.clone())));
        
        Self {
            schema_service,
            validation_service,
            llm_service: Arc::new(RwLock::new(llm_service)),
            semantic_index: Arc::new(semantic_index),
            catalog,
            plugin_manager,
        }
    }
    
    /// Configure the LLM service with an API key.
    pub async fn configure_llm(&self, provider: &str, api_key: String) -> Result<(), String> {
        let service = match provider.to_lowercase().as_str() {
            "anthropic" => LLMService::new_anthropic(api_key),
            "openai" => LLMService::new(api_key),
            _ => return Err(format!("Unknown provider: {}. Use 'anthropic' or 'openai'", provider)),
        };
        
        let mut llm = self.llm_service.write().await;
        *llm = Some(service);
        Ok(())
    }
    
    /// Check if LLM service is configured.
    pub async fn is_llm_configured(&self) -> bool {
        let llm = self.llm_service.read().await;
        llm.is_some()
    }
    
    /// Get the current LLM provider name.
    pub async fn llm_provider_name(&self) -> Option<String> {
        let llm = self.llm_service.read().await;
        llm.as_ref().map(|s| match s.provider() {
            LLMProvider::OpenAI => "openai".to_string(),
            LLMProvider::Anthropic => "anthropic".to_string(),
        })
    }
}

/// Create the API router with all endpoints.
fn create_api_router() -> Router<AppState> {
    Router::new()
        // Schema endpoints (Requirement 7.1)
        .route("/schema", get(api::get_schema))
        // Validation endpoints (Requirement 7.2)
        .route("/validate", post(api::validate_model))
        // Generation endpoints (Requirement 7.3)
        .route("/generate", post(api::generate_artifacts))
        // LLM research endpoints (Requirement 7.4)
        .route("/llm/research", post(api::llm_research))
        .route("/llm/research/batch", post(api::llm_research_batch))
        // LLM configuration endpoints
        .route("/llm/config", get(api::get_llm_config).post(api::set_llm_config))
        // Health check endpoint
        .route("/health", get(api::health_check))
        // Index endpoints (Requirements 2-7, 18-19)
        .nest("/index", api::index::create_index_router())
        // Catalog endpoints
        .nest("/catalog", api::catalog::create_catalog_router())
}

/// Create the main application router.
///
/// This sets up all API routes with CORS support for local development.
///
/// # Requirements
/// - 7.6: THE API_Server SHALL support CORS for local development with
///   configurable allowed origins
/// - 1.1: THE API_Server SHALL initialize a Semantic_Index instance on startup
/// - 1.2: THE API_Server SHALL store the Semantic_Index in AppState
pub async fn create_app() -> Router {
    let state = AppState::new().await;

    // Configure CORS for local development (Requirement 7.6)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve the frontend SPA from editor-ui/dist with index.html fallback
    let static_dir = "editor-ui/dist";
    let serve_dir = ServeDir::new(static_dir)
        .not_found_service(ServeFile::new(format!("{}/index.html", static_dir)));

    Router::new()
        .nest("/api", create_api_router())
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state)
}

/// Default server address.
pub const DEFAULT_ADDR: &str = "127.0.0.1:8080";

/// Run the editor server on the specified address.
///
/// # Arguments
///
/// * `addr` - The socket address to bind the server to
///
/// # Errors
///
/// Returns an error if the server fails to bind or encounters a runtime error.
pub async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
    let app = create_app().await;

    println!("🚀 OCSF Semantic Model Editor starting...");
    println!("   Server: http://{}", addr);
    println!("   API:    http://{}/api", addr);
    
    // Log index backend configuration
    if std::env::var("OCSF_INDEX_PATH").is_ok() {
        println!("   Index:  SQLite (persistent)");
    } else {
        println!("   Index:  In-memory (development)");
    }
    
    println!();
    println!("Press Ctrl+C to stop the server.");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_new() {
        let state = AppState::new().await;
        // Verify semantic_index is initialized
        assert!(state.semantic_index.config().cache_max_entries > 0);
    }

    #[tokio::test]
    async fn test_create_app() {
        let _app = create_app().await;
    }

    #[test]
    fn test_default_addr_is_valid() {
        let addr: Result<SocketAddr, _> = DEFAULT_ADDR.parse();
        assert!(addr.is_ok());
    }
    
    #[test]
    fn test_create_index_backend_in_memory() {
        // Without OCSF_INDEX_PATH set, should create in-memory backend
        std::env::remove_var("OCSF_INDEX_PATH");
        let backend = AppState::create_index_backend();
        // Backend should be created successfully (we can't easily check the type,
        // but if it doesn't panic, it worked)
        let _ = backend;
    }
}
