# Tech Stack

## Languages & Frameworks

- Rust (primary language)
- Property-based testing with `proptest`

## Build System

- Cargo workspace

## Key Dependencies

- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tokio` - Async runtime
- `anyhow`, `thiserror` - Error handling
- `clap` - CLI framework
- `reqwest` - HTTP client for GitHub schema fetching
- `qdrant-client` or in-memory vector store
- `candle` or `ort` - ML inference for embeddings
- `svg`, `resvg` - Visualization export

## Common Commands

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run property tests (may take longer)
cargo test --workspace -- --include-ignored

# Run specific crate
cargo run -p ocsf-cli -- <command>

# Check formatting
cargo fmt --check

# Lint
cargo clippy --workspace
```

## Environment Setup

- Rust 1.75+ (for async traits)
- Optional: OpenAI API key for embedding generation
