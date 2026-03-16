# Requirements Document

## Introduction

The OCSF Semantic Index Layer is a metadata management system that tracks physical table metadata, source-to-OCSF field lineage, partition information, and query statistics. It serves as a bridge between the ETL pipeline (ocsf-warehouse) and the query engine (ocsf-semantic), enabling query optimization through partition pruning, statistics-based planning, and result caching. The index supports multi-version OCSF schemas and provides lineage tracking for data governance and debugging.

## Glossary

- **Semantic_Index**: The top-level component that coordinates all index operations and provides the public API
- **Index_Backend**: The storage backend trait abstraction for persisting index metadata (SQLite for embedded, PostgreSQL for production)
- **Table_Registry**: Component that tracks which physical tables exist for each OCSF event class
- **Source_Lineage**: Metadata tracking which raw data sources contributed to each OCSF table
- **Field_Lineage**: Metadata tracking how raw source fields map to OCSF fields including transformations
- **Partition_Metadata**: Information about table partitions including time bounds and row counts
- **Table_Statistics**: Cardinality, null rates, and value distributions for query optimization
- **Query_Cache**: Result caching layer with TTL-based and lineage-based invalidation
- **ETL_Generator**: Existing component in ocsf-warehouse that generates SQL for observable extraction
- **Semantic_Query**: Existing query definition in ocsf-semantic with entity, filters, and metrics
- **OCSF_Version**: A specific version of the OCSF schema (e.g., 1.3.0, 1.4.0)
- **Lineage_Record**: A single source-to-target mapping with metadata about the transformation

## Requirements

### Requirement 1: Table Registry

**User Story:** As a security data engineer, I want to register and track physical tables for OCSF event classes, so that the query engine knows which tables to query for each semantic entity.

#### Acceptance Criteria

1. THE Table_Registry SHALL store table metadata including table name, OCSF class UID, schema name, warehouse dialect, and creation timestamp
2. WHEN a table is registered, THE Table_Registry SHALL validate that the OCSF class UID is a valid positive integer
3. WHEN querying for tables by OCSF class, THE Table_Registry SHALL return all registered tables for that class
4. THE Table_Registry SHALL support multiple tables per OCSF class (for partitioned or sharded data)
5. WHEN a table is deregistered, THE Table_Registry SHALL mark it as inactive rather than deleting metadata
6. THE Table_Registry SHALL serialize table metadata to JSON for persistence

### Requirement 2: Source Lineage Tracking

**User Story:** As a data governance analyst, I want to track which raw data sources contributed to each OCSF table, so that I can trace data provenance and debug data quality issues.

#### Acceptance Criteria

1. THE Source_Lineage SHALL record source system name, source table or topic name, and ingestion timestamp for each OCSF table
2. WHEN a lineage record is created, THE Source_Lineage SHALL associate it with a registered table from the Table_Registry
3. THE Source_Lineage SHALL support multiple sources contributing to a single OCSF table
4. WHEN querying lineage by target table, THE Source_Lineage SHALL return all source records ordered by ingestion timestamp
5. THE Source_Lineage SHALL serialize lineage records to JSON for persistence

### Requirement 3: Field Lineage Tracking

**User Story:** As a security data engineer, I want to track how raw source fields map to OCSF fields including transformations, so that I can understand data transformations and debug mapping issues.

#### Acceptance Criteria

1. THE Field_Lineage SHALL record source field path, target OCSF field path, and optional transformation expression for each field mapping
2. WHEN a field mapping includes a transformation, THE Field_Lineage SHALL store the SQL expression used
3. THE Field_Lineage SHALL support one-to-many mappings where one source field maps to multiple OCSF fields
4. THE Field_Lineage SHALL support many-to-one mappings where multiple source fields combine into one OCSF field
5. WHEN querying field lineage by target field, THE Field_Lineage SHALL return all source fields and transformations
6. THE Field_Lineage SHALL serialize field mappings to JSON for persistence

### Requirement 4: Partition Metadata

**User Story:** As a query optimizer, I want to track partition metadata including time bounds and row counts, so that I can prune partitions and estimate query costs.

#### Acceptance Criteria

1. THE Partition_Metadata SHALL store partition key, start time, end time, row count, and size in bytes for each partition
2. WHEN a partition is updated, THE Partition_Metadata SHALL update the row count, time bounds, and last modified timestamp
3. WHEN given a time range, THE Partition_Metadata SHALL return only partitions that overlap the specified range
4. IF a partition has no data, THEN THE Partition_Metadata SHALL mark it as empty but retain the metadata record
5. THE Partition_Metadata SHALL serialize partition information to JSON for persistence

### Requirement 5: Table Statistics

**User Story:** As a query optimizer, I want to track column statistics including cardinality and null rates, so that I can make better query planning decisions.

#### Acceptance Criteria

1. THE Table_Statistics SHALL store distinct count, null count, total count, and optional min/max values for each column
2. WHEN statistics are collected for a large table, THE Table_Statistics SHALL use sampling with a configurable sample rate
3. THE Table_Statistics SHALL store a collected timestamp to indicate freshness
4. WHEN querying statistics, THE Table_Statistics SHALL return the collected timestamp alongside the statistics
5. THE Table_Statistics SHALL serialize statistics to JSON for persistence

### Requirement 6: Query Result Caching

**User Story:** As a security analyst, I want query results to be cached, so that repeated queries return faster without re-executing against the warehouse.

#### Acceptance Criteria

1. THE Query_Cache SHALL store query results with a configurable time-to-live in seconds
2. WHEN a cached query is requested and the TTL has not expired, THE Query_Cache SHALL return the cached result
3. WHEN a cached query is requested and the TTL has expired, THE Query_Cache SHALL return a cache miss
4. THE Query_Cache SHALL generate cache keys from normalized query parameters
5. IF cache storage exceeds the configured maximum entries, THEN THE Query_Cache SHALL evict least-recently-used entries
6. THE Query_Cache SHALL serialize cache entries to JSON for persistence

### Requirement 7: Storage Backend Abstraction

**User Story:** As a platform engineer, I want the index to support multiple storage backends, so that I can use SQLite for development and PostgreSQL for production.

#### Acceptance Criteria

1. THE Index_Backend SHALL define an async trait for storage operations including create, read, update, delete, and list
2. THE Index_Backend SHALL provide a SQLite implementation for embedded and single-node deployments
3. THE Index_Backend SHALL provide a PostgreSQL implementation for production deployments
4. WHEN switching between backends, THE Semantic_Index SHALL maintain the same public API
5. THE Index_Backend SHALL support connection pooling for database backends
6. THE Index_Backend implementations SHALL handle serialization and deserialization of index records

### Requirement 8: Multi-Version OCSF Support

**User Story:** As a security data engineer, I want the index to support multiple OCSF schema versions, so that I can query data across version boundaries.

#### Acceptance Criteria

1. THE Table_Registry SHALL store the OCSF schema version for each registered table
2. WHEN a table is registered, THE Semantic_Index SHALL record the OCSF version alongside the table metadata
3. THE Semantic_Index SHALL integrate with the existing versioning module in ocsf-semantic for version comparison
4. WHEN a field is renamed between OCSF versions, THE Field_Lineage SHALL support tracking the version-specific field paths
5. WHEN querying across tables with different OCSF versions, THE Semantic_Index SHALL provide version information for field translation

### Requirement 9: ETL Integration

**User Story:** As a data engineer, I want the index to capture lineage during ETL execution, so that I don't have to manually track data transformations.

#### Acceptance Criteria

1. THE Semantic_Index SHALL provide a LineageCapture struct that ETL pipelines can use to record lineage
2. THE LineageCapture SHALL accept source table, target table, and field mappings as input
3. WHEN LineageCapture is finalized, THE Semantic_Index SHALL persist source lineage and field lineage records
4. THE Semantic_Index SHALL provide a method to update partition metadata after ETL completion
5. THE Semantic_Index SHALL provide a method to trigger statistics collection for a table

### Requirement 10: Query Engine Integration

**User Story:** As a query engine, I want to use the index for query optimization, so that queries execute faster with partition pruning and statistics.

#### Acceptance Criteria

1. WHEN given a Semantic_Query with time filters, THE Semantic_Index SHALL return relevant partition information
2. THE Semantic_Index SHALL provide table statistics for query cost estimation
3. THE Semantic_Index SHALL check the Query_Cache before returning partition information
4. WHEN a query result is provided, THE Semantic_Index SHALL store it in the Query_Cache if caching is enabled
5. THE Semantic_Index SHALL provide table location and dialect information for SQL generation

### Requirement 11: Editor API Integration

**User Story:** As a user of the OCSF Editor, I want to view lineage information and index status through the API, so that I can understand data provenance and monitor index health.

#### Acceptance Criteria

1. THE Semantic_Index SHALL expose a method to query source lineage by target table name
2. THE Semantic_Index SHALL expose a method to query field lineage by target field path
3. THE Semantic_Index SHALL expose a method to retrieve partition metadata for a table
4. THE Semantic_Index SHALL expose a method to retrieve table statistics
5. THE Semantic_Index SHALL expose a method to query cache status including hit rate and entry count
6. THE Semantic_Index SHALL return all query results as serializable structs compatible with JSON

### Requirement 12: Lineage Visualization Support

**User Story:** As a frontend developer, I want the index API to return lineage data in structures suitable for graph visualization, so that I can display lineage in the editor UI.

#### Acceptance Criteria

1. THE Semantic_Index SHALL return source lineage as a list of LineageEdge structs containing source, target, and timestamp
2. THE Semantic_Index SHALL return field lineage as a list of FieldMapping structs containing source path, target path, and transformation
3. THE Semantic_Index SHALL support filtering lineage queries by time range with start and end timestamps
4. THE Semantic_Index SHALL support pagination for lineage queries with offset and limit parameters
5. THE Semantic_Index SHALL return partition metadata as a list suitable for timeline visualization with time bounds and row counts
6. THE Semantic_Index SHALL include freshness timestamps in statistics responses for staleness indicators

