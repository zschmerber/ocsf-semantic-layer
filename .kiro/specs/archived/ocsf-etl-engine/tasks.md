# Tasks: OCSF ETL Engine

## Task 1: Workspace scaffold

- [x] 1.1 Create `ocsf-etl-engine/Cargo.toml` workspace manifest listing 5 crates: `etl-core`, `codegen-adapter`, `semantic-bridge`, `warehouse-gen`, `etl-server`
- [x] 1.2 Create `ocsf-etl-engine/crates/etl-core/Cargo.toml` with deps: `serde`, `serde_json`, `uuid`, `thiserror`; dev-deps: `proptest`
- [x] 1.3 Create `ocsf-etl-engine/crates/codegen-adapter/Cargo.toml` with deps: `etl-core`, `ocsf_delta_tangent::pipeline-codegen`, `ocsf-core`, `ocsf-semantic`, `serde_yaml`, `anyhow`
- [x] 1.4 Create `ocsf-etl-engine/crates/semantic-bridge/Cargo.toml` with deps: `etl-core`, `ocsf-catalog` (models only), `reqwest`, `async-trait`, `anyhow`; dev-deps: `proptest`, `tokio` (test)
- [x] 1.5 Create `ocsf-etl-engine/crates/warehouse-gen/Cargo.toml` with deps: `etl-core`, `ocsf-warehouse`, `ocsf-semantic`, `ocsf-core`, `anyhow`; dev-deps: `proptest`
- [x] 1.6 Create `ocsf-etl-engine/crates/etl-server/Cargo.toml` with deps: all crates + `axum`, `tokio`, `tower-http`, `uuid`, `serde_json`

## Task 2: `etl-core` — pure data types

- [x] 2.1 Implement `MappingSpec` with `source_field`, `target_ocsf_path`, `transformation: Option<Transform>`, `confidence: f32`; constructor clamps confidence to [0.0, 1.0]
- [x] 2.2 Implement `Transform` enum: `Lower`, `Upper`, `Trim`, `Cast(SqlType)` with Serialize/Deserialize; implement `SqlType` enum: `Integer`, `BigInt`, `Float`, `Double`, `Boolean`, `String`, `Timestamp`, `Date` with Serialize/Deserialize
- [x] 2.3 Implement `SourceConfig` enum: `File { path }`, `Tcp { bind_address }`, `Sqs { queue_url, region }`, `Kafka { brokers, topic }` with Serialize/Deserialize
- [x] 2.4 Implement `S3Config` struct with Serialize/Deserialize
- [x] 2.5 Implement `CatalogRef` struct: `entry_id: u64`, `catalog_version: u64`, `catalog_url: Option<String>` with Serialize/Deserialize
- [x] 2.6 Implement `EtlJob` struct with all fields per Requirement 1.4, with Serialize/Deserialize
- [x] 2.7 Implement `EtlJobStatus` enum: `Pending`, `Generating`, `Compiling`, `Testing`, `Running`, `Completed`, `Failed(String)` with Serialize/Deserialize
- [x] 2.8 Implement `EtlJobStatus` state machine: `fn transition(&self, next: &EtlJobStatus) -> Result<EtlJobStatus>` enforcing valid transitions (Pending→Generating→Compiling→Testing→Running→Completed, Failed from any active state)

## Task 3: `etl-core` — property tests

- [x] 3.1 [PBT: Property 1] Proptest: for any valid `MappingSpec`, JSON round-trip produces an equal value
- [x] 3.2 [PBT: Property 1] Proptest: for any valid `EtlJob`, JSON round-trip produces an equal value
- [x] 3.3 [PBT: Property 2] Proptest: for any `f32` confidence input, `MappingSpec::new` produces confidence in [0.0, 1.0]
- [x] 3.4 [PBT: Property 9] Proptest: for any random sequence of `EtlJobStatus` transitions, only valid transitions succeed and invalid transitions return errors

## Task 4: `semantic-bridge` — trait and implementations

- [x] 4.1 Define `SemanticBridge` trait with `resolve_job(entry_id: u64) -> Result<EtlJob>` and `list_entries() -> Result<Vec<CatalogEntrySummary>>`
- [x] 4.2 Define `CatalogEntrySummary` struct: `id: u64`, `entity_name: String`, `ocsf_version: String`, `catalog_version: u64`
- [x] 4.3 Implement `ManualBridge`: `resolve_job` returns error, `list_entries` returns empty vec
- [x] 4.4 Implement `CatalogBridge` with `reqwest` HTTP client and `base_url`
- [x] 4.5 Implement `CatalogBridge::resolve_job`: GET `/api/catalog/entries/{id}`, convert `CatalogFieldLineage` → `MappingSpec` (confidence=1.0), infer `ocsf_class_uid` from `source_event_classes[0]`, populate `catalog_ref`
- [x] 4.6 Implement `CatalogBridge::list_entries`: GET `/api/catalog/entries`, map to `Vec<CatalogEntrySummary>`
- [x] 4.7 Implement `convert_field_lineage(lineage: &[CatalogFieldLineage]) -> Vec<MappingSpec>` helper that sets confidence=1.0 and parses transformation strings

## Task 5: `semantic-bridge` — tests

- [x] 5.1 Unit test: `ManualBridge::list_entries` returns empty vec
- [x] 5.2 [PBT: Property 3] Proptest: for any `Vec<CatalogFieldLineage>`, `convert_field_lineage` produces all `MappingSpec` with confidence == 1.0
- [x] 5.3 [PBT: Property 4] Proptest: for any `CatalogEntry` with non-empty `source_event_classes`, the resulting `EtlJob` has `ocsf_class_uid == source_event_classes[0]` and `catalog_ref` matching entry ID and version

## Task 6: `codegen-adapter` — core adapter

- [x] 6.1 Implement `CodegenAdapter` struct with `confidence_threshold: f32` (default 0.5)
- [x] 6.2 Implement `to_field_lineage_record(mapping: &MappingSpec) -> FieldLineageRecord` conversion, including transformation expression mapping
- [x] 6.3 Implement `CodegenAdapter::generate(schema, job) -> Result<AdapterOutput>`: filter by confidence, convert mappings, build `PipelineGenRequest`, delegate to `pipeline_codegen::generate_pipeline()`
- [x] 6.4 Implement `generate_semantic_model(schema, job) -> Result<SemanticModel>`: derive entities, metrics, computed fields, query templates, threat intel joins from the EtlJob and CompiledSchema
- [x] 6.5 Implement schema loading: load `CompiledSchema` from `OCSF_SCHEMA_PATH` env var, return descriptive error if missing

## Task 7: `codegen-adapter` — tests

- [x] 7.1 Unit test: DNS Activity (uid=4003) generates correct struct names and class UIDs in Go output
- [x] 7.2 Unit test: generated `tangent.yaml` contains `ocsf_semantic_model` multiline block
- [x] 7.3 [PBT: Property 5] Proptest: for any `EtlJob` with mixed-confidence mappings, only mappings at or above threshold appear in `PipelineGenRequest`, and each preserves source_field, target_ocsf_path, and transformation
- [x] 7.4 [PBT: Property 6] Proptest: for any `EtlJob` and `CompiledSchema`, calling `generate` twice produces identical output
- [x] 7.5 [PBT: Property 7] Proptest: for any `EtlJob` with at least one mapping, `AdapterOutput.semantic_model_yaml` is valid YAML that deserializes to a `SemanticModel` with at least one entity
- [x] 7.6 [PBT: Property 5] Proptest: for any `EtlJob` and two thresholds `t1 < t2`, the number of mappings passing `t2` is ≤ the number passing `t1` (confidence gating monotonicity)
- [x] 7.7 [PBT: Property 11] Proptest: for any `EtlJob` and `CompiledSchema`, every `MappingSpec.target_ocsf_path` references a valid path in the `CompiledSchema` (schema path validation)
- [x] 7.8 [PBT: Property 12] Proptest: calling `generate_semantic_model(job, schema)` twice with the same inputs produces identical YAML output (semantic model idempotency, separate from plugin idempotency P6)

## Task 8: `warehouse-gen` — warehouse artifact generation

- [x] 8.1 Implement `WarehouseGen` struct with `dialect: WarehouseDialect`
- [x] 8.2 Implement `WarehouseGen::generate(schema, job, semantic_model) -> Result<WarehouseArtifacts>`: validate non-empty mappings, generate table DDL via `TableGenerator`, semantic views via `ViewGenerator`, dbt artifacts via `DBTGenerator`, materialized views via `MaterializedViewGenerator`
- [x] 8.3 Implement `WarehouseArtifacts` struct: `table_ddl: String`, `semantic_views: String`, `dbt_artifacts: DBTArtifacts`, `materialized_views: String`

## Task 9: `warehouse-gen` — tests

- [x] 9.1 Unit test: warehouse artifacts for DNS Activity contain both `CREATE TABLE` and `CREATE VIEW` statements
- [x] 9.2 [PBT: Property 8] Proptest: for any valid `EtlJob`, `SemanticModel`, and `WarehouseDialect`, all four artifact fields in `WarehouseArtifacts` are non-empty
- [x] 9.3 [PBT: Property 10] Proptest: for any `EtlJob` with zero mappings, `WarehouseGen::generate` returns an error

## Task 10: `etl-server` — `generate_all` orchestration

- [x] 10.1 Implement `generate_all(job, schema, dialect) -> Result<GenerationResult>`: validate non-empty mappings, call `CodegenAdapter::generate`, call `WarehouseGen::generate`, write all files to output directory
- [x] 10.2 Implement `GenerationResult` struct: codegen output paths, semantic model YAML, warehouse artifact paths
- [x] 10.3 Implement file writing: organize output as `{output_dir}/{job_id}/plugin/` (Go files), `{output_dir}/{job_id}/warehouse/` (DDL, views, dbt), `{output_dir}/{job_id}/semantic-model.yaml`

## Task 11: `etl-server` — job lifecycle

- [x] 11.1 Define `AppState`: `jobs`, `bridge`, `schema`, `codegen`, `warehouse`, `tangent_bin`, `output_dir`
- [x] 11.2 Define `JobRecord`: `job`, `status`, `codegen_output`, `warehouse_output`, `log_buffer`, `process`
- [x] 11.3 Implement `POST /api/jobs` — accept `EtlJob`, store as `Pending`, return `job_id`
- [x] 11.4 Implement `GET /api/jobs` — list all jobs with status
- [x] 11.5 Implement `GET /api/jobs/:id` — return job detail + artifact file list
- [x] 11.6 Implement `POST /api/jobs/:id/generate` — call `generate_all`, write files, transition to `Compiling`
- [x] 11.7 Implement `POST /api/jobs/:id/compile` — spawn `tangent plugin compile`, stream to log buffer
- [x] 11.8 Implement `POST /api/jobs/:id/test` — spawn `tangent plugin test`, stream to log buffer
- [x] 11.9 Implement `POST /api/jobs/:id/run` — spawn `tangent run`, keep child handle
- [x] 11.10 Implement `POST /api/jobs/:id/stop` — kill child process, set status `Completed`

## Task 12: `etl-server` — catalog proxy, SSE, health

- [x] 12.1 Implement `GET /api/catalog/entries` — proxy to `bridge.list_entries()`
- [x] 12.2 Implement `POST /api/jobs/from-catalog/:entry_id` — call `bridge.resolve_job(id)`, submit as new job
- [x] 12.3 Implement `GET /api/jobs/:id/logs` — SSE endpoint streaming `log_buffer` lines
- [x] 12.4 Implement `GET /api/health` — return server status + tangent binary presence check
- [x] 12.5 Wire all routes into `create_router()`, add CORS layer
- [x] 12.6 Implement `CompiledSchema` loading from `OCSF_SCHEMA_PATH` at startup

## Task 13: Integration test

- [x] 13.1 Write an integration test that:
  1. Creates an `EtlJob` for DNS Activity (uid=4003) with 3 field mappings (confidence=1.0)
  2. Calls `generate_all` with a test `CompiledSchema` and `WarehouseDialect::Snowflake`
  3. Asserts codegen output contains `DnsActivity` struct name and `ClassUID: 4003`
  4. Asserts `tangent.yaml` contains `ocsf_semantic_model` block
  5. Asserts `semantic-model.yaml` is valid YAML with at least one entity
  6. Asserts warehouse DDL contains `CREATE TABLE`
  7. Asserts warehouse views contain `CREATE VIEW` or `CREATE OR REPLACE VIEW`
  8. Asserts dbt artifacts contain `semantic_models` YAML key
