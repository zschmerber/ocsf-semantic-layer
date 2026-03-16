//! Semantic metric definitions.
//!
//! This module contains the semantic metric data structures for defining
//! business-level security metrics and KPIs.

use serde::{Deserialize, Serialize};

use crate::entity::OCSFMapping;

/// Aggregation function for metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Aggregation {
    /// Count of records.
    #[default]
    Count,
    /// Sum of values.
    Sum,
    /// Average of values.
    Avg,
    /// Minimum value.
    Min,
    /// Maximum value.
    Max,
    /// Count of distinct values.
    CountDistinct,
}


impl Aggregation {
    /// Returns the SQL function name for this aggregation.
    pub fn sql_function(&self) -> &'static str {
        match self {
            Self::Count => "COUNT",
            Self::Sum => "SUM",
            Self::Avg => "AVG",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::CountDistinct => "COUNT(DISTINCT",
        }
    }
}

/// Time granularity for time-based aggregations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeGranularity {
    /// Minute-level granularity.
    Minute,
    /// Hour-level granularity.
    Hour,
    /// Day-level granularity.
    Day,
    /// Week-level granularity.
    Week,
    /// Month-level granularity.
    Month,
}

impl TimeGranularity {
    /// Returns the SQL date truncation expression for this granularity.
    pub fn sql_date_trunc(&self) -> &'static str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

/// A semantic metric definition.
///
/// Metrics represent business-level KPIs and measurements that can be
/// calculated from OCSF event data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticMetric {
    /// Metric name (e.g., "auth_attempts", "failed_auth_rate").
    pub name: String,

    /// Human-readable caption.
    #[serde(default)]
    pub caption: String,

    /// Detailed description of the metric.
    #[serde(default)]
    pub description: String,

    /// Aggregation function to apply.
    #[serde(default)]
    pub aggregation: Aggregation,

    /// Source field or expression for the measure.
    #[serde(default)]
    pub measure: OCSFMapping,

    /// Dimension names that can be used to slice this metric.
    #[serde(default)]
    pub dimensions: Vec<String>,

    /// Supported time granularities for this metric.
    #[serde(default)]
    pub time_granularities: Vec<TimeGranularity>,

    /// Whether this metric operates on the hot path (observables table).
    #[serde(default)]
    pub is_hot_path: bool,

    /// Observable type_id if this metric operates on the observables table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observable_type_id: Option<u32>,
}

impl SemanticMetric {
    /// Creates a new semantic metric with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            caption: String::new(),
            description: String::new(),
            aggregation: Aggregation::default(),
            measure: OCSFMapping::default(),
            dimensions: Vec::new(),
            time_granularities: Vec::new(),
            is_hot_path: false,
            observable_type_id: None,
        }
    }

    /// Sets the caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the aggregation function.
    pub fn with_aggregation(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = aggregation;
        self
    }

    /// Sets the measure from a field reference.
    pub fn with_field_measure(mut self, field: impl Into<String>) -> Self {
        self.measure = OCSFMapping::from_field(field);
        self
    }

    /// Sets the measure from an expression.
    pub fn with_expression_measure(mut self, expression: impl Into<String>) -> Self {
        self.measure = OCSFMapping::from_expression(expression);
        self
    }

    /// Sets the measure mapping.
    pub fn with_measure(mut self, measure: OCSFMapping) -> Self {
        self.measure = measure;
        self
    }

    /// Sets the dimensions.
    pub fn with_dimensions(mut self, dimensions: Vec<String>) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Adds a dimension.
    pub fn add_dimension(mut self, dimension: impl Into<String>) -> Self {
        self.dimensions.push(dimension.into());
        self
    }

    /// Sets the time granularities.
    pub fn with_time_granularities(mut self, granularities: Vec<TimeGranularity>) -> Self {
        self.time_granularities = granularities;
        self
    }

    /// Adds a time granularity.
    pub fn add_time_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.time_granularities.push(granularity);
        self
    }

    /// Marks this metric as a hot path metric.
    pub fn as_hot_path(mut self) -> Self {
        self.is_hot_path = true;
        self
    }

    /// Sets the observable type_id for hot path metrics.
    pub fn with_observable_type_id(mut self, type_id: u32) -> Self {
        self.observable_type_id = Some(type_id);
        self.is_hot_path = true;
        self
    }

    /// Returns true if this metric supports the given time granularity.
    pub fn supports_granularity(&self, granularity: TimeGranularity) -> bool {
        self.time_granularities.contains(&granularity)
    }

    /// Returns true if this metric can be sliced by the given dimension.
    pub fn supports_dimension(&self, dimension: &str) -> bool {
        self.dimensions.iter().any(|d| d == dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_sql_function() {
        assert_eq!(Aggregation::Count.sql_function(), "COUNT");
        assert_eq!(Aggregation::Sum.sql_function(), "SUM");
        assert_eq!(Aggregation::Avg.sql_function(), "AVG");
        assert_eq!(Aggregation::Min.sql_function(), "MIN");
        assert_eq!(Aggregation::Max.sql_function(), "MAX");
        assert_eq!(Aggregation::CountDistinct.sql_function(), "COUNT(DISTINCT");
    }

    #[test]
    fn test_time_granularity_sql_date_trunc() {
        assert_eq!(TimeGranularity::Minute.sql_date_trunc(), "minute");
        assert_eq!(TimeGranularity::Hour.sql_date_trunc(), "hour");
        assert_eq!(TimeGranularity::Day.sql_date_trunc(), "day");
        assert_eq!(TimeGranularity::Week.sql_date_trunc(), "week");
        assert_eq!(TimeGranularity::Month.sql_date_trunc(), "month");
    }

    #[test]
    fn test_semantic_metric_builder() {
        let metric = SemanticMetric::new("auth_attempts")
            .with_caption("Authentication Attempts")
            .with_description("Count of authentication attempts")
            .with_aggregation(Aggregation::Count)
            .with_field_measure("metadata.uid")
            .with_dimensions(vec!["user_email".to_string(), "auth_result".to_string()])
            .with_time_granularities(vec![
                TimeGranularity::Minute,
                TimeGranularity::Hour,
                TimeGranularity::Day,
            ]);

        assert_eq!(metric.name, "auth_attempts");
        assert_eq!(metric.caption, "Authentication Attempts");
        assert_eq!(metric.aggregation, Aggregation::Count);
        assert_eq!(metric.dimensions.len(), 2);
        assert_eq!(metric.time_granularities.len(), 3);
        assert!(!metric.is_hot_path);
        assert!(metric.observable_type_id.is_none());
    }

    #[test]
    fn test_hot_path_metric() {
        let metric = SemanticMetric::new("threat_intel_matches")
            .with_caption("Threat Intel Matches")
            .with_aggregation(Aggregation::Count)
            .with_field_measure("observable_value")
            .with_observable_type_id(2);

        assert!(metric.is_hot_path);
        assert_eq!(metric.observable_type_id, Some(2));
    }

    #[test]
    fn test_metric_supports_granularity() {
        let metric = SemanticMetric::new("test")
            .add_time_granularity(TimeGranularity::Hour)
            .add_time_granularity(TimeGranularity::Day);

        assert!(!metric.supports_granularity(TimeGranularity::Minute));
        assert!(metric.supports_granularity(TimeGranularity::Hour));
        assert!(metric.supports_granularity(TimeGranularity::Day));
        assert!(!metric.supports_granularity(TimeGranularity::Week));
    }

    #[test]
    fn test_metric_supports_dimension() {
        let metric = SemanticMetric::new("test")
            .add_dimension("user_email")
            .add_dimension("source_ip");

        assert!(metric.supports_dimension("user_email"));
        assert!(metric.supports_dimension("source_ip"));
        assert!(!metric.supports_dimension("nonexistent"));
    }

    #[test]
    fn test_metric_serialization() {
        let metric = SemanticMetric::new("failed_auth_rate")
            .with_caption("Failed Authentication Rate")
            .with_aggregation(Aggregation::Avg)
            .with_expression_measure("CASE WHEN status_id != 1 THEN 1.0 ELSE 0.0 END")
            .add_dimension("user_email")
            .add_time_granularity(TimeGranularity::Hour)
            .add_time_granularity(TimeGranularity::Day);

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: SemanticMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    #[test]
    fn test_metric_with_observable_type_id_serialization() {
        let metric = SemanticMetric::new("ip_matches")
            .with_observable_type_id(2)
            .as_hot_path();

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: SemanticMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
        assert!(deserialized.is_hot_path);
        assert_eq!(deserialized.observable_type_id, Some(2));
    }
}
