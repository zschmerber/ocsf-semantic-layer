# Requirements Document

## Introduction

This document specifies requirements for a browser-based GUI for editing OCSF semantic models with LLM-assisted research capabilities. The editor enables security data engineers to visually browse OCSF schemas, create and modify semantic entities and metrics, and leverage LLM research to enrich models with descriptions, synonyms, and security context.

## Glossary

- **Editor**: The browser-based semantic model editing application
- **Schema_Browser**: Component for navigating OCSF categories, event classes, objects, and attributes
- **Entity_Editor**: Component for creating and modifying semantic entities with field mappings
- **Metric_Builder**: Component for defining semantic metrics with aggregations and dimensions
- **LLM_Assistant**: Component that queries LLMs to generate semantic enrichments
- **Validation_Engine**: Component that validates mappings against the OCSF schema in real-time
- **API_Server**: Rust backend serving JSON API endpoints for the frontend
- **Semantic_Model**: A YAML file containing entities, metrics, and observable configurations
- **OCSF_Schema**: The compiled OCSF schema JSON containing categories, classes, objects, and attributes

## Requirements

### Requirement 1: Schema Browser

**User Story:** As a security data engineer, I want to browse the OCSF schema hierarchy visually, so that I can understand available event classes and attributes for mapping.

#### Acceptance Criteria

1. WHEN the Editor loads, THE Schema_Browser SHALL display OCSF categories as expandable tree nodes
2. WHEN a user expands a category, THE Schema_Browser SHALL display event classes within that category
3. WHEN a user expands an event class, THE Schema_Browser SHALL display all attributes with their types and descriptions
4. WHEN a user selects an attribute, THE Schema_Browser SHALL display detailed metadata including type, requirement level, and enum values
5. THE Schema_Browser SHALL support text search to filter categories, classes, and attributes by name or description
6. WHEN a user searches, THE Schema_Browser SHALL highlight matching nodes and collapse non-matching branches

### Requirement 2: Semantic Entity Editor

**User Story:** As a security data engineer, I want to create and edit semantic entities through a visual interface, so that I can define business-level concepts without manually editing YAML.

#### Acceptance Criteria

1. THE Entity_Editor SHALL display a form for creating new semantic entities with name, caption, and description fields
2. WHEN a user creates an entity, THE Entity_Editor SHALL allow selection of source OCSF event classes from a dropdown
3. THE Entity_Editor SHALL display a drag-drop interface for mapping OCSF attributes to semantic attributes
4. WHEN a user drags an OCSF attribute to the entity, THE Entity_Editor SHALL create a semantic attribute with auto-populated field mapping
5. THE Entity_Editor SHALL allow editing of semantic attribute properties including name, caption, type, and dimension flag
6. WHEN a user modifies an entity, THE Entity_Editor SHALL update the in-memory model immediately
7. THE Entity_Editor SHALL display existing entities in a list with edit and delete actions
8. WHEN a user deletes an entity, THE Entity_Editor SHALL prompt for confirmation before removal

### Requirement 3: Metric Builder

**User Story:** As a security data engineer, I want to define metrics visually with aggregation selection, so that I can create reusable KPIs without writing expressions manually.

#### Acceptance Criteria

1. THE Metric_Builder SHALL display a form for creating metrics with name, caption, and description fields
2. THE Metric_Builder SHALL provide a dropdown for selecting aggregation type (count, sum, avg, min, max)
3. WHEN a user selects an aggregation, THE Metric_Builder SHALL display appropriate measure configuration options
4. THE Metric_Builder SHALL allow selection of dimensions from available entity attributes
5. THE Metric_Builder SHALL allow selection of time granularities (minute, hour, day, week, month)
6. WHEN a user creates a metric with expression measure, THE Metric_Builder SHALL provide a SQL expression editor with syntax highlighting
7. THE Metric_Builder SHALL display existing metrics in a list with edit and delete actions

### Requirement 4: LLM Research Assistant

**User Story:** As a security data engineer, I want LLM assistance to generate descriptions, synonyms, and security context, so that I can enrich semantic models with minimal manual effort.

#### Acceptance Criteria

1. THE LLM_Assistant SHALL provide a research panel accessible from entity and attribute editors
2. WHEN a user requests research for an entity, THE LLM_Assistant SHALL query the LLM for a description based on OCSF class metadata
3. WHEN a user requests research for an attribute, THE LLM_Assistant SHALL generate synonyms relevant to security analytics
4. THE LLM_Assistant SHALL generate security context explaining threat detection relevance for observable attributes
5. WHEN the LLM returns suggestions, THE LLM_Assistant SHALL display them in a review panel with accept/reject actions
6. WHEN a user accepts a suggestion, THE LLM_Assistant SHALL apply it to the corresponding entity or attribute
7. THE LLM_Assistant SHALL support batch research to enrich multiple attributes in a single operation
8. IF the LLM request fails, THEN THE LLM_Assistant SHALL display an error message with retry option

### Requirement 5: Real-time Validation

**User Story:** As a security data engineer, I want real-time validation of my mappings, so that I can catch errors before exporting the model.

#### Acceptance Criteria

1. WHEN a user modifies a field mapping, THE Validation_Engine SHALL validate the path against the OCSF schema within 500ms
2. IF a field path is invalid, THEN THE Validation_Engine SHALL display an inline error with the invalid path highlighted
3. THE Validation_Engine SHALL validate that referenced event classes exist in the loaded schema
4. THE Validation_Engine SHALL validate that metric dimensions reference existing entity attributes
5. WHEN validation errors exist, THE Editor SHALL display a validation summary panel with clickable error links
6. WHEN a user clicks a validation error, THE Editor SHALL navigate to the corresponding entity or metric

### Requirement 6: Export and Import

**User Story:** As a security data engineer, I want to save and load semantic models as YAML files, so that I can persist my work and share models with teammates.

#### Acceptance Criteria

1. THE Editor SHALL provide an export button that downloads the current model as a YAML file
2. THE Editor SHALL provide an import button that loads a semantic model from a YAML file
3. WHEN importing a model, THE Editor SHALL validate the YAML structure before loading
4. IF the imported YAML is invalid, THEN THE Editor SHALL display parsing errors with line numbers
5. THE Editor SHALL support auto-save to browser local storage every 30 seconds
6. WHEN the Editor loads, THE Editor SHALL restore the last auto-saved model if available
7. THE Editor SHALL provide a "New Model" action that clears the current model after confirmation

### Requirement 7: API Server Integration

**User Story:** As a developer, I want the frontend to communicate with the Rust backend via JSON API, so that I can leverage existing ocsf-cli functionality.

#### Acceptance Criteria

1. THE API_Server SHALL expose a GET /api/schema endpoint that returns the loaded OCSF schema as JSON
2. THE API_Server SHALL expose a POST /api/validate endpoint that validates a semantic model against the schema
3. THE API_Server SHALL expose a POST /api/generate endpoint that generates warehouse artifacts from a model
4. THE API_Server SHALL expose a POST /api/llm/research endpoint that queries the LLM for semantic enrichments
5. WHEN an API request fails, THE API_Server SHALL return appropriate HTTP status codes with error details
6. THE API_Server SHALL support CORS for local development with configurable allowed origins

### Requirement 8: User Interface Design

**User Story:** As a user, I want a dark-themed interface consistent with the existing dashboard, so that I have a cohesive visual experience.

#### Acceptance Criteria

1. THE Editor SHALL use a dark theme matching the existing demo/viz/index.html color scheme
2. THE Editor SHALL display a header with application title and primary navigation tabs
3. THE Editor SHALL use a split-pane layout with schema browser on the left and editor panels on the right
4. THE Editor SHALL provide keyboard shortcuts for common actions (save, undo, redo)
5. THE Editor SHALL display loading indicators during API operations
6. THE Editor SHALL be responsive and usable on screens 1280px wide and larger
