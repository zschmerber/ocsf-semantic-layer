# Design Document: Schema-Driven Semantic Layer Generation

## Overview

This design describes the architecture for automatically generating semantic layer entities, dimensions, and metrics from the compiled OCSF schema. The system parses the official OCSF schema JSON (produced by the ocsf-schema-compiler) and intelligently generates a complete semantic model by leveraging the rich metadata already present in OCSF.

The key insight is that OCSF already contains most of what we need for a semantic layer:
- **Objects** (user, device, actor) → Semantic Entities
- **Classes** (authentication, file_activity) → Event Entities
- **Requirement levels** (required, recommended) → Dimension candidates
- **Enum attributes** (status_id, activity_id) → Categorical dimensions
- **Attribute groups** (primary, context) → Priority indicators
- **Nested object references** → Field path generation

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI Layer                                     │
│  ocsf-cli generate-from-schema --schema schema.json --output model.yaml │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Generation Pipeline                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │Schema Parser │→ │Entity Gen    │→ │Model Output  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
│         │                 │                                          │
│         ▼                 ▼                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │Path Generator│  │Dimension Inf │  │Metric Suggest│              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Output: semantic-model.yaml                       │
│  - Entities from Objects (User, Device, Actor, etc.)                │
│  - Entities from Classes (Authentication, FileActivity, etc.)       │
│  - Inferred Dimensions (enums, required attrs, primary group)       │
│  - Suggested Metrics (count, duration, severity, status)            │
└─────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Schema Parser

Parses the compiled OCSF schema JSON into internal data structures.

```rust
/// Compiled OCSF schema representation
pub struct CompiledSchema {
    pub version: String,
    pub categories: HashMap<String, CategoryDef>,
    pub classes: HashMap<String, ClassDef>,
    pub objects: HashMap<String, ObjectDef>,
    pub profiles: HashMap<String, ProfileDef>,
}

/// Category definition from compiled schema
pub struct CategoryDef {
    pub uid: u32,
    pub caption: String,
    pub description: String,
}

/// Class (event type) definition
pub struct ClassDef {
    pub uid: u32,
    pub name: String,
    pub caption: String,
    pub description: String,
    pub category_uid: u32,
    pub category_name: String,
    pub attributes: HashMap<String, AttributeDef>,
    pub profiles: Vec<String>,
}

/// Object definition
pub struct ObjectDef {
    pub name: String,
    pub caption: String,
    pub description: String,
    pub attributes: HashMap<String, AttributeDef>,
    pub extends: Option<String>,
}

/// Attribute definition
pub struct AttributeDef {
    pub name: String,
    pub caption: String,
    pub description: String,
    pub attr_type: String,           // string_t, integer_t, object_t, etc.
    pub type_name: String,           // Human-readable type name
    pub requirement: Requirement,     // required, recommended, optional
    pub group: Option<String>,        // primary, context, etc.
    pub object_type: Option<String>,  // For object_t types
    pub is_array: bool,
    pub enum_values: Option<HashMap<String, EnumValue>>,
    pub observable: Option<u32>,      // Observable type_id if present
    pub sibling: Option<String>,      // Sibling attribute name
}

/// Enum value definition
pub struct EnumValue {
    pub value: i64,
    pub caption: String,
    pub description: String,
}

/// Schema parser trait
pub trait SchemaParser {
    fn parse(&self, json: &str) -> Result<CompiledSchema, ParseError>;
    fn parse_file(&self, path: &Path) -> Result<CompiledSchema, ParseError>;
}
```

### Entity Generator

Generates semantic entities from OCSF objects and classes.

```rust
/// Configuration for entity generation
pub struct GenerationConfig {
    /// Categories to include (empty = all)
    pub category_filter: Vec<String>,
    /// Specific classes to include (empty = all)
    pub class_filter: Vec<String>,
    /// Specific objects to include (empty = high-usage)
    pub object_filter: Vec<String>,
    /// Exclude patterns (regex)
    pub exclude_patterns: Vec<String>,
    /// Maximum nesting depth for path generation
    pub max_nesting_depth: usize,
    /// Include deprecated attributes
    pub include_deprecated: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            category_filter: vec![],
            class_filter: vec![],
            object_filter: vec![],
            exclude_patterns: vec![],
            max_nesting_depth: 3,
            include_deprecated: false,
        }
    }
}

/// Entity generator trait
pub trait EntityGenerator {
    /// Generate entities from OCSF objects
    fn generate_from_objects(
        &self,
        schema: &CompiledSchema,
        config: &GenerationConfig,
    ) -> Vec<SemanticEntity>;
    
    /// Generate entities from OCSF classes
    fn generate_from_classes(
        &self,
        schema: &CompiledSchema,
        config: &GenerationConfig,
    ) -> Vec<SemanticEntity>;
    
    /// Generate complete semantic model
    fn generate_model(
        &self,
        schema: &CompiledSchema,
        config: &GenerationConfig,
    ) -> SemanticModel;
}

/// High-usage objects that should become entities by default
const HIGH_USAGE_OBJECTS: &[&str] = &[
    "user", "device", "actor", "network_endpoint", 
    "file", "process", "cloud", "account", "group"
];
```

### Dimension Inferrer

Infers which attributes should be dimensions based on schema metadata.

```rust
/// Dimension inference result
pub struct DimensionInfo {
    pub is_dimension: bool,
    pub dimension_type: DimensionType,
    pub priority: DimensionPriority,
    pub sibling_attribute: Option<String>,
}

/// Type of dimension
pub enum DimensionType {
    Categorical,  // Enum-based
    Temporal,     // Timestamp
    Identifier,   // UID, name
    Searchable,   // Observable
    Numeric,      // Integer/float for ranges
}

/// Priority level for dimensions
pub enum DimensionPriority {
    High,    // Required or primary group
    Medium,  // Recommended
    Low,     // Optional
}

/// Dimension inference rules
pub trait DimensionInferrer {
    fn infer(&self, attr: &AttributeDef) -> DimensionInfo;
}

/// Default dimension inference implementation
impl DimensionInferrer for DefaultDimensionInferrer {
    fn infer(&self, attr: &AttributeDef) -> DimensionInfo {
        let is_enum = attr.enum_values.is_some();
        let is_required = attr.requirement == Requirement::Required;
        let is_primary = attr.group.as_deref() == Some("primary");
        let is_timestamp = attr.attr_type == "timestamp_t";
        let has_observable = attr.observable.is_some();
        
        let dimension_type = if is_enum {
            DimensionType::Categorical
        } else if is_timestamp {
            DimensionType::Temporal
        } else if has_observable {
            DimensionType::Searchable
        } else if attr.name.ends_with("_uid") || attr.name.ends_with("_id") {
            DimensionType::Identifier
        } else {
            DimensionType::Numeric
        };
        
        let priority = if is_required || is_primary {
            DimensionPriority::High
        } else if attr.requirement == Requirement::Recommended {
            DimensionPriority::Medium
        } else {
            DimensionPriority::Low
        };
        
        DimensionInfo {
            is_dimension: is_enum || is_required || is_primary || has_observable,
            dimension_type,
            priority,
            sibling_attribute: attr.sibling.clone(),
        }
    }
}
```

### Metric Suggester

Suggests metrics based on attribute patterns.

```rust
/// Suggested metric
pub struct SuggestedMetric {
    pub name: String,
    pub caption: String,
    pub description: String,
    pub aggregation: Aggregation,
    pub measure: MeasureDefinition,
    pub dimensions: Vec<String>,
    pub source_attribute: Option<String>,
}

/// Metric suggestion rules
pub trait MetricSuggester {
    fn suggest_metrics(&self, class: &ClassDef) -> Vec<SuggestedMetric>;
}

impl MetricSuggester for DefaultMetricSuggester {
    fn suggest_metrics(&self, class: &ClassDef) -> Vec<SuggestedMetric> {
        let mut metrics = vec![];
        
        // Always suggest event count
        metrics.push(SuggestedMetric {
            name: format!("{}_count", class.name),
            caption: format!("{} Count", class.caption),
            description: format!("Count of {} events", class.caption),
            aggregation: Aggregation::Count,
            measure: MeasureDefinition::field("metadata.uid"),
            dimensions: self.get_default_dimensions(class),
            source_attribute: None,
        });
        
        // Check for duration attribute
        if class.attributes.contains_key("duration") {
            metrics.push(SuggestedMetric {
                name: format!("{}_avg_duration", class.name),
                caption: format!("Average {} Duration", class.caption),
                aggregation: Aggregation::Avg,
                measure: MeasureDefinition::field("duration"),
                dimensions: self.get_default_dimensions(class),
                source_attribute: Some("duration".to_string()),
                ..Default::default()
            });
        }
        
        // Check for status_id (success/failure rate)
        if class.attributes.contains_key("status_id") {
            metrics.push(SuggestedMetric {
                name: format!("{}_success_rate", class.name),
                caption: format!("{} Success Rate", class.caption),
                aggregation: Aggregation::Avg,
                measure: MeasureDefinition::expression(
                    "CASE WHEN status_id = 1 THEN 1.0 ELSE 0.0 END"
                ),
                dimensions: self.get_default_dimensions(class),
                source_attribute: Some("status_id".to_string()),
                ..Default::default()
            });
        }
        
        // Check for severity_id
        if class.attributes.contains_key("severity_id") {
            metrics.push(SuggestedMetric {
                name: format!("{}_high_severity_count", class.name),
                caption: format!("High Severity {} Count", class.caption),
                aggregation: Aggregation::Count,
                measure: MeasureDefinition::expression(
                    "CASE WHEN severity_id >= 4 THEN 1 END"
                ),
                dimensions: self.get_default_dimensions(class),
                source_attribute: Some("severity_id".to_string()),
                ..Default::default()
            });
        }
        
        metrics
    }
}
```

### Path Generator

Generates OCSF field paths from nested object relationships.

```rust
/// Field path representation
pub struct FieldPath {
    pub segments: Vec<PathSegment>,
}

pub struct PathSegment {
    pub name: String,
    pub is_array: bool,
}

impl FieldPath {
    /// Create from dot notation string
    pub fn parse(path: &str) -> Result<Self, PathError>;
    
    /// Convert to dot notation string
    pub fn to_string(&self) -> String;
    
    /// Append a segment
    pub fn push(&mut self, name: &str, is_array: bool);
}

/// Path generator for resolving nested object references
pub trait PathGenerator {
    /// Generate all field paths for an object
    fn generate_paths(
        &self,
        schema: &CompiledSchema,
        object_name: &str,
        max_depth: usize,
    ) -> Vec<(FieldPath, AttributeDef)>;
    
    /// Resolve a single path to its attribute definition
    fn resolve_path(
        &self,
        schema: &CompiledSchema,
        path: &FieldPath,
    ) -> Option<AttributeDef>;
}
```

## Data Models

### Generated Semantic Model

The output semantic model extends the existing `SemanticModel` structure:

```rust
/// Extended semantic model with generation metadata
pub struct GeneratedSemanticModel {
    /// Base semantic model
    pub model: SemanticModel,
    /// Generation metadata
    pub generation: GenerationMetadata,
}

pub struct GenerationMetadata {
    /// Source schema version
    pub schema_version: String,
    /// Source schema file path
    pub schema_path: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Generation configuration used
    pub config: GenerationConfig,
    /// Statistics about generation
    pub stats: GenerationStats,
}

pub struct GenerationStats {
    pub objects_processed: usize,
    pub classes_processed: usize,
    pub entities_generated: usize,
    pub dimensions_inferred: usize,
    pub metrics_suggested: usize,
}
```

### Entity Categories

Generated entities are organized by their source:

```yaml
# Generated semantic model structure
version: "1.0"
ocsf_version: "1.6.0"
name: "generated-from-ocsf"
description: "Auto-generated semantic layer from OCSF schema"

generation:
  schema_path: "./schema/ocsf-compiled-v1.6.0.json"
  generated_at: "2026-01-22T18:45:00Z"
  stats:
    entities_generated: 90
    dimensions_inferred: 450
    metrics_suggested: 164

# Entities from OCSF Objects (reusable across events)
object_entities:
  - name: user
    caption: User
    source_type: object
    attributes:
      - name: uid
        caption: Unique ID
        type: string
        ocsf_mapping:
          field: uid
        is_dimension: true
        dimension_type: identifier
      # ... more attributes

# Entities from OCSF Classes (event types)
event_entities:
  - name: authentication
    caption: Authentication
    source_type: class
    source_class_uid: 3002
    source_category: iam
    attributes:
      - name: user_email
        caption: User Email
        type: string
        ocsf_mapping:
          field: actor.user.email_addr
        is_dimension: true
      # ... flattened attributes from nested objects

# Suggested metrics
metrics:
  - name: authentication_count
    caption: Authentication Count
    source_class: authentication
    aggregation: count
    # ...
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Schema Parsing Round-Trip
*For any* valid compiled OCSF schema JSON, parsing then serializing back to JSON SHALL produce a semantically equivalent structure (categories, classes, objects preserved).
**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

### Property 2: Malformed Schema Error Handling
*For any* malformed JSON input (missing required fields, invalid types, truncated content), the Schema_Parser SHALL return an error rather than panic or produce invalid output.
**Validates: Requirements 1.6**

### Property 3: Object Entity Generation Completeness
*For any* OCSF object in the high-usage list, the Entity_Generator SHALL produce a semantic entity with caption and description matching the source object.
**Validates: Requirements 2.1, 2.2, 2.3**

### Property 4: Object Attribute Inclusion
*For any* OCSF object, the generated entity SHALL include all attributes marked as required or recommended in the source object.
**Validates: Requirements 2.4**

### Property 5: Nested Path Generation
*For any* object attribute that references another object (type=object_t), the Entity_Generator SHALL produce a valid dot-notation path to nested attributes.
**Validates: Requirements 2.5, 6.1, 6.2**

### Property 6: Class Entity Generation
*For any* OCSF class, the Entity_Generator SHALL produce a semantic entity with the class UID and category information preserved.
**Validates: Requirements 3.1, 3.3, 3.4**

### Property 7: Class Attribute Flattening
*For any* OCSF class with nested object attributes, the generated entity SHALL include flattened attributes with correct path mappings.
**Validates: Requirements 3.2, 3.5**

### Property 8: Enum Dimension Inference
*For any* attribute with enum values, the Dimension_Inferrer SHALL mark it as a categorical dimension.
**Validates: Requirements 2.6, 3.6, 4.2**

### Property 9: Dimension Inference Rules
*For any* attribute, the Dimension_Inferrer SHALL correctly apply inference rules: required→dimension, primary group→high priority, timestamp→temporal, observable→searchable.
**Validates: Requirements 4.1, 4.3, 4.4, 4.5**

### Property 10: Sibling Attribute Linking
*For any* attribute with a sibling field, the Dimension_Inferrer SHALL link the ID attribute to its human-readable sibling.
**Validates: Requirements 4.6**

### Property 11: Metric Suggestion Completeness
*For any* OCSF class, the Metric_Suggester SHALL suggest at least a count metric, plus additional metrics for duration, status_id, and severity_id if present.
**Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

### Property 12: Metric Dimension Association
*For any* suggested metric, the Metric_Suggester SHALL include appropriate dimensions for grouping based on the class's categorical attributes.
**Validates: Requirements 5.6**

### Property 13: Path Round-Trip
*For any* valid field path, parsing from string then converting back to string SHALL produce the original path.
**Validates: Requirements 6.5**

### Property 14: Array Path Indication
*For any* attribute marked as is_array, the generated path SHALL indicate array access.
**Validates: Requirements 6.3**

### Property 15: Object Type Resolution
*For any* attribute with object_type reference, the Path_Generator SHALL resolve it to the actual object definition in the schema.
**Validates: Requirements 6.4**

### Property 16: Category Filter Application
*For any* category filter, the Entity_Generator SHALL only produce entities for classes whose category matches the filter.
**Validates: Requirements 7.1**

### Property 17: Class Filter Application
*For any* class filter, the Entity_Generator SHALL only produce entities for classes in the filter list.
**Validates: Requirements 7.2**

### Property 18: Object Filter Application
*For any* object filter, the Entity_Generator SHALL only produce entities for objects in the filter list.
**Validates: Requirements 7.3**

### Property 19: Default Generation Coverage
*For any* schema with no filters applied, the Entity_Generator SHALL produce entities for all classes and all high-usage objects.
**Validates: Requirements 7.4**

### Property 20: Semantic Model Round-Trip
*For any* valid generated semantic model, serializing to YAML then deserializing SHALL produce an equivalent model.
**Validates: Requirements 8.1, 8.5**

### Property 21: Generation Metadata Inclusion
*For any* generated model, the output SHALL include schema version, generation timestamp, and source path.
**Validates: Requirements 8.2, 8.3**

## Error Handling

```rust
/// Errors during schema parsing
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    
    #[error("Missing required field: {field} in {context}")]
    MissingField { field: String, context: String },
    
    #[error("Invalid schema version: {0}")]
    InvalidVersion(String),
    
    #[error("Unsupported schema version: {version}, supported: {supported:?}")]
    UnsupportedVersion { version: String, supported: Vec<String> },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors during entity generation
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    
    #[error("Class not found: {0}")]
    ClassNotFound(String),
    
    #[error("Circular reference detected: {path:?}")]
    CircularReference { path: Vec<String> },
    
    #[error("Max nesting depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },
}

/// Errors during path operations
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("Empty path")]
    EmptyPath,
    
    #[error("Invalid path segment: {0}")]
    InvalidSegment(String),
    
    #[error("Unresolved object type: {0}")]
    UnresolvedObjectType(String),
}
```

## Testing Strategy

### Unit Tests
- Schema parsing with various valid/invalid inputs
- Individual dimension inference rules
- Individual metric suggestion rules
- Path parsing and formatting

### Property-Based Tests
- Schema round-trip (parse → serialize → parse)
- Path round-trip (parse → to_string → parse)
- Model round-trip (generate → serialize → deserialize)
- Filter application correctness
- Dimension inference rule coverage
- Metric suggestion completeness

### Integration Tests
- End-to-end generation from real OCSF schema
- CLI command execution
- Output file validation

### Test Configuration
- Property tests: minimum 100 iterations
- Use `proptest` crate for Rust property-based testing
- Tag format: **Feature: schema-driven-generation, Property {number}: {property_text}**
