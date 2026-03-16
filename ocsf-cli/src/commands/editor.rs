//! Editor server command.
//!
//! This command starts the OCSF Semantic Model Editor server,
//! serving the frontend static files and API endpoints.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;
use crate::error::CliError;

/// Runs the editor command.
///
/// Starts the OCSF Semantic Model Editor server with the specified options.
///
/// # Arguments
///
/// * `port` - Port to run the server on (default: 8080)
/// * `schema_path` - Optional path to compiled OCSF schema JSON file to load
/// * `static_dir` - Optional path to frontend static files directory
/// * `no_open` - If true, don't automatically open browser
/// * `config` - CLI configuration
/// * `verbose` - Enable verbose output
///
/// # Requirements
///
/// - 7.1: THE API_Server SHALL expose a GET /api/schema endpoint
pub async fn run(
    port: u16,
    schema_path: Option<PathBuf>,
    static_dir: Option<PathBuf>,
    no_open: bool,
    _config: &Config,
    verbose: bool,
) -> Result<()> {
    use ocsf_editor::{AppState, SchemaService};

    // Create schema service
    let schema_service = Arc::new(SchemaService::new());

    // Load schema if path provided
    if let Some(schema_path) = &schema_path {
        if !schema_path.exists() {
            return Err(CliError::file_not_found(schema_path).into());
        }

        if verbose {
            eprintln!("Loading OCSF schema from: {}", schema_path.display());
        }

        // The SchemaService expects a compiled schema JSON file
        // (e.g., ocsf-compiled-v1.6.0.json from the ocsf-schema-compiler)
        if schema_path.is_file() {
            // Load compiled schema using the SchemaService
            schema_service.load_schema(schema_path).await
                .map_err(|e| CliError::invalid_file_format(
                    schema_path,
                    format!("Failed to load compiled schema: {}", e)
                ))?;
            
            if verbose {
                let version = schema_service.get_version_async().await.unwrap_or_default();
                eprintln!("Loaded compiled schema version: {}", version);
            }
        } else {
            return Err(CliError::invalid_file_format(
                schema_path,
                "Schema path must be a compiled OCSF schema JSON file (e.g., ocsf-compiled-v1.6.0.json)".to_string()
            ).into());
        }
    } else {
        // Try to find a default compiled schema in common locations
        let default_paths = [
            PathBuf::from("demo/schema/ocsf-compiled-v1.6.0.json"),
            PathBuf::from("schema/ocsf-compiled.json"),
        ];
        
        let mut loaded = false;
        for path in &default_paths {
            if path.exists() {
                if verbose {
                    eprintln!("Loading default schema from: {}", path.display());
                }
                if schema_service.load_schema(path).await.is_ok() {
                    loaded = true;
                    if verbose {
                        let version = schema_service.get_version_async().await.unwrap_or_default();
                        eprintln!("Loaded compiled schema version: {}", version);
                    }
                    break;
                }
            }
        }
        
        if !loaded && verbose {
            eprintln!("No schema loaded. Use --schema to specify a compiled schema JSON file.");
            eprintln!("Example: ocsf editor --schema demo/schema/ocsf-compiled-v1.6.0.json");
        }
    }

    // Create app state with the schema service
    let state = AppState::with_schema_service(schema_service).await;

    // Build the server address
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| CliError::configuration(format!("Invalid port: {}", e)))?;

    // Determine static files directory
    let static_path = static_dir.unwrap_or_else(|| {
        // Default to editor-ui/dist relative to workspace root
        PathBuf::from("editor-ui/dist")
    });

    // Print startup banner
    println!();
    println!("🚀 OCSF Semantic Model Editor");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Server:    http://{}", addr);
    println!("   API:       http://{}/api", addr);
    
    if static_path.exists() {
        println!("   Frontend:  http://{}", addr);
        println!("   Static:    {}", static_path.display());
    } else {
        println!("   Frontend:  Not available (build with 'npm run build' in editor-ui/)");
    }
    
    println!();
    println!("   Press Ctrl+C to stop the server.");
    println!();

    // Open browser if requested
    if !no_open {
        let url = format!("http://{}", addr);
        if let Err(e) = open_browser(&url) {
            if verbose {
                eprintln!("Could not open browser: {}", e);
            }
        }
    }

    // Create the router with static file serving
    let app = create_editor_app(state, &static_path);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| CliError::configuration(format!("Failed to bind to {}: {}", addr, e)))?;
    
    axum::serve(listener, app).await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

/// Create the editor application router with static file serving.
fn create_editor_app(state: ocsf_editor::AppState, static_dir: &PathBuf) -> axum::Router {
    use axum::{
        routing::{get, post},
        Router,
    };
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::services::ServeDir;

    // Configure CORS for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create API router
    let api_router = Router::new()
        .route("/schema", get(ocsf_editor::api::get_schema))
        .route("/validate", post(ocsf_editor::api::validate_model))
        .route("/generate", post(ocsf_editor::api::generate_artifacts))
        .route("/llm/research", post(ocsf_editor::api::llm_research))
        .route("/llm/research/batch", post(ocsf_editor::api::llm_research_batch))
        .route("/llm/config", get(ocsf_editor::api::get_llm_config).post(ocsf_editor::api::set_llm_config))
        .route("/health", get(ocsf_editor::api::health_check));

    // Build the main router
    let mut router = Router::new()
        .nest("/api", api_router)
        .layer(cors)
        .with_state(state);

    // Add static file serving if directory exists
    if static_dir.exists() {
        let serve_dir = ServeDir::new(static_dir)
            .append_index_html_on_directories(true);
        
        router = router.fallback_service(serve_dir);
    }

    router
}

/// Attempt to open the default browser to the given URL.
fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }

    Ok(())
}
