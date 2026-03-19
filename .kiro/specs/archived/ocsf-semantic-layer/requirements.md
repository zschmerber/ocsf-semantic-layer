# Requirements Document

## Introduction

This feature defines a Semantic Layer for the Open Cybersecurity Schema Framework (OCSF) that bridges the gap between OCSF's physical schema (JSON-based event classes, categories, attributes) and business-level security concepts. The system will provide visual representation of the semantic-to-physical layer interchange and leverage vector embeddings to assist with intelligent data mapping when security telemetry lands in a data warehouse.

## Glossary

- **OCSF**: Open Cybersecurity Schema Framework - an open-source, vendor-agnostic schema for security events
- **Semantic_Layer**: A logical abstraction layer that defines business-level security concepts, metrics, and relationships on top of physical data models
- **Physical_Layer**: The underlying OCSF JSON schema including event classes, categories, objects, and attributes
- **Event_Class**: An OCSF template defining required/optional fields for a specific security activity type (e.g., Authentication, Network Activity)
- **Category**: A grouping of related event classes in OCSF (e.g., System Activity, Identity & Access Management)
- **Vector_Embedding**: A numerical representation of data that captures semantic meaning, enabling similarity-based search
- **Mapping_Engine**: The component that translates between semantic queries and physical OCSF schema
- **Warehouse**: The target data storage system (Snowflake, Databricks, BigQuery, etc.) where OCSF data lands
- **Entity**: A business-level security concept (e.g., User, Device, Threat, Vulnerability) that may span multiple OCSF event classes

## Requirements

### Requirement 1: OCSF Schema Ingestion

**User Story:** As a security data engineer, I want to ingest and parse the OCSF schema definition, so that I can build a semantic layer on top of the physical schema structure.

#### Acceptance Criteria

1. WHEN the system starts, THE Schema_Ingester SHALL load OCSF schema JSON files from a configurable source (local path or GitHub repository)
2. WHEN parsing OCSF schema, THE Schema_Ingester SHALL extract all categories, event classes, objects, and attributes with their metadata
3. WHEN an attribute has type information, THE Schema_Ingester SHALL preserve data types, requirement flags (required/recommended/optional), and descriptions
4. IF the OCSF schema version changes, THEN THE Schema_Ingester SHALL detect version differences and support schema evolution
5. WHEN schema ingestion completes, THE Schema_Ingester SHALL produce a normalized internal representation suitable for semantic mapping

### Requirement 2: Semantic Entity Definition

**User Story:** As a security analyst, I want to define semantic entities that represent business-level security concepts, so that I can query security data using familiar terminology rather than raw OCSF field names.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support defining entities that map to one or more OCSF event classes
2. WHEN a user defines a semantic entity, THE Semantic_Layer SHALL allow specification of entity attributes with business-friendly names
3. WHEN mapping entity attributes, THE Semantic_Layer SHALL support expressions that combine or transform multiple OCSF fields
4. THE Semantic_Layer SHALL support defining relationships between entities (e.g., User performs Authentication on Device)
5. WHEN an entity definition references non-existent OCSF fields, THE Semantic_Layer SHALL report validation errors with specific field references

### Requirement 3: Semantic Metrics and Dimensions

**User Story:** As a security operations manager, I want to define security metrics and dimensions in business terms, so that I can create consistent KPIs across different security data sources.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support defining metrics with aggregation functions (count, sum, avg, min, max, distinct count)
2. WHEN defining a metric, THE Semantic_Layer SHALL require specification of the underlying OCSF fields and calculation logic
3. THE Semantic_Layer SHALL support defining dimensions for slicing metrics (e.g., by severity, by category, by time period)
4. WHEN a metric references time-based aggregation, THE Semantic_Layer SHALL support configurable time granularities
5. THE Semantic_Layer SHALL validate that metric definitions produce deterministic results across equivalent queries

### Requirement 4: Visual Layer Interchange Representation

**User Story:** As a data architect, I want to visualize how semantic concepts map to physical OCSF schema elements, so that I can understand and communicate the data model to stakeholders.

#### Acceptance Criteria

1. THE Visualization_Engine SHALL render a graph showing semantic entities and their relationships
2. THE Visualization_Engine SHALL render connections between semantic entities and underlying OCSF event classes
3. WHEN a user selects a semantic entity, THE Visualization_Engine SHALL highlight all connected OCSF elements (categories, event classes, attributes)
4. THE Visualization_Engine SHALL support different view modes: semantic-only, physical-only, and combined interchange view
5. WHEN displaying the interchange view, THE Visualization_Engine SHALL use visual differentiation (color, shape, line style) to distinguish semantic from physical elements
6. THE Visualization_Engine SHALL support exporting visualizations in standard formats (SVG, PNG, JSON graph format)

### Requirement 5: Vector Embedding Generation

**User Story:** As a data engineer, I want to generate vector embeddings for OCSF schema elements and semantic definitions, so that I can enable intelligent similarity-based mapping.

#### Acceptance Criteria

1. THE Embedding_Generator SHALL create vector embeddings for OCSF attribute names, descriptions, and sample values
2. THE Embedding_Generator SHALL create vector embeddings for semantic entity definitions and their descriptions
3. WHEN generating embeddings, THE Embedding_Generator SHALL use a configurable embedding model (default: sentence-transformers or OpenAI embeddings)
4. THE Embedding_Generator SHALL store embeddings in a vector store for efficient similarity search
5. WHEN new schema elements are added, THE Embedding_Generator SHALL incrementally update embeddings without full regeneration

### Requirement 6: Intelligent Mapping Assistance

**User Story:** As a data engineer mapping source data to OCSF, I want AI-assisted suggestions for field mappings, so that I can accelerate the mapping process and reduce errors.

#### Acceptance Criteria

1. WHEN a user provides source field metadata, THE Mapping_Assistant SHALL suggest matching OCSF attributes ranked by similarity score
2. THE Mapping_Assistant SHALL use vector similarity search to find semantically related OCSF fields
3. WHEN multiple mapping candidates exist, THE Mapping_Assistant SHALL explain the reasoning for each suggestion
4. THE Mapping_Assistant SHALL learn from user-confirmed mappings to improve future suggestions
5. IF a source field has no good OCSF match, THEN THE Mapping_Assistant SHALL suggest creating a custom extension attribute

### Requirement 7: Warehouse Integration

**User Story:** As a data platform engineer, I want to generate warehouse-specific artifacts from the semantic layer, so that I can deploy the semantic model to my target data warehouse.

#### Acceptance Criteria

1. THE Warehouse_Generator SHALL produce semantic layer definitions compatible with target warehouse formats
2. THE Warehouse_Generator SHALL support generating dbt semantic layer YAML files
3. THE Warehouse_Generator SHALL support generating Cube.js schema files
4. WHEN generating warehouse artifacts, THE Warehouse_Generator SHALL include OCSF-to-warehouse type mappings
5. THE Warehouse_Generator SHALL generate SQL views that implement semantic entity definitions on top of physical OCSF tables
6. WHEN the semantic model changes, THE Warehouse_Generator SHALL produce migration scripts for existing warehouse deployments

### Requirement 8: Query Translation

**User Story:** As a security analyst, I want to query data using semantic terms and have it translated to proper OCSF-based queries, so that I can analyze security data without knowing the physical schema details.

#### Acceptance Criteria

1. WHEN a user submits a semantic query, THE Query_Translator SHALL convert it to equivalent SQL against OCSF physical tables
2. THE Query_Translator SHALL resolve semantic entity references to their underlying OCSF event class joins
3. THE Query_Translator SHALL apply metric calculations as defined in the semantic layer
4. IF a semantic query references undefined entities or metrics, THEN THE Query_Translator SHALL return descriptive error messages
5. THE Query_Translator SHALL optimize generated SQL for the target warehouse dialect

### Requirement 9: Semantic Model Persistence

**User Story:** As a data team lead, I want to version and persist semantic model definitions, so that I can track changes and collaborate with my team.

#### Acceptance Criteria

1. THE Model_Store SHALL persist semantic model definitions in YAML format
2. THE Model_Store SHALL support versioning of semantic model definitions
3. WHEN loading a semantic model, THE Model_Store SHALL validate it against the current OCSF schema version
4. THE Model_Store SHALL support importing and exporting semantic models for sharing between environments
5. WHEN a semantic model is modified, THE Model_Store SHALL track change history with timestamps and optional change descriptions

### Requirement 10: Observable Analysis and Semantic Integration

**User Story:** As a security architect, I want to understand how OCSF observables relate to semantic entities, so that I can determine whether observables are still needed or can be superseded by the semantic layer.

#### Acceptance Criteria

1. THE Observable_Analyzer SHALL extract and catalog all observable definitions from the OCSF schema (by type, attribute, object, event class, and attribute path)
2. THE Observable_Analyzer SHALL map each observable type_id to corresponding semantic entities that reference the same underlying data
3. WHEN a semantic entity fully covers an observable's query use case, THE Observable_Analyzer SHALL flag the observable as "semantically redundant"
4. THE Observable_Analyzer SHALL generate a compatibility report showing which observables are essential, redundant, or partially covered by semantic definitions
5. WHEN generating warehouse artifacts, THE Warehouse_Generator SHALL optionally exclude observable arrays if semantic layer provides equivalent query capability
6. THE Semantic_Layer SHALL support defining semantic entities that directly reference observable type_ids for backward compatibility
7. THE Visualization_Engine SHALL display observable-to-semantic-entity mappings in the interchange view, highlighting coverage gaps and redundancies

### Requirement 11: Observable Hot Path Analytics Support

**User Story:** As a threat intelligence analyst, I want to leverage the observable extraction pattern for high-speed analytics, so that I can run threat intel matching against a denormalized observable table and reverse-lookup original OCSF events when matches are found.

#### Acceptance Criteria

1. THE Warehouse_Generator SHALL generate a dedicated observables table schema that consolidates observables from all OCSF event classes
2. THE Observable_Table SHALL include columns for: observable type_id, observable value, source event UID, source event class_uid, event timestamp, and original event reference
3. WHEN OCSF events are ingested, THE ETL_Pipeline SHALL extract observables into the dedicated observables table while preserving reverse-lookup capability
4. THE Semantic_Layer SHALL support defining threat intel matching metrics that operate on the observables table
5. WHEN a threat intel match is found in the observables table, THE Query_Translator SHALL generate efficient reverse-lookup queries to retrieve full original OCSF event data
6. THE Semantic_Layer SHALL support defining "hot path" entities that query the observables table for high-speed analytics and "cold path" entities that query full event tables for detailed investigation
7. THE Visualization_Engine SHALL display the hot path (observables table) and cold path (full events) data flow in the architecture view
8. THE Warehouse_Generator SHALL generate materialized view definitions for the observables table optimized for threat intel matching workloads
