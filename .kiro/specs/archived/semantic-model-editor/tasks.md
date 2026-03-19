# Implementation Plan: Semantic Model Editor

## Overview

This plan implements a browser-based GUI for editing OCSF semantic models with LLM-assisted research. The implementation follows a backend-first approach, building the Rust API server before the React frontend, ensuring stable APIs for frontend development.

## Tasks

- [x] 1. Set up project structure and dependencies
  - [x] 1.1 Create ocsf-editor crate with Axum dependencies
    - Add new crate to Cargo.toml workspace
    - Add dependencies: axum, tower, tower-http (cors), serde_json
    - Create basic main.rs with Axum server skeleton
    - _Requirements: 7.1, 7.6_
  
  - [x] 1.2 Set up frontend project with Vite and React
    - Create editor-ui directory with Vite + React + TypeScript template
    - Add dependencies: zustand, @tanstack/react-query, react-dnd, monaco-editor
    - Configure Vite proxy for API development
    - _Requirements: 8.1, 8.2_

- [x] 2. Implement Schema Service and API
  - [x] 2.1 Create SchemaService for loading and caching OCSF schema
    - Implement schema loading from file path
    - Build tree structure from CompiledSchema
    - Add search functionality with name/description matching
    - _Requirements: 1.1, 1.5_
  
  - [x] 2.2 Implement GET /api/schema endpoint
    - Return schema tree as JSON
    - Include categories, classes, objects, attributes
    - Add caching headers for browser caching
    - _Requirements: 7.1_
  
  - [ ]* 2.3 Write property test for schema search accuracy
    - **Property 4: Search Result Accuracy**
    - **Validates: Requirements 1.5**

- [x] 3. Implement Validation Service and API
  - [x] 3.1 Create ValidationService for model validation
    - Implement field path validation against schema
    - Implement event class existence validation
    - Implement dimension reference validation
    - _Requirements: 5.1, 5.2, 5.3, 5.4_
  
  - [x] 3.2 Implement POST /api/validate endpoint
    - Accept SemanticModel JSON body (currently returns placeholder)
    - Integrate ValidationService to return real validation results
    - Include path information for error navhccdxxfvgbg nhigation
    - _Requirements: 7.2_
  
  - [ ]* 3.3 Write property tests for validation
    - **Property 14: Invalid Path Error Detection**
    - **Property 15: Event Class Existence Validation**
    - **Property 16: Dimension Reference Validation**
    - **Validates: Requirements 5.2, 5.3, 5.4**

- [x] 4. Checkpoint - Backend validation complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement LLM Service and API
  - [x] 5.1 Create LLMService for semantic enrichment
    - Implement OpenAI API client with configurable model
    - Build prompts for entity description generation
    - Build prompts for attribute synonym generation
    - Build prompts for security context generation
    - _Requirements: 4.2, 4.3, 4.4_
  
  - [x] 5.2 Implement POST /api/llm/research endpoint
    - Accept ResearchRequest with target type and context (currently returns placeholder)
    - Integrate LLMService to return real suggestions with confidence scores
    - Support batch research for multiple targets
    - _Requirements: 7.4, 4.7_
  
  - [ ]* 5.3 Write property tests for LLM service
    - **Property 9: LLM Research Context Inclusion**
    - **Property 12: Batch Research Single Request**
    - **Validates: Requirements 4.2, 4.7**

- [-] 6. Implement Generation API
  - [x] 6.1 Implement POST /api/generate endpoint
    - Accept model, dialect, and artifact types (currently returns placeholder)
    - Integrate existing ocsf-warehouse generation functions
    - Return generated files as JSON array
    - _Requirements: 7.3_
  
  - [ ]* 6.2 Write property test for generation output
    - **Property 20: Generation API Artifact Output**
    - **Validates: Requirements 7.3**

- [x] 7. Implement API error handling
  - [x] 7.1 Create EditorApiError enum with thiserror
    - Define error variants for all failure cases
    - Implement status code mapping
    - Implement IntoResponse for Axum
    - _Requirements: 7.5_
  
  - [ ]* 7.2 Write property test for error responses
    - **Property 21: API Error Response Format**
    - **Validates: Requirements 7.5**

- [x] 8. Checkpoint - Backend API complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Implement frontend state management
  - [x] 9.1 Create Zustand store for editor state
    - Define EditorState interface with model, schema, UI state
    - Implement entity CRUD actions
    - Implement metric CRUD actions
    - Implement validation state management
    - _Requirements: 2.6, 2.7, 3.7_
  
  - [x] 9.2 Implement localStorage auto-save
    - Save model state every 30 seconds
    - Restore state on editor load
    - Handle storage quota errors
    - _Requirements: 6.5, 6.6_
  
  - [ ]* 9.3 Write property test for state restoration
    - **Property 18: Auto-Save State Restoration**
    - **Validates: Requirements 6.6**

- [x] 10. Implement Schema Browser component
  - [x] 10.1 Create SchemaTreeNode component
    - Render expandable tree nodes for categories, classes, attributes
    - Handle expand/collapse state
    - Display attribute metadata on selection
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  
  - [x] 10.2 Create schema search functionality
    - Add search input with debounced filtering
    - Filter tree nodes by name/description match
    - Highlight matching text in results
    - _Requirements: 1.5, 1.6_
  
  - [ ]* 10.3 Write property tests for schema tree
    - **Property 1: Schema Tree Expansion Consistency**
    - **Property 2: Class Attribute Completeness**
    - **Property 3: Attribute Metadata Display**
    - **Validates: Requirements 1.2, 1.3, 1.4**

- [x] 11. Implement Entity Editor component
  - [x] 11.1 Create EntityForm component
    - Form fields for name, caption, description
    - Event class multi-select dropdown
    - Entity list with edit/delete actions
    - _Requirements: 2.1, 2.2, 2.7, 2.8_
  
  - [x] 11.2 Create AttributeMappingZone with drag-drop
    - Implement react-dnd drop zone for OCSF attributes
    - Auto-create semantic attribute on drop
    - Attribute property editor (name, type, dimension flag)
    - _Requirements: 2.3, 2.4, 2.5_
  
  - [ ]* 11.3 Write property tests for entity editing
    - **Property 5: Drag-Drop Mapping Creation**
    - **Property 6: Entity Modification State Sync**
    - **Validates: Requirements 2.4, 2.6**

- [x] 12. Implement Metric Builder component
  - [x] 12.1 Create MetricForm component
    - Form fields for name, caption, description
    - Aggregation type dropdown
    - Measure configuration (field or expression)
    - _Requirements: 3.1, 3.2, 3.3, 3.6_
  
  - [x] 12.2 Create dimension and granularity selectors
    - Multi-select for dimensions from entity attributes
    - Checkbox group for time granularities
    - Metric list with edit/delete actions
    - _Requirements: 3.4, 3.5, 3.7_
  
  - [ ]* 12.3 Write property tests for metric builder
    - **Property 7: Aggregation-Specific Options**
    - **Property 8: Dimension Selection from Entity Attributes**
    - **Validates: Requirements 3.3, 3.4**

- [x] 13. Checkpoint - Core editor components complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 14. Implement LLM Research Panel
  - [x] 14.1 Create LLMResearchPanel component
    - Research button in entity/attribute editors
    - Suggestion display with accept/reject actions
    - Loading and error states
    - _Requirements: 4.1, 4.5, 4.8_
  
  - [x] 14.2 Implement suggestion acceptance flow
    - Apply accepted suggestions to model state
    - Support batch research for multiple attributes
    - _Requirements: 4.6, 4.7_
  
  - [ ]* 14.3 Write property tests for LLM integration
    - **Property 10: Observable Security Context Generation**
    - **Property 11: Suggestion Acceptance Application**
    - **Validates: Requirements 4.4, 4.6**

- [x] 15. Implement Validation Panel
  - [x] 15.1 Create ValidationPanel component
    - Display validation errors and warnings
    - Clickable error links for navigation
    - Real-time validation on model changes
    - _Requirements: 5.5, 5.6_
  
  - [x] 15.2 Implement inline validation indicators
    - Show validation errors next to invalid fields
    - Debounce validation calls (500ms)
    - _Requirements: 5.1, 5.2_
  
  - [ ]* 15.3 Write property test for validation timing
    - **Property 13: Field Path Validation Timing**
    - **Validates: Requirements 5.1**

- [x] 16. Implement Export/Import functionality
  - [x] 16.1 Create export functionality
    - Export button downloads model as YAML file
    - Use existing SemanticModel.to_yaml()
    - _Requirements: 6.1_
  
  - [x] 16.2 Create import functionality
    - Import button opens file picker
    - Validate YAML structure before loading
    - Display parse errors with line numbers
    - _Requirements: 6.2, 6.3, 6.4_
  
  - [x] 16.3 Implement New Model action
    - Clear current model with confirmation dialog
    - _Requirements: 6.7_
  
  - [ ]* 16.4 Write property test for YAML validation
    - **Property 17: YAML Import Structure Validation**
    - **Validates: Requirements 6.3**

- [x] 17. Implement UI shell and navigation
  - [x] 17.1 Create App shell with header and tabs
    - Header with application title
    - Tab navigation (Schema, Entities, Metrics, Validation)
    - Split-pane layout
    - _Requirements: 8.2, 8.3_
  
  - [x] 17.2 Implement keyboard shortcuts
    - Ctrl+S for save/export
    - Ctrl+Z for undo
    - Ctrl+Y for redo
    - _Requirements: 8.4_
  
  - [x] 17.3 Add loading indicators
    - Show spinners during API operations
    - Disable interactions while loading
    - _Requirements: 8.5_
  
  - [ ]* 17.4 Write property test for keyboard shortcuts
    - **Property 22: Keyboard Shortcut Action Mapping**
    - **Validates: Requirements 8.4**

- [x] 18. Apply dark theme styling
  - [x] 18.1 Create CSS variables matching demo/viz theme
    - Background colors: #0d1117, #161b22, #21262d
    - Text colors: #c9d1d9, #8b949e
    - Accent colors: #58a6ff, #7ee787, #f0883e
    - _Requirements: 8.1_
  
  - [x] 18.2 Style all components with dark theme
    - Apply consistent styling across all components
    - Ensure responsive layout for 1280px+ screens
    - _Requirements: 8.1, 8.6_

- [x] 19. Final checkpoint - All features complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 20. Integration and wiring
  - [x] 20.1 Wire frontend to backend API
    - Configure TanStack Query for API calls
    - Set up error handling and retry logic
    - Test full flow: load schema → edit model → validate → export
    - _Requirements: 7.1, 7.2, 7.3, 7.4_
  
  - [x] 20.2 Add CLI command to start editor server
    - Add `ocsf-cli editor` command
    - Serve frontend static files from backend
    - Open browser on startup
    - _Requirements: 7.1_

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Backend is implemented first to provide stable APIs for frontend development
