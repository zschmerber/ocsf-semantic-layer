# Requirements: OCSF ETL Engine

## Introduction

The OCSF ETL Engine orchestrates the full pipeline from raw log configuration through OCSF transformation to warehouse-ready output. The core flow is: **raw log → OCSF transformation (Go WASM plugin via Tangent) → simultaneous semantic layer generation → warehouse artifact generation** — where physical OCSF tables and semantic OCSF views coexist in the data warehouse.

This workspace composes with existing crates rather than forking them:
- `ocsf_delta_tangent::pipeline-codegen` for Go WASM plugin generation
- `ocsf_delta_tangent::delta-reader` for reading Delta table metadata
- `ocsf-warehouse` for warehouse artifact generation (dbt, views, DDL)
- `ocsf-catalog` for semantic catalog access
- `ocsf-semantic` for `SemanticModel`, `SemanticEntity`, `SemanticMetric`
- `ocsf-core` for `CompiledSchema`, `CompiledClass`, `CompiledAttribute`

## Glossary

- **ETL_Engine**: The OCSF ETL Engine system comprising five crates: `etl-core`, `codegen-adapter`, `semantic-bridge`, `warehouse-gen`, and `etl-server`.
- **MappingSpec**: A data type representing one source-to-OCSF field mapping with confidence and optional transformation.
- **EtlJob**: A data type representing a complete ETL job specification including mappings, source config, and catalog reference.
- **EtlJobStatus**: An enum representing the lifecycle states of an ETL job: Pending, Generating, Compiling, Testing, Running, Completed, Failed.
- **CodegenAdapter**: The crate that converts an `EtlJob` into a `PipelineGenRequest` and delegates to `pipeline-codegen`.
- **SemanticBridge**: A trait abstracting how `EtlJob` instances are resolved from catalog entries or manual input.
- **CatalogBridge**: An implementation of `SemanticBridge` that reads `CatalogEntry` from the OCSF 2 catalog and converts `CatalogFieldLineage` records into `MappingSpec` with default confidence of 1.0.
- **WarehouseGenerator**: The crate that produces dbt models, semantic SQL views, materialized views, and DDL from an `EtlJob` and its generated `SemanticModel`.
- **CompiledSchema**: The OCSF schema loaded from `OCSF_SCHEMA_PATH`, providing class definitions, attribute types, and category UIDs.
- **PipelineGenRequest**: The input type for `pipeline-codegen::generate_pipeline()`, containing plugin name, class UID, field mappings, and sink configuration.
- **Tangent**: The external sidecar process that compiles Go WASM plugins, runs tests, and executes pipelines.

## Requirements

### Requirement 1: Mapping Spec and Job Types

**User Story:** As a security data engineer, I want well-defined types for field mappings and ETL jobs, so that I can specify how raw log fields map to OCSF fields with confidence scores and transformations.

#### Acceptance Criteria

1. THE ETL_Engine SHALL define a `MappingSpec` type with fields: `source_field: String`, `target_ocsf_path: String`, `transformation: Option<Transform>`, and `confidence: f32`
2. THE ETL_Engine SHALL define a `Transform` enum with variants: `Lower`, `Upper`, `Trim`, `Cast(SqlType)` where `SqlType` is an enum with variants: `Integer`, `BigInt`, `Float`, `Double`, `Boolean`, `String`, `Timestamp`, `Date`
3. WHEN a `MappingSpec` is created, THE ETL_Engine SHALL enforce that `confidence` is in the range [0.0, 1.0]
4. THE ETL_Engine SHALL define an `EtlJob` type with fields: `job_id: Uuid`, `plugin_name: String`, `ocsf_class_uid: u32`, `ocsf_version: String`, `mappings: Vec<MappingSpec>`, `source_config: SourceConfig`, `delta_table_uri: String`, `s3_config: Option<S3Config>`, and `catalog_ref: Option<CatalogRef>`
5. THE ETL_Engine SHALL define a `SourceConfig` enum with variants: `File { path }`, `Tcp { bind_address }`, `Sqs { queue_url, region }`, `Kafka { brokers, topic }`
6. THE ETL_Engine SHALL define an `EtlJobStatus` enum with variants: `Pending`, `Generating`, `Compiling`, `Testing`, `Running`, `Completed`, `Failed(String)`
7. THE ETL_Engine SHALL define a `CatalogRef` type with fields: `entry_id: u64`, `catalog_version: u64`, `catalog_url: Option<String>`
8. WHEN an `EtlJob` is serialized to JSON and deserialized back, THE ETL_Engine SHALL produce an equivalent `EtlJob` (round-trip property)
9. WHEN a `MappingSpec` is serialized to JSON and deserialized back, THE ETL_Engine SHALL produce an equivalent `MappingSpec` (round-trip property)

### Requirement 2: Semantic Bridge

**User Story:** As a security data engineer, I want to resolve ETL jobs from the semantic catalog or from manual input, so that I can drive ETL pipelines from catalog entries or standalone configurations.

#### Acceptance Criteria

1. THE ETL_Engine SHALL define a `SemanticBridge` trait with methods: `resolve_job(entry_id: u64) -> Result<EtlJob>` and `list_entries() -> Result<Vec<CatalogEntrySummary>>`
2. THE ETL_Engine SHALL provide a `CatalogBridge` implementation that reads a `CatalogEntry` from the OCSF 2 `SemanticCatalog` via HTTP
3. WHEN the CatalogBridge converts `CatalogFieldLineage` records into `MappingSpec` instances, THE CatalogBridge SHALL set `confidence` to 1.0 for all catalog-sourced mappings
4. WHEN the CatalogBridge resolves a job, THE CatalogBridge SHALL infer `ocsf_class_uid` from the first element of `source_event_classes`
5. WHEN the CatalogBridge resolves a job, THE CatalogBridge SHALL populate `catalog_ref` with the entry ID and catalog version from the `CatalogEntry`
6. THE ETL_Engine SHALL provide a `ManualBridge` implementation that accepts a user-supplied `EtlJob` directly without catalog dependency
7. WHEN the ManualBridge `list_entries` method is called, THE ManualBridge SHALL return an empty list

### Requirement 3: Codegen Adapter

**User Story:** As a security data engineer, I want the ETL engine to generate Go WASM plugins by adapting existing pipeline-codegen, so that I can reuse Tangent's proven codegen without duplicating code.

#### Acceptance Criteria

1. THE CodegenAdapter SHALL convert an `EtlJob` into a `PipelineGenRequest` by mapping `MappingSpec` instances to `FieldLineageRecord` instances
2. WHEN converting mappings, THE CodegenAdapter SHALL apply confidence-gated logic: mappings with `confidence` below a configurable threshold (default 0.5) SHALL be excluded from the generated plugin
3. WHEN a `MappingSpec` has a `transformation` set, THE CodegenAdapter SHALL include the transformation expression in the `FieldLineageRecord`
4. THE CodegenAdapter SHALL delegate to `pipeline-codegen::generate_pipeline()` for actual Go code, `go.mod`, `tangent.yaml`, and test fixture generation
5. THE CodegenAdapter SHALL require a `CompiledSchema` reference to resolve OCSF class names, attribute types, and category UIDs
6. WHEN the `CompiledSchema` is not available at the configured `OCSF_SCHEMA_PATH`, THE CodegenAdapter SHALL return a descriptive error
7. THE CodegenAdapter SHALL generate a `SemanticModel` snapshot from the `EtlJob` at codegen time, including entities, metrics, computed fields, query templates, and threat intel joins
8. THE CodegenAdapter SHALL serialize the `SemanticModel` as YAML and embed it in the `tangent.yaml` sink block under `ocsf_semantic_model`
9. THE CodegenAdapter SHALL write the `SemanticModel` as a standalone `semantic-model.yaml` file in the plugin output directory
10. WHEN the same `EtlJob` and `CompiledSchema` are provided twice, THE CodegenAdapter SHALL produce identical output (idempotency property)

### Requirement 4: Warehouse Generation

**User Story:** As a data architect, I want the ETL engine to generate warehouse artifacts alongside the ETL pipeline, so that physical OCSF tables and semantic views coexist in the data warehouse.

#### Acceptance Criteria

1. THE WarehouseGenerator SHALL accept an `EtlJob` and a generated `SemanticModel` as input
2. THE WarehouseGenerator SHALL generate a physical OCSF table DDL using `ocsf-warehouse::TableGenerator` for the event class specified by `ocsf_class_uid`
3. THE WarehouseGenerator SHALL generate semantic SQL views using `ocsf-warehouse::ViewGenerator` from the `SemanticModel` entities
4. THE WarehouseGenerator SHALL generate dbt semantic layer artifacts using `ocsf-warehouse::DBTGenerator` from the `SemanticModel`
5. THE WarehouseGenerator SHALL generate materialized view definitions using `ocsf-warehouse::MaterializedViewGenerator` for observable hot-path analytics
6. THE WarehouseGenerator SHALL support multiple `WarehouseDialect` values: Snowflake, BigQuery, Databricks, Postgres
7. THE WarehouseGenerator SHALL produce a `WarehouseArtifacts` struct containing: table DDL, semantic view SQL, dbt artifacts, and materialized view SQL
8. WHEN the `EtlJob` has no mappings, THE WarehouseGenerator SHALL return an error indicating that at least one mapping is required

### Requirement 5: Simultaneous Generation

**User Story:** As a security data engineer, I want the ETL engine to generate the OCSF plugin, semantic model, and warehouse artifacts in a single operation, so that the full pipeline from raw log to warehouse-ready structure is produced atomically.

#### Acceptance Criteria

1. THE ETL_Engine SHALL provide a `generate_all` function that accepts an `EtlJob`, `CompiledSchema`, and `WarehouseDialect`, and produces all artifacts in one call
2. WHEN `generate_all` is called, THE ETL_Engine SHALL produce: Go WASM plugin files (via CodegenAdapter), a `SemanticModel` snapshot, and warehouse artifacts (via WarehouseGenerator)
3. WHEN `generate_all` is called, THE ETL_Engine SHALL write all output files to a single output directory organized by artifact type
4. THE ETL_Engine SHALL return a `GenerationResult` struct containing: codegen output paths, semantic model YAML, and warehouse artifact paths
5. IF the `EtlJob` contains zero mappings, THEN THE ETL_Engine SHALL return an error before generating any artifacts

### Requirement 6: ETL Server

**User Story:** As a security data engineer, I want an HTTP server to manage ETL job lifecycle, so that I can submit, generate, compile, test, and run ETL pipelines through a REST API.

#### Acceptance Criteria

1. THE ETL_Engine SHALL provide an Axum HTTP server with endpoints: `POST /api/jobs` (submit job), `GET /api/jobs` (list jobs), `GET /api/jobs/:id` (get job detail), `POST /api/jobs/:id/generate` (run generation), `POST /api/jobs/:id/compile` (compile plugin), `POST /api/jobs/:id/test` (test plugin), `POST /api/jobs/:id/run` (start pipeline), `POST /api/jobs/:id/stop` (stop pipeline)
2. THE ETL_Engine SHALL provide `GET /api/catalog/entries` to proxy catalog entry listing via the configured `SemanticBridge`
3. THE ETL_Engine SHALL provide `POST /api/jobs/from-catalog/:entry_id` to resolve a catalog entry into an `EtlJob` and submit it
4. THE ETL_Engine SHALL provide `GET /api/jobs/:id/logs` as an SSE endpoint streaming Tangent process stdout and stderr
5. THE ETL_Engine SHALL provide `GET /api/health` returning server status and Tangent binary availability
6. THE ETL_Engine SHALL store job state in memory with optional file persistence via `jobs.json`
7. WHEN a job transitions through lifecycle states, THE ETL_Engine SHALL follow the state machine: Pending → Generating → Compiling → Testing → Running → Completed, with Failed reachable from any active state
8. THE ETL_Engine SHALL support CORS for local development
9. WHEN the `POST /api/jobs/:id/generate` endpoint is called, THE ETL_Engine SHALL invoke `generate_all` to produce OCSF plugin, semantic model, and warehouse artifacts simultaneously
10. THE ETL_Engine SHALL load `CompiledSchema` from the path specified by `OCSF_SCHEMA_PATH` environment variable at startup

### Requirement 7: Delta Reading

**User Story:** As a security data engineer, I want to read semantic metadata back from Delta tables, so that I can verify what was embedded at write time and detect drift from the catalog.

#### Acceptance Criteria

1. THE ETL_Engine SHALL use `ocsf_delta_tangent::delta-reader` directly for reading Delta table metadata, without creating a separate crate
2. WHEN reading a Delta table, THE ETL_Engine SHALL expose: `read_semantic_model(table_uri) -> Result<SemanticModel>`, `read_catalog_ref(table_uri) -> Result<CatalogRef>`, and `read_field_lineage(table_uri) -> Result<Vec<MappingSpec>>`
3. THE ETL_Engine SHALL support both local Delta tables and S3/MinIO-backed tables

### Requirement 8: Workspace Structure

**User Story:** As a developer, I want a lean workspace that composes with existing crates, so that there is no code duplication and each crate has a clear responsibility.

#### Acceptance Criteria

1. THE ETL_Engine workspace SHALL contain exactly five crates: `etl-core`, `codegen-adapter`, `semantic-bridge`, `warehouse-gen`, and `etl-server`
2. THE `etl-core` crate SHALL have zero dependencies on Tangent, Delta, or OCSF 2 crates — it contains only pure data types with `serde` serialization
3. THE `codegen-adapter` crate SHALL depend on `etl-core`, `ocsf_delta_tangent::pipeline-codegen`, `ocsf-core`, and `ocsf-semantic`
4. THE `semantic-bridge` crate SHALL depend on `etl-core` and optionally `reqwest` for HTTP catalog access
5. THE `warehouse-gen` crate SHALL depend on `etl-core`, `ocsf-warehouse`, `ocsf-semantic`, and `ocsf-core`
6. THE `etl-server` crate SHALL depend on all other crates plus `axum`, `tokio`, and `tower-http`

### Requirement 9: Testing and Correctness

**User Story:** As a developer, I want comprehensive property-based tests, so that I can verify correctness invariants hold across all valid inputs.

#### Acceptance Criteria

1. THE ETL_Engine SHALL include proptest-based round-trip tests for `MappingSpec` JSON serialization and deserialization
2. THE ETL_Engine SHALL include proptest-based round-trip tests for `EtlJob` JSON serialization and deserialization
3. THE ETL_Engine SHALL include a proptest verifying that `MappingSpec.confidence` is always in the range [0.0, 1.0] after construction
4. THE ETL_Engine SHALL include a proptest verifying that codegen is idempotent: the same `EtlJob` and `CompiledSchema` produce identical output on repeated calls
5. THE ETL_Engine SHALL include a proptest verifying that every `EtlJob` has at least one mapping (non-empty mappings invariant)
6. THE ETL_Engine SHALL include a proptest verifying that `CatalogBridge` always sets confidence to 1.0 when converting `CatalogFieldLineage` records
7. THE ETL_Engine SHALL include unit tests verifying generated Go code contains correct struct names and class UIDs for a known OCSF class (DNS Activity, uid=4003)
8. THE ETL_Engine SHALL include unit tests verifying warehouse artifacts contain both physical table DDL and semantic view SQL for the same event class
9. THE ETL_Engine SHALL include a proptest verifying confidence gating monotonicity: for any `EtlJob` and two thresholds where `t1 < t2`, the number of mappings passing threshold `t2` is less than or equal to the number passing `t1`
10. THE ETL_Engine SHALL include a proptest verifying schema path validation: for any `EtlJob` and `CompiledSchema`, every `MappingSpec.target_ocsf_path` in the job must reference a valid path in the `CompiledSchema`
11. THE ETL_Engine SHALL include a proptest verifying semantic model generation idempotency: calling `generate_semantic_model(job, schema)` twice with the same inputs produces identical YAML output (separate from plugin idempotency in Requirement 3.10)
