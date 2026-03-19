# Implementation Plan: OCSF Semantic Index Layer

## Overview

This implementation plan creates the `ocsf-index` crate as a new workspace member. The implementation follows a bottom-up approach: core types → storage backend → individual stores → integration points → API exposure.

## Tasks

- [x] 1. Set up ocsf-index crate structure
  - Create `ocsf-index/` directory with Cargo.toml
  - Add crate to workspace Cargo.toml
  - Set up module structure (lib.rs, error.rs, types.rs)
  - Add dependencies: serde, tokio, thiserror, chrono, rusqlite
  - _Requirements: 7.1, 7.2_

- [ ] 2. Implement core types and error handling
  - [x] 2.1 Implement ID types (TableId, LineageId, PartitionId, RecordId)
    - Create newtype wrappers with Serialize/Deserialize
    - Implement Display, From<u64> traits
    - _Requirements: 1.1, 2.1, 3.1, 4.1_
  
  - [x] 2.2 Implement IndexConfig and enums
    - Create IndexConfig with cache and statistics settings
    - Implement WarehouseDialect re-export from ocsf-semantic
    - Create ConfidenceLevel, Severity, TLPLevel enums
    - _Requirements: 6.1, 7.4_
  
  - [x] 2.3 Implement IndexError and BackendError
    - Define error types with thiserror
    - Create IndexResult type alias
    - _Requirements: 1.2, 7.6_
  
  - [ ]* 2.4 Write property test for ID type serialization round-trip
    - **Property 1: IndexRecord JSON Round-Trip**
    - **Validates: Requirements 1.6, 2.5, 3.6, 4.5, 5.5, 6.6**

- [ ] 3. Implement TableEntry and DetectionCoverage
  - [x] 3.1 Implement DetectionCoverage struct
    - Create struct with MITRE techniques, tactics, data sources
    - Add kill chain phases, confidence, severity, TLP fields
    - Implement builder pattern methods
    - Implement covers_technique() and covers_tactic() methods
    - _Requirements: 1.1_
  
  - [x] 3.2 Implement TableEntry struct
    - Create struct with table metadata fields
    - Add detection_coverage optional field
    - Implement builder pattern (new, with_schema, with_dialect, etc.)
    - _Requirements: 1.1, 1.6, 8.1_
  
  - [ ]* 3.3 Write property test for TableEntry JSON round-trip
    - **Property 1: IndexRecord JSON Round-Trip**
    - **Validates: Requirements 1.6**
  
  - [ ]* 3.4 Write property test for class UID validation
    - **Property 2: Table Registration Class UID Validation**
    - **Validates: Requirements 1.2**

- [ ] 4. Implement lineage record types
  - [x] 4.1 Implement SourceLineageRecord struct
    - Create struct with source system, table, target, timestamp
    - Add record_count and metadata fields
    - Implement builder pattern
    - _Requirements: 2.1, 2.5_
  
  - [x] 4.2 Implement FieldLineageRecord struct
    - Create struct with source/target fields, transformation
    - Add source_lineage_id and ocsf_version fields
    - Implement builder pattern
    - _Requirements: 3.1, 3.6_
  
  - [x] 4.3 Implement LineageEdge and FieldMapping for visualization
    - Create LineageEdge with source, target, timestamp
    - Create FieldMapping with source_path, target_path, transformation
    - Implement From conversions
    - _Requirements: 12.1, 12.2_
  
  - [ ]* 4.4 Write property test for lineage record serialization
    - **Property 1: IndexRecord JSON Round-Trip**
    - **Validates: Requirements 2.5, 3.6**

- [ ] 5. Implement partition and statistics types
  - [x] 5.1 Implement PartitionEntry struct
    - Create struct with partition key, time bounds, row count, size
    - Implement overlaps() method for time range checking
    - Implement builder pattern
    - _Requirements: 4.1, 4.5_
  
  - [x] 5.2 Implement TableStatistics and ColumnStatistics
    - Create ColumnStatistics with distinct/null/total counts, min/max
    - Implement null_rate() and cardinality() methods
    - Create TableStatistics with columns, sample_rate, collected_at
    - Implement is_stale() and get_column() methods
    - _Requirements: 5.1, 5.3, 5.5_
  
  - [ ]* 5.3 Write property test for partition overlap calculation
    - **Property 7: Partition Time Range Filtering**
    - **Validates: Requirements 4.3**
  
  - [ ]* 5.4 Write property test for statistics freshness
    - **Property 9: Statistics Freshness Tracking**
    - **Validates: Requirements 5.3, 5.4**

- [ ] 6. Implement query cache types
  - [x] 6.1 Implement QueryCacheKey struct
    - Create struct with query_hash and table_names
    - Implement from_query() method with normalization
    - Implement Hash and Eq traits
    - _Requirements: 6.4_
  
  - [x] 6.2 Implement CachedResult and CacheStats
    - Create CachedResult with key, result, timestamps, hit_count
    - Implement is_expired() method
    - Create CacheStats with counters
    - Implement hit_rate() method
    - _Requirements: 6.1, 6.2, 6.6_
  
  - [ ]* 6.3 Write property test for cache key normalization
    - **Property 10: Cache Key Normalization Idempotence**
    - **Validates: Requirements 6.4**

- [x] 7. Checkpoint - Core types complete
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement IndexBackend trait and in-memory backend
  - [x] 8.1 Define IndexBackend async trait
    - Create trait with create, read, update, delete, list methods
    - Add batch_create and batch_delete methods
    - Define IndexRecord trait for serializable records
    - _Requirements: 7.1_
  
  - [x] 8.2 Implement RecordFilter struct
    - Create filter with record_type, table_name, time_range
    - Add limit and offset for pagination
    - Implement builder pattern
    - _Requirements: 12.3, 12.4_
  
  - [x] 8.3 Implement InMemoryBackend
    - Create backend with HashMap storage
    - Implement all IndexBackend methods
    - Add thread-safe access with RwLock
    - _Requirements: 7.1, 7.4_
  
  - [ ]* 8.4 Write property test for backend CRUD operations
    - **Property 12: Backend API Consistency**
    - **Validates: Requirements 7.4**

- [ ] 9. Implement SQLite backend
  - [x] 9.1 Create SQLite schema and migrations
    - Define tables for all record types
    - Create indexes for common queries
    - Implement migration system
    - _Requirements: 7.2_
  
  - [x] 9.2 Implement SqliteBackend struct
    - Create backend with connection pool
    - Implement IndexBackend trait methods
    - Handle serialization/deserialization
    - _Requirements: 7.2, 7.5, 7.6_
  
  - [ ]* 9.3 Write property test for SQLite vs in-memory consistency
    - **Property 12: Backend API Consistency**
    - **Validates: Requirements 7.4**

- [ ] 10. Implement SemanticIndex core
  - [x] 10.1 Implement SemanticIndex struct and construction
    - Create struct with backend and component stores
    - Implement new() and open() async constructors
    - _Requirements: 7.4_
  
  - [x] 10.2 Implement table registry methods
    - Implement register_table() with validation
    - Implement deregister_table() with soft delete
    - Implement get_tables_by_class() and get_table()
    - _Requirements: 1.2, 1.3, 1.4, 1.5_
  
  - [ ]* 10.3 Write property test for query by class returns all tables
    - **Property 3: Query By Class Returns All Tables**
    - **Validates: Requirements 1.3, 1.4**
  
  - [ ]* 10.4 Write property test for soft delete preserves metadata
    - **Property 4: Soft Delete Preserves Metadata**
    - **Validates: Requirements 1.5**

- [ ] 11. Implement lineage methods
  - [x] 11.1 Implement source lineage methods
    - Implement record_source_lineage()
    - Implement get_source_lineage() with ordering
    - _Requirements: 2.2, 2.3, 2.4_
  
  - [x] 11.2 Implement field lineage methods
    - Implement record_field_lineage()
    - Implement get_field_lineage() for target field queries
    - _Requirements: 3.3, 3.4, 3.5_
  
  - [ ]* 11.3 Write property test for multiple sources per target
    - **Property 5: Multiple Sources Per Target Table**
    - **Validates: Requirements 2.3, 2.4**
  
  - [ ]* 11.4 Write property test for field lineage multiple mappings
    - **Property 6: Field Lineage Multiple Mappings**
    - **Validates: Requirements 3.3, 3.4, 3.5**

- [ ] 12. Implement partition and statistics methods
  - [x] 12.1 Implement partition metadata methods
    - Implement update_partition()
    - Implement get_partitions() and get_partitions_in_range()
    - Handle empty partition marking
    - _Requirements: 4.2, 4.3, 4.4_
  
  - [x] 12.2 Implement statistics methods
    - Implement update_statistics()
    - Implement get_statistics() with freshness
    - Stub collect_statistics() for future implementation
    - _Requirements: 5.2, 5.3, 5.4_
  
  - [ ]* 12.3 Write property test for partition time range filtering
    - **Property 7: Partition Time Range Filtering**
    - **Validates: Requirements 4.3**
  
  - [ ]* 12.4 Write property test for empty partition handling
    - **Property 8: Empty Partition Handling**
    - **Validates: Requirements 4.4**

- [x] 13. Checkpoint - Core index complete
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Implement query cache
  - [x] 14.1 Implement QueryCache struct
    - Create cache with max_entries and default TTL
    - Implement LRU eviction with access tracking
    - _Requirements: 6.1, 6.5_
  
  - [x] 14.2 Implement cache methods on SemanticIndex
    - Implement cache_query_result() with TTL
    - Implement get_cached_result() with expiry check
    - Implement invalidate_cache() by table name
    - Implement get_cache_stats()
    - _Requirements: 6.2, 6.3_
  
  - [ ]* 14.3 Write property test for LRU eviction
    - **Property 11: Cache LRU Eviction**
    - **Validates: Requirements 6.5**

- [ ] 15. Implement LineageCapture for ETL integration
  - [x] 15.1 Implement LineageCapture builder
    - Create struct with source/target tables and field mappings
    - Implement add_field_mapping() and add_field_mapping_with_transform()
    - Implement with_record_count() and with_metadata()
    - _Requirements: 9.1, 9.2_
  
  - [x] 15.2 Implement finalize() method
    - Create source lineage record
    - Create field lineage records for all mappings
    - Persist all records atomically
    - _Requirements: 9.3_
  
  - [ ]* 15.3 Write property test for LineageCapture persistence
    - **Property 13: LineageCapture Persistence Round-Trip**
    - **Validates: Requirements 9.3**

- [ ] 16. Implement query integration
  - [x] 16.1 Implement QueryPlanHints struct
    - Create struct with tables, partitions, statistics, cached_result
    - Create TableHint and PartitionHint structs
    - _Requirements: 10.1, 10.2, 10.5_
  
  - [x] 16.2 Implement get_query_plan_hints() method
    - Extract time range from SemanticQuery
    - Get relevant partitions with pruning
    - Get table statistics
    - Check cache for existing result
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  
  - [ ]* 16.3 Write property test for query partition pruning
    - **Property 14: Query Partition Pruning Correctness**
    - **Validates: Requirements 10.1**

- [ ] 17. Implement detection coverage analysis
  - [x] 17.1 Implement detection coverage query methods
    - Implement get_tables_by_mitre_technique()
    - Implement get_tables_by_mitre_tactic()
    - _Requirements: 1.1_
  
  - [x] 17.2 Implement DetectionCoverageSummary
    - Create summary struct with technique/tactic/data source coverage
    - Implement get_detection_coverage_summary()
    - Aggregate coverage across all tables
    - _Requirements: 1.1_

- [ ] 18. Implement lineage query pagination and filtering
  - [x] 18.1 Implement LineageQueryResult and FieldLineageQueryResult
    - Create result structs with edges/mappings, total_count, has_more
    - _Requirements: 12.1, 12.2_
  
  - [x] 18.2 Implement filtered lineage queries
    - Add time range filtering to lineage queries
    - Add pagination with offset/limit
    - _Requirements: 12.3, 12.4, 12.5, 12.6_
  
  - [ ]* 18.3 Write property test for lineage time range filtering
    - **Property 15: Lineage Time Range Filtering**
    - **Validates: Requirements 12.3**
  
  - [ ]* 18.4 Write property test for lineage pagination
    - **Property 16: Lineage Pagination Correctness**
    - **Validates: Requirements 12.4**

- [x] 19. Checkpoint - Full implementation complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 20. Integration with ocsf-warehouse
  - [x] 20.1 Add ocsf-index dependency to ocsf-warehouse
    - Update Cargo.toml
    - _Requirements: 9.1_
  
  - [x] 20.2 Extend ETLGenerator with lineage capture
    - Add method to create LineageCapture from ETLConfig
    - Document integration pattern in module docs
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 21. Integration with ocsf-semantic
  - [x] 21.1 Add ocsf-index dependency to ocsf-semantic
    - Update Cargo.toml
    - _Requirements: 10.1_
  
  - [x] 21.2 Extend SqlGenerator with index integration
    - Add optional SemanticIndex parameter
    - Use QueryPlanHints for table resolution
    - Document integration pattern
    - _Requirements: 10.1, 10.5_

- [x] 22. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.
  - Run `cargo clippy --workspace` and fix any warnings
  - Run `cargo fmt --check` and format if needed

## Notes

- Tasks marked with `*` are optional property-based tests
- Each property test references a specific property from the design document
- Checkpoints ensure incremental validation
- Integration tasks (20, 21) modify existing crates

