//! Lineage query result types for the OCSF Semantic Index.
//!
//! This module provides result types for paginated lineage queries, supporting
//! both source lineage (table-to-table) and field lineage (field-to-field) queries.
//! These types are designed for API responses and graph visualization in the UI.

use serde::{Deserialize, Serialize};

use crate::field_lineage::FieldMapping;
use crate::source_lineage::LineageEdge;

// ============================================================================
// LineageQueryResult
// ============================================================================

/// Result of a paginated source lineage query.
///
/// This struct contains the lineage edges for a query along with pagination
/// metadata. It is designed for API responses and supports efficient pagination
/// through large lineage graphs.
///
/// # Example
///
/// ```rust
/// use ocsf_index::lineage_query::LineageQueryResult;
/// use ocsf_index::source_lineage::LineageEdge;
/// use chrono::Utc;
///
/// let edges = vec![
///     LineageEdge {
///         source: "splunk:raw_logs".to_string(),
///         target: "network_activity".to_string(),
///         timestamp: Utc::now(),
///         record_count: Some(1000),
///     },
/// ];
///
/// let result = LineageQueryResult::new(edges, 100, true);
///
/// assert_eq!(result.edges.len(), 1);
/// assert_eq!(result.total_count, 100);
/// assert!(result.has_more);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageQueryResult {
    /// The lineage edges returned by the query.
    pub edges: Vec<LineageEdge>,

    /// Total count of matching records (before pagination).
    pub total_count: u64,

    /// Whether there are more records beyond the current page.
    pub has_more: bool,
}

impl LineageQueryResult {
    /// Creates a new LineageQueryResult with the given edges and pagination info.
    ///
    /// # Arguments
    ///
    /// * `edges` - The lineage edges for this page
    /// * `total_count` - Total count of matching records
    /// * `has_more` - Whether there are more records beyond this page
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::LineageQueryResult;
    ///
    /// let result = LineageQueryResult::new(vec![], 0, false);
    /// assert!(result.edges.is_empty());
    /// assert_eq!(result.total_count, 0);
    /// assert!(!result.has_more);
    /// ```
    pub fn new(edges: Vec<LineageEdge>, total_count: u64, has_more: bool) -> Self {
        Self {
            edges,
            total_count,
            has_more,
        }
    }

    /// Creates an empty LineageQueryResult with no edges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::LineageQueryResult;
    ///
    /// let result = LineageQueryResult::empty();
    /// assert!(result.edges.is_empty());
    /// assert_eq!(result.total_count, 0);
    /// assert!(!result.has_more);
    /// ```
    pub fn empty() -> Self {
        Self {
            edges: Vec::new(),
            total_count: 0,
            has_more: false,
        }
    }

    /// Creates a LineageQueryResult from a full list of edges with pagination.
    ///
    /// This helper method applies offset and limit to a full list of edges
    /// and calculates the pagination metadata.
    ///
    /// # Arguments
    ///
    /// * `all_edges` - The complete list of lineage edges
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::LineageQueryResult;
    /// use ocsf_index::source_lineage::LineageEdge;
    /// use chrono::Utc;
    ///
    /// let edges: Vec<LineageEdge> = (0..10).map(|i| LineageEdge {
    ///     source: format!("source:{}", i),
    ///     target: "target".to_string(),
    ///     timestamp: Utc::now(),
    ///     record_count: None,
    /// }).collect();
    ///
    /// let result = LineageQueryResult::from_edges_paginated(edges, 2, 3);
    /// assert_eq!(result.edges.len(), 3);
    /// assert_eq!(result.total_count, 10);
    /// assert!(result.has_more); // 2 + 3 = 5, still have 5 more
    /// ```
    pub fn from_edges_paginated(all_edges: Vec<LineageEdge>, offset: usize, limit: usize) -> Self {
        let total_count = all_edges.len() as u64;
        let edges: Vec<LineageEdge> = all_edges.into_iter().skip(offset).take(limit).collect();
        let has_more = (offset + edges.len()) < total_count as usize;

        Self {
            edges,
            total_count,
            has_more,
        }
    }

    /// Returns the number of edges in this result page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::LineageQueryResult;
    ///
    /// let result = LineageQueryResult::new(vec![], 100, true);
    /// assert_eq!(result.page_size(), 0);
    /// ```
    pub fn page_size(&self) -> usize {
        self.edges.len()
    }

    /// Returns true if this result page is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::LineageQueryResult;
    ///
    /// let result = LineageQueryResult::empty();
    /// assert!(result.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

impl Default for LineageQueryResult {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// FieldLineageQueryResult
// ============================================================================

/// Result of a paginated field lineage query.
///
/// This struct contains the field mappings for a query along with pagination
/// metadata. It is designed for API responses and supports efficient pagination
/// through large field lineage graphs.
///
/// # Example
///
/// ```rust
/// use ocsf_index::lineage_query::FieldLineageQueryResult;
/// use ocsf_index::field_lineage::FieldMapping;
///
/// let mappings = vec![
///     FieldMapping {
///         source_path: "src_ip".to_string(),
///         target_path: "src_endpoint.ip".to_string(),
///         transformation: None,
///     },
/// ];
///
/// let result = FieldLineageQueryResult::new(mappings, 50, false);
///
/// assert_eq!(result.mappings.len(), 1);
/// assert_eq!(result.total_count, 50);
/// assert!(!result.has_more);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldLineageQueryResult {
    /// The field mappings returned by the query.
    pub mappings: Vec<FieldMapping>,

    /// Total count of matching records (before pagination).
    pub total_count: u64,

    /// Whether there are more records beyond the current page.
    pub has_more: bool,
}

impl FieldLineageQueryResult {
    /// Creates a new FieldLineageQueryResult with the given mappings and pagination info.
    ///
    /// # Arguments
    ///
    /// * `mappings` - The field mappings for this page
    /// * `total_count` - Total count of matching records
    /// * `has_more` - Whether there are more records beyond this page
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::FieldLineageQueryResult;
    ///
    /// let result = FieldLineageQueryResult::new(vec![], 0, false);
    /// assert!(result.mappings.is_empty());
    /// assert_eq!(result.total_count, 0);
    /// assert!(!result.has_more);
    /// ```
    pub fn new(mappings: Vec<FieldMapping>, total_count: u64, has_more: bool) -> Self {
        Self {
            mappings,
            total_count,
            has_more,
        }
    }

    /// Creates an empty FieldLineageQueryResult with no mappings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::FieldLineageQueryResult;
    ///
    /// let result = FieldLineageQueryResult::empty();
    /// assert!(result.mappings.is_empty());
    /// assert_eq!(result.total_count, 0);
    /// assert!(!result.has_more);
    /// ```
    pub fn empty() -> Self {
        Self {
            mappings: Vec::new(),
            total_count: 0,
            has_more: false,
        }
    }

    /// Creates a FieldLineageQueryResult from a full list of mappings with pagination.
    ///
    /// This helper method applies offset and limit to a full list of mappings
    /// and calculates the pagination metadata.
    ///
    /// # Arguments
    ///
    /// * `all_mappings` - The complete list of field mappings
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::FieldLineageQueryResult;
    /// use ocsf_index::field_lineage::FieldMapping;
    ///
    /// let mappings: Vec<FieldMapping> = (0..10).map(|i| FieldMapping {
    ///     source_path: format!("field_{}", i),
    ///     target_path: format!("ocsf_field_{}", i),
    ///     transformation: None,
    /// }).collect();
    ///
    /// let result = FieldLineageQueryResult::from_mappings_paginated(mappings, 0, 5);
    /// assert_eq!(result.mappings.len(), 5);
    /// assert_eq!(result.total_count, 10);
    /// assert!(result.has_more);
    /// ```
    pub fn from_mappings_paginated(
        all_mappings: Vec<FieldMapping>,
        offset: usize,
        limit: usize,
    ) -> Self {
        let total_count = all_mappings.len() as u64;
        let mappings: Vec<FieldMapping> =
            all_mappings.into_iter().skip(offset).take(limit).collect();
        let has_more = (offset + mappings.len()) < total_count as usize;

        Self {
            mappings,
            total_count,
            has_more,
        }
    }

    /// Returns the number of mappings in this result page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::FieldLineageQueryResult;
    ///
    /// let result = FieldLineageQueryResult::new(vec![], 100, true);
    /// assert_eq!(result.page_size(), 0);
    /// ```
    pub fn page_size(&self) -> usize {
        self.mappings.len()
    }

    /// Returns true if this result page is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::lineage_query::FieldLineageQueryResult;
    ///
    /// let result = FieldLineageQueryResult::empty();
    /// assert!(result.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

impl Default for FieldLineageQueryResult {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ========================================================================
    // LineageQueryResult Tests
    // ========================================================================

    #[test]
    fn test_lineage_query_result_new() {
        let edges = vec![LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp: Utc::now(),
            record_count: Some(1000),
        }];

        let result = LineageQueryResult::new(edges.clone(), 100, true);

        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.total_count, 100);
        assert!(result.has_more);
    }

    #[test]
    fn test_lineage_query_result_empty() {
        let result = LineageQueryResult::empty();

        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_lineage_query_result_default() {
        let result = LineageQueryResult::default();

        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_lineage_query_result_from_edges_paginated() {
        let edges: Vec<LineageEdge> = (0..10)
            .map(|i| LineageEdge {
                source: format!("source:{}", i),
                target: "target".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            })
            .collect();

        // First page
        let result = LineageQueryResult::from_edges_paginated(edges.clone(), 0, 3);
        assert_eq!(result.edges.len(), 3);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);

        // Middle page
        let result = LineageQueryResult::from_edges_paginated(edges.clone(), 3, 3);
        assert_eq!(result.edges.len(), 3);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);

        // Last page
        let result = LineageQueryResult::from_edges_paginated(edges.clone(), 8, 3);
        assert_eq!(result.edges.len(), 2);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);

        // Beyond end
        let result = LineageQueryResult::from_edges_paginated(edges.clone(), 15, 3);
        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);
    }

    #[test]
    fn test_lineage_query_result_from_edges_paginated_empty() {
        let edges: Vec<LineageEdge> = vec![];
        let result = LineageQueryResult::from_edges_paginated(edges, 0, 10);

        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_lineage_query_result_page_size() {
        let edges = vec![
            LineageEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            },
            LineageEdge {
                source: "c".to_string(),
                target: "d".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            },
        ];

        let result = LineageQueryResult::new(edges, 100, true);
        assert_eq!(result.page_size(), 2);
    }

    #[test]
    fn test_lineage_query_result_is_empty() {
        let empty_result = LineageQueryResult::empty();
        assert!(empty_result.is_empty());

        let non_empty_result = LineageQueryResult::new(
            vec![LineageEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            }],
            1,
            false,
        );
        assert!(!non_empty_result.is_empty());
    }

    #[test]
    fn test_lineage_query_result_json_serialization() {
        let edges = vec![LineageEdge {
            source: "splunk:raw_logs".to_string(),
            target: "network_activity".to_string(),
            timestamp: Utc::now(),
            record_count: Some(1000),
        }];

        let result = LineageQueryResult::new(edges, 100, true);

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LineageQueryResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_lineage_query_result_equality() {
        let timestamp = Utc::now();
        let edges = vec![LineageEdge {
            source: "a".to_string(),
            target: "b".to_string(),
            timestamp,
            record_count: None,
        }];

        let result1 = LineageQueryResult::new(edges.clone(), 10, true);
        let result2 = LineageQueryResult::new(edges.clone(), 10, true);
        let result3 = LineageQueryResult::new(edges, 20, true);

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_lineage_query_result_clone() {
        let result = LineageQueryResult::new(
            vec![LineageEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            }],
            10,
            true,
        );

        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ========================================================================
    // FieldLineageQueryResult Tests
    // ========================================================================

    #[test]
    fn test_field_lineage_query_result_new() {
        let mappings = vec![FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: None,
        }];

        let result = FieldLineageQueryResult::new(mappings.clone(), 50, false);

        assert_eq!(result.mappings.len(), 1);
        assert_eq!(result.total_count, 50);
        assert!(!result.has_more);
    }

    #[test]
    fn test_field_lineage_query_result_empty() {
        let result = FieldLineageQueryResult::empty();

        assert!(result.mappings.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_field_lineage_query_result_default() {
        let result = FieldLineageQueryResult::default();

        assert!(result.mappings.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_field_lineage_query_result_from_mappings_paginated() {
        let mappings: Vec<FieldMapping> = (0..10)
            .map(|i| FieldMapping {
                source_path: format!("field_{}", i),
                target_path: format!("ocsf_field_{}", i),
                transformation: None,
            })
            .collect();

        // First page
        let result = FieldLineageQueryResult::from_mappings_paginated(mappings.clone(), 0, 5);
        assert_eq!(result.mappings.len(), 5);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);

        // Last page
        let result = FieldLineageQueryResult::from_mappings_paginated(mappings.clone(), 7, 5);
        assert_eq!(result.mappings.len(), 3);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);

        // Exact fit
        let result = FieldLineageQueryResult::from_mappings_paginated(mappings.clone(), 0, 10);
        assert_eq!(result.mappings.len(), 10);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);
    }

    #[test]
    fn test_field_lineage_query_result_from_mappings_paginated_empty() {
        let mappings: Vec<FieldMapping> = vec![];
        let result = FieldLineageQueryResult::from_mappings_paginated(mappings, 0, 10);

        assert!(result.mappings.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_field_lineage_query_result_page_size() {
        let mappings = vec![
            FieldMapping {
                source_path: "a".to_string(),
                target_path: "b".to_string(),
                transformation: None,
            },
            FieldMapping {
                source_path: "c".to_string(),
                target_path: "d".to_string(),
                transformation: Some("UPPER(c)".to_string()),
            },
        ];

        let result = FieldLineageQueryResult::new(mappings, 100, true);
        assert_eq!(result.page_size(), 2);
    }

    #[test]
    fn test_field_lineage_query_result_is_empty() {
        let empty_result = FieldLineageQueryResult::empty();
        assert!(empty_result.is_empty());

        let non_empty_result = FieldLineageQueryResult::new(
            vec![FieldMapping {
                source_path: "a".to_string(),
                target_path: "b".to_string(),
                transformation: None,
            }],
            1,
            false,
        );
        assert!(!non_empty_result.is_empty());
    }

    #[test]
    fn test_field_lineage_query_result_json_serialization() {
        let mappings = vec![FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
        }];

        let result = FieldLineageQueryResult::new(mappings, 50, true);

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: FieldLineageQueryResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_field_lineage_query_result_equality() {
        let mappings = vec![FieldMapping {
            source_path: "a".to_string(),
            target_path: "b".to_string(),
            transformation: None,
        }];

        let result1 = FieldLineageQueryResult::new(mappings.clone(), 10, true);
        let result2 = FieldLineageQueryResult::new(mappings.clone(), 10, true);
        let result3 = FieldLineageQueryResult::new(mappings, 20, true);

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_field_lineage_query_result_clone() {
        let result = FieldLineageQueryResult::new(
            vec![FieldMapping {
                source_path: "a".to_string(),
                target_path: "b".to_string(),
                transformation: None,
            }],
            10,
            true,
        );

        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_pagination_with_zero_limit() {
        let edges: Vec<LineageEdge> = (0..5)
            .map(|i| LineageEdge {
                source: format!("source:{}", i),
                target: "target".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            })
            .collect();

        let result = LineageQueryResult::from_edges_paginated(edges, 0, 0);
        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);
    }

    #[test]
    fn test_pagination_with_large_offset() {
        let mappings: Vec<FieldMapping> = (0..5)
            .map(|i| FieldMapping {
                source_path: format!("field_{}", i),
                target_path: format!("ocsf_field_{}", i),
                transformation: None,
            })
            .collect();

        let result = FieldLineageQueryResult::from_mappings_paginated(mappings, 100, 10);
        assert!(result.mappings.is_empty());
        assert_eq!(result.total_count, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn test_has_more_boundary_condition() {
        // Test exact boundary: offset + limit == total_count
        let edges: Vec<LineageEdge> = (0..10)
            .map(|i| LineageEdge {
                source: format!("source:{}", i),
                target: "target".to_string(),
                timestamp: Utc::now(),
                record_count: None,
            })
            .collect();

        // offset=5, limit=5, total=10 -> 5+5=10, has_more should be false
        let result = LineageQueryResult::from_edges_paginated(edges, 5, 5);
        assert_eq!(result.edges.len(), 5);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);
    }
}
