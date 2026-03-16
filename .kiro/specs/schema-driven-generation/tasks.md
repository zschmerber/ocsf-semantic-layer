# Implementation Plan: Schema-Driven Semantic Layer Generation

## Overview

This implementation plan converts the compiled OCSF schema into a semantic layer by parsing the official schema JSON, generating entities from objects and classes, inferring dimensions from metadata, and suggesting metrics based on attribute patterns. The implementation is in Rust using the existing workspace structure.

## Tasks

- [x] 1. Add compiled schema types to ocsf-core
  - [ ] 1.1 Create compiled schema data structures
    - Add `CompiledSchema`, `CategoryDef`, `ClassDef`, `ObjectDef`, `AttributeDef`, `EnumValue` structs
    - Add `Requirement` enum (Required, Recommended, Optional)
    - Place in new file `ocsf-core/src/compiled_schema.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [ ]* 1.2 Write property test for schema parsing round-trip
    - **Property 1: Schema Parsing Round-Trip**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

  - [ ] 1.3 Implement schema parser
    - Parse compiled OCSF JSON format
    - Handle version extraction
    - Parse categories, classes, objects, profiles
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [ ]* 1.4 Write property test for malformed schema handling
    - **Property 2: Malformed Schema Error Handling**
    - **Validates: Requirements 1.6**

- [x] 2. Checkpoint - Ensure schema parsing works
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement dimension inference in ocsf-semantic
  - [x] 3.1 Create dimension inference types
    - Add `DimensionInfo`, `DimensionType`, `DimensionPriority` structs
    - Add `DimensionInferrer` trait
    - Place in new file `ocsf-semantic/src/dimension_inference.rs`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

  - [x] 3.2 Implement default dimension inference rules
    - Enum attributes → Categorical dimension
    - Required/primary group → High priority
    - Timestamp → Temporal dimension
    - Observable → Searchable dimension
    - Sibling attribute linking
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

  - [ ]* 3.3 Write property test for dimension inference
    - **Property 8: Enum Dimension Inference**
    - **Property 9: Dimension Inference Rules**
    - **Property 10: Sibling Attribute Linking**
    - **Validates: Requirements 2.6, 3.6, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6**

- [x] 4. Implement path generation
  - [x] 4.1 Create path types and parser
    - Add `FieldPath`, `PathSegment` structs
    - Implement path parsing from dot notation
    - Implement path serialization to string
    - Place in new file `ocsf-semantic/src/field_path.rs`
    - _Requirements: 6.1, 6.2, 6.5_

  - [ ]* 4.2 Write property test for path round-trip
    - **Property 13: Path Round-Trip**
    - **Validates: Requirements 6.5**

  - [x] 4.3 Implement path generator
    - Generate paths for nested object references
    - Handle array attributes
    - Resolve object type references
    - Respect max nesting depth
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ]* 4.4 Write property tests for path generation
    - **Property 5: Nested Path Generation**
    - **Property 14: Array Path Indication**
    - **Property 15: Object Type Resolution**
    - **Validates: Requirements 2.5, 6.1, 6.2, 6.3, 6.4**

- [x] 5. Checkpoint - Ensure dimension and path modules work
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement entity generation
  - [x] 6.1 Create generation config and types
    - Add `GenerationConfig` struct with filters
    - Add `GenerationMetadata`, `GenerationStats` structs
    - Define `HIGH_USAGE_OBJECTS` constant
    - Place in new file `ocsf-semantic/src/schema_generator.rs`
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [x] 6.2 Implement entity generation from objects
    - Generate semantic entities from OCSF objects
    - Include required/recommended attributes
    - Apply dimension inference to attributes
    - Generate nested paths for object references
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ]* 6.3 Write property tests for object entity generation
    - **Property 3: Object Entity Generation Completeness**
    - **Property 4: Object Attribute Inclusion**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4**

  - [x] 6.4 Implement entity generation from classes
    - Generate semantic entities from OCSF classes
    - Preserve class UID and category info
    - Flatten nested object attributes with paths
    - Apply dimension inference
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

  - [ ]* 6.5 Write property tests for class entity generation
    - **Property 6: Class Entity Generation**
    - **Property 7: Class Attribute Flattening**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

- [x] 7. Implement metric suggestion
  - [x] 7.1 Create metric suggestion types
    - Add `SuggestedMetric` struct
    - Add `MetricSuggester` trait
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [x] 7.2 Implement default metric suggester
    - Always suggest count metric
    - Suggest duration metrics when duration attribute present
    - Suggest success rate when status_id present
    - Suggest severity metrics when severity_id present
    - Associate appropriate dimensions
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [ ]* 7.3 Write property tests for metric suggestion
    - **Property 11: Metric Suggestion Completeness**
    - **Property 12: Metric Dimension Association**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6**

- [x] 8. Checkpoint - Ensure entity and metric generation work
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Implement filter application
  - [x] 9.1 Implement category, class, and object filters
    - Apply category filter to class generation
    - Apply class filter to class generation
    - Apply object filter to object generation
    - Handle empty filters (include all)
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [ ]* 9.2 Write property tests for filter application
    - **Property 16: Category Filter Application**
    - **Property 17: Class Filter Application**
    - **Property 18: Object Filter Application**
    - **Property 19: Default Generation Coverage**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4**

- [x] 10. Implement model output
  - [x] 10.1 Implement semantic model serialization
    - Serialize generated model to YAML
    - Include generation metadata
    - Include schema version and timestamp
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

  - [ ]* 10.2 Write property test for model round-trip
    - **Property 20: Semantic Model Round-Trip**
    - **Property 21: Generation Metadata Inclusion**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.5**

- [x] 11. Implement CLI command
  - [x] 11.1 Add generate-from-schema command
    - Add `generate-from-schema` subcommand to ocsf-cli
    - Accept `--schema` path argument
    - Accept `--output` path argument
    - Accept optional filter arguments
    - _Requirements: 9.1, 9.2, 9.3, 9.4_

  - [x] 11.2 Implement command execution
    - Load and parse schema file
    - Apply configuration and filters
    - Generate semantic model
    - Write output YAML file
    - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 12. Integration testing
  - [x] 12.1 Test with real OCSF schema
    - Run generation against `demo/schema/ocsf-compiled-v1.6.0.json`
    - Verify output model structure
    - Verify entity count matches expectations
    - _Requirements: 10.1, 10.2_

  - [x] 12.2 Update demo scripts
    - Update `demo/run-e2e-local.sh` to use generated model
    - Verify end-to-end workflow
    - _Requirements: 10.1, 10.2_

- [x] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Implementation uses existing `ocsf-semantic` entity types where possible
