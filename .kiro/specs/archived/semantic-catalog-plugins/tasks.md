# Implementation Plan: Semantic Catalog Plugins

## Overview

Implement the `ocsf-catalog` crate with a binary sidecar file, `SemanticCatalog` and `CatalogPlugin` traits, four engine plugins, vector embedding integration, editor API extensions, and editor UI components. Tasks are ordered to build incrementally: scaffold → data models → sidecar → catalog trait → vector → plugins → API → UI.

## Tasks

- [x] 1. Scaffold `ocsf-catalog` crate and add to workspace
  - Create `ocsf-catalog/` directory with `Cargo.toml` and `src/lib.rs`
  - Add `ocsf-catalog` to the workspace `Cargo.toml` members list
  - Add dependencies: `bincode`, `memmap2`, `crc32fast`, `serde`, `serde_json`, `tokio`, `async-trait`, `thiserror`, `uuid`, `chrono`, `anyhow`
  - Add dev-dependencies: `proptest`, `tempfile`
  - Declare public modules: `error`, `models`, `sidecar`, `catalog`, `plugin`, `manager`
  - _Requirements: 1.1, 2.1, 3.1, 5.1, 14.1_

- [x] 2. Implement data models
  - [x] 2.1 Define `CatalogEntryId`, `CatalogEntry`, and supporting types
    - Implement `CatalogEntryId(u64)` with `Serialize`/`Deserialize`/`Copy`/`Hash`
    - Implement `CatalogEntry` struct with all fields from the design: `id`, `entity_name`, `caption`, `description`, `ocsf_version`, `source_event_classes`, `attributes`, `relationships`, `covers_observables`, `detection_coverage`, `source_lineage`, `field_lineage`, `catalog_version`, `updated_at`
    - Implement `CatalogLineageRecord` and `CatalogFieldLineage` structs
    - Implement `SyncDiff`, `PushResult`, `PullResult`, `SyncResult`, `PluginInfo` structs
    - _Requirements: 1.1, 1.2, 1.4, 1.5_

  - [x] 2.2 Implement `CatalogError` enum and `CatalogResult` type alias
    - Define all variants: `Io`, `Serialization`, `Corruption`, `DuplicateEntry`, `NotFound`, `Plugin`
    - Derive `thiserror::Error` with display messages matching the design
    - _Requirements: 14.1, 14.2, 14.3_

  - [ ]* 2.3 Write property test for `CatalogEntry` serialization round-trip
    - Implement `proptest` `Arbitrary` strategy for `CatalogEntry` (random names, versions from `["1.3.0","1.4.0","2.0.0"]`, 0–10 attributes, 0–5 relationships, optional detection coverage, 0–3 lineage records)
    - **Property 1: Sidecar file round-trip** — bincode serialize then deserialize a `CatalogEntry` produces an equivalent value
    - **Validates: Requirements 1.5, 2.6**

- [x] 3. Implement sidecar file format
  - [x] 3.1 Implement `SidecarHeader` with fixed-size binary layout
    - Define `SidecarHeader` struct (56 bytes: magic `b"OCSF"`, `format_version: u32`, `catalog_version: u64`, `ocsf_version: [u8; 32]`, `entry_count: u64`, `data_checksum: u32`)
    - Implement `SidecarHeader::to_bytes()` and `SidecarHeader::from_bytes()` using fixed-size encoding
    - _Requirements: 2.1_

  - [x] 3.2 Implement `SidecarCatalog` open/create and memory-mapped reads
    - Implement `SidecarCatalog::create(path)` — writes a fresh header with magic and `format_version = 1`, empty data section
    - Implement `SidecarCatalog::open(path)` — validates magic number and format version (`CatalogError::Corruption` on mismatch), validates CRC32 checksum of data section, loads entries into in-memory index
    - Implement memory-mapped read path using `memmap2::Mmap` for the data section
    - _Requirements: 2.2, 2.3, 2.4, 2.5_

  - [ ]* 3.3 Write unit tests for sidecar file validation
    - Test corrupted magic number returns `CatalogError::Corruption`
    - Test invalid format version returns `CatalogError::Corruption`
    - Test checksum mismatch returns `CatalogError::Corruption`
    - _Requirements: 2.3, 2.4_

- [x] 4. Implement `SemanticCatalog` trait and `SidecarCatalog` CRUD
  - [x] 4.1 Define `SemanticCatalog` trait
    - Declare all async methods: `add`, `get`, `get_by_name`, `update`, `remove`, `list`, `list_versions`, `catalog_version`
    - Use `#[async_trait]` and `Send + Sync` bounds
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.4_

  - [x] 4.2 Implement `SemanticCatalog` for `SidecarCatalog`
    - `add`: check in-memory index for duplicate `(entity_name, ocsf_version)` → `CatalogError::DuplicateEntry`; assign next ID; bincode-serialize entry; append to data section; recompute CRC32; update header; update in-memory index
    - `get` / `get_by_name`: look up in-memory index, deserialize from mmap
    - `update`: replace entry bytes in data section; increment `catalog_version`; recompute checksum
    - `remove`: remove from data section; rebuild file; decrement `entry_count`; increment `catalog_version`
    - `list` / `list_versions`: iterate in-memory index
    - _Requirements: 3.1–3.8, 4.1–4.4_

  - [ ]* 4.3 Write property test: add returns unique IDs
    - **Property 2: Add returns unique IDs** — adding N entries with distinct `(entity_name, ocsf_version)` pairs yields N distinct `CatalogEntryId` values
    - **Validates: Requirements 1.3**

  - [ ]* 4.4 Write property test: add then retrieve by ID and name
    - **Property 3: Add then retrieve by ID and name** — after `add`, `get(id)` and `get_by_name(name, version)` return an entry with matching fields
    - **Validates: Requirements 3.2, 3.3**

  - [ ]* 4.5 Write property test: remove then get returns None
    - **Property 4: Remove then get returns None** — after `remove(id)`, `get(id)` returns `None`
    - **Validates: Requirements 3.5**

  - [ ]* 4.6 Write property test: duplicate add returns error
    - **Property 5: Duplicate add returns error** — first `add` succeeds; second `add` with same `(entity_name, ocsf_version)` returns `CatalogError::DuplicateEntry`
    - **Validates: Requirements 3.8**

  - [ ]* 4.7 Write property test: write operations increment catalog version
    - **Property 6: Write operations increment catalog version** — `catalog_version()` after any write (add/update/remove) is strictly greater than before
    - **Validates: Requirements 3.4, 3.7**

  - [ ]* 4.8 Write property test: version filtering correctness
    - **Property 7: Version filtering correctness** — `list(Some(v))` returns only entries with `ocsf_version == v`; `list(None)` returns all; `list_versions()` returns exactly the distinct versions present
    - **Validates: Requirements 4.2, 4.3, 4.4**

- [x] 5. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Integrate vector embeddings via `ocsf-vector`
  - [x] 6.1 Add `ocsf-vector` as a dependency in `ocsf-catalog/Cargo.toml`
    - Wire `EmbeddingGenerator` and `InMemoryVectorStore` into `SidecarCatalog`
    - On `add` and `update`, generate an embedding for the entry's `entity_name + description + attributes` text and store it in the vector store with the `CatalogEntryId` as the key
    - _Requirements: (vector integration — supports semantic search)_

  - [x] 6.2 Add `semantic_search` method to `SemanticCatalog` trait
    - Signature: `async fn semantic_search(&self, query: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>>`
    - Implement in `SidecarCatalog`: generate embedding for query, call `VectorStore::similarity_search`, resolve IDs to entries
    - _Requirements: (semantic search endpoint)_

  - [x] 6.3 Add `field_mapping_suggestions` method to `SemanticCatalog` trait
    - Signature: `async fn field_mapping_suggestions(&self, source_field: &str, top_k: usize) -> CatalogResult<Vec<CatalogEntry>>`
    - Delegate to `MappingAssistant` from `ocsf-vector`
    - _Requirements: (field mapping suggestions)_

- [x] 7. Implement `CatalogPlugin` trait and `PluginManager`
  - [x] 7.1 Define `CatalogPlugin` trait
    - Declare async methods: `engine_name() -> &'static str`, `push(entries: &[CatalogEntry]) -> CatalogResult<PushResult>`, `pull() -> CatalogResult<Vec<CatalogEntry>>`, `diff(catalog_entries: &[CatalogEntry]) -> CatalogResult<SyncDiff>`
    - Use `#[async_trait]` and `Send + Sync` bounds
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 7.2 Implement `PluginManager`
    - `new(catalog: Arc<dyn SemanticCatalog>)`, `register(plugin)`, `list_plugins() -> Vec<PluginInfo>`
    - `push(engine)`: fetch all entries from catalog, call `plugin.push(&entries)`
    - `pull(engine)`: call `plugin.pull()`, merge entries into catalog using last-writer-wins on `catalog_version`
    - `diff(engine)`: fetch all entries, call `plugin.diff(&entries)`
    - `full_sync(engine)`: compute diff → push local changes → pull remote changes → resolve conflicts → return `SyncResult`
    - _Requirements: 6.1–6.5_

  - [ ]* 7.3 Write property test: plugin push/pull round-trip (in-memory mock plugin)
    - Implement a `MockPlugin` that stores entries in a `HashMap` keyed by `(entity_name, ocsf_version)`
    - **Property 8: Plugin push/pull round-trip** — push N entries then pull returns entries with equivalent `entity_name`, `ocsf_version`, `attributes`, `detection_coverage`, lineage
    - **Validates: Requirements 6.1, 7.1, 7.2, 8.1, 8.2, 9.1, 9.2, 9.3, 10.1, 10.2**

  - [ ]* 7.4 Write property test: diff correctness
    - **Property 9: Diff correctness** — given catalog state S and engine state E, `diff` produces `added` = S \ E, `modified` = S ∩ E with differing `catalog_version`, `removed` = E \ S (by entity name + version)
    - **Validates: Requirements 6.3**

  - [ ]* 7.5 Write property test: last-writer-wins conflict resolution
    - **Property 10: Last-writer-wins conflict resolution** — when both catalog and engine have the same entry with different `catalog_version`, `full_sync` retains the entry with the higher `catalog_version` in both stores
    - **Validates: Requirements 6.5**

- [x] 8. Implement engine plugins
  - [x] 8.1 Implement `IcebergPlugin`
    - Create `ocsf-catalog/src/plugins/iceberg.rs`
    - `push`: serialize each `CatalogEntry` sub-field to JSON; write as Iceberg table properties with `ocsf.semantic.*` key prefix (use a mock/stub HTTP client or `iceberg-rust` if available; otherwise write a trait-compatible stub with a configurable `HashMap` backend for testing)
    - `pull`: read properties with `ocsf.semantic.*` prefix; deserialize JSON back to `CatalogEntry`
    - `diff`: compare by `(entity_name, ocsf_version)` and `catalog_version`
    - _Requirements: 7.1, 7.2, 7.3_

  - [x] 8.2 Implement `DeltaPlugin`
    - Create `ocsf-catalog/src/plugins/delta.rs`
    - Same `ocsf.semantic.*` key prefix convention as Iceberg, targeting Delta Lake table properties
    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 8.3 Implement `ClickHousePlugin`
    - Create `ocsf-catalog/src/plugins/clickhouse.rs`
    - `push`: write JSON summary as table comment; insert rows into `ocsf_semantic_catalog` dictionary table
    - `pull`: read from `ocsf_semantic_catalog` table; reconstruct `CatalogEntry`
    - _Requirements: 9.1, 9.2, 9.3_

  - [x] 8.4 Implement `DuckDBPlugin`
    - Create `ocsf-catalog/src/plugins/duckdb.rs`
    - `push`: upsert rows into `ocsf_semantic_catalog` table with JSON columns for attributes, detection coverage, lineage
    - `pull`: read from `ocsf_semantic_catalog` table; reconstruct `CatalogEntry`
    - _Requirements: 10.1, 10.2, 10.3_

- [x] 9. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Implement editor API (`ocsf-editor`)
  - [x] 10.1 Add `ocsf-catalog` as a dependency in `ocsf-editor/Cargo.toml`
    - Extend `AppState` to hold `Arc<dyn SemanticCatalog>` and `Arc<PluginManager>`
    - _Requirements: 11.1_

  - [x] 10.2 Implement catalog CRUD handlers in `ocsf-editor/src/api/catalog.rs`
    - `GET /api/catalog/entries` → `list_entries` (optional `?ocsf_version=` query param)
    - `POST /api/catalog/entries` → `create_entry` (returns 201 with entry + assigned ID + catalog version)
    - `GET /api/catalog/entries/:id` → `get_entry` (404 on `NotFound`)
    - `PUT /api/catalog/entries/:id` → `update_entry`
    - `DELETE /api/catalog/entries/:id` → `delete_entry`
    - `GET /api/catalog/versions` → `list_versions`
    - Map `CatalogError` variants to HTTP status codes: `NotFound` → 404, `DuplicateEntry` → 409, `Corruption`/`Io`/`Serialization` → 500, `Plugin` → 502
    - _Requirements: 11.1, 11.5_

  - [x] 10.3 Implement plugin sync handlers
    - `GET /api/catalog/plugins` → `list_plugins`
    - `POST /api/catalog/plugins/:engine/push` → `push_sync`
    - `POST /api/catalog/plugins/:engine/pull` → `pull_sync`
    - `GET /api/catalog/plugins/:engine/diff` → `get_diff`
    - `POST /api/catalog/plugins/:engine/sync` → `full_sync`
    - _Requirements: 11.2, 11.3, 11.4_

  - [x] 10.4 Implement semantic search and field mapping suggestion handlers
    - `GET /api/catalog/search?q=...&top_k=...` → `semantic_search`
    - `GET /api/catalog/suggest?field=...&top_k=...` → `field_mapping_suggestions`

  - [x] 10.5 Register `create_catalog_router()` in `create_api_router` in `ocsf-editor/src/lib.rs`
    - Nest the catalog router under `/api/catalog` following the existing `create_index_router` pattern
    - _Requirements: 11.1_

  - [ ]* 10.6 Write unit tests for catalog CRUD handlers
    - Use an in-memory `MockCatalog` implementing `SemanticCatalog`
    - Test create returns 201 with ID; get returns 200; get unknown returns 404; duplicate create returns 409
    - _Requirements: 11.1, 11.5_

  - [ ]* 10.7 Write unit tests for plugin sync handlers
    - Use a `MockPlugin` and `MockCatalog`
    - Test push/pull/diff/full-sync return correct HTTP status and body shapes
    - _Requirements: 11.2, 11.3, 11.4_

- [x] 11. Implement editor UI components
  - [x] 11.1 Implement `CatalogBrowser` component
    - Fetch entries from `GET /api/catalog/entries`
    - Display table with columns: name, `ocsf_version`, source event class count, attribute count, detection coverage summary
    - Add `<select>` filter for OCSF schema version (populated from `GET /api/catalog/versions`)
    - Add delete button per row with confirmation dialog
    - _Requirements: 12.1, 12.4, 12.5_

  - [x] 11.2 Implement `CatalogEntryForm` component
    - Create/edit form for `CatalogEntry` fields
    - Pre-populate from existing `SemanticEntity` definitions when available
    - Submit to `POST /api/catalog/entries` (create) or `PUT /api/catalog/entries/:id` (edit)
    - _Requirements: 12.2_

  - [x] 11.3 Implement `CatalogEntryDetail` component
    - Read-only detail view showing all entry fields: attributes, OCSF mappings, relationships, detection coverage, lineage
    - _Requirements: 12.3_

  - [x] 11.4 Implement `PluginDashboard` component
    - Fetch plugin list from `GET /api/catalog/plugins`
    - Display engine name and connection status per plugin
    - Buttons: "Diff", "Push", "Pull", "Full Sync" per plugin row
    - On sync completion, display summary (entries pushed/pulled/conflicts resolved)
    - On sync failure, display error message from API response
    - _Requirements: 13.1, 13.3, 13.4, 13.5_

  - [x] 11.5 Implement `SyncDiffView` component
    - Display diff result with three sections: Added, Modified, Removed
    - Triggered by "Diff" button in `PluginDashboard`
    - _Requirements: 13.2_

  - [x] 11.6 Add semantic search bar to `MappingBuilder`
    - Add a search input that calls `GET /api/catalog/search?q=...`
    - Display top-K matching catalog entries as suggestions
    - _Requirements: (semantic search in MappingBuilder)_

- [x] 12. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Property tests reference design document properties by number for traceability
- Engine plugins (task 8) may use stub/mock backends where native SDK integration is not yet available — the `CatalogPlugin` trait boundary keeps them swappable
- The `ocsf-vector` integration (task 6) requires `EmbeddingGenerator` to be configured at startup; a `MockEmbeddingGenerator` is used in tests
