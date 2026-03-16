# Project Structure

```
.
├── .kiro/
│   ├── steering/           # AI assistant guidance
│   └── specs/              # Feature specifications
│       └── ocsf-semantic-layer/
├── Cargo.toml              # Workspace manifest
├── ocsf-core/              # OCSF schema data structures
│   └── src/
├── ocsf-semantic/          # Semantic model definitions
│   └── src/
├── ocsf-vector/            # Embeddings and vector store
│   └── src/
├── ocsf-warehouse/         # Warehouse artifact generation
│   └── src/
├── ocsf-viz/               # Visualization engine
│   └── src/
└── ocsf-cli/               # Command-line interface
    └── src/
```

## Directory Conventions

- `ocsf-core/`: Core OCSF schema types, parsing, observable extraction
- `ocsf-semantic/`: Semantic entities, metrics, validation, model store
- `ocsf-vector/`: Embedding generation, vector store, mapping assistant
- `ocsf-warehouse/`: dbt, Cube.js, SQL generation, ETL pipelines
- `ocsf-viz/`: Graph generation, view modes, export formats
- `ocsf-cli/`: CLI commands, configuration, integration

## File Naming

- Rust modules: `snake_case.rs`
- Test files: `*_test.rs` or in `tests/` directory
- Config files: `*.yaml` or `*.toml`

## Architecture Patterns

- Layered architecture: Core → Semantic → Vector → Query → Output
- Trait-based abstractions for embedding models and vector stores
- Builder pattern for complex struct construction
- Result-based error handling with `thiserror`
