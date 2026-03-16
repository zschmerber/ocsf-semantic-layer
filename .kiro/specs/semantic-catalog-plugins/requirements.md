# Requirements Document

## Introduction

The Semantic Catalog Plugins feature introduces a universal semantic catalog (`ocsf-catalog` crate) that stores the semantic meaning of OCSF log entities in a compact, fast-to-access binary sidecar file. The catalog captures entity definitions, field mappings, OCSF class associations, detection coverage, threat relevance, and lineage metadata. A plugin system enables bidirectional synchronization of semantic annotations with multiple storage/query engines (Apache Iceberg, Delta Lake, ClickHouse, DuckDB), using each engine's native metadata facilities. The existing editor UI serves as the primary interface for managing catalog entries, browsing plugin sync status, and triggering sync operations.

## Glossary

- **Catalog**: The in-process semantic metadata store backed by a compact binary sidecar file, containing all semantic entity definitions, field mappings, detection coverage, and lineage for a given OCSF deployment.
- **Sidecar_File**: The on-disk binary file that serves as the canonical source of truth for the Catalog. It is memory-mappable for sub-millisecond reads.
- **Catalog_Entry**: A single record in the Catalog representing one semantic entity with its attributes, OCSF mappings, detection coverage, relationships, and lineage metadata.
- **Catalog_API**: The Rust trait (`SemanticCatalog`) that defines read/write operations on the Catalog.
- **Plugin**: An implementation of the `CatalogPlugin` trait that synchronizes Catalog_Entry records to and from a specific storage engine's native metadata system.
- **Sync_Operation**: A bidirectional transfer of semantic annotations between the Sidecar_File and a Plugin's target engine.
- **Catalog_Version**: A monotonically increasing version number embedded in the Sidecar_File header, incremented on each write.
- **OCSF_Schema_Version**: The version string of the OCSF schema (e.g., "1.3.0", "1.4.0") that a Catalog_Entry is associated with.
- **Iceberg_Plugin**: A Plugin that syncs semantic annotations into Apache Iceberg table properties and manifest metadata.
- **Delta_Plugin**: A Plugin that syncs semantic annotations into Delta Lake transaction log metadata (commitInfo and table properties).
- **ClickHouse_Plugin**: A Plugin that syncs semantic annotations into ClickHouse table comments and system dictionaries.
- **DuckDB_Plugin**: A Plugin that syncs semantic annotations into DuckDB catalog schema metadata tables.
- **Diff**: A computed set of changes between two states of the Catalog or between the Catalog and a Plugin's metadata, used to drive incremental Sync_Operations.
- **Editor_API**: The HTTP API layer in `ocsf-editor` that exposes Catalog and Plugin operations to the editor UI.
- **Editor_UI**: The existing React-based editor application that provides visual management of semantic models, index data, and now catalog entries and plugin sync.

## Requirements

### Requirement 1: Catalog Entry Data Model

**User Story:** As a security data engineer, I want a structured representation of semantic metadata for each OCSF entity, so that I can store and retrieve entity definitions, field mappings, detection coverage, and lineage in a single record.

#### Acceptance Criteria

1. THE Catalog_Entry SHALL contain the entity name, caption, description, source OCSF event class UIDs, semantic attributes with OCSF mappings, entity relationships, observable type IDs, and detection coverage metadata.
2. THE Catalog_Entry SHALL contain lineage metadata including source system identifiers, transformation descriptions, and field-level lineage records.
3. WHEN a Catalog_Entry is created, THE Catalog_API SHALL assign a unique identifier to the Catalog_Entry.
4. THE Catalog_Entry SHALL reference exactly one OCSF_Schema_Version.
5. THE Catalog_Entry SHALL be serializable to and deserializable from a binary representation using serde.

### Requirement 2: Sidecar File Format

**User Story:** As a security data engineer, I want a compact binary sidecar file that stores the full semantic catalog, so that I can achieve sub-millisecond reads through memory mapping.

#### Acceptance Criteria

1. THE Sidecar_File SHALL contain a fixed-size header with a magic number, format version, Catalog_Version, OCSF_Schema_Version reference, entry count, and checksum of the data section.
2. THE Sidecar_File SHALL store Catalog_Entry records in a binary-encoded data section following the header.
3. WHEN the Catalog_API opens a Sidecar_File, THE Catalog_API SHALL validate the magic number and format version before reading entries.
4. IF the Sidecar_File header checksum does not match the data section, THEN THE Catalog_API SHALL return a corruption error.
5. THE Sidecar_File SHALL support memory-mapped reads so that the Catalog_API can access entries without loading the entire file into heap memory.
6. THE Catalog_API SHALL serialize Catalog_Entry records to the Sidecar_File binary format and deserialize them back, producing equivalent Catalog_Entry values (round-trip property).

### Requirement 3: Catalog API (SemanticCatalog Trait)

**User Story:** As a developer, I want a Rust trait that defines read/write operations on the semantic catalog, so that I can interact with the catalog through a consistent interface regardless of the underlying storage.

#### Acceptance Criteria

1. THE Catalog_API SHALL provide an async method to add a new Catalog_Entry and return its unique identifier.
2. THE Catalog_API SHALL provide an async method to retrieve a Catalog_Entry by its unique identifier.
3. THE Catalog_API SHALL provide an async method to retrieve a Catalog_Entry by entity name and OCSF_Schema_Version.
4. THE Catalog_API SHALL provide an async method to update an existing Catalog_Entry, incrementing the Catalog_Version.
5. THE Catalog_API SHALL provide an async method to remove a Catalog_Entry by its unique identifier.
6. THE Catalog_API SHALL provide an async method to list all Catalog_Entry records, optionally filtered by OCSF_Schema_Version.
7. WHEN a write operation completes, THE Catalog_API SHALL increment the Catalog_Version in the Sidecar_File header.
8. IF a Catalog_Entry with the same entity name and OCSF_Schema_Version already exists during an add operation, THEN THE Catalog_API SHALL return a duplicate entry error.

### Requirement 4: Catalog Version Support

**User Story:** As a security data engineer, I want the catalog to support multiple OCSF schema versions simultaneously, so that I can manage entities across schema upgrades without data loss.

#### Acceptance Criteria

1. THE Catalog SHALL store Catalog_Entry records for multiple OCSF_Schema_Version values in the same Sidecar_File.
2. WHEN listing Catalog_Entry records with an OCSF_Schema_Version filter, THE Catalog_API SHALL return only entries matching that version.
3. WHEN listing Catalog_Entry records without a version filter, THE Catalog_API SHALL return entries across all OCSF_Schema_Version values.
4. THE Catalog_API SHALL provide a method to list all distinct OCSF_Schema_Version values present in the Catalog.

### Requirement 5: Plugin Trait (CatalogPlugin)

**User Story:** As a developer, I want a common trait that all engine plugins implement, so that I can add new engine targets without modifying the core catalog logic.

#### Acceptance Criteria

1. THE Plugin trait SHALL define an async method to push a set of Catalog_Entry records from the Catalog to the engine's native metadata system.
2. THE Plugin trait SHALL define an async method to pull semantic annotations from the engine's native metadata system and return them as Catalog_Entry records.
3. THE Plugin trait SHALL define a method that returns the engine name as a static string (e.g., "iceberg", "delta", "clickhouse", "duckdb").
4. THE Plugin trait SHALL define an async method to compute a Diff between the current Catalog state and the engine's metadata state.
5. WHEN a Plugin push operation encounters an engine-specific error, THE Plugin SHALL return a typed error that includes the engine name and a description of the failure.

### Requirement 6: Bidirectional Sync

**User Story:** As a security data engineer, I want plugins to both push semantic annotations to engines and pull discovered metadata back, so that I can keep the catalog and engine metadata in sync.

#### Acceptance Criteria

1. WHEN a push Sync_Operation is performed, THE Plugin SHALL write the Catalog_Entry semantic annotations into the target engine's native metadata format.
2. WHEN a pull Sync_Operation is performed, THE Plugin SHALL read semantic annotations from the engine's native metadata and convert them into Catalog_Entry records.
3. WHEN a Diff is computed, THE Catalog SHALL identify entries that are added, modified, or removed relative to the engine's current metadata state.
4. THE Catalog SHALL provide an async method to execute a full sync cycle: compute Diff, push local changes to the engine, and pull remote changes back to the Catalog.
5. IF a conflict arises during sync where both the Catalog and the engine have modified the same entry, THEN THE Catalog SHALL use a last-writer-wins strategy based on the Catalog_Version.

### Requirement 7: Iceberg Plugin

**User Story:** As a data engineer using Apache Iceberg, I want semantic annotations stored in Iceberg table properties, so that I can access OCSF semantic metadata alongside my Iceberg tables.

#### Acceptance Criteria

1. WHEN pushing Catalog_Entry records, THE Iceberg_Plugin SHALL write semantic annotations as Iceberg table properties with a `ocsf.semantic.` key prefix.
2. WHEN pulling metadata, THE Iceberg_Plugin SHALL read Iceberg table properties with the `ocsf.semantic.` prefix and reconstruct Catalog_Entry records.
3. THE Iceberg_Plugin SHALL serialize Catalog_Entry field mappings, detection coverage, and lineage as JSON-encoded values within table properties.

### Requirement 8: Delta Lake Plugin

**User Story:** As a data engineer using Delta Lake, I want semantic annotations stored in Delta transaction log metadata, so that I can version-track OCSF semantic metadata alongside my Delta tables.

#### Acceptance Criteria

1. WHEN pushing Catalog_Entry records, THE Delta_Plugin SHALL write semantic annotations as Delta Lake table properties with a `ocsf.semantic.` key prefix.
2. WHEN pulling metadata, THE Delta_Plugin SHALL read Delta Lake table properties with the `ocsf.semantic.` prefix and reconstruct Catalog_Entry records.
3. THE Delta_Plugin SHALL serialize Catalog_Entry field mappings, detection coverage, and lineage as JSON-encoded values within table properties.

### Requirement 9: ClickHouse Plugin

**User Story:** As a data engineer using ClickHouse, I want semantic annotations stored in ClickHouse table comments and a dedicated dictionary table, so that I can query OCSF semantic metadata using SQL.

#### Acceptance Criteria

1. WHEN pushing Catalog_Entry records, THE ClickHouse_Plugin SHALL write a JSON-encoded summary of the Catalog_Entry as the table comment for the corresponding ClickHouse table.
2. WHEN pushing Catalog_Entry records, THE ClickHouse_Plugin SHALL insert detailed semantic metadata (attributes, mappings, detection coverage) into a dedicated `ocsf_semantic_catalog` dictionary table.
3. WHEN pulling metadata, THE ClickHouse_Plugin SHALL read the `ocsf_semantic_catalog` dictionary table and reconstruct Catalog_Entry records.

### Requirement 10: DuckDB Plugin

**User Story:** As a data engineer using DuckDB, I want semantic annotations stored in DuckDB catalog metadata tables, so that I can query OCSF semantic metadata alongside my analytical queries.

#### Acceptance Criteria

1. WHEN pushing Catalog_Entry records, THE DuckDB_Plugin SHALL create or update rows in a `ocsf_semantic_catalog` table within the DuckDB database.
2. WHEN pulling metadata, THE DuckDB_Plugin SHALL read the `ocsf_semantic_catalog` table and reconstruct Catalog_Entry records.
3. THE DuckDB_Plugin SHALL store Catalog_Entry field mappings, detection coverage, and lineage as JSON columns in the metadata table.

### Requirement 11: Editor API Integration

**User Story:** As a security data engineer, I want to manage the semantic catalog through the existing editor HTTP API, so that the editor UI can display and manipulate catalog entries and trigger plugin sync operations.

#### Acceptance Criteria

1. THE Editor_API SHALL expose REST endpoints under `/api/catalog/` for creating, reading, updating, deleting, and listing Catalog_Entry records.
2. THE Editor_API SHALL expose a REST endpoint to list all registered Plugin engines and their connection status.
3. THE Editor_API SHALL expose REST endpoints to trigger push, pull, and full-sync operations for a specified Plugin.
4. THE Editor_API SHALL expose a REST endpoint to compute and return a Diff between the Catalog and a specified Plugin's metadata.
5. WHEN a Catalog_Entry is created or updated through the Editor_API, THE Editor_API SHALL return the updated Catalog_Entry with its assigned identifier and current Catalog_Version.

### Requirement 12: Editor UI Catalog Management

**User Story:** As a security data engineer, I want to browse, create, edit, and delete catalog entries in the editor UI, so that I can visually manage the semantic catalog without using the CLI.

#### Acceptance Criteria

1. THE Editor_UI SHALL display a catalog browser view that lists all Catalog_Entry records with name, OCSF_Schema_Version, source event class count, attribute count, and detection coverage summary.
2. THE Editor_UI SHALL provide a form for creating and editing Catalog_Entry records, pre-populated from existing SemanticEntity definitions in the semantic model when available.
3. THE Editor_UI SHALL provide a detail view for a selected Catalog_Entry showing all attributes, OCSF mappings, relationships, detection coverage, and lineage metadata.
4. WHEN the user deletes a Catalog_Entry in the Editor_UI, THE Editor_UI SHALL prompt for confirmation before executing the deletion.
5. THE Editor_UI SHALL allow filtering the catalog browser by OCSF_Schema_Version.

### Requirement 13: Editor UI Plugin Sync Dashboard

**User Story:** As a security data engineer, I want a plugin sync dashboard in the editor UI, so that I can see which engines are connected, view sync diffs, and trigger sync operations visually.

#### Acceptance Criteria

1. THE Editor_UI SHALL display a plugin dashboard listing all registered Plugin engines with their engine name and connection status.
2. THE Editor_UI SHALL provide a button to trigger a Diff computation for a selected Plugin and display the resulting added, modified, and removed entries.
3. THE Editor_UI SHALL provide buttons to trigger push, pull, and full-sync operations for a selected Plugin.
4. WHEN a Sync_Operation completes, THE Editor_UI SHALL display a summary of the changes applied (entries pushed, pulled, conflicts resolved).
5. IF a Sync_Operation fails, THEN THE Editor_UI SHALL display the error message returned by the Editor_API.

### Requirement 14: Error Handling

**User Story:** As a developer, I want clear, typed errors for all catalog and plugin operations, so that I can diagnose and handle failures programmatically.

#### Acceptance Criteria

1. THE Catalog_API SHALL define a `CatalogError` enum with variants for IO errors, serialization errors, corruption errors, duplicate entry errors, not-found errors, and plugin errors.
2. WHEN a Plugin operation fails, THE `CatalogError` SHALL include a plugin variant that wraps the engine name and the underlying error description.
3. THE Catalog_API SHALL use `Result<T, CatalogError>` as the return type for all fallible operations.
