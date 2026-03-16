# Requirements Document

## Introduction

This document specifies the requirements for integrating the `ocsf-index` crate into the OCSF Semantic Model Editor application. The integration will provide users with lineage visualization, MITRE ATT&CK detection coverage dashboards, table registry management, and statistics viewing capabilities through a web-based interface.

The `ocsf-index` crate provides a metadata management system that tracks physical table locations, source-to-OCSF lineage, partition metadata, column statistics, and query result caching. This integration exposes these capabilities through REST API endpoints in the `ocsf-editor` backend and React components in the `editor-ui` frontend.

## Glossary

- **Semantic_Index**: The main entry point for all index operations, coordinating table registry, lineage tracking, statistics, and caching
- **Table_Registry**: Component that manages physical table metadata and OCSF class mappings
- **Source_Lineage**: Records tracking which raw data sources contributed to each OCSF table
- **Field_Lineage**: Records tracking field-level transformations from source to OCSF fields
- **Detection_Coverage**: Metadata tracking MITRE ATT&CK techniques, tactics, and data sources covered by a table
- **Lineage_Graph**: Visual representation of data flow from source systems to OCSF tables
- **MITRE_ATT&CK_Matrix**: A framework for categorizing adversary tactics and techniques
- **Table_Statistics**: Column-level statistics including cardinality, null rates, and value ranges
- **Partition_Metadata**: Information about table partitions including time bounds and row counts
- **API_Server**: The Axum-based HTTP server in `ocsf-editor` that serves REST endpoints
- **Index_Backend**: Storage abstraction supporting SQLite and in-memory backends

## Requirements

### Requirement 1: Backend Index Integration

**User Story:** As a developer, I want the ocsf-index crate integrated into the ocsf-editor backend, so that index functionality is available through the API server.

#### Acceptance Criteria

1. THE API_Server SHALL initialize a Semantic_Index instance with configurable backend (SQLite or in-memory) on startup
2. THE API_Server SHALL store the Semantic_Index in AppState for access by API handlers
3. WHEN the API_Server starts, THE Semantic_Index SHALL be initialized with default IndexConfig values
4. IF the SQLite backend path is configured via environment variable, THEN THE API_Server SHALL use SQLite backend
5. IF no backend path is configured, THEN THE API_Server SHALL use in-memory backend for development
6. THE API_Server SHALL expose index configuration through environment variables (OCSF_INDEX_BACKEND, OCSF_INDEX_PATH)

### Requirement 2: Table Registry API

**User Story:** As a data engineer, I want to register and manage OCSF tables through the API, so that I can track which tables exist in my data warehouse.

#### Acceptance Criteria

1. WHEN a POST request is made to /api/index/tables with a valid TableEntry JSON body, THE API_Server SHALL register the table and return the assigned TableId
2. WHEN a GET request is made to /api/index/tables, THE API_Server SHALL return a list of all active registered tables
3. WHEN a GET request is made to /api/index/tables/{id}, THE API_Server SHALL return the TableEntry for that ID
4. WHEN a GET request is made to /api/index/tables with query parameter class_uid, THE API_Server SHALL return tables filtered by OCSF class UID
5. WHEN a DELETE request is made to /api/index/tables/{id}, THE API_Server SHALL soft-delete the table (set is_active to false)
6. WHEN a PUT request is made to /api/index/tables/{id} with updated TableEntry, THE API_Server SHALL update the table metadata
7. IF a table registration request has class_uid of 0, THEN THE API_Server SHALL return a 400 Bad Request error
8. THE API_Server SHALL return table entries with detection_coverage metadata when present

### Requirement 3: Source Lineage API

**User Story:** As a security engineer, I want to query source lineage through the API, so that I can understand data provenance for OCSF tables.

#### Acceptance Criteria

1. WHEN a POST request is made to /api/index/lineage/source with a valid SourceLineageRecord JSON body, THE API_Server SHALL record the lineage and return the assigned LineageId
2. WHEN a GET request is made to /api/index/lineage/source with query parameter target_table, THE API_Server SHALL return all source lineage records for that target table
3. WHEN a GET request is made to /api/index/lineage/source with query parameter source_system, THE API_Server SHALL return all lineage records from that source system
4. THE API_Server SHALL return lineage records ordered by ingestion_timestamp ascending
5. WHEN a GET request is made to /api/index/lineage/graph, THE API_Server SHALL return lineage data formatted as LineageEdge array for graph visualization
6. THE API_Server SHALL support pagination via limit and offset query parameters on lineage endpoints

### Requirement 4: Field Lineage API

**User Story:** As a data engineer, I want to query field-level lineage through the API, so that I can trace how source fields map to OCSF fields.

#### Acceptance Criteria

1. WHEN a POST request is made to /api/index/lineage/field with a valid FieldLineageRecord JSON body, THE API_Server SHALL record the field lineage and return the assigned LineageId
2. WHEN a GET request is made to /api/index/lineage/field with query parameter target_field, THE API_Server SHALL return all field lineage records for that target field
3. WHEN a GET request is made to /api/index/lineage/field with query parameter source_lineage_id, THE API_Server SHALL return all field mappings for that source lineage record
4. THE API_Server SHALL return field lineage records with transformation expressions when present
5. THE API_Server SHALL return field lineage as FieldMapping array for visualization

### Requirement 5: Detection Coverage API

**User Story:** As a detection engineer, I want to query MITRE ATT&CK coverage through the API, so that I can identify detection gaps.

#### Acceptance Criteria

1. WHEN a GET request is made to /api/index/coverage/summary, THE API_Server SHALL return a DetectionCoverageSummary with aggregated coverage across all tables
2. WHEN a GET request is made to /api/index/coverage/techniques/{technique_id}, THE API_Server SHALL return all tables covering that MITRE technique
3. WHEN a GET request is made to /api/index/coverage/tactics/{tactic}, THE API_Server SHALL return all tables covering that MITRE tactic
4. THE DetectionCoverageSummary SHALL include mitre_techniques, mitre_tactics, data_sources, and kill_chain_coverage arrays
5. WHEN a GET request is made to /api/index/coverage/matrix, THE API_Server SHALL return coverage data formatted for MITRE ATT&CK matrix visualization
6. THE API_Server SHALL include table_count and tables list for each technique and tactic in the coverage response

### Requirement 6: Table Statistics API

**User Story:** As an analyst, I want to view table statistics through the API, so that I can understand data quality and distribution.

#### Acceptance Criteria

1. WHEN a GET request is made to /api/index/statistics/{table_name}, THE API_Server SHALL return TableStatistics for that table
2. WHEN a POST request is made to /api/index/statistics/{table_name}/collect with optional sample_rate parameter, THE API_Server SHALL trigger statistics collection
3. THE TableStatistics response SHALL include column-level statistics with distinct_count, null_count, min_value, and max_value
4. THE API_Server SHALL return collected_at timestamp and is_stale indicator based on statistics_max_age_secs config
5. IF statistics do not exist for a table, THEN THE API_Server SHALL return a 404 Not Found error

### Requirement 7: Partition Metadata API

**User Story:** As a data engineer, I want to view partition metadata through the API, so that I can understand table partitioning for query optimization.

#### Acceptance Criteria

1. WHEN a GET request is made to /api/index/partitions/{table_name}, THE API_Server SHALL return all PartitionEntry records for that table
2. WHEN a GET request is made to /api/index/partitions/{table_name} with start_time and end_time query parameters, THE API_Server SHALL return partitions overlapping that time range
3. THE PartitionEntry response SHALL include partition_key, start_time, end_time, row_count, and size_bytes
4. WHEN a POST request is made to /api/index/partitions with a valid PartitionEntry JSON body, THE API_Server SHALL update or create the partition metadata

### Requirement 8: Lineage Visualization Component

**User Story:** As a security engineer, I want to see an interactive lineage graph in the UI, so that I can visualize data flow from source systems to OCSF tables.

#### Acceptance Criteria

1. WHEN the Lineage_Visualization component loads, THE component SHALL fetch lineage data from /api/index/lineage/graph
2. THE Lineage_Visualization component SHALL render source systems as nodes on the left side of the graph
3. THE Lineage_Visualization component SHALL render OCSF tables as nodes on the right side of the graph
4. THE Lineage_Visualization component SHALL render edges between source and target nodes with record_count labels when available
5. WHEN a user clicks on a lineage edge, THE component SHALL display field-level mappings for that source-to-target relationship
6. WHEN a user clicks on a table node, THE component SHALL display table details including detection_coverage metadata
7. THE Lineage_Visualization component SHALL support filtering by source_system or target_table
8. THE Lineage_Visualization component SHALL display loading and error states appropriately

### Requirement 9: Detection Coverage Dashboard Component

**User Story:** As a detection engineer, I want to see a MITRE ATT&CK coverage dashboard in the UI, so that I can identify detection gaps.

#### Acceptance Criteria

1. WHEN the Detection_Coverage_Dashboard component loads, THE component SHALL fetch coverage data from /api/index/coverage/summary
2. THE Detection_Coverage_Dashboard component SHALL display a MITRE ATT&CK matrix with techniques colored by coverage level
3. WHEN a technique has table_count > 0, THE component SHALL display it as covered (green)
4. WHEN a technique has table_count = 0, THE component SHALL display it as a gap (red or gray)
5. WHEN a user clicks on a technique cell, THE component SHALL display the list of tables covering that technique
6. THE Detection_Coverage_Dashboard component SHALL display summary statistics including total techniques covered and coverage percentage
7. THE Detection_Coverage_Dashboard component SHALL support filtering by tactic to focus on specific kill chain phases
8. THE Detection_Coverage_Dashboard component SHALL display data_sources coverage as a separate section
9. THE Detection_Coverage_Dashboard component SHALL display kill_chain_coverage as a visual indicator

### Requirement 10: Table Registry Browser Component

**User Story:** As a data engineer, I want to browse and manage registered tables in the UI, so that I can maintain the table registry.

#### Acceptance Criteria

1. WHEN the Table_Registry_Browser component loads, THE component SHALL fetch tables from /api/index/tables
2. THE Table_Registry_Browser component SHALL display tables in a searchable, sortable list
3. THE Table_Registry_Browser component SHALL display table_name, class_uid, ocsf_version, dialect, and is_active for each table
4. WHEN a user clicks on a table row, THE component SHALL display full table details including metadata and detection_coverage
5. THE Table_Registry_Browser component SHALL provide a form to register new tables with required fields
6. THE Table_Registry_Browser component SHALL provide actions to edit and deactivate existing tables
7. THE Table_Registry_Browser component SHALL support filtering by class_uid and is_active status
8. WHEN a table has detection_coverage, THE component SHALL display MITRE techniques and tactics as tags

### Requirement 11: Statistics Viewer Component

**User Story:** As an analyst, I want to view table statistics in the UI, so that I can understand data quality.

#### Acceptance Criteria

1. WHEN the Statistics_Viewer component loads for a table, THE component SHALL fetch statistics from /api/index/statistics/{table_name}
2. THE Statistics_Viewer component SHALL display column-level statistics in a table format
3. THE Statistics_Viewer component SHALL display null_rate as a percentage with visual indicator
4. THE Statistics_Viewer component SHALL display cardinality (distinct_count / total_count) as a percentage
5. THE Statistics_Viewer component SHALL display min_value and max_value for each column
6. THE Statistics_Viewer component SHALL display collected_at timestamp and staleness indicator
7. WHEN statistics are stale, THE component SHALL display a warning and offer to refresh
8. THE Statistics_Viewer component SHALL provide a button to trigger statistics collection

### Requirement 12: Frontend API Client Extensions

**User Story:** As a frontend developer, I want API client functions for index endpoints, so that I can easily fetch index data in components.

#### Acceptance Criteria

1. THE API client SHALL export functions for all index endpoints following existing patterns (get, post, put, del)
2. THE API client SHALL export TypeScript types matching the Rust backend types (TableEntry, SourceLineageRecord, DetectionCoverageSummary, etc.)
3. THE API client SHALL use TanStack Query hooks for data fetching with caching and refetching
4. THE API client SHALL handle errors consistently with existing error handling patterns
5. THE API client SHALL support query parameters for filtering and pagination

### Requirement 13: Navigation and Layout Integration

**User Story:** As a user, I want to access index features from the main navigation, so that I can easily find lineage, coverage, and registry views.

#### Acceptance Criteria

1. THE editor-ui application SHALL add an "Index" section to the main navigation
2. THE Index section SHALL include links to Lineage, Coverage, Tables, and Statistics views
3. WHEN the Index backend is not configured, THE navigation SHALL display a configuration prompt
4. THE Index views SHALL integrate with the existing layout and styling patterns

### Requirement 14: Raw Log Import and OCSF Transformation

**User Story:** As a security engineer, I want to load raw log samples into the editor and transform them to OCSF format, so that I can build mappings interactively.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide a Log_Import component for loading raw log samples
2. WHEN a user pastes or uploads raw log data, THE Log_Import component SHALL detect the log format (JSON, CSV, syslog, key-value)
3. THE Log_Import component SHALL display the parsed log structure with field names and sample values
4. THE editor-ui application SHALL provide a Mapping_Builder component for defining source-to-OCSF field mappings
5. WHEN a user maps a source field to an OCSF field, THE Mapping_Builder SHALL record the mapping as a FieldLineageRecord
6. THE Mapping_Builder component SHALL suggest OCSF target fields based on source field names and values
7. THE Mapping_Builder component SHALL support transformation expressions for field value conversion
8. WHEN mappings are complete, THE Mapping_Builder SHALL generate a preview of the transformed OCSF event
9. THE Log_Import component SHALL support loading multiple log samples to validate mappings across variations

### Requirement 15: Index Model Building

**User Story:** As a data engineer, I want to build the ocsf-index model from my mappings, so that I can track lineage and register tables.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide an Index_Builder component for constructing index models
2. WHEN a user completes field mappings, THE Index_Builder SHALL create SourceLineageRecord entries automatically
3. THE Index_Builder component SHALL allow users to specify table metadata (table_name, schema_name, dialect)
4. THE Index_Builder component SHALL allow users to add detection coverage metadata (MITRE techniques, tactics, data sources)
5. WHEN a user saves the index model, THE Index_Builder SHALL register the table and lineage records via the API
6. THE Index_Builder component SHALL validate that all required fields are mapped before allowing save
7. THE Index_Builder component SHALL display a summary of the index model including table count, lineage count, and coverage

### Requirement 16: Semantic Model Building from Mappings

**User Story:** As a data engineer, I want to build the ocsf-semantic model from my mappings, so that I can define semantic entities and metrics.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide integration between the Mapping_Builder and existing Entity_Editor
2. WHEN field mappings are defined, THE editor SHALL suggest semantic attributes based on the mapped OCSF fields
3. THE editor SHALL allow users to mark mapped fields as dimensions or observables
4. THE editor SHALL allow users to define metrics based on the mapped fields
5. WHEN a user creates a semantic entity, THE editor SHALL populate ocsf_mapping from the field mappings
6. THE editor SHALL track which OCSF event classes are covered by the semantic model

### Requirement 17: Model Validation

**User Story:** As a user, I want to validate both semantic and index models in the GUI, so that I can ensure correctness before export.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide a Validation_Panel for index model validation
2. WHEN validating the index model, THE Validation_Panel SHALL check that all registered tables have valid class_uid values
3. WHEN validating the index model, THE Validation_Panel SHALL check that all lineage records reference existing tables
4. WHEN validating the index model, THE Validation_Panel SHALL check that detection coverage references valid MITRE technique IDs
5. THE Validation_Panel SHALL display validation errors and warnings with paths to the problematic elements
6. THE existing semantic model validation SHALL continue to work alongside index validation
7. THE Validation_Panel SHALL provide a combined validation view showing both semantic and index validation results

### Requirement 18: Model Export

**User Story:** As a user, I want to export both semantic and index models, so that I can use them in my data pipeline.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide export functionality for the index model as JSON
2. THE editor-ui application SHALL provide export functionality for the semantic model as YAML (existing)
3. WHEN exporting the index model, THE export SHALL include all registered tables, lineage records, and detection coverage
4. THE editor-ui application SHALL provide a combined export option that bundles semantic and index models together
5. THE export functionality SHALL support downloading as a file or copying to clipboard
6. WHEN exporting, THE editor SHALL validate the models and warn if there are validation errors
7. THE export SHALL include metadata such as export timestamp and model versions

### Requirement 19: Import Existing Models

**User Story:** As a user, I want to import existing semantic and index models, so that I can continue editing previous work.

#### Acceptance Criteria

1. THE editor-ui application SHALL provide import functionality for index models from JSON
2. WHEN importing an index model, THE editor SHALL load tables, lineage records, and detection coverage into the index
3. THE editor-ui application SHALL provide import functionality for semantic models from YAML (existing)
4. THE editor SHALL support importing a combined bundle containing both semantic and index models
5. WHEN importing, THE editor SHALL validate the model format and display errors for invalid files
6. THE import functionality SHALL support file upload or paste from clipboard
