# Implementation Plan: OCSF Semantic Layer

## Overview

This implementation plan builds the OCSF Semantic Layer in Rust, leveraging its performance characteristics for ETL pipelines and query translation, with strong type safety for schema handling. The implementation follows a bottom-up approach: core data structures → schema ingestion → semantic layer → vector embeddings → query translation → warehouse generation → visualization.

## Tasks

- [ ] 1. Project Setup and Core Data Structures
  - [x] 1.1 Initialize Rust project with Cargo workspace structure
    - Create workspace with crates: `ocsf-core`, `ocsf-semantic`, `ocsf-vector`, `ocsf-warehouse`, `ocsf-viz`
    - Configure shared dependencies: serde, tokio, anyhow, thiserror
    - Set up CI configuration
    - _Requirements: 1.1_

  - [x] 1.2 Define OCSF schema data structures in `ocsf-core`
    - Implement `Category`, `EventClass`, `OCSFObject`, `Attribute` structs
    - Implement `ObservableDefinition` with all definition types
    - Derive Serialize/Deserialize for all types
    - _Requirements: 1.2, 1.3_

  - [x] 1.3 Write property test for schema struct serialization round-trip
    - **Property 1: Schema Parsing Completeness and Fidelity**
    - **Validates: Requirements 1.2, 1.3, 1.5**

- [x] 2. Schema Ingester Implementation
  - [x] 2.1 Implement schema loading from local files
    - Parse OCSF JSON schema files
    - Build internal `OCSFSchema` representation
    - Handle nested object references
    - _Requirements: 1.1, 1.2_

  - [x] 2.2 Implement schema loading from GitHub
    - Fetch schema from ocsf/ocsf-schema repository
    - Support version tags
    - Cache downloaded schemas locally
    - _Requirements: 1.1_

  - [x] 2.3 Implement observable extraction
    - Extract observables by type, attribute, object, event class, and path
    - Build observable catalog with type_id mappings
    - _Requirements: 10.1_

  - [x] 2.4 Write property test for observable extraction completeness
    - **Property 15: Observable Extraction Completeness**
    - **Validates: Requirements 10.1**

  - [x] 2.5 Implement schema version detection and comparison
    - Parse version strings
    - Generate schema diff between versions
    - _Requirements: 1.4_

- [x] 3. Checkpoint - Schema Ingestion Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Semantic Model Core in `ocsf-semantic`
  - [x] 4.1 Define semantic entity data structures
    - Implement `SemanticEntity`, `SemanticAttribute`, `OCSFMapping`
    - Implement `EntityRelationship` with cardinality
    - Support expression-based mappings
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 4.2 Define semantic metric data structures
    - Implement `SemanticMetric` with aggregation types
    - Support dimensions and time granularities
    - Add hot_path flag and observable_type_id
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 11.4_

  - [x] 4.3 Implement YAML serialization for semantic models
    - Use serde_yaml for serialization
    - Support the semantic-model.yaml format from design
    - _Requirements: 9.1_

  - [x] 4.4 Write property test for semantic entity round-trip
    - **Property 2: Semantic Entity Definition Round-Trip**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 9.1, 9.4**

  - [x] 4.5 Write property test for metric definition round-trip
    - **Property 4: Metric Definition Round-Trip**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**

- [-] 5. Semantic Model Validation
  - [x] 5.1 Implement OCSF field reference validation
    - Validate entity attribute mappings against schema
    - Report specific invalid field paths
    - _Requirements: 2.5_

  - [x] 5.2 Write property test for invalid field reference detection
    - **Property 3: Invalid Field Reference Detection**
    - **Validates: Requirements 2.5**

  - [x] 5.3 Implement metric determinism validation
    - Check for non-deterministic expressions
    - Validate aggregation compatibility
    - _Requirements: 3.5_

  - [x] 5.4 Implement model versioning and change tracking
    - Track modification history
    - Support version comparison
    - _Requirements: 9.2, 9.5_

- [x] 6. Observable Analyzer
  - [x] 6.1 Implement observable-to-entity mapping
    - Map observable type_ids to covering semantic entities
    - Calculate coverage percentage
    - _Requirements: 10.2_

  - [x] 6.2 Implement redundancy detection
    - Flag observables fully covered by semantic layer
    - Generate coverage report
    - _Requirements: 10.3, 10.4_

  - [x] 6.3 Write property test for observable coverage analysis
    - **Property 16: Observable Coverage Analysis**
    - **Validates: Requirements 10.2, 10.3, 10.4**

- [x] 7. Checkpoint - Semantic Layer Core Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Vector Embedding Layer in `ocsf-vector`
  - [x] 8.1 Implement embedding generator interface
    - Define trait for embedding models
    - Support sentence-transformers via candle or ort
    - Support OpenAI API embeddings
    - _Requirements: 5.1, 5.2, 5.3_

  - [x] 8.2 Implement vector store with similarity search
    - Use qdrant-client or implement in-memory store
    - Support cosine similarity search
    - Store metadata with embeddings
    - _Requirements: 5.4_

  - [x] 8.3 Write property test for embedding consistency
    - **Property 7: Embedding Generation Consistency**
    - **Validates: Requirements 5.1, 5.2**

  - [x] 8.4 Implement incremental embedding updates
    - Detect new/changed schema elements
    - Update only affected embeddings
    - _Requirements: 5.5_

- [x] 9. Mapping Assistant
  - [x] 9.1 Implement mapping suggestion engine
    - Query vector store for similar fields
    - Rank by similarity score
    - Generate reasoning for suggestions
    - _Requirements: 6.1, 6.2, 6.3_

  - [x] 9.2 Write property test for similarity search ordering
    - **Property 8: Vector Similarity Search Ordering**
    - **Validates: Requirements 6.1, 6.2**

  - [x] 9.3 Write property test for suggestion completeness
    - **Property 9: Mapping Suggestion Completeness**
    - **Validates: Requirements 6.3**

  - [x] 9.4 Implement learning from confirmed mappings
    - Store confirmed mappings
    - Boost similar mappings in future suggestions
    - _Requirements: 6.4_

  - [x] 9.5 Implement extension attribute suggestion
    - Detect low-confidence matches
    - Suggest custom extension creation
    - _Requirements: 6.5_

- [x] 10. Checkpoint - Vector Layer Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 11. Query Translator
  - [x] 11.1 Implement semantic query parser
    - Parse semantic query structure
    - Validate entity and metric references
    - _Requirements: 8.1_

  - [x] 11.2 Implement SQL generation for entity queries
    - Generate SELECT with attribute mappings
    - Generate JOINs for entity relationships
    - _Requirements: 8.2_

  - [x] 11.3 Implement metric aggregation in SQL
    - Apply aggregation functions
    - Handle time granularities
    - _Requirements: 8.3_

  - [x] 11.4 Write property test for query translation correctness
    - **Property 13: Query Translation Correctness**
    - **Validates: Requirements 8.1, 8.2, 8.3**

  - [x] 11.5 Write property test for invalid query error handling
    - **Property 14: Invalid Query Error Handling**
    - **Validates: Requirements 8.4**

  - [x] 11.6 Implement hot path query routing
    - Detect hot path metrics
    - Route to observables table
    - _Requirements: 11.4, 11.6_

  - [x] 11.7 Write property test for hot path query routing
    - **Property 20: Hot Path Query Routing**
    - **Validates: Requirements 11.4, 11.6**

  - [x] 11.8 Implement reverse lookup query generation
    - Generate query from observable matches to full events
    - Optimize for batch lookups
    - _Requirements: 11.5_

  - [x] 11.9 Write property test for reverse lookup query generation
    - **Property 19: Reverse Lookup Query Generation**
    - **Validates: Requirements 11.5**

  - [x] 11.10 Implement warehouse dialect support
    - Support Snowflake, Databricks, BigQuery, PostgreSQL dialects
    - Apply dialect-specific optimizations
    - _Requirements: 8.5_

- [x] 12. Checkpoint - Query Layer Complete
  - Ensure all tests pass, ask the user if questions arise.


- [x] 13. Warehouse Generator in `ocsf-warehouse`
  - [x] 13.1 Implement OCSF table schema generation
    - Generate CREATE TABLE for each event class
    - Map OCSF types to warehouse types
    - _Requirements: 7.1, 7.4_

  - [x] 13.2 Implement observables table generation
    - Generate dedicated observables table schema
    - Include all required columns
    - Add indexes for threat intel matching
    - _Requirements: 11.1, 11.2_

  - [x] 13.3 Write property test for observables table schema completeness
    - **Property 17: Observables Table Schema Completeness**
    - **Validates: Requirements 11.1, 11.2**

  - [x] 13.4 Implement dbt semantic layer YAML generation
    - Generate semantic_manifest.yml
    - Generate model SQL files
    - Generate sources.yml
    - _Requirements: 7.2_

  - [x] 13.5 Write property test for dbt artifact validity
    - **Property 10: DBT Artifact Validity**
    - **Validates: Requirements 7.2**

  - [x] 13.6 Implement Cube.js schema generation
    - Generate cube definitions
    - Map entities to cubes, metrics to measures
    - _Requirements: 7.3_

  - [x] 13.7 Write property test for Cube.js schema validity
    - **Property 11: Cube.js Schema Validity**
    - **Validates: Requirements 7.3**

  - [x] 13.8 Implement SQL view generation for semantic entities
    - Generate CREATE VIEW for each entity
    - Include OCSF field mappings and expressions
    - _Requirements: 7.5_

  - [x] 13.9 Write property test for SQL view syntax validity
    - **Property 12: SQL View Syntax Validity**
    - **Validates: Requirements 7.5**

  - [x] 13.10 Implement ETL pipeline generation
    - Generate observable extraction logic
    - Preserve reverse-lookup references
    - _Requirements: 11.3_

  - [x] 13.11 Write property test for observable extraction from events
    - **Property 18: Observable Extraction from Events**
    - **Validates: Requirements 11.3**

  - [x] 13.12 Implement materialized view generation for observables
    - Generate optimized materialized views
    - Configure refresh strategies
    - _Requirements: 11.8_

  - [x] 13.13 Implement migration script generation
    - Compare model versions
    - Generate ALTER statements
    - _Requirements: 7.6_

- [x] 14. Checkpoint - Warehouse Generator Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 15. Visualization Engine in `ocsf-viz`
  - [x] 15.1 Implement graph data structure
    - Define `GraphNode`, `GraphEdge`, `GraphData`
    - Support node types: semantic_entity, ocsf_category, ocsf_class, observable, hot_path, cold_path
    - _Requirements: 4.1_

  - [x] 15.2 Implement graph generation from semantic model
    - Create nodes for entities and OCSF elements
    - Create edges for mappings and relationships
    - _Requirements: 4.1, 4.2_

  - [x] 15.3 Write property test for graph generation completeness
    - **Property 5: Graph Generation Completeness**
    - **Validates: Requirements 4.1, 4.2**

  - [x] 15.4 Implement view mode filtering
    - Filter for semantic_only, physical_only, interchange modes
    - Apply visual differentiation
    - _Requirements: 4.4, 4.5_

  - [x] 15.5 Write property test for view mode filtering
    - **Property 6: View Mode Filtering**
    - **Validates: Requirements 4.4**

  - [x] 15.6 Implement entity highlighting
    - Highlight selected entity and connected elements
    - _Requirements: 4.3_

  - [x] 15.7 Implement observable-to-entity mapping visualization
    - Show coverage relationships
    - Highlight redundant observables
    - _Requirements: 10.7_

  - [x] 15.8 Implement hot/cold path visualization
    - Show data flow from observables table to full events
    - Visualize threat intel matching flow
    - _Requirements: 11.7_

  - [x] 15.9 Implement export formats
    - Export to SVG using svg crate
    - Export to PNG using resvg
    - Export to JSON graph format
    - _Requirements: 4.6_

- [x] 16. Checkpoint - Visualization Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 17. CLI and Integration
  - [x] 17.1 Implement CLI with clap
    - Commands: ingest, validate, generate, visualize, query
    - Support configuration file
    - _Requirements: All_

  - [x] 17.2 Implement end-to-end integration
    - Wire all components together
    - Implement main workflows
    - _Requirements: All_

  - [x] 17.3 Add comprehensive error handling
    - Implement error types for each component
    - Provide actionable error messages
    - _Requirements: All_

- [x] 18. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.
  - Review all property tests are passing
  - Verify end-to-end workflows

## Notes

- All tasks including property-based tests are required
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use `proptest` crate for Rust property-based testing
- Unit tests validate specific examples and edge cases
