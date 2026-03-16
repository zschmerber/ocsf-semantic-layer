//! Query translator for semantic queries.
//!
//! This module provides the query translator that converts semantic queries
//! into warehouse-specific SQL, supporting entity queries, metric aggregations,
//! hot path routing, and reverse lookups.
//!
//! # Integration with ocsf-index
//!
//! **Note on Cyclic Dependencies**: The `ocsf-index` crate depends on `ocsf-semantic`
//! for types like [`SemanticQuery`] and [`WarehouseDialect`]. Adding `ocsf-index` as a
//! dependency of `ocsf-semantic` would create a cyclic dependency, which is not allowed
//! in Rust. Therefore, integration between these crates happens at the **application level**,
//! not within either crate.
//!
//! ## Query Optimization with SemanticIndex
//!
//! The `ocsf-index` crate provides [`QueryPlanHints`](https://docs.rs/ocsf-index) that can
//! be used to optimize query execution through:
//!
//! - **Partition pruning**: Filter to only relevant time-based partitions
//! - **Statistics-based planning**: Use column cardinality and null rates for cost estimation
//! - **Result caching**: Check for cached results before executing expensive queries
//! - **Table resolution**: Get physical table locations and warehouse dialects
//!
//! ## Integration Pattern
//!
//! Applications that use both `ocsf-semantic` and `ocsf-index` should follow this pattern:
//!
//! ```rust,ignore
//! use ocsf_semantic::{SemanticQuery, SqlGenerator, QueryValidator, WarehouseDialect};
//! use ocsf_semantic::model::SemanticModel;
//! // use ocsf_index::{SemanticIndex, IndexBackend, QueryPlanHints};
//!
//! /// Execute a semantic query with index-based optimization.
//! ///
//! /// This function demonstrates the recommended integration pattern between
//! /// ocsf-semantic (query translation) and ocsf-index (query optimization).
//! async fn execute_optimized_query<B: /* IndexBackend */>(
//!     query: &SemanticQuery,
//!     model: &SemanticModel,
//!     // index: &SemanticIndex<B>,
//!     dialect: WarehouseDialect,
//! ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
//!     // Step 1: Validate the semantic query against the model
//!     let validator = QueryValidator::new(model);
//!     let validated = validator.validate(query)?;
//!
//!     // Step 2: Get optimization hints from the semantic index
//!     // let hints: QueryPlanHints = index.get_query_plan_hints(query).await?;
//!
//!     // Step 3: Check cache first (if available)
//!     // if let Some(cached) = hints.cached_result {
//!     //     if !cached.is_expired() {
//!     //         // Return cached result directly
//!     //         return Ok(serde_json::from_value(cached.result)?);
//!     //     }
//!     // }
//!
//!     // Step 4: Use partition hints for query optimization
//!     // The hints.partitions field contains only partitions that overlap
//!     // with the query's time range, enabling partition pruning.
//!     // for partition in &hints.partitions {
//!     //     // Filter to relevant partitions based on:
//!     //     // - partition.partition_key
//!     //     // - partition.estimated_rows (for cost estimation)
//!     // }
//!
//!     // Step 5: Use table hints for physical table resolution
//!     // The hints.tables field provides physical table locations:
//!     // for table_hint in &hints.tables {
//!     //     // table_hint.table_name - physical table name
//!     //     // table_hint.schema_name - optional schema
//!     //     // table_hint.dialect - warehouse dialect for SQL generation
//!     //     // table_hint.ocsf_version - for version-aware field mapping
//!     // }
//!
//!     // Step 6: Use statistics for query planning
//!     // The hints.statistics field provides column-level statistics:
//!     // for stats in &hints.statistics {
//!     //     // stats.total_rows - for cardinality estimation
//!     //     // stats.get_column("field_name") - for selectivity estimation
//!     //     // stats.is_stale(max_age_secs) - check if stats need refresh
//!     // }
//!
//!     // Step 7: Generate SQL using the validated query
//!     let generator = SqlGenerator::new(dialect);
//!     let translated = generator.generate(&validated);
//!
//!     // Step 8: Execute the SQL against the warehouse
//!     // (implementation depends on your warehouse client)
//!     // let results = warehouse_client.execute(&translated.sql).await?;
//!
//!     // Step 9: Cache the results for future queries
//!     // let cache_key = QueryCacheKey::from_query(query);
//!     // index.cache_query_result(&cache_key, &results, ttl_secs).await?;
//!
//!     // Return placeholder for example
//!     Ok(vec![])
//! }
//! ```
//!
//! ## QueryPlanHints Structure
//!
//! The `QueryPlanHints` struct from `ocsf-index` contains:
//!
//! ```rust,ignore
//! pub struct QueryPlanHints {
//!     /// Physical table information for SQL generation
//!     pub tables: Vec<TableHint>,
//!     /// Partitions that overlap with the query's time range
//!     pub partitions: Vec<PartitionHint>,
//!     /// Column statistics for cost estimation
//!     pub statistics: Vec<TableStatistics>,
//!     /// Cached result if available and not expired
//!     pub cached_result: Option<CachedResult>,
//! }
//!
//! pub struct TableHint {
//!     pub table_name: String,
//!     pub schema_name: Option<String>,
//!     pub dialect: WarehouseDialect,
//!     pub ocsf_version: String,
//! }
//!
//! pub struct PartitionHint {
//!     pub table_name: String,
//!     pub partition_key: String,
//!     pub estimated_rows: u64,
//! }
//! ```
//!
//! ## Benefits of Index Integration
//!
//! 1. **Faster queries**: Partition pruning reduces data scanned
//! 2. **Better planning**: Statistics enable cost-based optimization
//! 3. **Reduced load**: Caching avoids redundant query execution
//! 4. **Version awareness**: Table hints include OCSF version for field mapping
//! 5. **Dialect flexibility**: Each table can have its own warehouse dialect

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entity::{EntityRelationship, SemanticAttribute, SemanticEntity};
use crate::metric::{SemanticMetric, TimeGranularity};
use crate::model::SemanticModel;

/// Errors that can occur during query translation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum QueryError {
    /// Referenced entity does not exist.
    #[error("Unknown entity: '{0}'. Available entities: {1}")]
    UnknownEntity(String, String),

    /// Referenced metric does not exist.
    #[error("Unknown metric: '{0}'. Available metrics: {1}")]
    UnknownMetric(String, String),

    /// Referenced attribute does not exist on the entity.
    #[error("Unknown attribute '{0}' on entity '{1}'. Available attributes: {2}")]
    UnknownAttribute(String, String, String),

    /// Referenced dimension is not valid for the metric.
    #[error("Invalid dimension '{0}' for metric '{1}'. Valid dimensions: {2}")]
    InvalidDimension(String, String, String),

    /// Time granularity not supported by the metric.
    #[error("Time granularity '{0:?}' not supported by metric '{1}'. Supported: {2}")]
    UnsupportedTimeGranularity(TimeGranularity, String, String),

    /// Query is empty (no selections).
    #[error("Query must select at least one attribute or metric")]
    EmptyQuery,

    /// Invalid time range.
    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),

    /// Hot path requested but metric doesn't support it.
    #[error("Metric '{0}' does not support hot path queries")]
    HotPathNotSupported(String),

    /// Relationship target entity not found.
    #[error("Relationship '{0}' references unknown entity '{1}'")]
    UnknownRelationshipTarget(String, String),
}

/// Result type for query operations.
pub type QueryResult<T> = Result<T, QueryError>;

/// Path preference for query execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PathPreference {
    /// Use hot path (observables table) for faster queries.
    Hot,
    /// Use cold path (full events table) for detailed data.
    Cold,
    /// Automatically choose based on query characteristics.
    #[default]
    Auto,
}

/// A time range for filtering queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (ISO 8601 format or relative like "-7d").
    pub start: String,
    /// End time (ISO 8601 format or relative like "now").
    pub end: String,
}

impl TimeRange {
    /// Creates a new time range.
    pub fn new(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

/// A filter condition for queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryFilter {
    /// The attribute to filter on.
    pub attribute: String,
    /// The comparison operator.
    pub operator: FilterOperator,
    /// The value to compare against.
    pub value: FilterValue,
}

impl QueryFilter {
    /// Creates a new filter.
    pub fn new(attribute: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            attribute: attribute.into(),
            operator,
            value,
        }
    }

    /// Creates an equality filter.
    pub fn eq(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(attribute, FilterOperator::Eq, FilterValue::String(value.into()))
    }

    /// Creates a not-equal filter.
    pub fn ne(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(attribute, FilterOperator::Ne, FilterValue::String(value.into()))
    }

    /// Creates a greater-than filter.
    pub fn gt(attribute: impl Into<String>, value: i64) -> Self {
        Self::new(attribute, FilterOperator::Gt, FilterValue::Integer(value))
    }

    /// Creates a less-than filter.
    pub fn lt(attribute: impl Into<String>, value: i64) -> Self {
        Self::new(attribute, FilterOperator::Lt, FilterValue::Integer(value))
    }

    /// Creates an IN filter.
    pub fn in_list(attribute: impl Into<String>, values: Vec<String>) -> Self {
        Self::new(attribute, FilterOperator::In, FilterValue::List(values))
    }
}

/// Filter comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    /// Equal to.
    Eq,
    /// Not equal to.
    Ne,
    /// Greater than.
    Gt,
    /// Greater than or equal to.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal to.
    Lte,
    /// In a list of values.
    In,
    /// Not in a list of values.
    NotIn,
    /// Like pattern match.
    Like,
    /// Is null.
    IsNull,
    /// Is not null.
    IsNotNull,
}

impl FilterOperator {
    /// Returns the SQL operator string.
    pub fn sql_operator(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::Like => "LIKE",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }
}

/// A filter value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    /// String value.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// List of string values (for IN operator).
    List(Vec<String>),
    /// Null value.
    Null,
}

impl FilterValue {
    /// Formats the value for SQL.
    pub fn to_sql(&self) -> String {
        match self {
            Self::String(s) => format!("'{}'", s.replace('\'', "''")),
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Self::List(items) => {
                let formatted: Vec<String> = items
                    .iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect();
                format!("({})", formatted.join(", "))
            }
            Self::Null => "NULL".to_string(),
        }
    }
}

/// A semantic query definition.
///
/// Represents a query against the semantic layer that will be translated
/// to warehouse-specific SQL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticQuery {
    /// The primary entity to query.
    pub entity: String,

    /// Attributes to select from the entity.
    #[serde(default)]
    pub select: Vec<String>,

    /// Filter conditions.
    #[serde(default)]
    pub filters: Vec<QueryFilter>,

    /// Metrics to calculate.
    #[serde(default)]
    pub metrics: Vec<String>,

    /// Attributes to group by (for metric queries).
    #[serde(default)]
    pub group_by: Vec<String>,

    /// Time range filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,

    /// Time granularity for time-based aggregations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_granularity: Option<TimeGranularity>,

    /// Path preference for query execution.
    #[serde(default)]
    pub path_preference: PathPreference,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Number of results to skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    /// Order by clauses.
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
}

/// Order by clause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBy {
    /// The attribute or metric to order by.
    pub field: String,
    /// Sort direction.
    #[serde(default)]
    pub direction: SortDirection,
}

impl OrderBy {
    /// Creates a new ascending order by clause.
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Asc,
        }
    }

    /// Creates a new descending order by clause.
    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Desc,
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending order.
    #[default]
    Asc,
    /// Descending order.
    Desc,
}

impl SortDirection {
    /// Returns the SQL keyword.
    pub fn sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

impl SemanticQuery {
    /// Creates a new semantic query for the given entity.
    pub fn new(entity: impl Into<String>) -> Self {
        Self {
            entity: entity.into(),
            select: Vec::new(),
            filters: Vec::new(),
            metrics: Vec::new(),
            group_by: Vec::new(),
            time_range: None,
            time_granularity: None,
            path_preference: PathPreference::default(),
            limit: None,
            offset: None,
            order_by: Vec::new(),
        }
    }

    /// Adds attributes to select.
    pub fn select(mut self, attributes: Vec<String>) -> Self {
        self.select = attributes;
        self
    }

    /// Adds a single attribute to select.
    pub fn add_select(mut self, attribute: impl Into<String>) -> Self {
        self.select.push(attribute.into());
        self
    }

    /// Adds a filter condition.
    pub fn add_filter(mut self, filter: QueryFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Sets the filters.
    pub fn with_filters(mut self, filters: Vec<QueryFilter>) -> Self {
        self.filters = filters;
        self
    }

    /// Adds metrics to calculate.
    pub fn with_metrics(mut self, metrics: Vec<String>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Adds a single metric.
    pub fn add_metric(mut self, metric: impl Into<String>) -> Self {
        self.metrics.push(metric.into());
        self
    }

    /// Sets the group by attributes.
    pub fn group_by(mut self, attributes: Vec<String>) -> Self {
        self.group_by = attributes;
        self
    }

    /// Adds a group by attribute.
    pub fn add_group_by(mut self, attribute: impl Into<String>) -> Self {
        self.group_by.push(attribute.into());
        self
    }

    /// Sets the time range.
    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    /// Sets the time granularity.
    pub fn with_time_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.time_granularity = Some(granularity);
        self
    }

    /// Sets the path preference.
    pub fn with_path_preference(mut self, preference: PathPreference) -> Self {
        self.path_preference = preference;
        self
    }

    /// Sets the limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset.
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Adds an order by clause.
    pub fn add_order_by(mut self, order: OrderBy) -> Self {
        self.order_by.push(order);
        self
    }

    /// Sets the order by clauses.
    pub fn with_order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        self.order_by = order_by;
        self
    }

    /// Returns true if this is a metric query (has metrics to calculate).
    pub fn is_metric_query(&self) -> bool {
        !self.metrics.is_empty()
    }

    /// Returns true if this query has any selections.
    pub fn has_selections(&self) -> bool {
        !self.select.is_empty() || !self.metrics.is_empty()
    }
}

/// Validates a semantic query against a model.
pub struct QueryValidator<'a> {
    model: &'a SemanticModel,
}

impl<'a> QueryValidator<'a> {
    /// Creates a new query validator.
    pub fn new(model: &'a SemanticModel) -> Self {
        Self { model }
    }

    /// Validates a semantic query.
    pub fn validate(&self, query: &SemanticQuery) -> QueryResult<ValidatedQuery> {
        // Check that query has selections
        if !query.has_selections() {
            return Err(QueryError::EmptyQuery);
        }

        // Validate entity exists
        let entity = self.validate_entity(&query.entity)?;

        // Validate selected attributes
        let mut validated_attributes = Vec::new();
        for attr_name in &query.select {
            let attr = self.validate_attribute(entity, attr_name)?;
            validated_attributes.push(attr.clone());
        }

        // Validate metrics
        let mut validated_metrics = Vec::new();
        for metric_name in &query.metrics {
            let metric = self.validate_metric(metric_name)?;
            validated_metrics.push(metric.clone());
        }

        // Validate group by attributes
        for group_attr in &query.group_by {
            self.validate_attribute(entity, group_attr)?;
        }

        // Validate filter attributes
        for filter in &query.filters {
            self.validate_attribute(entity, &filter.attribute)?;
        }

        // Validate time granularity against metrics
        if let Some(granularity) = query.time_granularity {
            for metric in &validated_metrics {
                if !metric.time_granularities.is_empty()
                    && !metric.supports_granularity(granularity)
                {
                    let supported: Vec<String> = metric
                        .time_granularities
                        .iter()
                        .map(|g| format!("{:?}", g))
                        .collect();
                    return Err(QueryError::UnsupportedTimeGranularity(
                        granularity,
                        metric.name.clone(),
                        supported.join(", "),
                    ));
                }
            }
        }

        // Validate dimensions for metrics
        for metric in &validated_metrics {
            for group_attr in &query.group_by {
                if !metric.dimensions.is_empty() && !metric.supports_dimension(group_attr) {
                    return Err(QueryError::InvalidDimension(
                        group_attr.clone(),
                        metric.name.clone(),
                        metric.dimensions.join(", "),
                    ));
                }
            }
        }

        // Validate hot path preference
        if query.path_preference == PathPreference::Hot {
            for metric in &validated_metrics {
                if !metric.is_hot_path {
                    return Err(QueryError::HotPathNotSupported(metric.name.clone()));
                }
            }
        }

        // Validate relationships for joins
        let mut required_joins = Vec::new();
        for rel in &entity.relationships {
            if self.model.get_entity(&rel.target_entity).is_none() {
                return Err(QueryError::UnknownRelationshipTarget(
                    rel.name.clone(),
                    rel.target_entity.clone(),
                ));
            }
            required_joins.push(rel.clone());
        }

        // Determine if hot path should be used
        let uses_hot_path = self.should_use_hot_path(query, &validated_metrics);

        Ok(ValidatedQuery {
            entity: entity.clone(),
            attributes: validated_attributes,
            metrics: validated_metrics,
            filters: query.filters.clone(),
            group_by: query.group_by.clone(),
            time_range: query.time_range.clone(),
            time_granularity: query.time_granularity,
            uses_hot_path,
            required_joins,
            limit: query.limit,
            offset: query.offset,
            order_by: query.order_by.clone(),
        })
    }

    /// Validates that an entity exists.
    fn validate_entity(&self, name: &str) -> QueryResult<&SemanticEntity> {
        self.model.get_entity(name).ok_or_else(|| {
            let available: Vec<&str> = self.model.entity_names().collect();
            QueryError::UnknownEntity(name.to_string(), available.join(", "))
        })
    }

    /// Validates that an attribute exists on an entity.
    fn validate_attribute<'b>(
        &self,
        entity: &'b SemanticEntity,
        name: &str,
    ) -> QueryResult<&'b SemanticAttribute> {
        entity.get_attribute(name).ok_or_else(|| {
            let available: Vec<&str> = entity.attributes.iter().map(|a| a.name.as_str()).collect();
            QueryError::UnknownAttribute(name.to_string(), entity.name.clone(), available.join(", "))
        })
    }

    /// Validates that a metric exists.
    fn validate_metric(&self, name: &str) -> QueryResult<&SemanticMetric> {
        self.model.get_metric(name).ok_or_else(|| {
            let available: Vec<&str> = self.model.metric_names().collect();
            QueryError::UnknownMetric(name.to_string(), available.join(", "))
        })
    }

    /// Determines if the hot path should be used for this query.
    fn should_use_hot_path(&self, query: &SemanticQuery, metrics: &[SemanticMetric]) -> bool {
        match query.path_preference {
            PathPreference::Hot => true,
            PathPreference::Cold => false,
            PathPreference::Auto => {
                // Use hot path if all metrics support it
                !metrics.is_empty() && metrics.iter().all(|m| m.is_hot_path)
            }
        }
    }
}

/// A validated semantic query ready for SQL generation.
#[derive(Debug, Clone)]
pub struct ValidatedQuery {
    /// The validated entity.
    pub entity: SemanticEntity,
    /// Validated attributes to select.
    pub attributes: Vec<SemanticAttribute>,
    /// Validated metrics to calculate.
    pub metrics: Vec<SemanticMetric>,
    /// Filter conditions.
    pub filters: Vec<QueryFilter>,
    /// Group by attribute names.
    pub group_by: Vec<String>,
    /// Time range filter.
    pub time_range: Option<TimeRange>,
    /// Time granularity.
    pub time_granularity: Option<TimeGranularity>,
    /// Whether to use hot path.
    pub uses_hot_path: bool,
    /// Required joins for relationships.
    pub required_joins: Vec<EntityRelationship>,
    /// Result limit.
    pub limit: Option<u32>,
    /// Result offset.
    pub offset: Option<u32>,
    /// Order by clauses.
    pub order_by: Vec<OrderBy>,
}

/// Warehouse SQL dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WarehouseDialect {
    /// Snowflake SQL dialect.
    #[default]
    Snowflake,
    /// Databricks SQL dialect.
    Databricks,
    /// Google BigQuery SQL dialect.
    BigQuery,
    /// PostgreSQL dialect.
    Postgres,
}

impl WarehouseDialect {
    /// Returns the date truncation function for this dialect.
    pub fn date_trunc_fn(&self, granularity: &str, column: &str) -> String {
        match self {
            Self::Snowflake => format!("DATE_TRUNC('{}', {})", granularity, column),
            Self::Databricks => format!("DATE_TRUNC('{}', {})", granularity, column),
            Self::BigQuery => format!("TIMESTAMP_TRUNC({}, {})", column, granularity.to_uppercase()),
            Self::Postgres => format!("DATE_TRUNC('{}', {})", granularity, column),
        }
    }

    /// Returns the limit/offset syntax for this dialect.
    pub fn limit_offset(&self, limit: Option<u32>, offset: Option<u32>) -> String {
        let mut parts = Vec::new();
        if let Some(l) = limit {
            parts.push(format!("LIMIT {}", l));
        }
        if let Some(o) = offset {
            parts.push(format!("OFFSET {}", o));
        }
        parts.join(" ")
    }

    /// Returns the string concatenation operator.
    pub fn concat_operator(&self) -> &'static str {
        match self {
            Self::BigQuery => "||",
            _ => "||",
        }
    }

    /// Returns the COALESCE function name for this dialect.
    pub fn coalesce_fn(&self) -> &'static str {
        match self {
            Self::BigQuery => "COALESCE",
            Self::Snowflake => "COALESCE",
            Self::Databricks => "COALESCE",
            Self::Postgres => "COALESCE",
        }
    }

    /// Returns the IFNULL/NVL function name for this dialect.
    pub fn ifnull_fn(&self) -> &'static str {
        match self {
            Self::BigQuery => "IFNULL",
            Self::Snowflake => "NVL",
            Self::Databricks => "COALESCE",
            Self::Postgres => "COALESCE",
        }
    }

    /// Returns the current timestamp function for this dialect.
    pub fn current_timestamp_fn(&self) -> &'static str {
        match self {
            Self::BigQuery => "CURRENT_TIMESTAMP()",
            Self::Snowflake => "CURRENT_TIMESTAMP()",
            Self::Databricks => "CURRENT_TIMESTAMP()",
            Self::Postgres => "CURRENT_TIMESTAMP",
        }
    }

    /// Returns the JSON extraction syntax for this dialect.
    pub fn json_extract(&self, column: &str, path: &str) -> String {
        match self {
            Self::BigQuery => format!("JSON_EXTRACT_SCALAR({}, '$.{}')", column, path),
            Self::Snowflake => format!("{}:{}", column, path),
            Self::Databricks => format!("{}:{}", column, path),
            Self::Postgres => format!("{} ->> '{}'", column, path),
        }
    }

    /// Returns the array contains function for this dialect.
    pub fn array_contains(&self, array_col: &str, value: &str) -> String {
        match self {
            Self::BigQuery => format!("{} IN UNNEST({})", value, array_col),
            Self::Snowflake => format!("ARRAY_CONTAINS({}, {})", value, array_col),
            Self::Databricks => format!("ARRAY_CONTAINS({}, {})", array_col, value),
            Self::Postgres => format!("{} = ANY({})", value, array_col),
        }
    }

    /// Returns the string type name for this dialect.
    pub fn string_type(&self) -> &'static str {
        match self {
            Self::BigQuery => "STRING",
            Self::Snowflake => "VARCHAR",
            Self::Databricks => "STRING",
            Self::Postgres => "TEXT",
        }
    }

    /// Returns the integer type name for this dialect.
    pub fn integer_type(&self) -> &'static str {
        match self {
            Self::BigQuery => "INT64",
            Self::Snowflake => "INTEGER",
            Self::Databricks => "BIGINT",
            Self::Postgres => "BIGINT",
        }
    }

    /// Returns the timestamp type name for this dialect.
    pub fn timestamp_type(&self) -> &'static str {
        match self {
            Self::BigQuery => "TIMESTAMP",
            Self::Snowflake => "TIMESTAMP_NTZ",
            Self::Databricks => "TIMESTAMP",
            Self::Postgres => "TIMESTAMP",
        }
    }

    /// Returns the boolean type name for this dialect.
    pub fn boolean_type(&self) -> &'static str {
        match self {
            Self::BigQuery => "BOOL",
            Self::Snowflake => "BOOLEAN",
            Self::Databricks => "BOOLEAN",
            Self::Postgres => "BOOLEAN",
        }
    }

    /// Returns the JSON type name for this dialect.
    pub fn json_type(&self) -> &'static str {
        match self {
            Self::BigQuery => "JSON",
            Self::Snowflake => "VARIANT",
            Self::Databricks => "STRING", // Databricks uses STRING for JSON
            Self::Postgres => "JSONB",
        }
    }

    /// Returns the dialect name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Snowflake => "snowflake",
            Self::Databricks => "databricks",
            Self::BigQuery => "bigquery",
            Self::Postgres => "postgres",
        }
    }

    /// Returns true if this dialect supports QUALIFY clause.
    pub fn supports_qualify(&self) -> bool {
        matches!(self, Self::Snowflake | Self::Databricks | Self::BigQuery)
    }

    /// Returns true if this dialect supports STRUCT types.
    pub fn supports_struct(&self) -> bool {
        matches!(self, Self::BigQuery | Self::Databricks)
    }

    /// Returns the comment syntax for this dialect.
    pub fn comment_syntax(&self) -> &'static str {
        match self {
            Self::BigQuery => "--",
            Self::Snowflake => "--",
            Self::Databricks => "--",
            Self::Postgres => "--",
        }
    }

    /// Wraps an identifier in the appropriate quotes for this dialect.
    pub fn quote_identifier(&self, identifier: &str) -> String {
        match self {
            Self::BigQuery => format!("`{}`", identifier),
            Self::Snowflake => format!("\"{}\"", identifier),
            Self::Databricks => format!("`{}`", identifier),
            Self::Postgres => format!("\"{}\"", identifier),
        }
    }
}

/// The result of translating a semantic query to SQL.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedQuery {
    /// The generated SQL statement.
    pub sql: String,
    /// Query parameters (for parameterized queries).
    pub parameters: Vec<(String, FilterValue)>,
    /// Whether this query uses the hot path (observables table).
    pub uses_hot_path: bool,
    /// The warehouse dialect used.
    pub dialect: WarehouseDialect,
}

impl TranslatedQuery {
    /// Creates a new translated query.
    pub fn new(sql: String, uses_hot_path: bool, dialect: WarehouseDialect) -> Self {
        Self {
            sql,
            parameters: Vec::new(),
            uses_hot_path,
            dialect,
        }
    }
}

/// SQL generator for semantic queries.
pub struct SqlGenerator {
    dialect: WarehouseDialect,
    observables_table: String,
}

impl SqlGenerator {
    /// Creates a new SQL generator with the given dialect.
    pub fn new(dialect: WarehouseDialect) -> Self {
        Self {
            dialect,
            observables_table: "ocsf_observables".to_string(),
        }
    }

    /// Sets the observables table name.
    pub fn with_observables_table(mut self, table: impl Into<String>) -> Self {
        self.observables_table = table.into();
        self
    }

    /// Generates SQL for a validated query.
    pub fn generate(&self, query: &ValidatedQuery) -> TranslatedQuery {
        if query.uses_hot_path {
            self.generate_hot_path_query(query)
        } else {
            self.generate_cold_path_query(query)
        }
    }

    /// Generates SQL for a cold path (full events) query.
    fn generate_cold_path_query(&self, query: &ValidatedQuery) -> TranslatedQuery {
        let mut sql_parts = Vec::new();

        // SELECT clause
        let select_clause = self.build_select_clause(query);
        sql_parts.push(format!("SELECT {}", select_clause));

        // FROM clause with table name based on entity
        let table_name = self.entity_to_table_name(&query.entity.name);
        sql_parts.push(format!("FROM {}", table_name));

        // JOIN clauses for relationships
        for join in &query.required_joins {
            let join_table = self.entity_to_table_name(&join.target_entity);
            sql_parts.push(format!(
                "LEFT JOIN {} ON {}",
                join_table, join.join_condition
            ));
        }

        // WHERE clause
        if let Some(where_clause) = self.build_where_clause(query) {
            sql_parts.push(format!("WHERE {}", where_clause));
        }

        // GROUP BY clause
        if let Some(group_by) = self.build_group_by_clause(query) {
            sql_parts.push(format!("GROUP BY {}", group_by));
        }

        // ORDER BY clause
        if let Some(order_by) = self.build_order_by_clause(query) {
            sql_parts.push(format!("ORDER BY {}", order_by));
        }

        // LIMIT/OFFSET
        let limit_offset = self.dialect.limit_offset(query.limit, query.offset);
        if !limit_offset.is_empty() {
            sql_parts.push(limit_offset);
        }

        TranslatedQuery::new(sql_parts.join("\n"), false, self.dialect)
    }

    /// Generates SQL for a hot path (observables table) query.
    fn generate_hot_path_query(&self, query: &ValidatedQuery) -> TranslatedQuery {
        let mut sql_parts = Vec::new();

        // SELECT clause for hot path metrics
        let select_clause = self.build_hot_path_select_clause(query);
        sql_parts.push(format!("SELECT {}", select_clause));

        // FROM observables table
        sql_parts.push(format!("FROM {}", self.observables_table));

        // WHERE clause with type_id filter
        let mut where_conditions = Vec::new();
        
        // Add type_id filter for hot path metrics
        let type_ids: Vec<String> = query
            .metrics
            .iter()
            .filter_map(|m| m.observable_type_id)
            .map(|id| id.to_string())
            .collect();
        
        if !type_ids.is_empty() {
            if type_ids.len() == 1 {
                where_conditions.push(format!("type_id = {}", type_ids[0]));
            } else {
                where_conditions.push(format!("type_id IN ({})", type_ids.join(", ")));
            }
        }

        // Add time range filter
        if let Some(ref time_range) = query.time_range {
            where_conditions.push(format!(
                "event_time BETWEEN '{}' AND '{}'",
                time_range.start, time_range.end
            ));
        }

        // Add other filters
        for filter in &query.filters {
            where_conditions.push(self.filter_to_sql(filter, &query.entity));
        }

        if !where_conditions.is_empty() {
            sql_parts.push(format!("WHERE {}", where_conditions.join(" AND ")));
        }

        // GROUP BY clause
        if let Some(group_by) = self.build_hot_path_group_by_clause(query) {
            sql_parts.push(format!("GROUP BY {}", group_by));
        }

        // ORDER BY clause
        if let Some(order_by) = self.build_order_by_clause(query) {
            sql_parts.push(format!("ORDER BY {}", order_by));
        }

        // LIMIT/OFFSET
        let limit_offset = self.dialect.limit_offset(query.limit, query.offset);
        if !limit_offset.is_empty() {
            sql_parts.push(limit_offset);
        }

        TranslatedQuery::new(sql_parts.join("\n"), true, self.dialect)
    }

    /// Builds the SELECT clause for entity queries.
    fn build_select_clause(&self, query: &ValidatedQuery) -> String {
        let mut columns = Vec::new();

        // Add time granularity column if specified
        if let Some(granularity) = query.time_granularity {
            let time_col = self.dialect.date_trunc_fn(
                granularity.sql_date_trunc(),
                "event_time",
            );
            columns.push(format!("{} AS time_bucket", time_col));
        }

        // Add selected attributes
        for attr in &query.attributes {
            let col_expr = self.attribute_to_sql(attr);
            columns.push(format!("{} AS {}", col_expr, attr.name));
        }

        // Add group by attributes that aren't already selected
        for group_attr in &query.group_by {
            if !query.attributes.iter().any(|a| &a.name == group_attr) {
                if let Some(attr) = query.entity.get_attribute(group_attr) {
                    let col_expr = self.attribute_to_sql(attr);
                    columns.push(format!("{} AS {}", col_expr, attr.name));
                }
            }
        }

        // Add metrics
        for metric in &query.metrics {
            let metric_expr = self.metric_to_sql(metric);
            columns.push(format!("{} AS {}", metric_expr, metric.name));
        }

        if columns.is_empty() {
            "*".to_string()
        } else {
            columns.join(", ")
        }
    }

    /// Builds the SELECT clause for hot path queries.
    fn build_hot_path_select_clause(&self, query: &ValidatedQuery) -> String {
        let mut columns = Vec::new();

        // Add time granularity column if specified
        if let Some(granularity) = query.time_granularity {
            let time_col = self.dialect.date_trunc_fn(
                granularity.sql_date_trunc(),
                "event_time",
            );
            columns.push(format!("{} AS time_bucket", time_col));
        }

        // Add metrics (hot path metrics operate on observables table)
        for metric in &query.metrics {
            let metric_expr = self.hot_path_metric_to_sql(metric);
            columns.push(format!("{} AS {}", metric_expr, metric.name));
        }

        if columns.is_empty() {
            "*".to_string()
        } else {
            columns.join(", ")
        }
    }

    /// Builds the WHERE clause.
    fn build_where_clause(&self, query: &ValidatedQuery) -> Option<String> {
        let mut conditions = Vec::new();

        // Add time range filter
        if let Some(ref time_range) = query.time_range {
            conditions.push(format!(
                "event_time BETWEEN '{}' AND '{}'",
                time_range.start, time_range.end
            ));
        }

        // Add filters
        for filter in &query.filters {
            conditions.push(self.filter_to_sql(filter, &query.entity));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }

    /// Builds the GROUP BY clause.
    fn build_group_by_clause(&self, query: &ValidatedQuery) -> Option<String> {
        if query.metrics.is_empty() && query.group_by.is_empty() && query.time_granularity.is_none() {
            return None;
        }

        let mut group_cols = Vec::new();

        // Add time bucket if time granularity is specified
        if query.time_granularity.is_some() {
            group_cols.push("time_bucket".to_string());
        }

        // Add group by attributes
        for group_attr in &query.group_by {
            if let Some(attr) = query.entity.get_attribute(group_attr) {
                group_cols.push(self.attribute_to_sql(attr));
            }
        }

        // Add selected attributes (non-aggregated)
        for attr in &query.attributes {
            let col_expr = self.attribute_to_sql(attr);
            if !group_cols.contains(&col_expr) {
                group_cols.push(col_expr);
            }
        }

        if group_cols.is_empty() {
            None
        } else {
            Some(group_cols.join(", "))
        }
    }

    /// Builds the GROUP BY clause for hot path queries.
    fn build_hot_path_group_by_clause(&self, query: &ValidatedQuery) -> Option<String> {
        let mut group_cols = Vec::new();

        // Add time bucket if time granularity is specified
        if query.time_granularity.is_some() {
            group_cols.push("time_bucket".to_string());
        }

        // Hot path queries typically group by type_id
        if query.metrics.iter().any(|m| m.observable_type_id.is_some()) {
            group_cols.push("type_id".to_string());
        }

        if group_cols.is_empty() {
            None
        } else {
            Some(group_cols.join(", "))
        }
    }

    /// Builds the ORDER BY clause.
    fn build_order_by_clause(&self, query: &ValidatedQuery) -> Option<String> {
        if query.order_by.is_empty() {
            return None;
        }

        let order_parts: Vec<String> = query
            .order_by
            .iter()
            .map(|o| format!("{} {}", o.field, o.direction.sql()))
            .collect();

        Some(order_parts.join(", "))
    }

    /// Converts an attribute to its SQL expression.
    fn attribute_to_sql(&self, attr: &SemanticAttribute) -> String {
        if let Some(ref expr) = attr.ocsf_mapping.expression {
            expr.clone()
        } else if let Some(ref field) = attr.ocsf_mapping.field {
            field.clone()
        } else {
            attr.name.clone()
        }
    }

    /// Converts a metric to its SQL expression.
    fn metric_to_sql(&self, metric: &SemanticMetric) -> String {
        let measure = if let Some(ref expr) = metric.measure.expression {
            expr.clone()
        } else if let Some(ref field) = metric.measure.field {
            field.clone()
        } else {
            "*".to_string()
        };

        match metric.aggregation {
            crate::metric::Aggregation::Count => format!("COUNT({})", measure),
            crate::metric::Aggregation::Sum => format!("SUM({})", measure),
            crate::metric::Aggregation::Avg => format!("AVG({})", measure),
            crate::metric::Aggregation::Min => format!("MIN({})", measure),
            crate::metric::Aggregation::Max => format!("MAX({})", measure),
            crate::metric::Aggregation::CountDistinct => format!("COUNT(DISTINCT {})", measure),
        }
    }

    /// Converts a hot path metric to its SQL expression.
    fn hot_path_metric_to_sql(&self, metric: &SemanticMetric) -> String {
        // Hot path metrics typically count or aggregate observable values
        match metric.aggregation {
            crate::metric::Aggregation::Count => "COUNT(*)".to_string(),
            crate::metric::Aggregation::CountDistinct => "COUNT(DISTINCT value)".to_string(),
            _ => self.metric_to_sql(metric),
        }
    }

    /// Converts a filter to its SQL expression.
    fn filter_to_sql(&self, filter: &QueryFilter, entity: &SemanticEntity) -> String {
        let column = if let Some(attr) = entity.get_attribute(&filter.attribute) {
            self.attribute_to_sql(attr)
        } else {
            filter.attribute.clone()
        };

        match filter.operator {
            FilterOperator::IsNull => format!("{} IS NULL", column),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", column),
            _ => format!(
                "{} {} {}",
                column,
                filter.operator.sql_operator(),
                filter.value.to_sql()
            ),
        }
    }

    /// Converts an entity name to a table name.
    fn entity_to_table_name(&self, entity_name: &str) -> String {
        // Convention: entity names map to OCSF event tables
        format!("ocsf_{}", entity_name)
    }

    /// Generates a reverse lookup query to retrieve full events from observable matches.
    ///
    /// This is used in the hot path → cold path flow where threat intel matches
    /// in the observables table need to be resolved to full OCSF events.
    pub fn generate_reverse_lookup(
        &self,
        matches: &[ObservableMatch],
        target_table: &str,
    ) -> TranslatedQuery {
        if matches.is_empty() {
            return TranslatedQuery::new(
                format!("SELECT * FROM {} WHERE 1=0", target_table),
                false,
                self.dialect,
            );
        }

        let mut sql_parts = Vec::new();

        // SELECT all columns from the target table
        sql_parts.push("SELECT *".to_string());
        sql_parts.push(format!("FROM {}", target_table));

        // Build WHERE clause based on event UIDs
        let event_uids: Vec<String> = matches
            .iter()
            .map(|m| format!("'{}'", m.event_uid.replace('\'', "''")))
            .collect();

        // Use IN clause for batch lookups (optimized)
        if event_uids.len() == 1 {
            sql_parts.push(format!("WHERE metadata.uid = {}", event_uids[0]));
        } else {
            sql_parts.push(format!("WHERE metadata.uid IN ({})", event_uids.join(", ")));
        }

        // Add ordering by event time for consistent results
        sql_parts.push("ORDER BY event_time DESC".to_string());

        TranslatedQuery::new(sql_parts.join("\n"), false, self.dialect)
    }

    /// Generates a reverse lookup query with additional filtering.
    ///
    /// This variant allows filtering by event class and time range for
    /// more efficient lookups.
    pub fn generate_reverse_lookup_filtered(
        &self,
        matches: &[ObservableMatch],
        target_table: &str,
        time_range: Option<&TimeRange>,
    ) -> TranslatedQuery {
        if matches.is_empty() {
            return TranslatedQuery::new(
                format!("SELECT * FROM {} WHERE 1=0", target_table),
                false,
                self.dialect,
            );
        }

        let mut sql_parts = Vec::new();

        // SELECT all columns from the target table
        sql_parts.push("SELECT *".to_string());
        sql_parts.push(format!("FROM {}", target_table));

        // Build WHERE clause
        let mut where_conditions = Vec::new();

        // Filter by event UIDs
        let event_uids: Vec<String> = matches
            .iter()
            .map(|m| format!("'{}'", m.event_uid.replace('\'', "''")))
            .collect();

        if event_uids.len() == 1 {
            where_conditions.push(format!("metadata.uid = {}", event_uids[0]));
        } else {
            where_conditions.push(format!("metadata.uid IN ({})", event_uids.join(", ")));
        }

        // Filter by event class UIDs if available
        let class_uids: Vec<u32> = matches
            .iter()
            .map(|m| m.event_class_uid)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if !class_uids.is_empty() {
            if class_uids.len() == 1 {
                where_conditions.push(format!("class_uid = {}", class_uids[0]));
            } else {
                let class_uid_strs: Vec<String> = class_uids.iter().map(|c| c.to_string()).collect();
                where_conditions.push(format!("class_uid IN ({})", class_uid_strs.join(", ")));
            }
        }

        // Add time range filter if provided
        if let Some(range) = time_range {
            where_conditions.push(format!(
                "event_time BETWEEN '{}' AND '{}'",
                range.start, range.end
            ));
        }

        sql_parts.push(format!("WHERE {}", where_conditions.join(" AND ")));

        // Add ordering by event time for consistent results
        sql_parts.push("ORDER BY event_time DESC".to_string());

        TranslatedQuery::new(sql_parts.join("\n"), false, self.dialect)
    }

    /// Generates a batch reverse lookup query using a subquery.
    ///
    /// This is optimized for large numbers of matches by using a subquery
    /// against the observables table.
    pub fn generate_batch_reverse_lookup(
        &self,
        observable_filter: &ObservableFilter,
        target_table: &str,
    ) -> TranslatedQuery {
        let mut sql_parts = Vec::new();

        // SELECT all columns from the target table
        sql_parts.push("SELECT e.*".to_string());
        sql_parts.push(format!("FROM {} e", target_table));

        // JOIN with observables table using subquery
        sql_parts.push(format!(
            "INNER JOIN (SELECT DISTINCT event_uid FROM {} WHERE {}) o",
            self.observables_table,
            self.build_observable_filter(observable_filter)
        ));
        sql_parts.push("ON e.metadata.uid = o.event_uid".to_string());

        // Add ordering
        sql_parts.push("ORDER BY e.event_time DESC".to_string());

        // Add limit if specified
        if let Some(limit) = observable_filter.limit {
            sql_parts.push(format!("LIMIT {}", limit));
        }

        TranslatedQuery::new(sql_parts.join("\n"), false, self.dialect)
    }

    /// Builds the WHERE clause for observable filtering.
    fn build_observable_filter(&self, filter: &ObservableFilter) -> String {
        let mut conditions = Vec::new();

        // Filter by type_id
        if !filter.type_ids.is_empty() {
            if filter.type_ids.len() == 1 {
                conditions.push(format!("type_id = {}", filter.type_ids[0]));
            } else {
                let type_id_strs: Vec<String> = filter.type_ids.iter().map(|t| t.to_string()).collect();
                conditions.push(format!("type_id IN ({})", type_id_strs.join(", ")));
            }
        }

        // Filter by values
        if !filter.values.is_empty() {
            let value_strs: Vec<String> = filter
                .values
                .iter()
                .map(|v| format!("'{}'", v.replace('\'', "''")))
                .collect();
            conditions.push(format!("value IN ({})", value_strs.join(", ")));
        }

        // Filter by time range
        if let Some(ref range) = filter.time_range {
            conditions.push(format!(
                "event_time BETWEEN '{}' AND '{}'",
                range.start, range.end
            ));
        }

        if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        }
    }
}

/// An observable match from the hot path query.
///
/// Represents a match found in the observables table that needs to be
/// resolved to a full OCSF event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableMatch {
    /// The observable type_id.
    pub type_id: u32,
    /// The observable value that matched.
    pub value: String,
    /// The source event UID for reverse lookup.
    pub event_uid: String,
    /// The source event class UID.
    pub event_class_uid: u32,
    /// The event timestamp.
    pub event_time: String,
}

impl ObservableMatch {
    /// Creates a new observable match.
    pub fn new(
        type_id: u32,
        value: impl Into<String>,
        event_uid: impl Into<String>,
        event_class_uid: u32,
        event_time: impl Into<String>,
    ) -> Self {
        Self {
            type_id,
            value: value.into(),
            event_uid: event_uid.into(),
            event_class_uid,
            event_time: event_time.into(),
        }
    }
}

/// Filter criteria for observable queries.
#[derive(Debug, Clone, Default)]
pub struct ObservableFilter {
    /// Observable type_ids to filter by.
    pub type_ids: Vec<u32>,
    /// Observable values to match.
    pub values: Vec<String>,
    /// Time range filter.
    pub time_range: Option<TimeRange>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl ObservableFilter {
    /// Creates a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds type_ids to filter by.
    pub fn with_type_ids(mut self, type_ids: Vec<u32>) -> Self {
        self.type_ids = type_ids;
        self
    }

    /// Adds values to match.
    pub fn with_values(mut self, values: Vec<String>) -> Self {
        self.values = values;
        self
    }

    /// Sets the time range.
    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    /// Sets the limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Cardinality, EntityRelationship, SemanticAttribute, SemanticType};
    use crate::metric::Aggregation;

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_field_mapping("actor.user.email_addr")
                            .as_dimension(),
                    )
                    .add_attribute(
                        SemanticAttribute::new("auth_result")
                            .with_type(SemanticType::String)
                            .with_expression_mapping(
                                "CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END",
                            )
                            .as_dimension(),
                    )
                    .add_attribute(
                        SemanticAttribute::new("source_ip")
                            .with_type(SemanticType::String)
                            .with_field_mapping("src_endpoint.ip")
                            .as_dimension(),
                    ),
            )
            .add_entity(
                SemanticEntity::new("user")
                    .with_source_event_classes(vec![3001])
                    .add_attribute(
                        SemanticAttribute::new("email")
                            .with_type(SemanticType::String)
                            .with_field_mapping("user.email_addr"),
                    )
                    .add_attribute(
                        SemanticAttribute::new("name")
                            .with_type(SemanticType::String)
                            .with_field_mapping("user.name"),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_attempts")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("metadata.uid")
                    .with_dimensions(vec![
                        "user_email".to_string(),
                        "auth_result".to_string(),
                        "source_ip".to_string(),
                    ])
                    .with_time_granularities(vec![
                        TimeGranularity::Minute,
                        TimeGranularity::Hour,
                        TimeGranularity::Day,
                    ]),
            )
            .add_metric(
                SemanticMetric::new("threat_intel_matches")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("observable_value")
                    .with_observable_type_id(2)
                    .as_hot_path(),
            )
    }

    fn create_model_with_relationships() -> SemanticModel {
        SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_source_event_classes(vec![3002])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_type(SemanticType::String)
                            .with_field_mapping("actor.user.email_addr")
                            .as_dimension(),
                    )
                    .add_relationship(
                        EntityRelationship::new(
                            "performed_by",
                            "user",
                            "authentication_event.user_email = user.email",
                        )
                        .with_cardinality(Cardinality::ManyToMany),
                    ),
            )
            .add_entity(
                SemanticEntity::new("user")
                    .with_source_event_classes(vec![3001])
                    .add_attribute(
                        SemanticAttribute::new("email")
                            .with_type(SemanticType::String)
                            .with_field_mapping("user.email_addr"),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_attempts")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("metadata.uid"),
            )
    }

    #[test]
    fn test_semantic_query_builder() {
        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .add_select("auth_result")
            .add_filter(QueryFilter::eq("auth_result", "failure"))
            .with_limit(100);

        assert_eq!(query.entity, "authentication_event");
        assert_eq!(query.select, vec!["user_email", "auth_result"]);
        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_metric_query_builder() {
        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .add_group_by("user_email")
            .with_time_granularity(TimeGranularity::Hour)
            .with_time_range(TimeRange::new("-24h", "now"));

        assert!(query.is_metric_query());
        assert_eq!(query.metrics, vec!["auth_attempts"]);
        assert_eq!(query.group_by, vec!["user_email"]);
        assert_eq!(query.time_granularity, Some(TimeGranularity::Hour));
    }

    #[test]
    fn test_query_validation_success() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .add_select("auth_result");

        let result = validator.validate(&query);
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.entity.name, "authentication_event");
        assert_eq!(validated.attributes.len(), 2);
    }

    #[test]
    fn test_query_validation_unknown_entity() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("nonexistent_entity").add_select("some_attr");

        let result = validator.validate(&query);
        assert!(matches!(result, Err(QueryError::UnknownEntity(_, _))));
    }

    #[test]
    fn test_query_validation_unknown_attribute() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query =
            SemanticQuery::new("authentication_event").add_select("nonexistent_attribute");

        let result = validator.validate(&query);
        assert!(matches!(result, Err(QueryError::UnknownAttribute(_, _, _))));
    }

    #[test]
    fn test_query_validation_unknown_metric() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event").add_metric("nonexistent_metric");

        let result = validator.validate(&query);
        assert!(matches!(result, Err(QueryError::UnknownMetric(_, _))));
    }

    #[test]
    fn test_query_validation_empty_query() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event");

        let result = validator.validate(&query);
        assert!(matches!(result, Err(QueryError::EmptyQuery)));
    }

    #[test]
    fn test_query_validation_metric_with_dimensions() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .add_group_by("user_email")
            .with_time_granularity(TimeGranularity::Hour);

        let result = validator.validate(&query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_validation_unsupported_time_granularity() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_granularity(TimeGranularity::Week); // Not supported

        let result = validator.validate(&query);
        assert!(matches!(
            result,
            Err(QueryError::UnsupportedTimeGranularity(_, _, _))
        ));
    }

    #[test]
    fn test_hot_path_detection() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        // Hot path metric with auto preference should use hot path
        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Auto);

        let validated = validator.validate(&query).unwrap();
        assert!(validated.uses_hot_path);

        // Non-hot path metric with auto preference should not use hot path
        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_path_preference(PathPreference::Auto);

        let validated = validator.validate(&query).unwrap();
        assert!(!validated.uses_hot_path);
    }

    #[test]
    fn test_hot_path_not_supported_error() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_path_preference(PathPreference::Hot);

        let result = validator.validate(&query);
        assert!(matches!(result, Err(QueryError::HotPathNotSupported(_))));
    }

    #[test]
    fn test_filter_value_to_sql() {
        assert_eq!(FilterValue::String("test".to_string()).to_sql(), "'test'");
        assert_eq!(
            FilterValue::String("it's".to_string()).to_sql(),
            "'it''s'"
        );
        assert_eq!(FilterValue::Integer(42).to_sql(), "42");
        assert_eq!(FilterValue::Float(3.14).to_sql(), "3.14");
        assert_eq!(FilterValue::Boolean(true).to_sql(), "TRUE");
        assert_eq!(FilterValue::Boolean(false).to_sql(), "FALSE");
        assert_eq!(
            FilterValue::List(vec!["a".to_string(), "b".to_string()]).to_sql(),
            "('a', 'b')"
        );
        assert_eq!(FilterValue::Null.to_sql(), "NULL");
    }

    #[test]
    fn test_filter_operators() {
        assert_eq!(FilterOperator::Eq.sql_operator(), "=");
        assert_eq!(FilterOperator::Ne.sql_operator(), "!=");
        assert_eq!(FilterOperator::Gt.sql_operator(), ">");
        assert_eq!(FilterOperator::Gte.sql_operator(), ">=");
        assert_eq!(FilterOperator::Lt.sql_operator(), "<");
        assert_eq!(FilterOperator::Lte.sql_operator(), "<=");
        assert_eq!(FilterOperator::In.sql_operator(), "IN");
        assert_eq!(FilterOperator::NotIn.sql_operator(), "NOT IN");
        assert_eq!(FilterOperator::Like.sql_operator(), "LIKE");
        assert_eq!(FilterOperator::IsNull.sql_operator(), "IS NULL");
        assert_eq!(FilterOperator::IsNotNull.sql_operator(), "IS NOT NULL");
    }

    // SQL Generator Tests

    #[test]
    fn test_sql_generator_simple_select() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .add_select("auth_result");

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(!result.uses_hot_path);
        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("actor.user.email_addr AS user_email"));
        assert!(result.sql.contains("CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END AS auth_result"));
        assert!(result.sql.contains("FROM ocsf_authentication_event"));
    }

    #[test]
    fn test_sql_generator_with_filter() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .add_filter(QueryFilter::eq("auth_result", "failure"));

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("WHERE"));
        assert!(result.sql.contains("= 'failure'"));
    }

    #[test]
    fn test_sql_generator_with_metric() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .add_group_by("user_email");

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("COUNT(metadata.uid) AS auth_attempts"));
        assert!(result.sql.contains("GROUP BY"));
    }

    #[test]
    fn test_sql_generator_with_time_granularity() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_granularity(TimeGranularity::Hour);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("DATE_TRUNC('hour', event_time) AS time_bucket"));
        assert!(result.sql.contains("GROUP BY"));
    }

    #[test]
    fn test_sql_generator_with_limit_offset() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .with_limit(100)
            .with_offset(50);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("LIMIT 100"));
        assert!(result.sql.contains("OFFSET 50"));
    }

    #[test]
    fn test_sql_generator_with_order_by() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email")
            .add_order_by(OrderBy::desc("user_email"));

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("ORDER BY user_email DESC"));
    }

    #[test]
    fn test_sql_generator_hot_path() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Hot);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.uses_hot_path);
        assert!(result.sql.contains("FROM ocsf_observables"));
        assert!(result.sql.contains("type_id = 2"));
    }

    #[test]
    fn test_sql_generator_with_joins() {
        let model = create_model_with_relationships();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_select("user_email");

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("LEFT JOIN ocsf_user ON"));
    }

    #[test]
    fn test_warehouse_dialect_date_trunc() {
        // Snowflake
        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(
            snowflake.date_trunc_fn("hour", "event_time"),
            "DATE_TRUNC('hour', event_time)"
        );

        // BigQuery
        let bigquery = WarehouseDialect::BigQuery;
        assert_eq!(
            bigquery.date_trunc_fn("hour", "event_time"),
            "TIMESTAMP_TRUNC(event_time, HOUR)"
        );

        // Postgres
        let postgres = WarehouseDialect::Postgres;
        assert_eq!(
            postgres.date_trunc_fn("hour", "event_time"),
            "DATE_TRUNC('hour', event_time)"
        );
    }

    #[test]
    fn test_warehouse_dialect_limit_offset() {
        let dialect = WarehouseDialect::Snowflake;
        
        assert_eq!(dialect.limit_offset(Some(100), None), "LIMIT 100");
        assert_eq!(dialect.limit_offset(None, Some(50)), "OFFSET 50");
        assert_eq!(dialect.limit_offset(Some(100), Some(50)), "LIMIT 100 OFFSET 50");
        assert_eq!(dialect.limit_offset(None, None), "");
    }

    #[test]
    fn test_warehouse_dialect_json_extract() {
        // BigQuery
        let bigquery = WarehouseDialect::BigQuery;
        assert_eq!(
            bigquery.json_extract("data", "user.name"),
            "JSON_EXTRACT_SCALAR(data, '$.user.name')"
        );

        // Snowflake
        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(
            snowflake.json_extract("data", "user.name"),
            "data:user.name"
        );

        // Postgres
        let postgres = WarehouseDialect::Postgres;
        assert_eq!(
            postgres.json_extract("data", "user.name"),
            "data ->> 'user.name'"
        );
    }

    #[test]
    fn test_warehouse_dialect_array_contains() {
        // BigQuery
        let bigquery = WarehouseDialect::BigQuery;
        assert_eq!(
            bigquery.array_contains("tags", "'important'"),
            "'important' IN UNNEST(tags)"
        );

        // Snowflake
        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(
            snowflake.array_contains("tags", "'important'"),
            "ARRAY_CONTAINS('important', tags)"
        );

        // Postgres
        let postgres = WarehouseDialect::Postgres;
        assert_eq!(
            postgres.array_contains("tags", "'important'"),
            "'important' = ANY(tags)"
        );
    }

    #[test]
    fn test_warehouse_dialect_types() {
        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(snowflake.string_type(), "VARCHAR");
        assert_eq!(snowflake.integer_type(), "INTEGER");
        assert_eq!(snowflake.timestamp_type(), "TIMESTAMP_NTZ");
        assert_eq!(snowflake.boolean_type(), "BOOLEAN");
        assert_eq!(snowflake.json_type(), "VARIANT");

        let bigquery = WarehouseDialect::BigQuery;
        assert_eq!(bigquery.string_type(), "STRING");
        assert_eq!(bigquery.integer_type(), "INT64");
        assert_eq!(bigquery.timestamp_type(), "TIMESTAMP");
        assert_eq!(bigquery.boolean_type(), "BOOL");
        assert_eq!(bigquery.json_type(), "JSON");

        let postgres = WarehouseDialect::Postgres;
        assert_eq!(postgres.string_type(), "TEXT");
        assert_eq!(postgres.integer_type(), "BIGINT");
        assert_eq!(postgres.timestamp_type(), "TIMESTAMP");
        assert_eq!(postgres.boolean_type(), "BOOLEAN");
        assert_eq!(postgres.json_type(), "JSONB");
    }

    #[test]
    fn test_warehouse_dialect_quote_identifier() {
        let bigquery = WarehouseDialect::BigQuery;
        assert_eq!(bigquery.quote_identifier("my_table"), "`my_table`");

        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(snowflake.quote_identifier("my_table"), "\"my_table\"");

        let postgres = WarehouseDialect::Postgres;
        assert_eq!(postgres.quote_identifier("my_table"), "\"my_table\"");
    }

    #[test]
    fn test_warehouse_dialect_features() {
        let snowflake = WarehouseDialect::Snowflake;
        assert!(snowflake.supports_qualify());
        assert!(!snowflake.supports_struct());

        let bigquery = WarehouseDialect::BigQuery;
        assert!(bigquery.supports_qualify());
        assert!(bigquery.supports_struct());

        let postgres = WarehouseDialect::Postgres;
        assert!(!postgres.supports_qualify());
        assert!(!postgres.supports_struct());
    }

    #[test]
    fn test_warehouse_dialect_name() {
        assert_eq!(WarehouseDialect::Snowflake.name(), "snowflake");
        assert_eq!(WarehouseDialect::Databricks.name(), "databricks");
        assert_eq!(WarehouseDialect::BigQuery.name(), "bigquery");
        assert_eq!(WarehouseDialect::Postgres.name(), "postgres");
    }

    #[test]
    fn test_warehouse_dialect_current_timestamp() {
        let snowflake = WarehouseDialect::Snowflake;
        assert_eq!(snowflake.current_timestamp_fn(), "CURRENT_TIMESTAMP()");

        let postgres = WarehouseDialect::Postgres;
        assert_eq!(postgres.current_timestamp_fn(), "CURRENT_TIMESTAMP");
    }

    #[test]
    fn test_all_aggregation_types() {
        // Create a model with metrics for each aggregation type
        let model = SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("events")
                    .add_attribute(
                        SemanticAttribute::new("value")
                            .with_type(SemanticType::Integer)
                            .with_field_mapping("data.value"),
                    ),
            )
            .add_metric(
                SemanticMetric::new("count_metric")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("id"),
            )
            .add_metric(
                SemanticMetric::new("sum_metric")
                    .with_aggregation(Aggregation::Sum)
                    .with_field_measure("data.value"),
            )
            .add_metric(
                SemanticMetric::new("avg_metric")
                    .with_aggregation(Aggregation::Avg)
                    .with_field_measure("data.value"),
            )
            .add_metric(
                SemanticMetric::new("min_metric")
                    .with_aggregation(Aggregation::Min)
                    .with_field_measure("data.value"),
            )
            .add_metric(
                SemanticMetric::new("max_metric")
                    .with_aggregation(Aggregation::Max)
                    .with_field_measure("data.value"),
            )
            .add_metric(
                SemanticMetric::new("count_distinct_metric")
                    .with_aggregation(Aggregation::CountDistinct)
                    .with_field_measure("user_id"),
            );

        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Test COUNT
        let query = SemanticQuery::new("events").add_metric("count_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("COUNT(id) AS count_metric"));

        // Test SUM
        let query = SemanticQuery::new("events").add_metric("sum_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("SUM(data.value) AS sum_metric"));

        // Test AVG
        let query = SemanticQuery::new("events").add_metric("avg_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("AVG(data.value) AS avg_metric"));

        // Test MIN
        let query = SemanticQuery::new("events").add_metric("min_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("MIN(data.value) AS min_metric"));

        // Test MAX
        let query = SemanticQuery::new("events").add_metric("max_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("MAX(data.value) AS max_metric"));

        // Test COUNT DISTINCT
        let query = SemanticQuery::new("events").add_metric("count_distinct_metric");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("COUNT(DISTINCT user_id) AS count_distinct_metric"));
    }

    #[test]
    fn test_all_time_granularities() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Test minute granularity
        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_granularity(TimeGranularity::Minute);
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("DATE_TRUNC('minute', event_time) AS time_bucket"));

        // Test hour granularity
        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_granularity(TimeGranularity::Hour);
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("DATE_TRUNC('hour', event_time) AS time_bucket"));

        // Test day granularity
        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_granularity(TimeGranularity::Day);
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);
        assert!(result.sql.contains("DATE_TRUNC('day', event_time) AS time_bucket"));
    }

    #[test]
    fn test_metric_with_expression_measure() {
        let model = SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("events")
                    .add_attribute(
                        SemanticAttribute::new("status")
                            .with_type(SemanticType::String)
                            .with_field_mapping("status"),
                    ),
            )
            .add_metric(
                SemanticMetric::new("failure_rate")
                    .with_aggregation(Aggregation::Avg)
                    .with_expression_measure("CASE WHEN status = 'failure' THEN 1.0 ELSE 0.0 END"),
            );

        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("events").add_metric("failure_rate");
        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("AVG(CASE WHEN status = 'failure' THEN 1.0 ELSE 0.0 END) AS failure_rate"));
    }

    #[test]
    fn test_metric_with_time_range() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("auth_attempts")
            .with_time_range(TimeRange::new("2024-01-01", "2024-01-31"));

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("WHERE"));
        assert!(result.sql.contains("event_time BETWEEN '2024-01-01' AND '2024-01-31'"));
    }

    // Hot Path Routing Tests

    #[test]
    fn test_hot_path_routing_with_observable_type_id() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Query with hot path metric
        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Hot);

        let validated = validator.validate(&query).unwrap();
        assert!(validated.uses_hot_path);

        let result = generator.generate(&validated);

        // Should query observables table
        assert!(result.uses_hot_path);
        assert!(result.sql.contains("FROM ocsf_observables"));
        // Should filter by type_id
        assert!(result.sql.contains("type_id = 2"));
    }

    #[test]
    fn test_hot_path_auto_detection() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Query with hot path metric and auto preference
        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Auto);

        let validated = validator.validate(&query).unwrap();
        // Auto should detect hot path metric
        assert!(validated.uses_hot_path);

        let result = generator.generate(&validated);
        assert!(result.uses_hot_path);
    }

    #[test]
    fn test_cold_path_forced() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Force cold path even for hot path metric
        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Cold);

        let validated = validator.validate(&query).unwrap();
        // Cold preference should override
        assert!(!validated.uses_hot_path);

        let result = generator.generate(&validated);
        assert!(!result.uses_hot_path);
        assert!(result.sql.contains("FROM ocsf_authentication_event"));
    }

    #[test]
    fn test_hot_path_with_time_range() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Hot)
            .with_time_range(TimeRange::new("2024-01-01", "2024-01-31"));

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.uses_hot_path);
        assert!(result.sql.contains("type_id = 2"));
        assert!(result.sql.contains("event_time BETWEEN '2024-01-01' AND '2024-01-31'"));
    }

    #[test]
    fn test_hot_path_with_time_granularity() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Hot)
            .with_time_granularity(TimeGranularity::Hour);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.uses_hot_path);
        assert!(result.sql.contains("time_bucket"));
        assert!(result.sql.contains("GROUP BY"));
    }

    #[test]
    fn test_hot_path_multiple_type_ids() {
        // Create model with multiple hot path metrics
        let model = SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("events")
                    .add_attribute(
                        SemanticAttribute::new("value")
                            .with_type(SemanticType::String)
                            .with_field_mapping("value"),
                    ),
            )
            .add_metric(
                SemanticMetric::new("ip_matches")
                    .with_aggregation(Aggregation::Count)
                    .with_observable_type_id(2)
                    .as_hot_path(),
            )
            .add_metric(
                SemanticMetric::new("email_matches")
                    .with_aggregation(Aggregation::Count)
                    .with_observable_type_id(5)
                    .as_hot_path(),
            );

        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        // Query with multiple hot path metrics
        let query = SemanticQuery::new("events")
            .add_metric("ip_matches")
            .add_metric("email_matches")
            .with_path_preference(PathPreference::Hot);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.uses_hot_path);
        // Should have IN clause for multiple type_ids
        assert!(result.sql.contains("type_id IN (2, 5)"));
    }

    #[test]
    fn test_custom_observables_table_name() {
        let model = create_test_model();
        let validator = QueryValidator::new(&model);
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake)
            .with_observables_table("custom_observables_table");

        let query = SemanticQuery::new("authentication_event")
            .add_metric("threat_intel_matches")
            .with_path_preference(PathPreference::Hot);

        let validated = validator.validate(&query).unwrap();
        let result = generator.generate(&validated);

        assert!(result.sql.contains("FROM custom_observables_table"));
    }

    // Reverse Lookup Tests

    #[test]
    fn test_reverse_lookup_single_match() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches = vec![ObservableMatch::new(
            2,
            "192.168.1.1",
            "event-uid-123",
            3002,
            "2024-01-15T10:30:00Z",
        )];

        let result = generator.generate_reverse_lookup(&matches, "ocsf_authentication");

        assert!(!result.uses_hot_path);
        assert!(result.sql.contains("SELECT *"));
        assert!(result.sql.contains("FROM ocsf_authentication"));
        assert!(result.sql.contains("metadata.uid = 'event-uid-123'"));
        assert!(result.sql.contains("ORDER BY event_time DESC"));
    }

    #[test]
    fn test_reverse_lookup_multiple_matches() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches = vec![
            ObservableMatch::new(2, "192.168.1.1", "event-uid-1", 3002, "2024-01-15T10:30:00Z"),
            ObservableMatch::new(2, "192.168.1.2", "event-uid-2", 3002, "2024-01-15T10:31:00Z"),
            ObservableMatch::new(2, "192.168.1.3", "event-uid-3", 3002, "2024-01-15T10:32:00Z"),
        ];

        let result = generator.generate_reverse_lookup(&matches, "ocsf_authentication");

        assert!(result.sql.contains("metadata.uid IN ("));
        assert!(result.sql.contains("'event-uid-1'"));
        assert!(result.sql.contains("'event-uid-2'"));
        assert!(result.sql.contains("'event-uid-3'"));
    }

    #[test]
    fn test_reverse_lookup_empty_matches() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches: Vec<ObservableMatch> = vec![];

        let result = generator.generate_reverse_lookup(&matches, "ocsf_authentication");

        // Should return a query that returns no results
        assert!(result.sql.contains("WHERE 1=0"));
    }

    #[test]
    fn test_reverse_lookup_filtered_with_time_range() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches = vec![
            ObservableMatch::new(2, "192.168.1.1", "event-uid-1", 3002, "2024-01-15T10:30:00Z"),
            ObservableMatch::new(2, "192.168.1.2", "event-uid-2", 3003, "2024-01-15T10:31:00Z"),
        ];

        let time_range = TimeRange::new("2024-01-01", "2024-01-31");
        let result = generator.generate_reverse_lookup_filtered(
            &matches,
            "ocsf_authentication",
            Some(&time_range),
        );

        assert!(result.sql.contains("metadata.uid IN ("));
        assert!(result.sql.contains("class_uid IN ("));
        assert!(result.sql.contains("event_time BETWEEN '2024-01-01' AND '2024-01-31'"));
    }

    #[test]
    fn test_reverse_lookup_filtered_deduplicates_class_uids() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches = vec![
            ObservableMatch::new(2, "192.168.1.1", "event-uid-1", 3002, "2024-01-15T10:30:00Z"),
            ObservableMatch::new(2, "192.168.1.2", "event-uid-2", 3002, "2024-01-15T10:31:00Z"),
            ObservableMatch::new(2, "192.168.1.3", "event-uid-3", 3002, "2024-01-15T10:32:00Z"),
        ];

        let result = generator.generate_reverse_lookup_filtered(&matches, "ocsf_authentication", None);

        // Should have single class_uid = 3002, not IN clause
        assert!(result.sql.contains("class_uid = 3002"));
    }

    #[test]
    fn test_batch_reverse_lookup() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let filter = ObservableFilter::new()
            .with_type_ids(vec![2])
            .with_values(vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()])
            .with_time_range(TimeRange::new("2024-01-01", "2024-01-31"))
            .with_limit(1000);

        let result = generator.generate_batch_reverse_lookup(&filter, "ocsf_authentication");

        assert!(result.sql.contains("SELECT e.*"));
        assert!(result.sql.contains("FROM ocsf_authentication e"));
        assert!(result.sql.contains("INNER JOIN"));
        assert!(result.sql.contains("SELECT DISTINCT event_uid FROM ocsf_observables"));
        assert!(result.sql.contains("type_id = 2"));
        assert!(result.sql.contains("value IN ("));
        assert!(result.sql.contains("LIMIT 1000"));
    }

    #[test]
    fn test_observable_match_creation() {
        let match_obj = ObservableMatch::new(
            2,
            "192.168.1.1",
            "event-uid-123",
            3002,
            "2024-01-15T10:30:00Z",
        );

        assert_eq!(match_obj.type_id, 2);
        assert_eq!(match_obj.value, "192.168.1.1");
        assert_eq!(match_obj.event_uid, "event-uid-123");
        assert_eq!(match_obj.event_class_uid, 3002);
        assert_eq!(match_obj.event_time, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_observable_filter_builder() {
        let filter = ObservableFilter::new()
            .with_type_ids(vec![2, 5])
            .with_values(vec!["test@example.com".to_string()])
            .with_time_range(TimeRange::new("2024-01-01", "2024-01-31"))
            .with_limit(100);

        assert_eq!(filter.type_ids, vec![2, 5]);
        assert_eq!(filter.values, vec!["test@example.com"]);
        assert!(filter.time_range.is_some());
        assert_eq!(filter.limit, Some(100));
    }

    #[test]
    fn test_reverse_lookup_escapes_quotes() {
        let generator = SqlGenerator::new(WarehouseDialect::Snowflake);

        let matches = vec![ObservableMatch::new(
            2,
            "test'value",
            "event-uid-with'quote",
            3002,
            "2024-01-15T10:30:00Z",
        )];

        let result = generator.generate_reverse_lookup(&matches, "ocsf_authentication");

        // Should escape single quotes
        assert!(result.sql.contains("event-uid-with''quote"));
    }
}
