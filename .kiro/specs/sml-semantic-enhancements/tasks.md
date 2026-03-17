# Implementation Plan: SML Semantic Enhancements

## Overview

This plan implements SML v1.4 enhancements across three layers: `ocsf-semantic` (new structs, field additions, validation), `ocsf-warehouse` (generator updates), and `editor-ui` (TypeScript types and components). Each task builds incrementally — core types first, then validation, then generators, then UI — ensuring no orphaned code.

## Tasks

- [x] 1. Add new structs and enums to ocsf-semantic
  - [x] 1.1 Create HierarchyLevel struct and MetricType enum
    - Add `HierarchyLevel` struct to `entity.rs` with `name` and `attribute_ref` fields, derive Serialize/Deserialize/Clone/PartialEq/Debug
    - Add `MetricType` enum (Additive, SemiAdditive, NonAdditive) to `metric.rs` with Default impl returning Additive
    - Add `is_false` serde helper to `lib.rs` if not already present
    - _Requirements: 1.1, 1.3, 2.1, 2.2_

  - [x] 1.2 Create Dataset struct in model.rs
    - Add `Dataset` struct with `name`, `dialect`, `table`, optional `schema_name`, optional `connection`
    - Use `#[serde(default, skip_serializing_if = "Option::is_none")]` on optional fields
    - _Requirements: 4.1_

  - [x] 1.3 Create catalog.rs with SemanticCatalog and CatalogEntry
    - Create new file `ocsf-semantic/src/catalog.rs`
    - Implement `SemanticCatalog` struct with `name`, `version`, optional `description`, `models: Vec<CatalogEntry>`
    - Implement `CatalogEntry` struct with `model_name`, `path`, `version`, `dependencies: Vec<String>`
    - Implement `to_yaml` and `from_yaml` methods on `SemanticCatalog`
    - Register module in `lib.rs` and re-export types
    - _Requirements: 7.1, 7.2, 7.5_

  - [ ]* 1.4 Write property test for SemanticCatalog YAML round-trip
    - **Property 4: SemanticCatalog YAML round-trip**
    - **Validates: Requirements 7.6**

- [x] 2. Extend existing structs with new fields
  - [x] 2.1 Add new fields to SemanticAttribute in entity.rs
    - Add `hierarchy: Vec<HierarchyLevel>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
    - Add `is_hidden: bool` with `#[serde(default, skip_serializing_if = "crate::is_false")]`
    - Add `folder: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
    - Update any builder methods to initialize new fields with defaults
    - Ensure attribute with non-empty `hierarchy` is treated as dimension regardless of `is_dimension`
    - _Requirements: 1.1, 1.2, 6.1, 6.2_

  - [x] 2.2 Add new fields to SemanticMetric in metric.rs
    - Add `metric_type: MetricType` with `#[serde(default)]`
    - Add `formula: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
    - Add `non_additive_dimensions: Vec<String>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
    - Add `is_hidden: bool` with `#[serde(default, skip_serializing_if = "crate::is_false")]`
    - Add `folder: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
    - Update any builder methods to initialize new fields with defaults
    - _Requirements: 2.1, 2.3, 3.1, 3.2, 6.3, 6.4_

  - [x] 2.3 Add datasets to SemanticModel and dataset_ref to SemanticEntity
    - Add `datasets: Vec<Dataset>` to `SemanticModel` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
    - Add `dataset_ref: Option<String>` to `SemanticEntity` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
    - Update builder methods as needed
    - _Requirements: 4.2, 4.3, 4.5_

  - [x] 2.4 Add role_alias to EntityRelationship in entity.rs
    - Add `role_alias: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
    - Update `to_join_sql` method to use `role_alias` as SQL table alias when present
    - _Requirements: 5.1, 5.3_

  - [ ]* 2.5 Write property tests for YAML round-trip on modified structs
    - **Property 1: SemanticEntity YAML round-trip** — entity with hierarchies, role aliases, hidden/folder attributes
    - **Property 2: SemanticMetric YAML round-trip** — metric with metric_type, formula, non_additive_dimensions, hidden/folder
    - **Property 3: SemanticModel YAML round-trip (with datasets)** — model with datasets and dataset_ref
    - **Validates: Requirements 1.7, 2.7, 3.7, 4.7, 5.5, 6.7**

  - [ ]* 2.6 Write property tests for hierarchy-implies-dimension and role alias JOIN SQL
    - **Property 5: Hierarchy implies dimension** — attribute with non-empty hierarchy appears in dimensions iterator
    - **Property 14: Role alias in JOIN SQL** — to_join_sql uses AS clause when role_alias set, omits when absent
    - **Property 15: Default visibility omission in YAML** — is_hidden=false and folder=None omitted from serialized YAML
    - **Validates: Requirements 1.2, 5.3, 6.5, 10.4**

- [x] 3. Implement formula reference extraction
  - [x] 3.1 Add extract_metric_references function
    - Implement `extract_metric_references(formula: &str) -> Vec<String>` in `metric.rs` or a suitable module
    - Extract metric name identifiers from formula strings with arithmetic operators
    - _Requirements: 3.5_

  - [ ]* 3.2 Write property test for formula metric name extraction
    - **Property 10: Formula metric name extraction**
    - **Validates: Requirements 3.5**

- [x] 4. Checkpoint — Core types compile and serialize
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement validation enhancements
  - [x] 5.1 Add new ValidationError variants to validation.rs
    - Add `InvalidHierarchyRef`, `DuplicateHierarchyRef`, `InvalidNonAdditiveDimension`, `InvalidFormulaRef`, `CircularMetricDependency`, `InvalidDatasetRef`, `MissingRoleAlias`, `DuplicateCatalogModelName`, `InvalidCatalogDependency` variants
    - _Requirements: 1.4, 1.5, 2.4, 3.3, 3.4, 4.4, 5.2, 7.3, 7.4_

  - [x] 5.2 Implement validate_hierarchies method
    - Verify every `attribute_ref` in a hierarchy resolves to an existing attribute in the same entity
    - Report error on duplicate `attribute_ref` values within a hierarchy
    - Wire into the main validation pass
    - _Requirements: 1.4, 1.5_

  - [x] 5.3 Implement validate_metric_type method
    - Verify every entry in `non_additive_dimensions` references a valid dimension attribute in at least one entity
    - Emit warning when semi-additive metric has empty `non_additive_dimensions`
    - Wire into the main validation pass
    - _Requirements: 2.4, 2.5_

  - [x] 5.4 Implement validate_formula method with circular dependency detection
    - Verify every metric name referenced in a formula exists in the model
    - Implement DFS-based circular dependency detection over the metric dependency graph
    - Wire into the main validation pass
    - _Requirements: 3.3, 3.4_

  - [x] 5.5 Implement validate_dataset_refs method
    - Verify every entity's `dataset_ref` resolves to a dataset in the model's `datasets` list
    - Wire into the main validation pass
    - _Requirements: 4.4_

  - [x] 5.6 Implement validate_role_aliases method
    - When two relationships in the same entity target the same entity, require distinct `role_alias` values
    - Wire into the main validation pass
    - _Requirements: 5.2_

  - [x] 5.7 Implement validate_catalog standalone function
    - Verify catalog dependency model names exist as other entries
    - Detect duplicate `model_name` values
    - _Requirements: 7.3, 7.4_

  - [ ]* 5.8 Write property tests for validation rules
    - **Property 6: Hierarchy attribute_ref validation** — invalid attribute_ref produces error
    - **Property 7: Non-additive dimension validation** — invalid dimension name produces error
    - **Property 8: Formula reference validation** — unknown metric name in formula produces error
    - **Property 9: Circular formula dependency detection** — cycle in metric graph produces error
    - **Property 11: Dataset ref validation** — invalid dataset_ref produces error
    - **Property 13: Role alias uniqueness validation** — duplicate target without distinct role_alias produces error
    - **Property 16: Catalog dependency validation** — missing dependency produces error
    - **Property 17: Catalog duplicate model name detection** — duplicate model_name produces error
    - **Validates: Requirements 1.4, 2.4, 3.3, 3.4, 4.4, 5.2, 7.3, 7.4**

- [x] 6. Checkpoint — Validation rules pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Update schema generator and metric suggester
  - [x] 7.1 Set metric_type based on aggregation in SchemaGenerator
    - Count/Sum aggregation → `MetricType::Additive`
    - Avg/Min/Max aggregation → `MetricType::NonAdditive`
    - Set `is_hidden: false`, `folder: None` on all generated attributes and metrics
    - Do not generate Dataset entries
    - _Requirements: 9.1, 9.4, 9.5_

  - [x] 7.2 Add hierarchy inference from OCSF object nesting
    - When an object has country/region/city fields, generate a geographic `HierarchyLevel` chain
    - Best-effort: no error if patterns don't match
    - _Requirements: 9.3_

  - [x] 7.3 Generate calculated metrics in MetricSuggester
    - When a class has a status field, generate a calculated success rate metric with `formula: Some("success_count / total_count".into())`
    - _Requirements: 9.2_

  - [ ]* 7.4 Write property tests for generator defaults
    - **Property 18: Generated metric type matches aggregation** — Count/Sum → Additive, Avg/Min/Max → NonAdditive
    - **Property 19: Generator default visibility and no datasets** — all generated attrs/metrics have is_hidden=false, folder=None, datasets empty
    - **Validates: Requirements 9.1, 9.4, 9.5**

- [ ] 8. Update warehouse generators
  - [x] 8.1 Update DBTGenerator for hierarchies, semi-additive measures, and derived metrics
    - Emit hierarchy levels as nested `dimensions` with `type: hierarchy` when attribute has non-empty hierarchy
    - Set `agg_time_dimension` constraints on semi-additive measures from `non_additive_dimensions`
    - Emit calculated metrics as `derived` metric type with `formula` expression
    - _Requirements: 10.1, 10.2, 10.3_

  - [x] 8.2 Update ViewGenerator for role aliases, dataset refs, and hidden exclusion
    - Use `role_alias` as SQL table alias in JOIN clauses
    - Resolve source table from `dataset_ref` → `Dataset.table` (qualified by `schema_name`), fallback to OCSF naming
    - Exclude `is_hidden` attributes and metrics from generated SQL views
    - _Requirements: 10.4, 10.5, 10.6_

  - [ ]* 8.3 Write property tests for warehouse generators
    - **Property 12: Dataset ref table resolution in views** — dataset_ref resolves to Dataset.table, fallback to OCSF naming
    - **Property 20: Hidden attributes excluded from generated views** — is_hidden=true attributes not in SELECT
    - **Property 21: DBT hierarchy dimension emission** — hierarchy attributes produce type: hierarchy dimensions
    - **Property 22: DBT semi-additive measure constraints** — SemiAdditive metrics get agg_time_dimension
    - **Property 23: DBT derived metric emission** — formula metrics emitted as derived type
    - **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5, 10.6**

- [x] 9. Checkpoint — Rust backend complete
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Update editor-ui TypeScript types
  - [x] 10.1 Add new types and update existing interfaces in types/index.ts
    - Add `HierarchyLevel` interface with `name` and `attribute_ref`
    - Add `MetricType` type alias (`'Additive' | 'SemiAdditive' | 'NonAdditive'`)
    - Add `Dataset` interface with `name`, `dialect`, `table`, optional `schema_name`, optional `connection`
    - Add optional `hierarchy`, `is_hidden`, `folder` to `SemanticAttribute`
    - Add optional `metric_type`, `formula`, `non_additive_dimensions`, `is_hidden`, `folder` to `SemanticMetric`
    - Add optional `datasets` to `SemanticModel`
    - Add optional `dataset_ref` to `SemanticEntity`
    - Add optional `role_alias` to `EntityRelationship`
    - _Requirements: 8.1_

- [ ] 11. Implement editor-ui components
  - [x] 11.1 Update AttributeEditor with hierarchy editor, hidden toggle, and folder input
    - Add `is_hidden` toggle control
    - Add `folder` text input
    - Add hierarchy level editor (ordered list with add/remove/reorder) shown when attribute is a dimension
    - Visually distinguish hidden items in list views (dimmed or tagged)
    - _Requirements: 8.2, 8.3, 8.8_

  - [x] 11.2 Update EntityForm with dataset_ref dropdown
    - Add `dataset_ref` dropdown populated from `model.datasets`
    - _Requirements: 8.4_

  - [x] 11.3 Create MetricEditor component
    - Create `editor-ui/src/components/MetricEditor/MetricEditor.tsx`
    - Implement `metric_type` select (Additive/SemiAdditive/NonAdditive)
    - Implement `formula` textarea shown when metric is calculated
    - Implement `non_additive_dimensions` multi-select shown when SemiAdditive
    - Add `is_hidden` toggle and `folder` text input
    - Visually distinguish hidden metrics in list views
    - _Requirements: 8.5, 8.8_

  - [x] 11.4 Create DatasetPanel component
    - Create `editor-ui/src/components/DatasetPanel/DatasetPanel.tsx`
    - Implement CRUD panel for Dataset entries on the model
    - _Requirements: 8.6_

  - [x] 11.5 Update relationship editor with role_alias input
    - Add `role_alias` text input to the EntityRelationship editor section
    - _Requirements: 8.7_

  - [x] 11.6 Add folder grouping to list views
    - When `folder` is set on attributes or metrics, group items by folder in list views
    - _Requirements: 8.9_

- [ ] 12. Update editor store
  - [x] 12.1 Add dataset and visibility actions/selectors to editorStore.ts
    - Add `addDataset`, `updateDataset`, `removeDataset` actions
    - Add `updateEntityDatasetRef` action
    - Add `selectDatasets`, `selectVisibleAttributes`, `selectAttributesByFolder` selectors
    - _Requirements: 8.1, 8.6_

- [x] 13. Final checkpoint — All layers integrated
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- All new serde fields use `#[serde(default)]` and `skip_serializing_if` for backward compatibility
- Property tests use `proptest` with minimum 100 iterations per property
- Each property test references its design property number and validated requirements
- Checkpoints ensure incremental validation across the three layers
