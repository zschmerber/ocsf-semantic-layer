//! Query result caching for the OCSF Semantic Index.
//!
//! This module provides query result caching with TTL-based expiration
//! and LRU eviction. Cache keys are normalized so that semantically
//! equivalent queries produce the same cache key.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ocsf_semantic::query::SemanticQuery;

// ============================================================================
// QueryCacheKey
// ============================================================================

/// A cache key for query results.
///
/// The cache key is generated from a normalized representation of the query,
/// ensuring that semantically equivalent queries produce the same key.
/// Normalization includes:
/// - Sorting field lists (select, metrics, group_by, order_by)
/// - Sorting filter conditions
/// - Ignoring whitespace differences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct QueryCacheKey {
    /// Hash of the normalized query parameters.
    pub query_hash: String,
    /// Table names involved in the query (sorted for consistency).
    pub table_names: Vec<String>,
}

impl QueryCacheKey {
    /// Creates a new QueryCacheKey from a SemanticQuery.
    ///
    /// The query is normalized before hashing to ensure that semantically
    /// equivalent queries produce the same cache key. This means:
    /// - Field ordering differences do not affect the key
    /// - Whitespace differences do not affect the key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_semantic::query::SemanticQuery;
    /// use ocsf_index::query_cache::QueryCacheKey;
    ///
    /// let query = SemanticQuery::new("network_activity")
    ///     .add_select("src_ip")
    ///     .add_select("dst_ip");
    ///
    /// let key = QueryCacheKey::from_query(&query);
    /// ```
    pub fn from_query(query: &SemanticQuery) -> Self {
        let normalized = NormalizedQuery::from(query);
        let query_hash = normalized.compute_hash();

        // Extract table names from the entity (primary table)
        // In a more complete implementation, this would also include
        // tables from joins/relationships
        let mut table_names = vec![query.entity.clone()];
        table_names.sort();

        Self {
            query_hash,
            table_names,
        }
    }

    /// Creates a QueryCacheKey directly from components.
    ///
    /// This is useful for testing or when you have pre-computed values.
    pub fn new(query_hash: impl Into<String>, table_names: Vec<String>) -> Self {
        let mut table_names = table_names;
        table_names.sort();
        Self {
            query_hash: query_hash.into(),
            table_names,
        }
    }
}

// ============================================================================
// NormalizedQuery (Internal)
// ============================================================================

/// Internal representation of a normalized query for hashing.
///
/// All fields are sorted and normalized to ensure consistent hashing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NormalizedQuery {
    /// The primary entity name (trimmed).
    entity: String,
    /// Sorted list of selected attributes.
    select: Vec<String>,
    /// Sorted list of normalized filters.
    filters: Vec<NormalizedFilter>,
    /// Sorted list of metrics.
    metrics: Vec<String>,
    /// Sorted list of group by attributes.
    group_by: Vec<String>,
    /// Normalized time range (if present).
    time_range: Option<(String, String)>,
    /// Time granularity (if present).
    time_granularity: Option<String>,
    /// Path preference.
    path_preference: String,
    /// Limit value.
    limit: Option<u32>,
    /// Offset value.
    offset: Option<u32>,
    /// Sorted list of order by clauses.
    order_by: Vec<(String, String)>,
}

impl NormalizedQuery {
    /// Computes a hash string for this normalized query.
    fn compute_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

impl From<&SemanticQuery> for NormalizedQuery {
    fn from(query: &SemanticQuery) -> Self {
        // Normalize and sort select fields
        let mut select: Vec<String> = query.select.iter().map(|s| s.trim().to_string()).collect();
        select.sort();

        // Normalize and sort filters
        let mut filters: Vec<NormalizedFilter> =
            query.filters.iter().map(NormalizedFilter::from).collect();
        filters.sort();

        // Normalize and sort metrics
        let mut metrics: Vec<String> = query.metrics.iter().map(|s| s.trim().to_string()).collect();
        metrics.sort();

        // Normalize and sort group_by
        let mut group_by: Vec<String> = query
            .group_by
            .iter()
            .map(|s| s.trim().to_string())
            .collect();
        group_by.sort();

        // Normalize time range
        let time_range = query
            .time_range
            .as_ref()
            .map(|tr| (tr.start.trim().to_string(), tr.end.trim().to_string()));

        // Normalize time granularity
        let time_granularity = query.time_granularity.map(|g| format!("{:?}", g));

        // Normalize path preference
        let path_preference = format!("{:?}", query.path_preference);

        // Normalize and sort order_by
        let mut order_by: Vec<(String, String)> = query
            .order_by
            .iter()
            .map(|o| (o.field.trim().to_string(), format!("{:?}", o.direction)))
            .collect();
        order_by.sort();

        Self {
            entity: query.entity.trim().to_string(),
            select,
            filters,
            metrics,
            group_by,
            time_range,
            time_granularity,
            path_preference,
            limit: query.limit,
            offset: query.offset,
            order_by,
        }
    }
}

// ============================================================================
// NormalizedFilter (Internal)
// ============================================================================

/// Internal representation of a normalized filter for hashing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NormalizedFilter {
    /// The attribute name (trimmed).
    attribute: String,
    /// The operator as a string.
    operator: String,
    /// The value as a normalized string.
    value: String,
}

impl From<&ocsf_semantic::query::QueryFilter> for NormalizedFilter {
    fn from(filter: &ocsf_semantic::query::QueryFilter) -> Self {
        Self {
            attribute: filter.attribute.trim().to_string(),
            operator: format!("{:?}", filter.operator),
            value: normalize_filter_value(&filter.value),
        }
    }
}

/// Normalizes a filter value to a consistent string representation.
fn normalize_filter_value(value: &ocsf_semantic::query::FilterValue) -> String {
    use ocsf_semantic::query::FilterValue;

    match value {
        FilterValue::String(s) => format!("String:{}", s.trim()),
        FilterValue::Integer(i) => format!("Integer:{}", i),
        FilterValue::Float(f) => format!("Float:{}", f),
        FilterValue::Boolean(b) => format!("Boolean:{}", b),
        FilterValue::List(items) => {
            let mut sorted: Vec<String> = items.iter().map(|s| s.trim().to_string()).collect();
            sorted.sort();
            format!("List:[{}]", sorted.join(","))
        }
        FilterValue::Null => "Null".to_string(),
    }
}

// ============================================================================
// CachedResult
// ============================================================================

/// A cached query result with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    /// The cache key for this result.
    pub key: QueryCacheKey,
    /// The cached result data.
    pub result: serde_json::Value,
    /// When the cache entry was created.
    pub created_at: DateTime<Utc>,
    /// When the cache entry expires.
    pub expires_at: DateTime<Utc>,
    /// Number of times this cache entry has been accessed.
    pub hit_count: u64,
}

impl CachedResult {
    /// Creates a new cached result.
    pub fn new(key: QueryCacheKey, result: serde_json::Value, ttl_secs: u64) -> Self {
        let now = Utc::now();
        Self {
            key,
            result,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            hit_count: 0,
        }
    }

    /// Returns true if this cache entry has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Increments the hit count and returns the new value.
    pub fn record_hit(&mut self) -> u64 {
        self.hit_count += 1;
        self.hit_count
    }
}

// ============================================================================
// CacheStats
// ============================================================================

/// Statistics about the query cache.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total number of entries currently in the cache.
    pub total_entries: u64,
    /// Total number of cache hits.
    pub hit_count: u64,
    /// Total number of cache misses.
    pub miss_count: u64,
    /// Total number of entries evicted.
    pub eviction_count: u64,
    /// Total size of cached data in bytes (approximate).
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Creates new empty cache statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the cache hit rate.
    ///
    /// Returns 0.0 if there have been no cache accesses.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }

    /// Records a cache hit.
    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }

    /// Records a cache miss.
    pub fn record_miss(&mut self) {
        self.miss_count += 1;
    }

    /// Records an eviction.
    pub fn record_eviction(&mut self) {
        self.eviction_count += 1;
        if self.total_entries > 0 {
            self.total_entries -= 1;
        }
    }

    /// Records a new entry being added.
    pub fn record_insert(&mut self, size_bytes: u64) {
        self.total_entries += 1;
        self.total_size_bytes += size_bytes;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::query::{FilterValue, QueryFilter, TimeRange};

    #[test]
    fn test_query_cache_key_from_query() {
        let query = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .add_select("dst_ip");

        let key = QueryCacheKey::from_query(&query);

        assert!(!key.query_hash.is_empty());
        assert_eq!(key.table_names, vec!["network_activity"]);
    }

    #[test]
    fn test_query_cache_key_normalization_field_order() {
        // Two queries with same fields in different order should produce same key
        let query1 = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .add_select("dst_ip")
            .add_select("protocol");

        let query2 = SemanticQuery::new("network_activity")
            .add_select("protocol")
            .add_select("src_ip")
            .add_select("dst_ip");

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_eq!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_normalization_whitespace() {
        // Queries with whitespace differences should produce same key
        let query1 = SemanticQuery::new("network_activity").add_select("src_ip");

        let query2 = SemanticQuery::new("  network_activity  ").add_select("  src_ip  ");

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_eq!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_different_queries() {
        // Different queries should produce different keys
        let query1 = SemanticQuery::new("network_activity").add_select("src_ip");

        let query2 = SemanticQuery::new("network_activity").add_select("dst_ip");

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_ne!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_filter_normalization() {
        // Filters in different order should produce same key
        let query1 = SemanticQuery::new("network_activity")
            .add_filter(QueryFilter::eq("src_ip", "192.168.1.1"))
            .add_filter(QueryFilter::eq("dst_ip", "10.0.0.1"));

        let query2 = SemanticQuery::new("network_activity")
            .add_filter(QueryFilter::eq("dst_ip", "10.0.0.1"))
            .add_filter(QueryFilter::eq("src_ip", "192.168.1.1"));

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_eq!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_metrics_normalization() {
        // Metrics in different order should produce same key
        let query1 = SemanticQuery::new("network_activity")
            .with_metrics(vec!["total_bytes".to_string(), "event_count".to_string()]);

        let query2 = SemanticQuery::new("network_activity")
            .with_metrics(vec!["event_count".to_string(), "total_bytes".to_string()]);

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_eq!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_group_by_normalization() {
        // Group by in different order should produce same key
        let query1 = SemanticQuery::new("network_activity")
            .group_by(vec!["src_ip".to_string(), "dst_ip".to_string()]);

        let query2 = SemanticQuery::new("network_activity")
            .group_by(vec!["dst_ip".to_string(), "src_ip".to_string()]);

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        assert_eq!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_with_time_range() {
        let query = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .with_time_range(TimeRange::new("-7d", "now"));

        let key = QueryCacheKey::from_query(&query);

        assert!(!key.query_hash.is_empty());
    }

    #[test]
    fn test_query_cache_key_with_limit_offset() {
        let query1 = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .with_limit(100)
            .with_offset(50);

        let query2 = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .with_limit(100)
            .with_offset(100);

        let key1 = QueryCacheKey::from_query(&query1);
        let key2 = QueryCacheKey::from_query(&query2);

        // Different offsets should produce different keys
        assert_ne!(key1.query_hash, key2.query_hash);
    }

    #[test]
    fn test_query_cache_key_serialization() {
        let key = QueryCacheKey::new("abc123", vec!["table1".to_string(), "table2".to_string()]);

        let json = serde_json::to_string(&key).unwrap();
        let deserialized: QueryCacheKey = serde_json::from_str(&json).unwrap();

        assert_eq!(key, deserialized);
    }

    #[test]
    fn test_query_cache_key_hash_eq() {
        use std::collections::HashSet;

        let key1 = QueryCacheKey::new("abc123", vec!["table1".to_string()]);
        let key2 = QueryCacheKey::new("abc123", vec!["table1".to_string()]);
        let key3 = QueryCacheKey::new("def456", vec!["table1".to_string()]);

        let mut set = HashSet::new();
        set.insert(key1.clone());
        set.insert(key2.clone());
        set.insert(key3.clone());

        // key1 and key2 are equal, so set should have 2 elements
        assert_eq!(set.len(), 2);
        assert!(set.contains(&key1));
        assert!(set.contains(&key3));
    }

    #[test]
    fn test_cached_result_expiration() {
        let key = QueryCacheKey::new("test", vec!["table".to_string()]);
        let result = serde_json::json!({"data": "test"});

        // Create with 0 TTL (immediately expired)
        let cached = CachedResult::new(key.clone(), result.clone(), 0);
        assert!(cached.is_expired());

        // Create with long TTL (not expired)
        let cached = CachedResult::new(key, result, 3600);
        assert!(!cached.is_expired());
    }

    #[test]
    fn test_cached_result_hit_count() {
        let key = QueryCacheKey::new("test", vec!["table".to_string()]);
        let result = serde_json::json!({"data": "test"});
        let mut cached = CachedResult::new(key, result, 3600);

        assert_eq!(cached.hit_count, 0);
        assert_eq!(cached.record_hit(), 1);
        assert_eq!(cached.record_hit(), 2);
        assert_eq!(cached.hit_count, 2);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::new();

        // No accesses = 0% hit rate
        assert_eq!(stats.hit_rate(), 0.0);

        // 3 hits, 1 miss = 75% hit rate
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hit_rate(), 0.75);
    }

    #[test]
    fn test_cache_stats_tracking() {
        let mut stats = CacheStats::new();

        stats.record_insert(1000);
        stats.record_insert(500);
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_size_bytes, 1500);

        stats.record_eviction();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.eviction_count, 1);
    }

    #[test]
    fn test_filter_value_normalization() {
        // Test list normalization (sorting)
        let value = FilterValue::List(vec!["c".to_string(), "a".to_string(), "b".to_string()]);
        let normalized = normalize_filter_value(&value);
        assert_eq!(normalized, "List:[a,b,c]");
    }

    #[test]
    fn test_query_cache_key_idempotence() {
        // Property 10: Generating a QueryCacheKey twice should produce identical keys
        let query = SemanticQuery::new("network_activity")
            .add_select("src_ip")
            .add_select("dst_ip")
            .add_filter(QueryFilter::eq("protocol", "TCP"))
            .with_metrics(vec!["total_bytes".to_string()])
            .group_by(vec!["src_ip".to_string()])
            .with_limit(100);

        let key1 = QueryCacheKey::from_query(&query);
        let key2 = QueryCacheKey::from_query(&query);

        assert_eq!(key1, key2);
        assert_eq!(key1.query_hash, key2.query_hash);
        assert_eq!(key1.table_names, key2.table_names);
    }
}
