# Requirements: SML Semantic Enhancements

## Introduction

This specification enhances the `ocsf-semantic` crate to align with SML (Semantic Modeling Language) v1.4 patterns and universal semantic model best practices. The enhancements add dimension hierarchies, semi-additive and calculated metric support, dataset/connection abstraction, role-playing relationships, and UI organization metadata. These changes deepen the semantic layer's expressiveness while preserving backward compatibility with existing YAML models and downstream warehouse artifact generators (`ocsf-warehouse`) and the editor GUI (`editor-ui`).

## Glossary

- **Semantic_Layer**: The `ocsf-semantic` crate providing semantic entity definitions, metrics, and validation on top of OCSF physical schema.
- **SemanticAttribute**: A named attribute within a `SemanticEntity`, optionally marked as a dimension, with an OCSF field mapping.
- **SemanticMetric**: A named metric with an aggregation function, measure mapping, allowed dimensions, and time granularities.
- **SemanticEntity**: A logical grouping of attributes and relationships representing a business-level security concept.
- **SemanticModel**: The top-level container holding entities, metrics, observable config, and model metadata.
- **SemanticModelStore**: A mutable wrapper around `SemanticModel` providing CRUD operations, indexing, and change history.
- **Dimension_Hierarchy**: An ordered sequence of levels within a dimension that supports drill-down navigation (e.g., Country → Region → City → IP).
- **Hierarchy_Level**: A single step in a `Dimension_Hierarchy`, referencing an attribute name and its position in the drill path.
- **Metric_Type**: A classification of a metric as additive, semi-additive, or non-additive, controlling which dimensions it can be aggregated across.
- **Calculated_Metric**: A derived metric whose value is computed from an expression referencing other metrics, rather than a raw OCSF field.
- **Dataset**: An abstraction representing a physical data source (connection string, dialect, table/view name) decoupled from semantic definitions.
- **Role_Alias**: A named role assigned to an `EntityRelationship` when the same target entity is joined multiple times with different semantic meanings.
- **Validator**: The `SemanticModelValidator` struct that checks model correctness against OCSF schema and internal consistency rules.
- **YAML_Serializer**: The serde-based YAML serialization/deserialization path for `SemanticModel` persistence.
- **Editor_GUI**: The React/TypeScript editor UI (`editor-ui/`) that provides visual editing of semantic models.
- **Model_Generator**: The `SchemaGenerator` and `MetricSuggester` in `ocsf-semantic/src/schema_generator.rs` that auto-generate entities and metrics from OCSF schema.
- **Warehouse_Generators**: The dbt, Cube.js, and SQL view generators in `ocsf-warehouse/` that produce warehouse artifacts from semantic models.

## Requirements

### Requirement 1: Dimension Hierarchies

**User Story:** As a security analyst, I want to define drill-down hierarchies on dimension attributes, so that I can navigate from high-level groupings (e.g., MITRE Tactic) down to specific details (e.g., Sub-technique) in my queries.

#### Acceptance Criteria

1. THE SemanticAttribute SHALL support an optional `hierarchy` field containing an ordered list of Hierarchy_Level values.
2. WHEN a SemanticAttribute has a non-empty `hierarchy` field, THE SemanticAttribute SHALL be treated as a dimension regardless of the `is_dimension` flag.
3. THE Hierarchy_Level SHALL contain a `name` field (String) and an `attribute_ref` field referencing another attribute name within the same entity.
4. WHEN a Dimension_Hierarchy is defined, THE Validator SHALL verify that every `attribute_ref` in the hierarchy resolves to an existing attribute within the same SemanticEntity.
5. WHEN a Dimension_Hierarchy contains duplicate `attribute_ref` values, THE Validator SHALL report a validation error.
6. THE YAML_Serializer SHALL serialize and deserialize Dimension_Hierarchy definitions, preserving level order.
7. FOR ALL valid SemanticEntity values containing hierarchies, serializing to YAML then deserializing SHALL produce an equivalent SemanticEntity (round-trip property).

### Requirement 2: Semi-Additive Metric Support

**User Story:** As a security data engineer, I want to classify metrics by their additivity type, so that query engines only aggregate metrics across valid dimensions and avoid incorrect rollups (e.g., summing "active sessions" across time).

#### Acceptance Criteria

1. THE SemanticMetric SHALL support a `metric_type` field with values: Additive, SemiAdditive, and NonAdditive.
2. WHEN the `metric_type` field is absent during deserialization, THE YAML_Serializer SHALL default `metric_type` to Additive.
3. WHEN a SemanticMetric has `metric_type` set to SemiAdditive, THE SemanticMetric SHALL support a `non_additive_dimensions` field listing dimension names across which the metric cannot be summed.
4. WHEN a SemanticMetric has `metric_type` set to SemiAdditive, THE Validator SHALL verify that every entry in `non_additive_dimensions` references a valid dimension attribute in at least one entity in the model.
5. WHEN a SemanticMetric has `metric_type` set to SemiAdditive and `non_additive_dimensions` is empty, THE Validator SHALL report a validation warning.
6. THE YAML_Serializer SHALL serialize and deserialize the `metric_type` and `non_additive_dimensions` fields.
7. FOR ALL valid SemanticMetric values with metric_type set, serializing to YAML then deserializing SHALL produce an equivalent SemanticMetric (round-trip property).

### Requirement 3: Calculated Metrics

**User Story:** As a security analyst, I want to define derived metrics computed from other metrics (e.g., alert-to-incident ratio, period-over-period change), so that I can express complex KPIs without duplicating aggregation logic.

#### Acceptance Criteria

1. THE SemanticMetric SHALL support an optional `formula` field containing a string expression that references other metric names.
2. WHEN a SemanticMetric has a non-empty `formula` field, THE SemanticMetric SHALL be treated as a Calculated_Metric and the `measure` field SHALL be ignored.
3. WHEN a SemanticMetric has a non-empty `formula` field, THE Validator SHALL verify that every metric name referenced in the formula exists in the SemanticModel.
4. WHEN a Calculated_Metric references itself directly or through a chain of other Calculated_Metrics, THE Validator SHALL report a circular dependency error.
5. THE Semantic_Layer SHALL provide a function to extract metric name references from a formula string.
6. THE YAML_Serializer SHALL serialize and deserialize the `formula` field, omitting the field when empty or absent.
7. FOR ALL valid SemanticMetric values with a formula, serializing to YAML then deserializing SHALL produce an equivalent SemanticMetric (round-trip property).

### Requirement 4: Dataset / Connection Abstraction

**User Story:** As a data architect, I want to separate physical data source details (connection, dialect, table name) from semantic definitions, so that the same semantic model can target different warehouse backends without modifying entity or metric definitions.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL define a Dataset struct containing fields: `name` (String), `dialect` (String), `table` (String), `schema_name` (optional String), and `connection` (optional String).
2. THE SemanticModel SHALL support an optional `datasets` field containing a list of Dataset values.
3. THE SemanticEntity SHALL support an optional `dataset_ref` field referencing a Dataset by name.
4. WHEN a SemanticEntity has a `dataset_ref` field, THE Validator SHALL verify that the referenced Dataset name exists in the SemanticModel `datasets` list.
5. WHEN a SemanticEntity has no `dataset_ref` field, THE Semantic_Layer SHALL continue to use the existing OCSF field path mapping behavior.
6. THE YAML_Serializer SHALL serialize and deserialize Dataset definitions and `dataset_ref` fields.
7. FOR ALL valid SemanticModel values containing datasets, serializing to YAML then deserializing SHALL produce an equivalent SemanticModel (round-trip property).

### Requirement 5: Role-Playing Relationships

**User Story:** As a security data engineer, I want to join the same dimension entity multiple times with different role aliases (e.g., "source_user" and "target_user" both referencing the User entity), so that I can model multi-role relationships without duplicating entity definitions.

#### Acceptance Criteria

1. THE EntityRelationship SHALL support an optional `role_alias` field (String) that provides a unique name for the relationship role.
2. WHEN two EntityRelationship values within the same SemanticEntity reference the same target entity, THE Validator SHALL require that each has a distinct `role_alias`.
3. WHEN an EntityRelationship has a `role_alias`, THE EntityRelationship `to_join_sql` method SHALL use the `role_alias` as the SQL table alias in the generated JOIN clause.
4. THE YAML_Serializer SHALL serialize and deserialize the `role_alias` field, omitting the field when absent.
5. FOR ALL valid SemanticEntity values containing role-playing relationships, serializing to YAML then deserializing SHALL produce an equivalent SemanticEntity (round-trip property).

### Requirement 6: Visibility and Folder Organization

**User Story:** As a data architect, I want to mark attributes and metrics as hidden and organize them into logical folders, so that downstream UI tools can present a clean, navigable interface to analysts.

#### Acceptance Criteria

1. THE SemanticAttribute SHALL support an `is_hidden` field (bool) defaulting to false.
2. THE SemanticAttribute SHALL support an optional `folder` field (String) for logical grouping.
3. THE SemanticMetric SHALL support an `is_hidden` field (bool) defaulting to false.
4. THE SemanticMetric SHALL support an optional `folder` field (String) for logical grouping.
5. WHEN `is_hidden` is false and `folder` is absent, THE YAML_Serializer SHALL omit both fields from serialized output to maintain backward compatibility.
6. THE YAML_Serializer SHALL serialize and deserialize `is_hidden` and `folder` fields.
7. FOR ALL valid SemanticAttribute and SemanticMetric values with visibility/folder metadata, serializing to YAML then deserializing SHALL produce equivalent values (round-trip property).

### Requirement 7: Catalog-Level Metadata

**User Story:** As a data architect managing multiple semantic models, I want a catalog manifest that tracks all models in a repository with their versions and dependencies, so that I can coordinate multi-model deployments and detect cross-model conflicts.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL define a SemanticCatalog struct containing fields: `name` (String), `version` (String), `description` (optional String), and `models` (list of CatalogEntry values).
2. THE CatalogEntry SHALL contain fields: `model_name` (String), `path` (String), `version` (String), and `dependencies` (optional list of model name Strings).
3. WHEN a CatalogEntry lists a dependency, THE Validator SHALL verify that the dependency model name exists as another CatalogEntry in the same SemanticCatalog.
4. WHEN two CatalogEntry values have the same `model_name`, THE Validator SHALL report a duplicate model name error.
5. THE SemanticCatalog SHALL provide `to_yaml` and `from_yaml` methods for persistence.
6. FOR ALL valid SemanticCatalog values, serializing to YAML then deserializing SHALL produce an equivalent SemanticCatalog (round-trip property).

### Requirement 8: Editor GUI Integration

**User Story:** As a data architect using the visual editor, I want the GUI to reflect all new SML fields (hierarchies, metric types, formulas, datasets, role aliases, visibility, folders), so that I can configure the full semantic model without hand-editing YAML.

#### Acceptance Criteria

1. THE Editor_GUI TypeScript types (`editor-ui/src/types/index.ts`) SHALL mirror all new Rust struct fields: `hierarchy` on SemanticAttribute, `metric_type`/`formula`/`non_additive_dimensions` on SemanticMetric, `datasets`/`dataset_ref` on SemanticModel/SemanticEntity, `role_alias` on EntityRelationship, `is_hidden`/`folder` on SemanticAttribute and SemanticMetric.
2. THE AttributeEditor component SHALL render `is_hidden` as a toggle and `folder` as a text input.
3. THE AttributeEditor component SHALL render a hierarchy level editor when the attribute is a dimension, allowing ordered add/remove/reorder of Hierarchy_Level entries.
4. THE EntityForm component SHALL render a `dataset_ref` dropdown populated from the model's `datasets` list.
5. THE Editor_GUI SHALL provide a metric editing panel that supports `metric_type` selection (Additive/SemiAdditive/NonAdditive), `formula` text input for calculated metrics, and `non_additive_dimensions` multi-select for semi-additive metrics.
6. THE Editor_GUI SHALL provide a dataset management panel for adding, editing, and removing Dataset entries on the SemanticModel.
7. THE EntityRelationship editor SHALL render a `role_alias` text input.
8. WHEN `is_hidden` is true on an attribute or metric, THE Editor_GUI SHALL visually distinguish hidden items (e.g., dimmed or tagged) in list views.
9. WHEN `folder` is set on attributes or metrics, THE Editor_GUI SHALL group items by folder in list views.

### Requirement 9: Model Generator Integration

**User Story:** As a data engineer auto-generating semantic models from OCSF schema, I want the SchemaGenerator and MetricSuggester to produce sensible defaults for new SML fields, so that generated models are immediately useful without manual field-by-field configuration.

#### Acceptance Criteria

1. THE SchemaGenerator SHALL set `metric_type` to Additive on all generated count/sum metrics and NonAdditive on generated avg/min/max metrics.
2. THE MetricSuggester SHALL generate calculated metrics for common patterns (e.g., success rate = success_count / total_count) when the source class has a status field.
3. THE SchemaGenerator SHALL infer dimension hierarchies from OCSF object nesting (e.g., when an object has country, region, and city fields, generate a geographic hierarchy).
4. THE SchemaGenerator SHALL set `is_hidden` to false and leave `folder` empty on all generated attributes and metrics (preserving current behavior as default).
5. THE SchemaGenerator SHALL NOT generate Dataset entries (datasets are user-configured, not schema-derived).

### Requirement 10: Warehouse Generator Integration

**User Story:** As a data engineer generating warehouse artifacts, I want the dbt, Cube.js, and SQL view generators to consume new SML fields, so that generated artifacts correctly represent hierarchies, semi-additive measures, calculated metrics, and role-aliased joins.

#### Acceptance Criteria

1. THE DBTGenerator SHALL emit hierarchy levels as nested `dimensions` with `type: hierarchy` in the dbt semantic manifest when a SemanticAttribute has a non-empty `hierarchy` field.
2. THE DBTGenerator SHALL set `agg_time_dimension` constraints on semi-additive measures based on `non_additive_dimensions`.
3. THE DBTGenerator SHALL emit calculated metrics as `derived` metric type in the dbt semantic manifest with the `formula` expression.
4. THE ViewGenerator SHALL use `role_alias` as the SQL table alias in JOIN clauses when generating entity views with role-playing relationships.
5. THE ViewGenerator SHALL resolve the source table from the entity's `dataset_ref` → Dataset `table` field when a dataset_ref is present, falling back to the existing OCSF table naming when absent.
6. THE ViewGenerator SHALL exclude attributes and metrics where `is_hidden` is true from generated SQL views.
