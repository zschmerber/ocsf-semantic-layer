//! OCSF Semantic Model Editor Server
//!
//! A browser-based GUI for editing OCSF semantic models with LLM-assisted
//! research capabilities. This server provides:
//!
//! - REST API endpoints for schema access, validation, and generation
//! - LLM integration for semantic enrichment
//! - Static file serving for the frontend application
//!
//! # API Endpoints
//!
//! - `GET /api/schema` - Returns the loaded OCSF schema as JSON
//! - `POST /api/validate` - Validates a semantic model against the schema
//! - `POST /api/generate` - Generates warehouse artifacts from a model
//! - `POST /api/llm/research` - Queries the LLM for semantic enrichments
//!
//! # Requirements
//!
//! - Requirements 7.1: GET /api/schema endpoint
//! - Requirements 7.6: CORS support for local development

use std::net::SocketAddr;

use ocsf_editor::{run_server, DEFAULT_ADDR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = DEFAULT_ADDR.parse()?;
    run_server(addr).await
}
