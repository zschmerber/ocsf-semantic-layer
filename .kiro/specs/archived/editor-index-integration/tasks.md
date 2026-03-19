# Implementation Plan: Editor Index Integration

## Overview

This implementation plan integrates the `ocsf-index` crate into the OCSF Semantic Model Editor, providing lineage visualization, MITRE ATT&CK coverage dashboards, table registry management, and statistics viewing through REST APIs and React components.

The implementation follows the existing patterns:
- Backend: Rust/Axum with State extraction and JSON responses
- Frontend: React/TypeScript with Zustand stores and TanStack Query
- Testing: Property-based tests with proptest, unit tests with vitest

## Tasks

- [x] 1. Backend: Integrate ocsf-index into ocsf-editor
  - [x] 1.1 Add ocsf-index dependency to ocsf-editor/Cargo.toml
    - Add `ocsf-index = { path = "../ocsf-index" }` to dependencies
    - _Requirements: 1.1_
  
  - [x] 1.2 Extend AppState with SemanticIndex
    - Add `semantic_index: Arc<SemanticIndex<Box<dyn IndexBackend>>>` field
    - Implement `create_index_backend()` with SQLite/in-memory selection
    - Initialize index in `AppState::new()`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 1.3 Create index API module structure
    - Create `ocsf-editor/src/api/index.rs` module
    - Add request/response types for tables, lineage, coverage, statistics
    - Export module from `ocsf-editor/src/api/mod.rs`
    - _Requirements: 2.1, 3.1, 4.1, 5.1, 6.1, 7.1_

- [x] 2. Backend: Table Registry API endpoints
  - [x] 2.1 Implement table CRUD handlers
    - `POST /api/index/tables` - register_table handler
    - `GET /api/index/tables` - list_tables handler with class_uid filter
    - `GET /api/index/tables/{id}` - get_table handler
    - `PUT /api/index/tables/{id}` - update_table handler
    - `DELETE /api/index/tables/{id}` - deactivate_table handler
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_
  
  - [ ]* 2.2 Write property test for table registration round-trip
    - **Property 1: Table Registration Round-Trip**
    - **Validates: Requirements 2.1, 2.3**
  
  - [ ]* 2.3 Write property test for table filtering
    - **Property 3: Table Filtering By Class UID**
    - **Validates: Requirements 2.4**

- [x] 3. Backend: Lineage API endpoints
  - [x] 3.1 Implement source lineage handlers
    - `POST /api/index/lineage/source` - record_source_lineage handler
    - `GET /api/index/lineage/source` - list_source_lineage with target_table/source_system filters
    - `GET /api/index/lineage/graph` - get_lineage_graph handler for visualization
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  
  - [x] 3.2 Implement field lineage handlers
    - `POST /api/index/lineage/field` - record_field_lineage handler
    - `GET /api/index/lineage/field` - list_field_lineage with target_field/source_lineage_id filters
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [ ]* 3.3 Write property test for lineage round-trip
    - **Property 6: Source Lineage Round-Trip**
    - **Validates: Requirements 3.1, 3.2, 3.3**
  
  - [ ]* 3.4 Write property test for lineage graph
    - **Property 8: Lineage Graph Contains All Edges**
    - **Validates: Requirements 3.5**

- [x] 4. Backend: Detection Coverage API endpoints
  - [x] 4.1 Implement coverage handlers
    - `GET /api/index/coverage/summary` - get_coverage_summary handler
    - `GET /api/index/coverage/techniques/{id}` - get_tables_by_technique handler
    - `GET /api/index/coverage/tactics/{tactic}` - get_tables_by_tactic handler
    - `GET /api/index/coverage/matrix` - get_coverage_matrix handler for MITRE visualization
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_
  
  - [ ]* 4.2 Write property test for coverage aggregation
    - **Property 11: Coverage Summary Aggregation**
    - **Validates: Requirements 5.1**

- [x] 5. Backend: Statistics and Partition API endpoints
  - [x] 5.1 Implement statistics handlers
    - `GET /api/index/statistics/{table_name}` - get_statistics handler
    - `POST /api/index/statistics/{table_name}/collect` - collect_statistics handler
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 5.2 Implement partition handlers
    - `GET /api/index/partitions/{table_name}` - list_partitions with time range filter
    - `POST /api/index/partitions` - update_partition handler
    - _Requirements: 7.1, 7.2, 7.3, 7.4_
  
  - [ ]* 5.3 Write property test for partition time range filtering
    - **Property 14: Partition Time Range Filtering**
    - **Validates: Requirements 7.2**

- [x] 6. Backend: Export/Import API endpoints
  - [x] 6.1 Implement export/import handlers
    - `GET /api/index/export` - export_index_model handler (JSON)
    - `POST /api/index/import` - import_index_model handler
    - Support combined=true query param for semantic+index bundle
    - _Requirements: 18.1, 18.3, 18.4, 19.1, 19.2_
  
  - [ ]* 6.2 Write property test for export/import round-trip
    - **Property 18: Index Model Export/Import Round-Trip**
    - **Validates: Requirements 18.1, 18.3, 19.1, 19.2**

- [x] 7. Checkpoint - Backend API complete
  - Ensure all backend tests pass
  - Verify API endpoints work with curl/httpie
  - Ask the user if questions arise

- [x] 8. Frontend: TypeScript types and API client
  - [x] 8.1 Add TypeScript types for index data
    - Add TableEntry, DetectionCoverage, SourceLineageRecord, FieldLineageRecord types
    - Add LineageGraph, DetectionCoverageSummary, CoverageMatrix types
    - Add TableStatistics, ColumnStatistics, PartitionEntry types
    - Add IndexModelExport, CombinedModelExport types
    - _Requirements: 12.2_
  
  - [x] 8.2 Implement API client functions
    - Create `editor-ui/src/api/index.ts` with CRUD functions
    - Implement getTables, registerTable, updateTable, deactivateTable
    - Implement getSourceLineage, recordSourceLineage, getLineageGraph
    - Implement getCoverageSummary, getCoverageMatrix, getTablesByTechnique
    - Implement getStatistics, collectStatistics
    - Implement exportIndexModel, importIndexModel
    - _Requirements: 12.1, 12.4, 12.5_
  
  - [x] 8.3 Implement TanStack Query hooks
    - Create hooks for tables: useTables, useTable, useRegisterTable
    - Create hooks for lineage: useLineageGraph, useSourceLineage
    - Create hooks for coverage: useCoverageSummary, useCoverageMatrix
    - Create hooks for statistics: useTableStatistics, useCollectStatistics
    - _Requirements: 12.3_

- [x] 9. Frontend: Zustand store for index state
  - [x] 9.1 Create indexStore
    - Create `editor-ui/src/store/indexStore.ts`
    - Add log import state: rawLogInput, parsedFields, detectedFormat
    - Add mapping state: fieldMappings, selectedSourceField, selectedTargetField
    - Add UI state: selectedTableId, selectedTechnique, lineageFilter
    - Implement actions for state updates
    - _Requirements: 14.1, 14.4_

- [x] 10. Frontend: Log Import and Mapping components
  - [x] 10.1 Implement log parser utilities
    - Create `editor-ui/src/utils/logParser.ts`
    - Implement detectLogFormat for JSON, CSV, syslog, key-value
    - Implement parseLogInput with format-specific parsers
    - Implement flattenObject for JSON field extraction
    - _Requirements: 14.2, 14.3_
  
  - [ ]* 10.2 Write property test for log format detection
    - **Property 15: Log Format Detection**
    - **Validates: Requirements 14.2**
  
  - [x] 10.3 Implement LogImport component
    - Create `editor-ui/src/components/LogImport/` directory
    - Implement LogImport.tsx with textarea and file upload
    - Implement ParsedFieldsPreview sub-component
    - Add LogImport.css styles
    - _Requirements: 14.1, 14.2, 14.3, 14.9_
  
  - [x] 10.4 Implement MappingBuilder component
    - Create `editor-ui/src/components/MappingBuilder/` directory
    - Implement MappingBuilder.tsx with source/target field selection
    - Implement OCSFFieldSelector sub-component
    - Implement MappingRow sub-component with transformation input
    - Add MappingBuilder.css styles
    - _Requirements: 14.4, 14.5, 14.6, 14.7, 14.8_

- [x] 11. Frontend: Lineage Visualization component
  - [x] 11.1 Implement LineageVisualization component
    - Create `editor-ui/src/components/LineageVisualization/` directory
    - Implement LineageVisualization.tsx with source/target columns
    - Implement LineageEdgePath SVG component for edges
    - Implement node click handlers for filtering
    - Add LineageVisualization.css styles
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

- [x] 12. Frontend: Detection Coverage Dashboard component
  - [x] 12.1 Implement DetectionCoverageDashboard component
    - Create `editor-ui/src/components/DetectionCoverage/` directory
    - Implement DetectionCoverageDashboard.tsx with MITRE matrix
    - Implement TechniqueDetailPanel sub-component
    - Implement coverage statistics display
    - Add tactic filtering
    - Add DetectionCoverageDashboard.css styles
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9_

- [x] 13. Frontend: Table Registry Browser component
  - [x] 13.1 Implement TableRegistryBrowser component
    - Create `editor-ui/src/components/TableRegistry/` directory
    - Implement TableRegistryBrowser.tsx with table list
    - Implement TableRow sub-component with coverage tags
    - Implement TableDetailPanel sub-component
    - Implement RegisterTableModal sub-component
    - Add TableRegistryBrowser.css styles
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8_

- [x] 14. Frontend: Statistics Viewer component
  - [x] 14.1 Implement StatisticsViewer component
    - Create `editor-ui/src/components/StatisticsViewer/` directory
    - Implement StatisticsViewer.tsx with column stats table
    - Implement ColumnStatsRow sub-component with rate bars
    - Implement staleness indicator and refresh button
    - Add StatisticsViewer.css styles
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8_

- [x] 15. Checkpoint - Frontend components complete
  - Ensure all frontend components render correctly
  - Test component interactions manually
  - Ask the user if questions arise

- [x] 16. Frontend: Index Model Building workflow
  - [x] 16.1 Implement IndexBuilder component
    - Create `editor-ui/src/components/IndexBuilder/` directory
    - Implement IndexBuilder.tsx for table registration from mappings
    - Implement DetectionCoverageForm sub-component
    - Implement IndexModelSummary sub-component
    - Wire up to registerTable and recordSourceLineage APIs
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7_
  
  - [x] 16.2 Integrate with EntityEditor for semantic model building
    - Add mapping suggestions to AttributeEditor
    - Add ocsf_mapping population from field mappings
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5, 16.6_

- [x] 17. Frontend: Validation and Export/Import
  - [x] 17.1 Implement index model validation
    - Add validateIndexModel function
    - Check class_uid validity, orphan lineage, MITRE ID format
    - Integrate with ValidationPanel
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6, 17.7_
  
  - [ ]* 17.2 Write property test for index model validation
    - **Property 17: Index Model Validation Correctness**
    - **Validates: Requirements 17.2, 17.3, 17.4**
  
  - [x] 17.3 Implement export/import UI
    - Add export buttons to Index views
    - Implement ImportModal component
    - Support JSON download and clipboard copy
    - _Requirements: 18.1, 18.2, 18.4, 18.5, 18.6, 18.7, 19.1, 19.3, 19.4, 19.5, 19.6_

- [x] 18. Frontend: Navigation integration
  - [x] 18.1 Add Index section to navigation
    - Add "Index" nav item with sub-items: Lineage, Coverage, Tables, Statistics
    - Add route definitions for index views
    - Add configuration prompt when backend not configured
    - _Requirements: 13.1, 13.2, 13.3, 13.4_

- [x] 19. Final checkpoint - Integration complete
  - Ensure all tests pass (cargo test, npm test)
  - Test full workflow: import log → map fields → register table → view lineage
  - Verify export/import round-trip works
  - Ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
