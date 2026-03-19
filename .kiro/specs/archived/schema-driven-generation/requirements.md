# Requirements Document

## Introduction

This specification defines the requirements for Schema-Driven Semantic Layer Generation - a feature that automatically generates semantic entities, dimensions, and metrics from the compiled OCSF schema. Instead of manually defining semantic models, the system will parse the OCSF schema structure (categories, classes, objects, attributes) and intelligently generate a complete semantic layer with proper field mappings, dimension inference, and metric suggestions.

The goal is to shift from an observable-centric approach to a schema-driven approach that leverages the rich metadata already present in OCSF (captions, descriptions, requirement levels, enum values, object nesting).

## Glossary

- **Compiled_Schema**: The JSON output from the OCSF schema compiler containing categories, classes, objects, dictionary, and profiles
- **Schema_Parser**: Component that reads and normalizes the compiled OCSF schema JSON
- **Entity_Generator**: Component that creates semantic entities from OCSF objects and classes
- **Dimension_Inferrer**: Component that identifies which attributes should be dimensions based on schema metadata
- **Metric_Suggester**: Component that proposes metrics based on attribute types and patterns
- **Path_Generator**: Component that creates OCSF field paths from nested object relationships
- **OCSF_Class**: An event type definition in OCSF (e.g., authentication, file_activity)
- **OCSF_Object**: A reusable structure in OCSF (e.g., user, device, actor)
- **Semantic_Entity**: A business-level abstraction generated from OCSF classes or objects

## Requirements

### Requirement 1: Parse Compiled OCSF Schema

**User Story:** As a security data engineer, I want to load a compiled OCSF schema JSON file, so that I can generate semantic entities from the official schema structure.

#### Acceptance Criteria

1. WHEN a compiled schema JSON file path is provided, THE Schema_Parser SHALL load and parse the JSON into an internal representation
2. WHEN the schema is parsed, THE Schema_Parser SHALL extract all categories with their UIDs and captions
3. WHEN the schema is parsed, THE Schema_Parser SHALL extract all classes with their attributes, requirements, and object references
4. WHEN the schema is parsed, THE Schema_Parser SHALL extract all objects with their nested attributes and relationships
5. WHEN the schema is parsed, THE Schema_Parser SHALL extract all enum definitions with their values and captions
6. IF the schema JSON is malformed, THEN THE Schema_Parser SHALL return a descriptive error
7. THE Schema_Parser SHALL support schema versions 1.4.0 through 1.6.0

### Requirement 2: Generate Semantic Entities from OCSF Objects

**User Story:** As a security data engineer, I want semantic entities automatically generated from OCSF objects like User, Device, and Actor, so that I have reusable business-level abstractions.

#### Acceptance Criteria

1. WHEN generating entities from objects, THE Entity_Generator SHALL create a semantic entity for each high-usage OCSF object (user, device, actor, network_endpoint, file, process, cloud, account)
2. WHEN creating an entity from an object, THE Entity_Generator SHALL use the object's caption as the entity caption
3. WHEN creating an entity from an object, THE Entity_Generator SHALL use the object's description as the entity description
4. WHEN creating an entity from an object, THE Entity_Generator SHALL include all required and recommended attributes
5. WHEN an object attribute references another object, THE Entity_Generator SHALL generate the nested field path (e.g., actor.user.email_addr)
6. WHEN an object has enum attributes, THE Entity_Generator SHALL mark them as categorical dimensions

### Requirement 3: Generate Semantic Entities from OCSF Classes

**User Story:** As a security data engineer, I want semantic entities automatically generated from OCSF event classes like Authentication and FileActivity, so that I can query events using business terms.

#### Acceptance Criteria

1. WHEN generating entities from classes, THE Entity_Generator SHALL create a semantic entity for each OCSF class
2. WHEN creating an entity from a class, THE Entity_Generator SHALL flatten nested object attributes into the entity with proper path mappings
3. WHEN creating an entity from a class, THE Entity_Generator SHALL include the class UID as metadata
4. WHEN creating an entity from a class, THE Entity_Generator SHALL include the category information
5. WHEN a class has required object attributes (e.g., actor, device), THE Entity_Generator SHALL include their key fields as entity attributes
6. WHEN a class has enum attributes (activity_id, status_id, disposition_id), THE Entity_Generator SHALL include them as categorical dimensions

### Requirement 4: Infer Dimensions from Schema Metadata

**User Story:** As a security analyst, I want dimensions automatically identified from schema metadata, so that I can filter and group data without manual configuration.

#### Acceptance Criteria

1. WHEN analyzing attributes, THE Dimension_Inferrer SHALL mark required attributes as dimensions
2. WHEN analyzing attributes, THE Dimension_Inferrer SHALL mark enum attributes as categorical dimensions
3. WHEN analyzing attributes, THE Dimension_Inferrer SHALL mark attributes with group "primary" as high-priority dimensions
4. WHEN analyzing attributes, THE Dimension_Inferrer SHALL mark timestamp attributes as time dimensions
5. WHEN analyzing attributes, THE Dimension_Inferrer SHALL mark attributes with observable type_ids as searchable dimensions
6. WHEN an attribute has a sibling (e.g., status_id has sibling status), THE Dimension_Inferrer SHALL link them for display purposes

### Requirement 5: Suggest Metrics from Attribute Patterns

**User Story:** As a security analyst, I want metrics automatically suggested based on attribute types, so that I have useful KPIs without manual definition.

#### Acceptance Criteria

1. WHEN analyzing a class, THE Metric_Suggester SHALL suggest a count metric for event counting
2. WHEN a class has a duration attribute, THE Metric_Suggester SHALL suggest sum and average duration metrics
3. WHEN a class has a size attribute, THE Metric_Suggester SHALL suggest sum and average size metrics
4. WHEN a class has severity_id, THE Metric_Suggester SHALL suggest a severity distribution metric
5. WHEN a class has status_id, THE Metric_Suggester SHALL suggest success/failure rate metrics
6. WHEN suggesting metrics, THE Metric_Suggester SHALL include appropriate dimensions for grouping

### Requirement 6: Generate Field Paths from Object Nesting

**User Story:** As a security data engineer, I want OCSF field paths automatically generated from object relationships, so that I don't have to manually trace nested structures.

#### Acceptance Criteria

1. WHEN an entity attribute maps to a nested object field, THE Path_Generator SHALL produce the full dot-notation path (e.g., actor.user.email_addr)
2. WHEN generating paths, THE Path_Generator SHALL handle multiple levels of nesting (e.g., actor.process.file.name)
3. WHEN an attribute is an array, THE Path_Generator SHALL indicate array access in the path
4. WHEN generating paths for a class, THE Path_Generator SHALL resolve object_type references to their actual object definitions
5. FOR ALL generated paths, printing then parsing SHALL produce an equivalent path (round-trip property)

### Requirement 7: Support Selective Entity Generation

**User Story:** As a security data engineer, I want to select which classes and objects to generate entities for, so that I can create focused semantic models.

#### Acceptance Criteria

1. WHEN a category filter is provided, THE Entity_Generator SHALL only generate entities for classes in those categories
2. WHEN a class list is provided, THE Entity_Generator SHALL only generate entities for those specific classes
3. WHEN an object list is provided, THE Entity_Generator SHALL only generate entities for those specific objects
4. WHEN no filters are provided, THE Entity_Generator SHALL generate entities for all classes and high-usage objects
5. THE Entity_Generator SHALL support include and exclude patterns for filtering

### Requirement 8: Output Generated Semantic Model

**User Story:** As a security data engineer, I want the generated semantic model output as YAML, so that I can review, customize, and use it with existing tools.

#### Acceptance Criteria

1. WHEN generation is complete, THE system SHALL output a valid semantic model YAML file
2. WHEN outputting the model, THE system SHALL include schema version metadata
3. WHEN outputting the model, THE system SHALL include generation timestamp and source schema path
4. WHEN outputting the model, THE system SHALL organize entities by category
5. FOR ALL valid semantic models, serializing then deserializing SHALL produce an equivalent model (round-trip property)
6. THE Pretty_Printer SHALL format the YAML with proper indentation and comments

### Requirement 9: CLI Integration

**User Story:** As a security data engineer, I want a CLI command to generate semantic models from OCSF schemas, so that I can automate the generation process.

#### Acceptance Criteria

1. WHEN the generate-from-schema command is invoked, THE CLI SHALL accept a schema file path argument
2. WHEN the generate-from-schema command is invoked, THE CLI SHALL accept an output file path argument
3. WHEN the generate-from-schema command is invoked, THE CLI SHALL accept optional category and class filters
4. WHEN the generate-from-schema command is invoked, THE CLI SHALL accept a --dry-run flag to preview without writing
5. IF generation succeeds, THEN THE CLI SHALL output a summary of generated entities and metrics
6. IF generation fails, THEN THE CLI SHALL output a descriptive error message

### Requirement 10: Deprecate Observable-Centric Features

**User Story:** As a maintainer, I want to deprecate observable-specific features that are now redundant with the semantic layer, so that the codebase is simpler.

#### Acceptance Criteria

1. WHEN the semantic layer covers an observable type, THE system SHALL mark the observable as redundant
2. THE system SHALL provide a migration guide from observable-based to semantic-based queries
3. THE system SHALL maintain backward compatibility with existing observable configurations
4. THE system SHALL log deprecation warnings when observable-specific features are used
