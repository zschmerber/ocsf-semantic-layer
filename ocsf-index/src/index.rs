//! SemanticIndex - the main entry point for the OCSF Semantic Index.
//!
//! The `SemanticIndex` struct is the primary interface for all index operations.
//! It coordinates the various components (table registry, lineage stores, partition
//! metadata, statistics, and query cache) and provides a unified API.
//!
//! # Example
//!
//! ```rust,ignore
//! use ocsf_index::{SemanticIndex, IndexConfig};
//! use ocsf_index::backend::InMemoryBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new index with in-memory backend
//!     let backend = InMemoryBackend::new();
//!     let config = IndexConfig::default();
//!     let index = SemanticIndex::new(backend, config).await?;
//!
//!     // Or open an existing index
//!     let backend = InMemoryBackend::new();
//!     let index = SemanticIndex::open(backend).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::backend::{IndexBackend, RecordFilter};
use crate::error::{IndexError, IndexResult};
use crate::field_lineage::FieldLineageRecord;
use crate::partition_metadata::PartitionEntry;
use crate::query_cache::{CacheStats, CachedResult, QueryCacheKey};
use crate::source_lineage::SourceLineageRecord;
use crate::table_registry::TableEntry;
use crate::types::{IndexConfig, LineageId, RecordId, TableId};

// ============================================================================
// Component Stores (marker structs for now)
// ============================================================================

/// Table registry component for managing physical table metadata.
///
/// This component tracks which physical tables exist for each OCSF event class,
/// including their schema, dialect, and detection coverage metadata.
#[derive(Debug, Default)]
pub struct TableRegistry {
    // Internal state will be added in subsequent tasks
}

impl TableRegistry {
    /// Creates a new empty TableRegistry.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Source lineage store component for tracking data provenance.
///
/// This component records which raw data sources contributed to each OCSF table,
/// enabling data provenance tracking and debugging of data quality issues.
#[derive(Debug, Default)]
pub struct SourceLineageStore {
    // Internal state will be added in subsequent tasks
}

impl SourceLineageStore {
    /// Creates a new empty SourceLineageStore.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Field lineage store component for tracking field-level transformations.
///
/// This component records how raw source fields map to OCSF fields,
/// including transformation expressions and version-specific mappings.
#[derive(Debug, Default)]
pub struct FieldLineageStore {
    // Internal state will be added in subsequent tasks
}

impl FieldLineageStore {
    /// Creates a new empty FieldLineageStore.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Partition metadata store component for query optimization.
///
/// This component tracks partition information including time bounds and row counts,
/// enabling partition pruning during query execution.
#[derive(Debug, Default)]
pub struct PartitionMetadataStore {
    // Internal state will be added in subsequent tasks
}

impl PartitionMetadataStore {
    /// Creates a new empty PartitionMetadataStore.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Table statistics store component for query planning.
///
/// This component tracks column statistics including cardinality and null rates,
/// enabling statistics-based query optimization.
#[derive(Debug, Default)]
pub struct TableStatisticsStore {
    // Internal state will be added in subsequent tasks
}

impl TableStatisticsStore {
    /// Creates a new empty TableStatisticsStore.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Query cache component for result caching.
///
/// This component caches query results with TTL-based expiration and LRU eviction,
/// improving query performance for repeated queries.
///
/// # LRU Eviction
///
/// When the cache exceeds `max_entries`, the least-recently-used entries are evicted.
/// Access time is tracked for each entry, and entries are evicted based on their
/// last access time (oldest first).
///
/// # TTL Expiration
///
/// Each cached entry has a time-to-live (TTL). Expired entries are not returned
/// on cache lookups and are cleaned up during eviction.
///
/// # Thread Safety
///
/// The cache uses interior mutability with `RwLock` to allow concurrent reads
/// and exclusive writes.
#[derive(Debug)]
pub struct QueryCache {
    /// Storage for cached results, keyed by QueryCacheKey.
    entries: std::sync::RwLock<std::collections::HashMap<QueryCacheKey, CacheEntry>>,
    /// Maximum number of entries allowed in the cache.
    max_entries: usize,
    /// Default TTL for cache entries in seconds.
    default_ttl_secs: u64,
    /// Cache statistics.
    stats: std::sync::RwLock<CacheStats>,
}

/// Internal cache entry with access tracking for LRU eviction.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached result.
    result: CachedResult,
    /// Last access time for LRU tracking.
    last_accessed: chrono::DateTime<chrono::Utc>,
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new_with_config(1000, 3600)
    }
}

impl QueryCache {
    /// Creates a new empty QueryCache with default settings.
    ///
    /// Default settings:
    /// - max_entries: 1000
    /// - default_ttl_secs: 3600 (1 hour)
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new QueryCache with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of entries allowed in the cache
    /// * `default_ttl_secs` - Default TTL for cache entries in seconds
    pub fn new_with_config(max_entries: usize, default_ttl_secs: u64) -> Self {
        Self {
            entries: std::sync::RwLock::new(std::collections::HashMap::new()),
            max_entries,
            default_ttl_secs,
            stats: std::sync::RwLock::new(CacheStats::new()),
        }
    }

    /// Creates a QueryCache from an IndexConfig.
    pub fn from_config(config: &IndexConfig) -> Self {
        Self::new_with_config(config.cache_max_entries, config.cache_default_ttl_secs)
    }

    /// Returns the maximum number of entries allowed in the cache.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Returns the default TTL for cache entries in seconds.
    pub fn default_ttl_secs(&self) -> u64 {
        self.default_ttl_secs
    }

    /// Inserts a cached result into the cache.
    ///
    /// If the cache is at capacity, the least-recently-used entry is evicted
    /// before inserting the new entry.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key for the result
    /// * `result` - The cached result to store
    ///
    /// # Returns
    ///
    /// The size in bytes of the cached result (approximate).
    pub fn insert(&self, key: QueryCacheKey, result: CachedResult) -> u64 {
        use chrono::Utc;

        let size_bytes = Self::estimate_size(&result);

        // Evict if necessary before inserting
        self.evict_if_needed();

        let entry = CacheEntry {
            result,
            last_accessed: Utc::now(),
        };

        {
            let mut entries = self.entries.write().unwrap();
            entries.insert(key, entry);
        }

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.record_insert(size_bytes);
        }

        size_bytes
    }

    /// Retrieves a cached result by key.
    ///
    /// If the entry exists and has not expired, it is returned and its
    /// access time is updated for LRU tracking. If the entry has expired,
    /// it is removed and `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to look up
    ///
    /// # Returns
    ///
    /// `Some(CachedResult)` if the entry exists and is not expired, `None` otherwise.
    pub fn get(&self, key: &QueryCacheKey) -> Option<CachedResult> {
        use chrono::Utc;

        let mut entries = self.entries.write().unwrap();

        if let Some(entry) = entries.get_mut(key) {
            // Check if expired
            if entry.result.is_expired() {
                // Remove expired entry
                entries.remove(key);
                let mut stats = self.stats.write().unwrap();
                stats.record_miss();
                return None;
            }

            // Update access time for LRU tracking
            entry.last_accessed = Utc::now();

            // Increment hit count on the result
            let mut result = entry.result.clone();
            result.record_hit();
            entry.result = result.clone();

            // Record hit in stats
            drop(entries);
            let mut stats = self.stats.write().unwrap();
            stats.record_hit();

            Some(result)
        } else {
            drop(entries);
            let mut stats = self.stats.write().unwrap();
            stats.record_miss();
            None
        }
    }

    /// Removes all cache entries associated with the given table name.
    ///
    /// This is used to invalidate cache entries when a table's data changes.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table whose cache entries should be invalidated
    ///
    /// # Returns
    ///
    /// The number of entries that were invalidated.
    pub fn invalidate_by_table(&self, table_name: &str) -> u64 {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let keys_to_remove: Vec<QueryCacheKey> = entries
            .keys()
            .filter(|key| key.table_names.contains(&table_name.to_string()))
            .cloned()
            .collect();

        let count = keys_to_remove.len() as u64;

        for key in keys_to_remove {
            entries.remove(&key);
            stats.record_eviction();
        }

        count
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let count = entries.len();
        entries.clear();

        // Record evictions for all cleared entries
        for _ in 0..count {
            stats.record_eviction();
        }
    }

    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a copy of the current cache statistics.
    pub fn stats(&self) -> CacheStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Evicts the least-recently-used entries if the cache is at or over capacity.
    ///
    /// This method also removes any expired entries during eviction.
    fn evict_if_needed(&self) {
        let mut entries = self.entries.write().unwrap();

        // First, remove any expired entries
        let expired_keys: Vec<QueryCacheKey> = entries
            .iter()
            .filter(|(_, entry)| entry.result.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        {
            let mut stats = self.stats.write().unwrap();
            for key in expired_keys {
                entries.remove(&key);
                stats.record_eviction();
            }
        }

        // If still at or over capacity, evict LRU entries
        while entries.len() >= self.max_entries {
            // Find the least recently used entry
            let lru_key = entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone());

            if let Some(key) = lru_key {
                entries.remove(&key);
                let mut stats = self.stats.write().unwrap();
                stats.record_eviction();
            } else {
                break;
            }
        }
    }

    /// Estimates the size of a cached result in bytes.
    fn estimate_size(result: &CachedResult) -> u64 {
        // Estimate based on JSON serialization
        serde_json::to_string(result)
            .map(|s| s.len() as u64)
            .unwrap_or(0)
    }
}

// ============================================================================
// SemanticIndex
// ============================================================================

/// The main entry point for the OCSF Semantic Index.
///
/// `SemanticIndex` coordinates all index operations and provides a unified API
/// for managing table metadata, lineage tracking, partition information,
/// statistics, and query caching.
///
/// # Type Parameters
///
/// * `B` - The storage backend type implementing `IndexBackend`
///
/// # Example
///
/// ```rust,ignore
/// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
/// use ocsf_index::backend::InMemoryBackend;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let backend = InMemoryBackend::new();
///     let config = IndexConfig::default();
///     let index = SemanticIndex::new(backend, config).await?;
///
///     // Register a table
///     let table = TableEntry::new("network_activity", 4001)
///         .with_schema("ocsf")
///         .with_ocsf_version("1.3.0");
///     let table_id = index.register_table(table).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
#[allow(dead_code)] // fields are part of planned architecture, not yet fully wired
pub struct SemanticIndex<B: IndexBackend> {
    /// The storage backend for persisting index data.
    backend: B,
    /// Table registry for managing physical table metadata.
    table_registry: TableRegistry,
    /// Source lineage store for tracking data provenance.
    source_lineage: SourceLineageStore,
    /// Field lineage store for tracking field-level transformations.
    field_lineage: FieldLineageStore,
    /// Partition metadata store for query optimization.
    partition_metadata: PartitionMetadataStore,
    /// Table statistics store for query planning.
    table_statistics: TableStatisticsStore,
    /// Query cache for result caching.
    query_cache: QueryCache,
    /// Index configuration.
    config: IndexConfig,
}

impl<B: IndexBackend> SemanticIndex<B> {
    /// Creates a new SemanticIndex with the given backend and configuration.
    ///
    /// This constructor initializes all component stores and prepares the index
    /// for use. The backend is used for persisting all index data.
    ///
    /// # Arguments
    ///
    /// * `backend` - The storage backend to use for persistence
    /// * `config` - The index configuration
    ///
    /// # Returns
    ///
    /// A new `SemanticIndex` instance wrapped in `IndexResult`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let config = IndexConfig::default()
    ///         .with_cache_max_entries(500)
    ///         .with_cache_default_ttl_secs(1800);
    ///     let index = SemanticIndex::new(backend, config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(backend: B, config: IndexConfig) -> IndexResult<Self> {
        Ok(Self {
            backend,
            table_registry: TableRegistry::new(),
            source_lineage: SourceLineageStore::new(),
            field_lineage: FieldLineageStore::new(),
            partition_metadata: PartitionMetadataStore::new(),
            table_statistics: TableStatisticsStore::new(),
            query_cache: QueryCache::from_config(&config),
            config,
        })
    }

    /// Opens an existing SemanticIndex with the given backend.
    ///
    /// This constructor opens an existing index using the default configuration.
    /// It loads any existing data from the backend and prepares the index for use.
    ///
    /// # Arguments
    ///
    /// * `backend` - The storage backend containing existing index data
    ///
    /// # Returns
    ///
    /// A `SemanticIndex` instance with default configuration wrapped in `IndexResult`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::SemanticIndex;
    /// use ocsf_index::backend::SqliteBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = SqliteBackend::open("index.db").await?;
    ///     let index = SemanticIndex::open(backend).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn open(backend: B) -> IndexResult<Self> {
        Self::new(backend, IndexConfig::default()).await
    }

    /// Returns a reference to the index configuration.
    pub fn config(&self) -> &IndexConfig {
        &self.config
    }

    /// Returns a reference to the storage backend.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    // ========================================================================
    // Table Registry Methods
    // ========================================================================

    /// Registers a new table in the index.
    ///
    /// This method validates the table entry and persists it to the backend.
    /// The class_uid must be a positive integer (> 0).
    ///
    /// # Arguments
    ///
    /// * `table` - The table entry to register
    ///
    /// # Returns
    ///
    /// The assigned `TableId` for the registered table.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::InvalidClassUid` if the class_uid is 0.
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let table = TableEntry::new("network_activity", 4001)
    ///         .with_schema("ocsf")
    ///         .with_ocsf_version("1.3.0");
    ///
    ///     let table_id = index.register_table(table).await?;
    ///     println!("Registered table with ID: {}", table_id);
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_table(&self, table: TableEntry) -> IndexResult<TableId> {
        // Validate class_uid is a positive integer (Requirement 1.2)
        if table.class_uid == 0 {
            return Err(IndexError::invalid_class_uid(table.class_uid));
        }

        // Persist the table entry to the backend
        let record_id = self.backend.create(&table).await?;

        Ok(TableId::new(record_id.0))
    }

    /// Deregisters a table by marking it as inactive (soft delete).
    ///
    /// This method performs a soft delete by setting `is_active` to false
    /// rather than physically deleting the table metadata. This preserves
    /// the table's history and allows for potential reactivation.
    ///
    /// # Arguments
    ///
    /// * `table_id` - The ID of the table to deregister
    ///
    /// # Errors
    ///
    /// Returns `IndexError::TableIdNotFound` if the table doesn't exist.
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let table = TableEntry::new("network_activity", 4001);
    ///     let table_id = index.register_table(table).await?;
    ///
    ///     // Soft delete the table
    ///     index.deregister_table(table_id).await?;
    ///
    ///     // Table still exists but is inactive
    ///     let table = index.get_table(table_id).await?;
    ///     assert!(!table.unwrap().is_active);
    ///     Ok(())
    /// }
    /// ```
    pub async fn deregister_table(&self, table_id: TableId) -> IndexResult<()> {
        let record_id = RecordId::new(table_id.0);

        // Read the existing table entry
        let table: Option<TableEntry> = self.backend.read(record_id).await?;

        match table {
            Some(mut entry) => {
                // Mark as inactive (soft delete) - Requirement 1.5
                entry.deactivate();

                // Update the record in the backend
                self.backend.update(record_id, &entry).await?;

                Ok(())
            }
            None => Err(IndexError::TableIdNotFound(table_id)),
        }
    }

    /// Retrieves all active tables registered for a specific OCSF class.
    ///
    /// This method returns all tables that are registered for the given
    /// class_uid and are currently active (not soft-deleted). Multiple
    /// tables can be registered for the same class to support partitioned
    /// or sharded data.
    ///
    /// # Arguments
    ///
    /// * `class_uid` - The OCSF class UID to query
    ///
    /// # Returns
    ///
    /// A vector of active `TableEntry` records for the specified class.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register multiple tables for the same class (partitioned data)
    ///     index.register_table(TableEntry::new("network_activity_2024", 4001)).await?;
    ///     index.register_table(TableEntry::new("network_activity_2023", 4001)).await?;
    ///
    ///     // Query all tables for class 4001
    ///     let tables = index.get_tables_by_class(4001).await?;
    ///     assert_eq!(tables.len(), 2);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tables_by_class(&self, class_uid: u32) -> IndexResult<Vec<TableEntry>> {
        // List all table entries from the backend
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        // Filter by class_uid and active status (Requirements 1.3, 1.4)
        let filtered_tables: Vec<TableEntry> = all_tables
            .into_iter()
            .filter(|table| table.class_uid == class_uid && table.is_active)
            .collect();

        Ok(filtered_tables)
    }

    /// Retrieves a single table by its ID.
    ///
    /// This method returns the table entry regardless of its active status,
    /// allowing retrieval of soft-deleted tables for auditing purposes.
    ///
    /// # Arguments
    ///
    /// * `table_id` - The ID of the table to retrieve
    ///
    /// # Returns
    ///
    /// `Some(TableEntry)` if the table exists, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let table = TableEntry::new("network_activity", 4001);
    ///     let table_id = index.register_table(table).await?;
    ///
    ///     // Retrieve the table
    ///     let retrieved = index.get_table(table_id).await?;
    ///     assert!(retrieved.is_some());
    ///     assert_eq!(retrieved.unwrap().table_name, "network_activity");
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_table(&self, table_id: TableId) -> IndexResult<Option<TableEntry>> {
        let record_id = RecordId::new(table_id.0);
        let table: Option<TableEntry> = self.backend.read(record_id).await?;
        Ok(table)
    }

    /// Lists all registered tables, optionally filtered by active status.
    ///
    /// This method returns all tables in the registry. Use the `active_only`
    /// parameter to filter out soft-deleted tables.
    ///
    /// # Arguments
    ///
    /// * `active_only` - If true, only return active tables; if false, return all tables
    ///
    /// # Returns
    ///
    /// A vector of `TableEntry` records.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register some tables
    ///     index.register_table(TableEntry::new("table1", 4001)).await?;
    ///     index.register_table(TableEntry::new("table2", 4002)).await?;
    ///
    ///     // List all active tables
    ///     let tables = index.list_tables(true).await?;
    ///     assert_eq!(tables.len(), 2);
    ///
    ///     // List all tables including inactive
    ///     let all_tables = index.list_tables(false).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_tables(&self, active_only: bool) -> IndexResult<Vec<TableEntry>> {
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        if active_only {
            Ok(all_tables.into_iter().filter(|t| t.is_active).collect())
        } else {
            Ok(all_tables)
        }
    }

    /// Updates an existing table entry.
    ///
    /// This method updates the table entry with the given ID. The table must
    /// exist in the registry. The `updated_at` timestamp is automatically
    /// set to the current time.
    ///
    /// # Arguments
    ///
    /// * `table_id` - The ID of the table to update
    /// * `table` - The updated table entry
    ///
    /// # Errors
    ///
    /// Returns `IndexError::TableIdNotFound` if the table doesn't exist.
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let table = TableEntry::new("network_activity", 4001);
    ///     let table_id = index.register_table(table).await?;
    ///
    ///     // Update the table
    ///     let mut updated = index.get_table(table_id).await?.unwrap();
    ///     updated.ocsf_version = "1.4.0".to_string();
    ///     index.update_table(table_id, updated).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_table(&self, table_id: TableId, mut table: TableEntry) -> IndexResult<()> {
        let record_id = RecordId::new(table_id.0);

        // Verify the table exists
        let existing: Option<TableEntry> = self.backend.read(record_id).await?;
        if existing.is_none() {
            return Err(IndexError::TableIdNotFound(table_id));
        }

        // Update the timestamp
        table.updated_at = chrono::Utc::now();

        // Persist the update
        self.backend.update(record_id, &table).await?;

        Ok(())
    }

    // ========================================================================
    // Source Lineage Methods
    // ========================================================================

    /// Records a source lineage entry for tracking data provenance.
    ///
    /// This method persists a source lineage record that tracks which raw data
    /// source contributed to an OCSF table. Multiple sources can contribute to
    /// a single target table (Requirement 2.3).
    ///
    /// # Arguments
    ///
    /// * `lineage` - The source lineage record to persist
    ///
    /// # Returns
    ///
    /// The assigned `LineageId` for the recorded lineage.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let lineage = SourceLineageRecord::new("splunk", "raw_network_logs", "network_activity")
    ///         .with_record_count(1_000_000);
    ///
    ///     let lineage_id = index.record_source_lineage(lineage).await?;
    ///     println!("Recorded lineage with ID: {}", lineage_id);
    ///     Ok(())
    /// }
    /// ```
    pub async fn record_source_lineage(
        &self,
        lineage: SourceLineageRecord,
    ) -> IndexResult<LineageId> {
        // Persist the lineage record to the backend (Requirements 2.2, 2.3)
        let record_id = self.backend.create(&lineage).await?;

        Ok(LineageId::new(record_id.0))
    }

    /// Retrieves all source lineage records for a target table.
    ///
    /// This method returns all source lineage records that contributed to the
    /// specified target table, ordered by ingestion timestamp ascending
    /// (Requirement 2.4).
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the target OCSF table
    ///
    /// # Returns
    ///
    /// A vector of `SourceLineageRecord` entries ordered by ingestion timestamp.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Record multiple sources for the same target
    ///     index.record_source_lineage(
    ///         SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
    ///     ).await?;
    ///     index.record_source_lineage(
    ///         SourceLineageRecord::new("kafka", "events", "network_activity")
    ///     ).await?;
    ///
    ///     // Query all sources for the target table
    ///     let lineage = index.get_source_lineage("network_activity").await?;
    ///     assert_eq!(lineage.len(), 2);
    ///
    ///     // Results are ordered by ingestion timestamp
    ///     for record in &lineage {
    ///         println!("Source: {}:{}", record.source_system, record.source_table);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_source_lineage(
        &self,
        table_name: &str,
    ) -> IndexResult<Vec<SourceLineageRecord>> {
        // List all source lineage records from the backend
        let filter = RecordFilter::new();
        let all_records: Vec<SourceLineageRecord> = self.backend.list(&filter).await?;

        // Filter by target table name
        let mut filtered_records: Vec<SourceLineageRecord> = all_records
            .into_iter()
            .filter(|record| record.target_table == table_name)
            .collect();

        // Sort by ingestion_timestamp ascending (Requirement 2.4)
        filtered_records.sort_by(|a, b| a.ingestion_timestamp.cmp(&b.ingestion_timestamp));

        Ok(filtered_records)
    }

    // ========================================================================
    // Field Lineage Methods
    // ========================================================================

    /// Records a field lineage entry for tracking field-level transformations.
    ///
    /// This method persists a field lineage record that tracks how a source field
    /// maps to an OCSF target field, optionally including the transformation
    /// expression used. This supports both one-to-many mappings (one source field
    /// to multiple OCSF fields) and many-to-one mappings (multiple source fields
    /// combining into one OCSF field).
    ///
    /// # Arguments
    ///
    /// * `lineage` - The field lineage record to persist
    ///
    /// # Returns
    ///
    /// The assigned `LineageId` for the recorded field lineage.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 3.3: Supports one-to-many mappings
    /// - Requirement 3.4: Supports many-to-one mappings
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Simple direct mapping
    ///     let lineage = FieldLineageRecord::new(
    ///         LineageId::new(1),
    ///         "src_ip",
    ///         "src_endpoint.ip"
    ///     );
    ///     let lineage_id = index.record_field_lineage(lineage).await?;
    ///
    ///     // Mapping with transformation
    ///     let lineage_with_transform = FieldLineageRecord::new(
    ///         LineageId::new(1),
    ///         "timestamp_str",
    ///         "time"
    ///     )
    ///     .with_transformation("CAST(timestamp_str AS TIMESTAMP)");
    ///     let lineage_id = index.record_field_lineage(lineage_with_transform).await?;
    ///
    ///     println!("Recorded field lineage with ID: {}", lineage_id);
    ///     Ok(())
    /// }
    /// ```
    pub async fn record_field_lineage(
        &self,
        lineage: FieldLineageRecord,
    ) -> IndexResult<LineageId> {
        // Persist the field lineage record to the backend (Requirements 3.3, 3.4)
        let record_id = self.backend.create(&lineage).await?;

        Ok(LineageId::new(record_id.0))
    }

    /// Retrieves all field lineage records for a target OCSF field.
    ///
    /// This method returns all field lineage records that map to the specified
    /// target field, including their source fields and transformations. This
    /// enables understanding how data flows from source fields to OCSF fields
    /// and supports debugging of field mapping issues.
    ///
    /// # Arguments
    ///
    /// * `target_field` - The target OCSF field path (e.g., "src_endpoint.ip")
    ///
    /// # Returns
    ///
    /// A vector of `FieldLineageRecord` entries that map to the target field.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 3.5: Returns all source fields and transformations for a target field
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Record multiple source fields mapping to the same target (many-to-one)
    ///     let lineage1 = FieldLineageRecord::new(
    ///         LineageId::new(1),
    ///         "first_name",
    ///         "actor.user.full_name"
    ///     )
    ///     .with_transformation("CONCAT(first_name, ' ', last_name)");
    ///
    ///     let lineage2 = FieldLineageRecord::new(
    ///         LineageId::new(1),
    ///         "last_name",
    ///         "actor.user.full_name"
    ///     )
    ///     .with_transformation("CONCAT(first_name, ' ', last_name)");
    ///
    ///     index.record_field_lineage(lineage1).await?;
    ///     index.record_field_lineage(lineage2).await?;
    ///
    ///     // Query all source fields for the target
    ///     let lineage = index.get_field_lineage("actor.user.full_name").await?;
    ///     assert_eq!(lineage.len(), 2);
    ///
    ///     for record in &lineage {
    ///         println!("Source: {} -> Target: {}", record.source_field, record.target_field);
    ///         if let Some(transform) = &record.transformation {
    ///             println!("  Transform: {}", transform);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_field_lineage(
        &self,
        target_field: &str,
    ) -> IndexResult<Vec<FieldLineageRecord>> {
        // List all field lineage records from the backend
        let filter = RecordFilter::new();
        let all_records: Vec<FieldLineageRecord> = self.backend.list(&filter).await?;

        // Filter by target field (Requirement 3.5)
        let filtered_records: Vec<FieldLineageRecord> = all_records
            .into_iter()
            .filter(|record| record.target_field == target_field)
            .collect();

        Ok(filtered_records)
    }

    // ========================================================================
    // Partition Metadata Methods
    // ========================================================================

    /// Updates or creates a partition entry in the index.
    ///
    /// This method creates a new partition entry if it doesn't exist, or updates
    /// an existing one. When updating, the row count, time bounds, and last modified
    /// timestamp are updated (Requirement 4.2).
    ///
    /// If the partition has no data (row_count = 0), it is marked as empty but
    /// the metadata record is retained (Requirement 4.4).
    ///
    /// # Arguments
    ///
    /// * `partition` - The partition entry to create or update
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 4.2: Updates row count, time bounds, and last modified timestamp
    /// - Requirement 4.4: Marks empty partitions but retains metadata
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let start = Utc::now();
    ///     let end = start + Duration::hours(1);
    ///
    ///     // Create a new partition
    ///     let partition = PartitionEntry::new("network_activity", "2024-01")
    ///         .with_time_range(start, end)
    ///         .with_row_count(1_000_000)
    ///         .with_size_bytes(500_000_000);
    ///
    ///     index.update_partition(partition).await?;
    ///
    ///     // Update an existing partition (e.g., after more data arrives)
    ///     let updated_partition = PartitionEntry::new("network_activity", "2024-01")
    ///         .with_time_range(start, end + Duration::hours(1))
    ///         .with_row_count(2_000_000)
    ///         .with_size_bytes(1_000_000_000);
    ///
    ///     index.update_partition(updated_partition).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_partition(&self, mut partition: PartitionEntry) -> IndexResult<()> {
        use chrono::Utc;

        // Check if partition already exists by looking for matching table_name and partition_key
        let filter = RecordFilter::new();
        let existing_partitions: Vec<PartitionEntry> = self.backend.list(&filter).await?;

        let existing = existing_partitions.into_iter().find(|p| {
            p.table_name == partition.table_name && p.partition_key == partition.partition_key
        });

        // Update last_modified timestamp (Requirement 4.2)
        partition.last_modified = Utc::now();

        // Ensure is_empty is correctly set based on row_count (Requirement 4.4)
        partition.is_empty = partition.row_count == 0;

        match existing {
            Some(existing_partition) => {
                // Update existing partition
                if let Some(id) = existing_partition.id {
                    partition.id = Some(id);
                    self.backend.update(RecordId::new(id.0), &partition).await?;
                }
            }
            None => {
                // Create new partition
                self.backend.create(&partition).await?;
            }
        }

        Ok(())
    }

    /// Retrieves all partitions for a given table.
    ///
    /// This method returns all partition entries for the specified table,
    /// including empty partitions (Requirement 4.4).
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to get partitions for
    ///
    /// # Returns
    ///
    /// A vector of `PartitionEntry` records for the specified table.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let start = Utc::now();
    ///
    ///     // Create partitions for different months
    ///     for month in 1..=3 {
    ///         let partition = PartitionEntry::new("network_activity", format!("2024-{:02}", month))
    ///             .with_time_range(start, start + Duration::days(30))
    ///             .with_row_count(1_000_000);
    ///         index.update_partition(partition).await?;
    ///     }
    ///
    ///     // Get all partitions for the table
    ///     let partitions = index.get_partitions("network_activity").await?;
    ///     assert_eq!(partitions.len(), 3);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_partitions(&self, table_name: &str) -> IndexResult<Vec<PartitionEntry>> {
        // List all partition entries from the backend
        let filter = RecordFilter::new();
        let all_partitions: Vec<PartitionEntry> = self.backend.list(&filter).await?;

        // Filter by table name
        let filtered_partitions: Vec<PartitionEntry> = all_partitions
            .into_iter()
            .filter(|partition| partition.table_name == table_name)
            .collect();

        Ok(filtered_partitions)
    }

    /// Retrieves partitions that overlap with the given time range.
    ///
    /// This method returns only partitions whose time range overlaps with the
    /// specified start and end times (Requirement 4.3). A partition overlaps
    /// if its start_time < end AND its end_time > start.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to get partitions for
    /// * `start` - The start of the time range to check
    /// * `end` - The end of the time range to check
    ///
    /// # Returns
    ///
    /// A vector of `PartitionEntry` records that overlap with the time range.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 4.3: Returns only partitions that overlap the specified range
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let base = Utc::now();
    ///
    ///     // Create partitions for different time ranges
    ///     let partition1 = PartitionEntry::new("network_activity", "2024-01")
    ///         .with_time_range(base, base + Duration::hours(2))
    ///         .with_row_count(1000);
    ///     let partition2 = PartitionEntry::new("network_activity", "2024-02")
    ///         .with_time_range(base + Duration::hours(2), base + Duration::hours(4))
    ///         .with_row_count(2000);
    ///     let partition3 = PartitionEntry::new("network_activity", "2024-03")
    ///         .with_time_range(base + Duration::hours(4), base + Duration::hours(6))
    ///         .with_row_count(3000);
    ///
    ///     index.update_partition(partition1).await?;
    ///     index.update_partition(partition2).await?;
    ///     index.update_partition(partition3).await?;
    ///
    ///     // Query for partitions in a specific time range
    ///     let query_start = base + Duration::hours(1);
    ///     let query_end = base + Duration::hours(3);
    ///
    ///     let partitions = index.get_partitions_in_range(
    ///         "network_activity",
    ///         query_start,
    ///         query_end
    ///     ).await?;
    ///
    ///     // Should return partition1 and partition2 (both overlap with query range)
    ///     assert_eq!(partitions.len(), 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_partitions_in_range(
        &self,
        table_name: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> IndexResult<Vec<PartitionEntry>> {
        // List all partition entries from the backend
        let filter = RecordFilter::new();
        let all_partitions: Vec<PartitionEntry> = self.backend.list(&filter).await?;

        // Filter by table name and time range overlap (Requirement 4.3)
        let filtered_partitions: Vec<PartitionEntry> = all_partitions
            .into_iter()
            .filter(|partition| {
                partition.table_name == table_name && partition.overlaps(start, end)
            })
            .collect();

        Ok(filtered_partitions)
    }

    // ========================================================================
    // Statistics Methods
    // ========================================================================

    /// Updates or creates table statistics in the index.
    ///
    /// This method creates new statistics if they don't exist for the table,
    /// or updates existing statistics. The `collected_at` timestamp indicates
    /// when the statistics were collected (Requirement 5.3).
    ///
    /// When statistics are collected for large tables, sampling with a configurable
    /// sample rate should be used (Requirement 5.2). The sample rate is stored
    /// in the `TableStatistics` struct.
    ///
    /// # Arguments
    ///
    /// * `stats` - The table statistics to create or update
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 5.2: Supports sampling with configurable sample rate
    /// - Requirement 5.3: Stores collected timestamp for freshness
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::table_statistics::{TableStatistics, ColumnStatistics};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Create statistics with sampling
    ///     let stats = TableStatistics::new("network_activity")
    ///         .with_total_rows(10_000_000)
    ///         .with_sample_rate(0.1) // 10% sample
    ///         .add_column(
    ///             ColumnStatistics::new("src_ip")
    ///                 .with_counts(50000, 100, 1_000_000)
    ///         );
    ///
    ///     index.update_statistics(stats).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_statistics(
        &self,
        stats: crate::table_statistics::TableStatistics,
    ) -> IndexResult<()> {
        // Check if statistics already exist for this table
        let filter = RecordFilter::new();
        let existing_stats: Vec<crate::table_statistics::TableStatistics> =
            self.backend.list(&filter).await?;

        let existing = existing_stats
            .into_iter()
            .find(|s| s.table_name == stats.table_name);

        match existing {
            Some(existing_stat) => {
                // Update existing statistics
                if let Some(id) = existing_stat.id {
                    let mut updated_stats = stats;
                    updated_stats.id = Some(id);
                    self.backend
                        .update(crate::types::RecordId::new(id.0), &updated_stats)
                        .await?;
                }
            }
            None => {
                // Create new statistics
                self.backend.create(&stats).await?;
            }
        }

        Ok(())
    }

    /// Retrieves table statistics with freshness information.
    ///
    /// This method returns the statistics for the specified table, including
    /// the `collected_at` timestamp to indicate freshness (Requirement 5.4).
    /// The caller can use `is_stale()` on the returned statistics to check
    /// if they need to be refreshed.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to get statistics for
    ///
    /// # Returns
    ///
    /// `Some(TableStatistics)` if statistics exist for the table, `None` otherwise.
    /// The returned statistics include the `collected_at` timestamp for freshness
    /// checking.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 5.4: Returns collected timestamp alongside statistics
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::table_statistics::{TableStatistics, ColumnStatistics};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // First, create some statistics
    ///     let stats = TableStatistics::new("network_activity")
    ///         .with_total_rows(1_000_000)
    ///         .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 1_000_000));
    ///     index.update_statistics(stats).await?;
    ///
    ///     // Retrieve statistics
    ///     if let Some(stats) = index.get_statistics("network_activity").await? {
    ///         println!("Total rows: {}", stats.total_rows);
    ///         println!("Collected at: {}", stats.collected_at);
    ///         println!("Sample rate: {}", stats.sample_rate);
    ///
    ///         // Check freshness (stale after 1 hour)
    ///         if stats.is_stale(3600) {
    ///             println!("Statistics are stale, consider refreshing");
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_statistics(
        &self,
        table_name: &str,
    ) -> IndexResult<Option<crate::table_statistics::TableStatistics>> {
        // List all statistics from the backend
        let filter = RecordFilter::new();
        let all_stats: Vec<crate::table_statistics::TableStatistics> =
            self.backend.list(&filter).await?;

        // Find statistics for the specified table
        let stats = all_stats.into_iter().find(|s| s.table_name == table_name);

        Ok(stats)
    }

    /// Collects statistics for a table using sampling.
    ///
    /// **Note:** This method is a stub for future implementation. Actual statistics
    /// collection requires access to the warehouse and is not yet implemented.
    /// Currently, this method returns an error indicating that the feature is
    /// not yet available.
    ///
    /// When implemented, this method will:
    /// - Connect to the warehouse to query the table
    /// - Use the specified sample rate for large tables (Requirement 5.2)
    /// - Compute column statistics (distinct count, null count, min/max)
    /// - Return statistics with the current timestamp (Requirement 5.3)
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to collect statistics for
    /// * `sample_rate` - The sampling rate (0.0 to 1.0) for large tables
    ///
    /// # Returns
    ///
    /// Currently returns an error. When implemented, will return `TableStatistics`
    /// with the collected data.
    ///
    /// # Errors
    ///
    /// Currently always returns `IndexError::StatisticsNotFound` as a placeholder.
    /// When implemented, may return:
    /// - `IndexError::TableNotFound` if the table doesn't exist
    /// - `IndexError::BackendError` if the warehouse query fails
    ///
    /// # Requirements
    ///
    /// - Requirement 5.2: Uses sampling with configurable sample rate
    /// - Requirement 5.3: Sets collected timestamp
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Collect statistics with 10% sampling
    ///     // Note: Currently returns an error as this is a stub
    ///     match index.collect_statistics("network_activity", 0.1).await {
    ///         Ok(stats) => {
    ///             println!("Collected {} rows", stats.total_rows);
    ///             // Store the collected statistics
    ///             index.update_statistics(stats).await?;
    ///         }
    ///         Err(e) => {
    ///             println!("Statistics collection not yet implemented: {}", e);
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn collect_statistics(
        &self,
        table_name: &str,
        _sample_rate: f64,
    ) -> IndexResult<crate::table_statistics::TableStatistics> {
        // Stub implementation - actual statistics collection requires warehouse access
        // and will be implemented in a future task when ocsf-warehouse integration is complete.
        //
        // The implementation would:
        // 1. Look up the table in the registry to get warehouse connection info
        // 2. Connect to the warehouse using the appropriate dialect
        // 3. Execute sampling queries to collect column statistics
        // 4. Return TableStatistics with collected_at set to now
        Err(IndexError::statistics_not_found(format!(
            "{} (collect_statistics not yet implemented - use update_statistics to manually set statistics)",
            table_name
        )))
    }

    // ========================================================================
    // Query Cache Methods
    // ========================================================================

    /// Caches a query result with the specified TTL.
    ///
    /// This method stores a query result in the cache with a time-to-live (TTL)
    /// in seconds. The cache key is used to identify the result for later retrieval.
    /// If the cache is at capacity, the least-recently-used entry is evicted
    /// before inserting the new entry (Requirement 6.5).
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key identifying this query result
    /// * `result` - The query result to cache (as JSON value)
    /// * `ttl_secs` - Time-to-live in seconds for this cache entry
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Requirements
    ///
    /// - Requirement 6.1: Stores query results with configurable TTL
    /// - Requirement 6.5: LRU eviction when cache exceeds max entries
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::query_cache::QueryCacheKey;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let key = QueryCacheKey::new("query_hash_123", vec!["network_activity".to_string()]);
    ///     let result = json!({"rows": [{"src_ip": "192.168.1.1"}]});
    ///
    ///     // Cache the result for 1 hour
    ///     index.cache_query_result(&key, &result, 3600).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn cache_query_result(
        &self,
        key: &QueryCacheKey,
        result: &serde_json::Value,
        ttl_secs: u64,
    ) -> IndexResult<()> {
        // Create a CachedResult with the provided TTL
        let cached_result = CachedResult::new(key.clone(), result.clone(), ttl_secs);

        // Insert into the cache (handles LRU eviction internally)
        self.query_cache.insert(key.clone(), cached_result);

        Ok(())
    }

    /// Retrieves a cached query result if it exists and has not expired.
    ///
    /// This method looks up a cached result by its key. If the entry exists
    /// and has not expired, it is returned. If the entry has expired, it is
    /// removed from the cache and `None` is returned (Requirement 6.3).
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to look up
    ///
    /// # Returns
    ///
    /// `Ok(Some(CachedResult))` if the entry exists and is not expired,
    /// `Ok(None)` if the entry doesn't exist or has expired.
    ///
    /// # Requirements
    ///
    /// - Requirement 6.2: Returns cached result if TTL has not expired
    /// - Requirement 6.3: Returns cache miss if TTL has expired
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::query_cache::QueryCacheKey;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     let key = QueryCacheKey::new("query_hash_123", vec!["network_activity".to_string()]);
    ///     let result = json!({"rows": [{"src_ip": "192.168.1.1"}]});
    ///
    ///     // Cache the result
    ///     index.cache_query_result(&key, &result, 3600).await?;
    ///
    ///     // Retrieve the cached result
    ///     if let Some(cached) = index.get_cached_result(&key).await? {
    ///         println!("Cache hit! Result: {}", cached.result);
    ///         println!("Hit count: {}", cached.hit_count);
    ///     } else {
    ///         println!("Cache miss");
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_cached_result(
        &self,
        key: &QueryCacheKey,
    ) -> IndexResult<Option<CachedResult>> {
        // Delegate to the QueryCache's get method which handles:
        // - Expiration checking (Requirement 6.3)
        // - Hit count tracking
        // - LRU access time updates
        Ok(self.query_cache.get(key))
    }

    /// Invalidates all cache entries associated with a specific table.
    ///
    /// This method removes all cached query results that involve the specified
    /// table. This is useful when a table's data changes and cached results
    /// may be stale.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table whose cache entries should be invalidated
    ///
    /// # Returns
    ///
    /// The number of cache entries that were invalidated.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::query_cache::QueryCacheKey;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Cache some results for network_activity table
    ///     let key1 = QueryCacheKey::new("hash1", vec!["network_activity".to_string()]);
    ///     let key2 = QueryCacheKey::new("hash2", vec!["network_activity".to_string()]);
    ///     let key3 = QueryCacheKey::new("hash3", vec!["process_activity".to_string()]);
    ///
    ///     index.cache_query_result(&key1, &json!({}), 3600).await?;
    ///     index.cache_query_result(&key2, &json!({}), 3600).await?;
    ///     index.cache_query_result(&key3, &json!({}), 3600).await?;
    ///
    ///     // Invalidate all cache entries for network_activity
    ///     let invalidated = index.invalidate_cache("network_activity").await?;
    ///     assert_eq!(invalidated, 2);
    ///
    ///     // process_activity cache entry should still exist
    ///     assert!(index.get_cached_result(&key3).await?.is_some());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn invalidate_cache(&self, table_name: &str) -> IndexResult<u64> {
        // Delegate to the QueryCache's invalidate_by_table method
        let invalidated_count = self.query_cache.invalidate_by_table(table_name);
        Ok(invalidated_count)
    }

    /// Returns current cache statistics.
    ///
    /// This method returns statistics about the query cache including:
    /// - Total number of entries
    /// - Hit count and miss count
    /// - Eviction count
    /// - Total size in bytes (approximate)
    /// - Hit rate (calculated from hit/miss counts)
    ///
    /// # Returns
    ///
    /// A `CacheStats` struct containing cache statistics.
    ///
    /// # Requirements
    ///
    /// - Requirement 11.5: Exposes cache status including hit rate and entry count
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::query_cache::QueryCacheKey;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Perform some cache operations
    ///     let key = QueryCacheKey::new("hash1", vec!["network_activity".to_string()]);
    ///     index.cache_query_result(&key, &json!({}), 3600).await?;
    ///     index.get_cached_result(&key).await?; // Hit
    ///     index.get_cached_result(&QueryCacheKey::new("nonexistent", vec![])).await?; // Miss
    ///
    ///     // Get cache statistics
    ///     let stats = index.get_cache_stats().await;
    ///     println!("Total entries: {}", stats.total_entries);
    ///     println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    ///     println!("Hits: {}, Misses: {}", stats.hit_count, stats.miss_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_cache_stats(&self) -> CacheStats {
        // Delegate to the QueryCache's stats method
        self.query_cache.stats()
    }

    // ========================================================================
    // Query Integration Methods
    // ========================================================================

    /// Returns query plan hints for optimizing query execution.
    ///
    /// This method provides hints to the query engine for optimizing query
    /// execution based on index metadata. The hints include:
    /// - Table locations and dialects for SQL generation
    /// - Partition information for partition pruning (based on time range)
    /// - Statistics for cost estimation
    /// - Cached results to avoid re-execution
    ///
    /// # Arguments
    ///
    /// * `query` - The semantic query to get hints for
    ///
    /// # Returns
    ///
    /// `QueryPlanHints` containing optimization information.
    ///
    /// # Requirements
    ///
    /// - Requirement 10.1: Returns relevant partition information for time filters
    /// - Requirement 10.2: Provides table statistics for query cost estimation
    /// - Requirement 10.3: Checks Query_Cache before returning partition information
    /// - Requirement 10.4: Provides table location and dialect information
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::partition_metadata::PartitionEntry;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use ocsf_semantic::query::{SemanticQuery, TimeRange};
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register a table for the entity
    ///     let table = TableEntry::new("network_activity", 4001)
    ///         .with_schema("ocsf")
    ///         .with_ocsf_version("1.3.0");
    ///     index.register_table(table).await?;
    ///
    ///     // Create partitions
    ///     let now = Utc::now();
    ///     let partition = PartitionEntry::new("network_activity", "2024-01")
    ///         .with_time_range(now - Duration::days(30), now)
    ///         .with_row_count(1_000_000);
    ///     index.update_partition(partition).await?;
    ///
    ///     // Create a query with time range
    ///     let query = SemanticQuery::new("network_activity")
    ///         .add_select("src_ip")
    ///         .with_time_range(TimeRange::new("-7d", "now"));
    ///
    ///     // Get query plan hints
    ///     let hints = index.get_query_plan_hints(&query).await?;
    ///
    ///     println!("Tables: {:?}", hints.tables);
    ///     println!("Partitions: {:?}", hints.partitions);
    ///     println!("Has cached result: {}", hints.has_valid_cache());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_query_plan_hints(
        &self,
        query: &ocsf_semantic::query::SemanticQuery,
    ) -> IndexResult<crate::query_hints::QueryPlanHints> {
        use crate::query_hints::{PartitionHint, QueryPlanHints, TableHint};

        let mut hints = QueryPlanHints::new();

        // Step 1: Check cache for existing result (Requirement 10.3)
        let cache_key = QueryCacheKey::from_query(query);
        if let Some(cached_result) = self.query_cache.get(&cache_key) {
            hints = hints.with_cached_result(cached_result);
        }

        // Step 2: Look up tables for the query's entity
        // The entity name in SemanticQuery corresponds to an OCSF class name.
        // We need to find tables that match this entity.
        // For now, we search all tables and match by table_name containing the entity
        // or by looking up tables registered for this entity.
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        // Find tables that match the query entity (by table name or entity mapping)
        // In a full implementation, there would be a mapping from entity names to class UIDs.
        // For now, we match tables whose name contains the entity name (case-insensitive).
        let entity_lower = query.entity.to_lowercase();
        let matching_tables: Vec<&TableEntry> = all_tables
            .iter()
            .filter(|t| t.is_active && t.table_name.to_lowercase().contains(&entity_lower))
            .collect();

        // Step 3: Create TableHint entries for each matching table (Requirement 10.4)
        for table in &matching_tables {
            let table_hint = TableHint::new(&table.table_name, table.dialect, &table.ocsf_version);
            let table_hint = if let Some(ref schema) = table.schema_name {
                table_hint.with_schema(schema)
            } else {
                table_hint
            };
            hints = hints.add_table(table_hint);
        }

        // Step 4: Get partitions with time range filtering (Requirement 10.1)
        // If the query has a time_range filter, get partitions that overlap with that range
        if let Some(ref time_range) = query.time_range {
            // Parse the time range strings to DateTime
            // Note: In a full implementation, we would handle relative times like "-7d" and "now"
            // For now, we try to parse as ISO 8601 or handle relative times
            if let (Some(start), Some(end)) = (
                parse_time_string(&time_range.start),
                parse_time_string(&time_range.end),
            ) {
                // Get partitions that overlap with the time range for each matching table
                for table in &matching_tables {
                    let partitions = self
                        .get_partitions_in_range(&table.table_name, start, end)
                        .await?;

                    for partition in partitions {
                        let partition_hint = PartitionHint::new(
                            &partition.table_name,
                            &partition.partition_key,
                            partition.row_count,
                        );
                        hints = hints.add_partition(partition_hint);
                    }
                }
            }
        } else {
            // No time range filter - include all partitions for matching tables
            for table in &matching_tables {
                let partitions = self.get_partitions(&table.table_name).await?;

                for partition in partitions {
                    let partition_hint = PartitionHint::new(
                        &partition.table_name,
                        &partition.partition_key,
                        partition.row_count,
                    );
                    hints = hints.add_partition(partition_hint);
                }
            }
        }

        // Step 5: Get statistics for each matching table (Requirement 10.2)
        for table in &matching_tables {
            if let Some(stats) = self.get_statistics(&table.table_name).await? {
                hints = hints.add_statistics(stats);
            }
        }

        Ok(hints)
    }

    // ========================================================================
    // Detection Coverage Analysis Methods
    // ========================================================================

    /// Retrieves all active tables that cover a specific MITRE ATT&CK technique.
    ///
    /// This method searches all registered tables and returns those that have
    /// detection coverage metadata indicating they can detect the specified
    /// MITRE technique.
    ///
    /// # Arguments
    ///
    /// * `technique` - The MITRE technique ID to search for (e.g., "T1071.004", "T1110.003")
    ///
    /// # Returns
    ///
    /// A vector of active `TableEntry` records that cover the specified technique.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 1.1: Uses detection_coverage metadata on TableEntry
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::table_registry::DetectionCoverage;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register tables with detection coverage
    ///     let coverage = DetectionCoverage::new()
    ///         .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()]);
    ///     let table = TableEntry::new("network_activity", 4001)
    ///         .with_detection_coverage(coverage);
    ///     index.register_table(table).await?;
    ///
    ///     // Query tables by MITRE technique
    ///     let tables = index.get_tables_by_mitre_technique("T1071.004").await?;
    ///     assert_eq!(tables.len(), 1);
    ///     assert_eq!(tables[0].table_name, "network_activity");
    ///
    ///     // Query for a technique not covered
    ///     let tables = index.get_tables_by_mitre_technique("T9999").await?;
    ///     assert!(tables.is_empty());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tables_by_mitre_technique(
        &self,
        technique: &str,
    ) -> IndexResult<Vec<TableEntry>> {
        // List all table entries from the backend
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        // Filter by active status and detection coverage containing the technique
        let filtered_tables: Vec<TableEntry> = all_tables
            .into_iter()
            .filter(|table| {
                table.is_active
                    && table
                        .detection_coverage
                        .as_ref()
                        .map(|dc| dc.covers_technique(technique))
                        .unwrap_or(false)
            })
            .collect();

        Ok(filtered_tables)
    }

    /// Retrieves all active tables that cover a specific MITRE ATT&CK tactic.
    ///
    /// This method searches all registered tables and returns those that have
    /// detection coverage metadata indicating they can detect the specified
    /// MITRE tactic.
    ///
    /// # Arguments
    ///
    /// * `tactic` - The MITRE tactic name to search for (e.g., "credential-access", "lateral-movement")
    ///
    /// # Returns
    ///
    /// A vector of active `TableEntry` records that cover the specified tactic.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 1.1: Uses detection_coverage metadata on TableEntry
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::table_registry::DetectionCoverage;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register tables with detection coverage
    ///     let coverage = DetectionCoverage::new()
    ///         .with_mitre_tactics(vec!["credential-access".to_string(), "lateral-movement".to_string()]);
    ///     let table = TableEntry::new("authentication_events", 3002)
    ///         .with_detection_coverage(coverage);
    ///     index.register_table(table).await?;
    ///
    ///     // Query tables by MITRE tactic
    ///     let tables = index.get_tables_by_mitre_tactic("credential-access").await?;
    ///     assert_eq!(tables.len(), 1);
    ///     assert_eq!(tables[0].table_name, "authentication_events");
    ///
    ///     // Query for a tactic not covered
    ///     let tables = index.get_tables_by_mitre_tactic("unknown-tactic").await?;
    ///     assert!(tables.is_empty());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tables_by_mitre_tactic(&self, tactic: &str) -> IndexResult<Vec<TableEntry>> {
        // List all table entries from the backend
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        // Filter by active status and detection coverage containing the tactic
        let filtered_tables: Vec<TableEntry> = all_tables
            .into_iter()
            .filter(|table| {
                table.is_active
                    && table
                        .detection_coverage
                        .as_ref()
                        .map(|dc| dc.covers_tactic(tactic))
                        .unwrap_or(false)
            })
            .collect();

        Ok(filtered_tables)
    }

    /// Retrieves a summary of detection coverage across all indexed tables.
    ///
    /// This method aggregates detection coverage metadata from all active tables
    /// in the index, providing a comprehensive view of:
    /// - Total number of tables with detection coverage
    /// - All unique MITRE ATT&CK techniques covered and which tables cover them
    /// - All unique MITRE ATT&CK tactics covered with technique counts
    /// - All unique data sources available across all tables
    /// - Coverage by kill chain phase
    ///
    /// This is useful for:
    /// - Gap analysis: Identifying which techniques/tactics are not covered
    /// - Detection engineering: Understanding current detection capabilities
    /// - Compliance reporting: Documenting security monitoring coverage
    ///
    /// # Returns
    ///
    /// A `DetectionCoverageSummary` containing aggregated coverage information.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 1.1: Aggregates detection_coverage metadata from TableEntry
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
    /// use ocsf_index::table_registry::DetectionCoverage;
    /// use ocsf_index::backend::InMemoryBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Register tables with detection coverage
    ///     let coverage1 = DetectionCoverage::new()
    ///         .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()])
    ///         .with_mitre_tactics(vec!["command-and-control".to_string()])
    ///         .with_data_sources(vec!["network_connection".to_string()])
    ///         .with_kill_chain_phases(vec!["delivery".to_string()]);
    ///     index.register_table(
    ///         TableEntry::new("network_activity", 4001).with_detection_coverage(coverage1)
    ///     ).await?;
    ///
    ///     let coverage2 = DetectionCoverage::new()
    ///         .with_mitre_techniques(vec!["T1071.004".to_string()]) // Shared technique
    ///         .with_mitre_tactics(vec!["credential-access".to_string()])
    ///         .with_data_sources(vec!["authentication_log".to_string()])
    ///         .with_kill_chain_phases(vec!["exploitation".to_string()]);
    ///     index.register_table(
    ///         TableEntry::new("auth_events", 3002).with_detection_coverage(coverage2)
    ///     ).await?;
    ///
    ///     // Get aggregated coverage summary
    ///     let summary = index.get_detection_coverage_summary().await?;
    ///
    ///     assert_eq!(summary.tables_with_coverage, 2);
    ///     assert_eq!(summary.mitre_techniques.len(), 2); // T1071.004 and T1110.003
    ///     assert_eq!(summary.mitre_tactics.len(), 2);    // command-and-control and credential-access
    ///     assert_eq!(summary.data_sources.len(), 2);     // network_connection and authentication_log
    ///     assert_eq!(summary.kill_chain_coverage.len(), 2); // delivery and exploitation
    ///
    ///     // Check that T1071.004 is covered by 2 tables
    ///     let t1071 = summary.mitre_techniques.iter()
    ///         .find(|t| t.technique_id == "T1071.004")
    ///         .unwrap();
    ///     assert_eq!(t1071.table_count, 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_detection_coverage_summary(
        &self,
    ) -> IndexResult<crate::detection_coverage::DetectionCoverageSummary> {
        use crate::detection_coverage::{
            DataSourceCoverage, DetectionCoverageSummary, KillChainCoverage, MitreTacticCoverage,
            MitreTechniqueCoverage,
        };
        use std::collections::{HashMap, HashSet};

        // List all table entries from the backend
        let filter = RecordFilter::new();
        let all_tables: Vec<TableEntry> = self.backend.list(&filter).await?;

        // Filter to active tables with detection coverage
        let tables_with_coverage: Vec<&TableEntry> = all_tables
            .iter()
            .filter(|table| table.is_active && table.detection_coverage.is_some())
            .collect();

        let tables_with_coverage_count = tables_with_coverage.len() as u64;

        // Aggregate MITRE techniques: technique_id -> (table_count, table_names)
        let mut technique_map: HashMap<String, Vec<String>> = HashMap::new();

        // Aggregate MITRE tactics: tactic -> (techniques_set, table_names)
        let mut tactic_map: HashMap<String, (HashSet<String>, HashSet<String>)> = HashMap::new();

        // Aggregate data sources: data_source -> table_names
        let mut data_source_map: HashMap<String, Vec<String>> = HashMap::new();

        // Aggregate kill chain phases: phase -> table_count
        let mut kill_chain_map: HashMap<String, u64> = HashMap::new();

        for table in &tables_with_coverage {
            let table_name = &table.table_name;
            if let Some(coverage) = &table.detection_coverage {
                // Aggregate techniques
                for technique in &coverage.mitre_techniques {
                    technique_map
                        .entry(technique.clone())
                        .or_default()
                        .push(table_name.clone());
                }

                // Aggregate tactics with their associated techniques
                for tactic in &coverage.mitre_tactics {
                    let entry = tactic_map.entry(tactic.clone()).or_default();
                    // Add all techniques from this table to the tactic's technique set
                    for technique in &coverage.mitre_techniques {
                        entry.0.insert(technique.clone());
                    }
                    // Add this table to the tactic's table set
                    entry.1.insert(table_name.clone());
                }

                // Aggregate data sources
                for data_source in &coverage.data_sources {
                    data_source_map
                        .entry(data_source.clone())
                        .or_default()
                        .push(table_name.clone());
                }

                // Aggregate kill chain phases
                for phase in &coverage.kill_chain_phases {
                    *kill_chain_map.entry(phase.clone()).or_insert(0) += 1;
                }
            }
        }

        // Build the summary structs
        let mut mitre_techniques: Vec<MitreTechniqueCoverage> = technique_map
            .into_iter()
            .map(|(technique_id, tables)| MitreTechniqueCoverage {
                technique_id,
                table_count: tables.len() as u64,
                tables,
            })
            .collect();
        // Sort by technique_id for consistent ordering
        mitre_techniques.sort_by(|a, b| a.technique_id.cmp(&b.technique_id));

        let mut mitre_tactics: Vec<MitreTacticCoverage> = tactic_map
            .into_iter()
            .map(|(tactic, (techniques, tables))| MitreTacticCoverage {
                tactic,
                technique_count: techniques.len() as u64,
                table_count: tables.len() as u64,
            })
            .collect();
        // Sort by tactic for consistent ordering
        mitre_tactics.sort_by(|a, b| a.tactic.cmp(&b.tactic));

        let mut data_sources: Vec<DataSourceCoverage> = data_source_map
            .into_iter()
            .map(|(data_source, tables)| DataSourceCoverage {
                data_source,
                table_count: tables.len() as u64,
                tables,
            })
            .collect();
        // Sort by data_source for consistent ordering
        data_sources.sort_by(|a, b| a.data_source.cmp(&b.data_source));

        let mut kill_chain_coverage: Vec<KillChainCoverage> = kill_chain_map
            .into_iter()
            .map(|(phase, table_count)| KillChainCoverage { phase, table_count })
            .collect();
        // Sort by phase for consistent ordering
        kill_chain_coverage.sort_by(|a, b| a.phase.cmp(&b.phase));

        Ok(DetectionCoverageSummary {
            tables_with_coverage: tables_with_coverage_count,
            mitre_techniques,
            mitre_tactics,
            data_sources,
            kill_chain_coverage,
        })
    }

    // ========================================================================
    // Filtered Lineage Query Methods
    // ========================================================================

    /// Retrieves source lineage records with time range filtering and pagination.
    ///
    /// This method returns source lineage records for the specified target table,
    /// filtered by the given time range and paginated with offset/limit.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the target OCSF table
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (inclusive)
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    ///
    /// A `LineageQueryResult` containing the filtered and paginated lineage edges,
    /// total count, and whether there are more records.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 12.3: Supports filtering lineage queries by time range
    /// - Requirement 12.4: Supports pagination with offset and limit
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Record some lineage
    ///     let now = Utc::now();
    ///     for i in 0..10 {
    ///         let lineage = SourceLineageRecord::new("splunk", format!("logs_{}", i), "network_activity")
    ///             .with_ingestion_timestamp(now - Duration::hours(i));
    ///         index.record_source_lineage(lineage).await?;
    ///     }
    ///
    ///     // Query with time range and pagination
    ///     let start = now - Duration::hours(5);
    ///     let end = now;
    ///     let result = index.get_source_lineage_filtered("network_activity", start, end, 0, 3).await?;
    ///
    ///     println!("Found {} edges (total: {})", result.edges.len(), result.total_count);
    ///     println!("Has more: {}", result.has_more);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_source_lineage_filtered(
        &self,
        table_name: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        offset: usize,
        limit: usize,
    ) -> IndexResult<crate::lineage_query::LineageQueryResult> {
        use crate::lineage_query::LineageQueryResult;
        use crate::source_lineage::LineageEdge;

        // List all source lineage records from the backend
        let filter = RecordFilter::new();
        let all_records: Vec<SourceLineageRecord> = self.backend.list(&filter).await?;

        // Filter by target table name and time range (Requirement 12.3)
        let mut filtered_records: Vec<SourceLineageRecord> = all_records
            .into_iter()
            .filter(|record| {
                record.target_table == table_name
                    && record.ingestion_timestamp >= start
                    && record.ingestion_timestamp <= end
            })
            .collect();

        // Sort by ingestion_timestamp ascending for consistent ordering
        filtered_records.sort_by(|a, b| a.ingestion_timestamp.cmp(&b.ingestion_timestamp));

        // Convert to LineageEdge for visualization
        let all_edges: Vec<LineageEdge> = filtered_records.iter().map(LineageEdge::from).collect();

        // Apply pagination (Requirement 12.4)
        Ok(LineageQueryResult::from_edges_paginated(
            all_edges, offset, limit,
        ))
    }

    /// Retrieves field lineage records with time range filtering and pagination.
    ///
    /// This method returns field lineage records for the specified target field,
    /// filtered by the time range of their associated source lineage records
    /// and paginated with offset/limit.
    ///
    /// Note: Field lineage records don't have their own timestamp, so filtering
    /// is based on the source lineage record's ingestion timestamp. This requires
    /// looking up the associated source lineage record.
    ///
    /// # Arguments
    ///
    /// * `target_field` - The target OCSF field path (e.g., "src_endpoint.ip")
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (inclusive)
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    ///
    /// A `FieldLineageQueryResult` containing the filtered and paginated field mappings,
    /// total count, and whether there are more records.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::BackendError` if the backend operation fails.
    ///
    /// # Requirements
    ///
    /// - Requirement 12.3: Supports filtering lineage queries by time range
    /// - Requirement 12.4: Supports pagination with offset and limit
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ocsf_index::{SemanticIndex, IndexConfig};
    /// use ocsf_index::source_lineage::SourceLineageRecord;
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    /// use ocsf_index::backend::InMemoryBackend;
    /// use chrono::{Utc, Duration};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let backend = InMemoryBackend::new();
    ///     let index = SemanticIndex::new(backend, IndexConfig::default()).await?;
    ///
    ///     // Record source lineage first
    ///     let now = Utc::now();
    ///     let source_lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
    ///         .with_ingestion_timestamp(now - Duration::hours(1));
    ///     let lineage_id = index.record_source_lineage(source_lineage).await?;
    ///
    ///     // Record field lineage
    ///     let field_lineage = FieldLineageRecord::new(lineage_id, "src_ip", "src_endpoint.ip");
    ///     index.record_field_lineage(field_lineage).await?;
    ///
    ///     // Query with time range and pagination
    ///     let start = now - Duration::hours(2);
    ///     let end = now;
    ///     let result = index.get_field_lineage_filtered("src_endpoint.ip", start, end, 0, 10).await?;
    ///
    ///     println!("Found {} mappings (total: {})", result.mappings.len(), result.total_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_field_lineage_filtered(
        &self,
        target_field: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        offset: usize,
        limit: usize,
    ) -> IndexResult<crate::lineage_query::FieldLineageQueryResult> {
        use crate::field_lineage::FieldMapping;
        use crate::lineage_query::FieldLineageQueryResult;
        use std::collections::HashMap;

        // List all source lineage records to build a timestamp lookup
        let source_filter = RecordFilter::new();
        let source_records: Vec<SourceLineageRecord> = self.backend.list(&source_filter).await?;

        // Build a map from source_lineage_id to ingestion_timestamp
        let timestamp_map: HashMap<u64, chrono::DateTime<chrono::Utc>> = source_records
            .iter()
            .filter_map(|record| record.id.map(|id| (id.0, record.ingestion_timestamp)))
            .collect();

        // List all field lineage records from the backend
        let field_filter = RecordFilter::new();
        let all_records: Vec<FieldLineageRecord> = self.backend.list(&field_filter).await?;

        // Filter by target field and time range (based on source lineage timestamp)
        let filtered_records: Vec<&FieldLineageRecord> = all_records
            .iter()
            .filter(|record| {
                if record.target_field != target_field {
                    return false;
                }
                // Look up the timestamp from the source lineage record
                if let Some(&timestamp) = timestamp_map.get(&record.source_lineage_id.0) {
                    timestamp >= start && timestamp <= end
                } else {
                    // If we can't find the source lineage, include it (conservative approach)
                    true
                }
            })
            .collect();

        // Convert to FieldMapping for visualization
        let all_mappings: Vec<FieldMapping> = filtered_records
            .iter()
            .map(|r| FieldMapping::from(*r))
            .collect();

        // Apply pagination (Requirement 12.4)
        Ok(FieldLineageQueryResult::from_mappings_paginated(
            all_mappings,
            offset,
            limit,
        ))
    }
}

/// Parses a time string to a DateTime<Utc>.
///
/// Supports:
/// - ISO 8601 format (e.g., "2024-01-15T10:30:00Z")
/// - Relative times: "now", "-Nd" (N days ago), "-Nh" (N hours ago)
///
/// Returns None if the string cannot be parsed.
fn parse_time_string(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{Duration, Utc};

    let s = s.trim();

    // Handle "now"
    if s.eq_ignore_ascii_case("now") {
        return Some(Utc::now());
    }

    // Handle relative times like "-7d", "-24h", "-30m"
    if s.starts_with('-') && s.len() >= 2 {
        let (num_str, unit) = s[1..].split_at(s.len() - 2);
        if let Ok(num) = num_str.parse::<i64>() {
            let duration = match unit.to_lowercase().as_str() {
                "d" => Some(Duration::days(num)),
                "h" => Some(Duration::hours(num)),
                "m" => Some(Duration::minutes(num)),
                "s" => Some(Duration::seconds(num)),
                _ => None,
            };
            if let Some(dur) = duration {
                return Some(Utc::now() - dur);
            }
        }
    }

    // Try parsing as ISO 8601
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::InMemoryBackend;
    use crate::field_lineage::FieldLineageRecord;
    use crate::source_lineage::SourceLineageRecord;
    use crate::table_registry::DetectionCoverage;
    use crate::types::{ConfidenceLevel, LineageId, WarehouseDialect};
    use chrono::{Datelike, Duration, Utc};

    #[tokio::test]
    async fn test_semantic_index_new() {
        let backend = InMemoryBackend::new();
        let config = IndexConfig::default();
        let index = SemanticIndex::new(backend, config).await;

        assert!(index.is_ok());
        let index = index.unwrap();
        assert_eq!(index.config().cache_max_entries, 1000);
        assert_eq!(index.config().cache_default_ttl_secs, 3600);
    }

    #[tokio::test]
    async fn test_semantic_index_new_with_custom_config() {
        let backend = InMemoryBackend::new();
        let config = IndexConfig::default()
            .with_cache_max_entries(500)
            .with_cache_default_ttl_secs(1800)
            .with_statistics_sample_rate(0.5)
            .with_statistics_max_age_secs(43200);

        let index = SemanticIndex::new(backend, config).await;

        assert!(index.is_ok());
        let index = index.unwrap();
        assert_eq!(index.config().cache_max_entries, 500);
        assert_eq!(index.config().cache_default_ttl_secs, 1800);
        assert_eq!(index.config().statistics_sample_rate, 0.5);
        assert_eq!(index.config().statistics_max_age_secs, 43200);
    }

    #[tokio::test]
    async fn test_semantic_index_open() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::open(backend).await;

        assert!(index.is_ok());
        let index = index.unwrap();
        // Should use default config
        assert_eq!(index.config().cache_max_entries, 1000);
        assert_eq!(index.config().cache_default_ttl_secs, 3600);
        assert_eq!(index.config().statistics_sample_rate, 0.1);
        assert_eq!(index.config().statistics_max_age_secs, 86400);
    }

    #[tokio::test]
    async fn test_semantic_index_backend_access() {
        let backend = InMemoryBackend::new();
        let config = IndexConfig::default();
        let index = SemanticIndex::new(backend, config).await.unwrap();

        // Should be able to access the backend
        let _backend_ref = index.backend();
    }

    #[tokio::test]
    async fn test_semantic_index_config_access() {
        let backend = InMemoryBackend::new();
        let config = IndexConfig::default().with_cache_max_entries(999);
        let index = SemanticIndex::new(backend, config).await.unwrap();

        assert_eq!(index.config().cache_max_entries, 999);
    }

    // ========================================================================
    // Table Registry Tests
    // ========================================================================

    #[tokio::test]
    async fn test_register_table_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.3.0");

        let result = index.register_table(table).await;
        assert!(result.is_ok());

        let table_id = result.unwrap();
        assert_eq!(table_id.0, 1); // First ID should be 1
    }

    #[tokio::test]
    async fn test_register_table_invalid_class_uid_zero() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // class_uid = 0 should fail validation (Requirement 1.2)
        let table = TableEntry::new("invalid_table", 0);

        let result = index.register_table(table).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IndexError::InvalidClassUid(uid) => assert_eq!(uid, 0),
            other => panic!("Expected InvalidClassUid error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_register_table_valid_class_uid_one() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // class_uid = 1 should be valid (minimum positive integer)
        let table = TableEntry::new("minimal_table", 1);

        let result = index.register_table(table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_table_valid_class_uid_max() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // class_uid = u32::MAX should be valid
        let table = TableEntry::new("max_uid_table", u32::MAX);

        let result = index.register_table(table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_multiple_tables_same_class() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register multiple tables for the same class (Requirement 1.4)
        let table1 = TableEntry::new("network_activity_2024", 4001);
        let table2 = TableEntry::new("network_activity_2023", 4001);
        let table3 = TableEntry::new("network_activity_2022", 4001);

        let id1 = index.register_table(table1).await.unwrap();
        let id2 = index.register_table(table2).await.unwrap();
        let id3 = index.register_table(table3).await.unwrap();

        // All should have unique IDs
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[tokio::test]
    async fn test_register_table_with_detection_coverage() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_confidence(ConfidenceLevel::High);

        let table = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage);

        let table_id = index.register_table(table).await.unwrap();

        // Verify the table was stored with coverage
        let retrieved = index.get_table(table_id).await.unwrap().unwrap();
        assert!(retrieved.detection_coverage.is_some());
        assert!(retrieved
            .detection_coverage
            .unwrap()
            .covers_technique("T1071.004"));
    }

    #[tokio::test]
    async fn test_get_table_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.4.0")
            .with_dialect(WarehouseDialect::BigQuery);

        let table_id = index.register_table(table).await.unwrap();

        let retrieved = index.get_table(table_id).await.unwrap();
        assert!(retrieved.is_some());

        let entry = retrieved.unwrap();
        assert_eq!(entry.table_name, "network_activity");
        assert_eq!(entry.class_uid, 4001);
        assert_eq!(entry.schema_name, Some("ocsf".to_string()));
        assert_eq!(entry.ocsf_version, "1.4.0");
        assert_eq!(entry.dialect, WarehouseDialect::BigQuery);
        assert!(entry.is_active);
    }

    #[tokio::test]
    async fn test_get_table_not_found() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Try to get a non-existent table
        let result = index.get_table(TableId::new(999)).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_deregister_table_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let table = TableEntry::new("network_activity", 4001);
        let table_id = index.register_table(table).await.unwrap();

        // Verify table is active
        let before = index.get_table(table_id).await.unwrap().unwrap();
        assert!(before.is_active);

        // Deregister (soft delete)
        let result = index.deregister_table(table_id).await;
        assert!(result.is_ok());

        // Verify table is now inactive (Requirement 1.5)
        let after = index.get_table(table_id).await.unwrap().unwrap();
        assert!(!after.is_active);

        // Verify other fields are preserved
        assert_eq!(after.table_name, "network_activity");
        assert_eq!(after.class_uid, 4001);
    }

    #[tokio::test]
    async fn test_deregister_table_not_found() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Try to deregister a non-existent table
        let result = index.deregister_table(TableId::new(999)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IndexError::TableIdNotFound(id) => assert_eq!(id.0, 999),
            other => panic!("Expected TableIdNotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_deregister_table_preserves_metadata() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);

        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.4.0")
            .with_dialect(WarehouseDialect::Databricks)
            .with_metadata("owner", "security-team")
            .with_detection_coverage(coverage);

        let table_id = index.register_table(table).await.unwrap();

        // Deregister
        index.deregister_table(table_id).await.unwrap();

        // Verify all metadata is preserved (Requirement 1.5)
        let after = index.get_table(table_id).await.unwrap().unwrap();
        assert!(!after.is_active);
        assert_eq!(after.table_name, "network_activity");
        assert_eq!(after.class_uid, 4001);
        assert_eq!(after.schema_name, Some("ocsf".to_string()));
        assert_eq!(after.ocsf_version, "1.4.0");
        assert_eq!(after.dialect, WarehouseDialect::Databricks);
        assert_eq!(
            after.metadata.get("owner"),
            Some(&"security-team".to_string())
        );
        assert!(after.detection_coverage.is_some());
    }

    #[tokio::test]
    async fn test_get_tables_by_class_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables for different classes
        index
            .register_table(TableEntry::new("network_activity_1", 4001))
            .await
            .unwrap();
        index
            .register_table(TableEntry::new("network_activity_2", 4001))
            .await
            .unwrap();
        index
            .register_table(TableEntry::new("process_activity", 1001))
            .await
            .unwrap();

        // Query by class 4001 (Requirement 1.3)
        let tables = index.get_tables_by_class(4001).await.unwrap();
        assert_eq!(tables.len(), 2);

        // Verify all returned tables have the correct class_uid
        for table in &tables {
            assert_eq!(table.class_uid, 4001);
            assert!(table.is_active);
        }

        // Query by class 1001
        let tables = index.get_tables_by_class(1001).await.unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].table_name, "process_activity");
    }

    #[tokio::test]
    async fn test_get_tables_by_class_empty() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query for a class with no tables
        let tables = index.get_tables_by_class(9999).await.unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_class_excludes_inactive() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables
        let id1 = index
            .register_table(TableEntry::new("network_activity_1", 4001))
            .await
            .unwrap();
        let _id2 = index
            .register_table(TableEntry::new("network_activity_2", 4001))
            .await
            .unwrap();
        let id3 = index
            .register_table(TableEntry::new("network_activity_3", 4001))
            .await
            .unwrap();

        // Deregister some tables
        index.deregister_table(id1).await.unwrap();
        index.deregister_table(id3).await.unwrap();

        // Query should only return active tables
        let tables = index.get_tables_by_class(4001).await.unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].table_name, "network_activity_2");
    }

    #[tokio::test]
    async fn test_get_tables_by_class_multiple_tables_per_class() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register many tables for the same class (Requirement 1.4 - partitioned/sharded data)
        for i in 0..10 {
            let table = TableEntry::new(format!("network_activity_{}", i), 4001);
            index.register_table(table).await.unwrap();
        }

        let tables = index.get_tables_by_class(4001).await.unwrap();
        assert_eq!(tables.len(), 10);
    }

    #[tokio::test]
    async fn test_table_registry_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Register a table
        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.3.0");
        let table_id = index.register_table(table).await.unwrap();

        // 2. Retrieve it
        let retrieved = index.get_table(table_id).await.unwrap().unwrap();
        assert_eq!(retrieved.table_name, "network_activity");
        assert!(retrieved.is_active);

        // 3. Query by class
        let tables = index.get_tables_by_class(4001).await.unwrap();
        assert_eq!(tables.len(), 1);

        // 4. Deregister (soft delete)
        index.deregister_table(table_id).await.unwrap();

        // 5. Table still retrievable but inactive
        let after_deregister = index.get_table(table_id).await.unwrap().unwrap();
        assert!(!after_deregister.is_active);

        // 6. Query by class no longer returns it
        let tables_after = index.get_tables_by_class(4001).await.unwrap();
        assert!(tables_after.is_empty());
    }

    // ========================================================================
    // Source Lineage Tests
    // ========================================================================

    #[tokio::test]
    async fn test_record_source_lineage_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let lineage = SourceLineageRecord::new("splunk", "raw_network_logs", "network_activity")
            .with_record_count(1_000_000);

        let result = index.record_source_lineage(lineage).await;
        assert!(result.is_ok());

        let lineage_id = result.unwrap();
        assert_eq!(lineage_id.0, 1); // First ID should be 1
    }

    #[tokio::test]
    async fn test_record_source_lineage_multiple_sources() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record multiple sources for the same target (Requirement 2.3)
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        let lineage2 = SourceLineageRecord::new("kafka", "events", "network_activity");
        let lineage3 = SourceLineageRecord::new("elastic", "security", "network_activity");

        let id1 = index.record_source_lineage(lineage1).await.unwrap();
        let id2 = index.record_source_lineage(lineage2).await.unwrap();
        let id3 = index.record_source_lineage(lineage3).await.unwrap();

        // All should have unique IDs
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[tokio::test]
    async fn test_record_source_lineage_with_metadata() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(500_000)
            .with_metadata("pipeline", "etl-v2")
            .with_metadata("batch_id", "batch-001");

        let lineage_id = index.record_source_lineage(lineage).await.unwrap();
        assert!(lineage_id.0 > 0);
    }

    #[tokio::test]
    async fn test_get_source_lineage_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record lineage
        let lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1000);
        index.record_source_lineage(lineage).await.unwrap();

        // Query lineage
        let records = index.get_source_lineage("network_activity").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source_system, "splunk");
        assert_eq!(records[0].source_table, "raw_logs");
        assert_eq!(records[0].target_table, "network_activity");
        assert_eq!(records[0].record_count, Some(1000));
    }

    #[tokio::test]
    async fn test_get_source_lineage_empty() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query for a table with no lineage
        let records = index.get_source_lineage("nonexistent_table").await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_get_source_lineage_multiple_sources() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record multiple sources for the same target (Requirement 2.3)
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        let lineage2 = SourceLineageRecord::new("kafka", "events", "network_activity");
        let lineage3 = SourceLineageRecord::new("elastic", "security", "network_activity");

        index.record_source_lineage(lineage1).await.unwrap();
        index.record_source_lineage(lineage2).await.unwrap();
        index.record_source_lineage(lineage3).await.unwrap();

        // Query should return all 3 sources
        let records = index.get_source_lineage("network_activity").await.unwrap();
        assert_eq!(records.len(), 3);

        // Verify all sources are present
        let systems: Vec<&str> = records.iter().map(|r| r.source_system.as_str()).collect();
        assert!(systems.contains(&"splunk"));
        assert!(systems.contains(&"kafka"));
        assert!(systems.contains(&"elastic"));
    }

    #[tokio::test]
    async fn test_get_source_lineage_filters_by_target() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record lineage for different targets
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity");
        let lineage2 = SourceLineageRecord::new("kafka", "events", "process_activity");
        let lineage3 = SourceLineageRecord::new("elastic", "security", "network_activity");

        index.record_source_lineage(lineage1).await.unwrap();
        index.record_source_lineage(lineage2).await.unwrap();
        index.record_source_lineage(lineage3).await.unwrap();

        // Query for network_activity should return 2 records
        let network_records = index.get_source_lineage("network_activity").await.unwrap();
        assert_eq!(network_records.len(), 2);

        // Query for process_activity should return 1 record
        let process_records = index.get_source_lineage("process_activity").await.unwrap();
        assert_eq!(process_records.len(), 1);
        assert_eq!(process_records[0].source_system, "kafka");
    }

    #[tokio::test]
    async fn test_get_source_lineage_ordered_by_timestamp() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Create records with specific timestamps (Requirement 2.4)
        let now = Utc::now();
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_ingestion_timestamp(now + Duration::hours(2)); // Latest
        let lineage2 = SourceLineageRecord::new("kafka", "events", "network_activity")
            .with_ingestion_timestamp(now); // Earliest
        let lineage3 = SourceLineageRecord::new("elastic", "security", "network_activity")
            .with_ingestion_timestamp(now + Duration::hours(1)); // Middle

        // Insert in non-chronological order
        index.record_source_lineage(lineage1).await.unwrap();
        index.record_source_lineage(lineage2).await.unwrap();
        index.record_source_lineage(lineage3).await.unwrap();

        // Query should return records ordered by ingestion_timestamp ascending
        let records = index.get_source_lineage("network_activity").await.unwrap();
        assert_eq!(records.len(), 3);

        // Verify order: kafka (earliest) -> elastic (middle) -> splunk (latest)
        assert_eq!(records[0].source_system, "kafka");
        assert_eq!(records[1].source_system, "elastic");
        assert_eq!(records[2].source_system, "splunk");

        // Verify timestamps are in ascending order
        assert!(records[0].ingestion_timestamp <= records[1].ingestion_timestamp);
        assert!(records[1].ingestion_timestamp <= records[2].ingestion_timestamp);
    }

    #[tokio::test]
    async fn test_source_lineage_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Record lineage from multiple sources
        let lineage1 = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_record_count(1_000_000)
            .with_metadata("pipeline", "etl-v1");
        let lineage2 = SourceLineageRecord::new("kafka", "events", "network_activity")
            .with_record_count(500_000)
            .with_metadata("pipeline", "streaming");

        let id1 = index.record_source_lineage(lineage1).await.unwrap();
        let id2 = index.record_source_lineage(lineage2).await.unwrap();

        assert_ne!(id1, id2);

        // 2. Query lineage for the target table
        let records = index.get_source_lineage("network_activity").await.unwrap();
        assert_eq!(records.len(), 2);

        // 3. Verify record details
        let total_records: u64 = records.iter().filter_map(|r| r.record_count).sum();
        assert_eq!(total_records, 1_500_000);

        // 4. Query for non-existent table returns empty
        let empty = index.get_source_lineage("nonexistent").await.unwrap();
        assert!(empty.is_empty());
    }

    // ========================================================================
    // Field Lineage Tests
    // ========================================================================

    #[tokio::test]
    async fn test_record_field_lineage_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");

        let result = index.record_field_lineage(lineage).await;
        assert!(result.is_ok());

        let lineage_id = result.unwrap();
        assert_eq!(lineage_id.0, 1); // First ID should be 1
    }

    #[tokio::test]
    async fn test_record_field_lineage_with_transformation() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let lineage = FieldLineageRecord::new(LineageId::new(1), "timestamp_str", "time")
            .with_transformation("CAST(timestamp_str AS TIMESTAMP)")
            .with_ocsf_version("1.3.0");

        let lineage_id = index.record_field_lineage(lineage).await.unwrap();
        assert!(lineage_id.0 > 0);
    }

    #[tokio::test]
    async fn test_record_field_lineage_one_to_many() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // One source field maps to multiple OCSF fields (Requirement 3.3)
        let lineage1 = FieldLineageRecord::new(LineageId::new(1), "ip_address", "src_endpoint.ip");
        let lineage2 = FieldLineageRecord::new(LineageId::new(1), "ip_address", "dst_endpoint.ip");

        let id1 = index.record_field_lineage(lineage1).await.unwrap();
        let id2 = index.record_field_lineage(lineage2).await.unwrap();

        // Both should have unique IDs
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_record_field_lineage_many_to_one() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Multiple source fields combine into one OCSF field (Requirement 3.4)
        let lineage1 =
            FieldLineageRecord::new(LineageId::new(1), "first_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");
        let lineage2 =
            FieldLineageRecord::new(LineageId::new(1), "last_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");

        let id1 = index.record_field_lineage(lineage1).await.unwrap();
        let id2 = index.record_field_lineage(lineage2).await.unwrap();

        // Both should have unique IDs
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_get_field_lineage_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record field lineage
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");
        index.record_field_lineage(lineage).await.unwrap();

        // Query field lineage (Requirement 3.5)
        let records = index.get_field_lineage("src_endpoint.ip").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source_field, "src_ip");
        assert_eq!(records[0].target_field, "src_endpoint.ip");
        assert_eq!(records[0].transformation, Some("LOWER(src_ip)".to_string()));
    }

    #[tokio::test]
    async fn test_get_field_lineage_empty() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query for a field with no lineage
        let records = index.get_field_lineage("nonexistent.field").await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_get_field_lineage_many_to_one_returns_all_sources() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record many-to-one mapping (Requirement 3.4, 3.5)
        let lineage1 =
            FieldLineageRecord::new(LineageId::new(1), "first_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");
        let lineage2 =
            FieldLineageRecord::new(LineageId::new(1), "last_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");

        index.record_field_lineage(lineage1).await.unwrap();
        index.record_field_lineage(lineage2).await.unwrap();

        // Query should return all source fields (Requirement 3.5)
        let records = index
            .get_field_lineage("actor.user.full_name")
            .await
            .unwrap();
        assert_eq!(records.len(), 2);

        // Verify all source fields are present
        let source_fields: Vec<&str> = records.iter().map(|r| r.source_field.as_str()).collect();
        assert!(source_fields.contains(&"first_name"));
        assert!(source_fields.contains(&"last_name"));

        // Verify transformations are present
        for record in &records {
            assert_eq!(
                record.transformation,
                Some("CONCAT(first_name, ' ', last_name)".to_string())
            );
        }
    }

    #[tokio::test]
    async fn test_get_field_lineage_filters_by_target() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record lineage for different targets
        let lineage1 = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");
        let lineage2 = FieldLineageRecord::new(LineageId::new(1), "dst_ip", "dst_endpoint.ip");
        let lineage3 = FieldLineageRecord::new(LineageId::new(1), "src_port", "src_endpoint.port");

        index.record_field_lineage(lineage1).await.unwrap();
        index.record_field_lineage(lineage2).await.unwrap();
        index.record_field_lineage(lineage3).await.unwrap();

        // Query for src_endpoint.ip should return 1 record
        let src_ip_records = index.get_field_lineage("src_endpoint.ip").await.unwrap();
        assert_eq!(src_ip_records.len(), 1);
        assert_eq!(src_ip_records[0].source_field, "src_ip");

        // Query for dst_endpoint.ip should return 1 record
        let dst_ip_records = index.get_field_lineage("dst_endpoint.ip").await.unwrap();
        assert_eq!(dst_ip_records.len(), 1);
        assert_eq!(dst_ip_records[0].source_field, "dst_ip");

        // Query for src_endpoint.port should return 1 record
        let port_records = index.get_field_lineage("src_endpoint.port").await.unwrap();
        assert_eq!(port_records.len(), 1);
        assert_eq!(port_records[0].source_field, "src_port");
    }

    #[tokio::test]
    async fn test_get_field_lineage_with_ocsf_version() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Record lineage with OCSF version
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_ocsf_version("1.3.0");
        index.record_field_lineage(lineage).await.unwrap();

        // Query and verify version is preserved
        let records = index.get_field_lineage("src_endpoint.ip").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].ocsf_version, Some("1.3.0".to_string()));
    }

    #[tokio::test]
    async fn test_field_lineage_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Record field lineage for a table transformation
        let lineage1 = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");
        let lineage2 = FieldLineageRecord::new(LineageId::new(1), "dst_ip", "dst_endpoint.ip")
            .with_transformation("LOWER(dst_ip)");
        let lineage3 =
            FieldLineageRecord::new(LineageId::new(1), "first_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");
        let lineage4 =
            FieldLineageRecord::new(LineageId::new(1), "last_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");

        let id1 = index.record_field_lineage(lineage1).await.unwrap();
        let id2 = index.record_field_lineage(lineage2).await.unwrap();
        let id3 = index.record_field_lineage(lineage3).await.unwrap();
        let id4 = index.record_field_lineage(lineage4).await.unwrap();

        // All IDs should be unique
        let ids = vec![id1, id2, id3, id4];
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j]);
            }
        }

        // 2. Query lineage for different target fields
        let src_ip_lineage = index.get_field_lineage("src_endpoint.ip").await.unwrap();
        assert_eq!(src_ip_lineage.len(), 1);

        let dst_ip_lineage = index.get_field_lineage("dst_endpoint.ip").await.unwrap();
        assert_eq!(dst_ip_lineage.len(), 1);

        // 3. Query many-to-one mapping
        let full_name_lineage = index
            .get_field_lineage("actor.user.full_name")
            .await
            .unwrap();
        assert_eq!(full_name_lineage.len(), 2);

        // 4. Query for non-existent field returns empty
        let empty = index.get_field_lineage("nonexistent.field").await.unwrap();
        assert!(empty.is_empty());
    }

    // ========================================================================
    // Partition Metadata Tests
    // ========================================================================

    #[tokio::test]
    async fn test_update_partition_create_new() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::hours(1);

        let partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(1_000_000)
                .with_size_bytes(500_000_000);

        let result = index.update_partition(partition).await;
        assert!(result.is_ok());

        // Verify partition was created
        let partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].table_name, "network_activity");
        assert_eq!(partitions[0].partition_key, "2024-01");
        assert_eq!(partitions[0].row_count, 1_000_000);
        assert_eq!(partitions[0].size_bytes, 500_000_000);
        assert!(!partitions[0].is_empty);
    }

    #[tokio::test]
    async fn test_update_partition_update_existing() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::hours(1);

        // Create initial partition
        let partition1 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(1_000_000)
                .with_size_bytes(500_000_000);

        index.update_partition(partition1).await.unwrap();

        // Update the same partition (Requirement 4.2)
        let new_end = end + Duration::hours(1);
        let partition2 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, new_end)
                .with_row_count(2_000_000)
                .with_size_bytes(1_000_000_000);

        index.update_partition(partition2).await.unwrap();

        // Verify only one partition exists with updated values
        let partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].row_count, 2_000_000);
        assert_eq!(partitions[0].size_bytes, 1_000_000_000);
        assert_eq!(partitions[0].end_time, new_end);
    }

    #[tokio::test]
    async fn test_update_partition_empty_partition() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::hours(1);

        // Create an empty partition (Requirement 4.4)
        let partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(0)
                .with_size_bytes(0);

        index.update_partition(partition).await.unwrap();

        // Verify partition is marked as empty but metadata is retained
        let partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions.len(), 1);
        assert!(partitions[0].is_empty);
        assert_eq!(partitions[0].row_count, 0);
        assert_eq!(partitions[0].table_name, "network_activity");
        assert_eq!(partitions[0].partition_key, "2024-01");
        // Time bounds should still be valid
        assert_eq!(partitions[0].start_time, start);
        assert_eq!(partitions[0].end_time, end);
    }

    #[tokio::test]
    async fn test_update_partition_updates_last_modified() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::hours(1);

        let partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(1000);

        let before_update = Utc::now();
        index.update_partition(partition).await.unwrap();

        let partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions.len(), 1);

        // last_modified should be updated (Requirement 4.2)
        assert!(partitions[0].last_modified >= before_update);
    }

    #[tokio::test]
    async fn test_get_partitions_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();

        // Create multiple partitions for the same table
        for i in 1..=3 {
            let partition = crate::partition_metadata::PartitionEntry::new(
                "network_activity",
                format!("2024-{:02}", i),
            )
            .with_time_range(
                start + Duration::days((i - 1) * 30),
                start + Duration::days(i * 30),
            )
            .with_row_count(1_000_000 * i as u64);

            index.update_partition(partition).await.unwrap();
        }

        let partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions.len(), 3);
    }

    #[tokio::test]
    async fn test_get_partitions_empty() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query for a table with no partitions
        let partitions = index.get_partitions("nonexistent_table").await.unwrap();
        assert!(partitions.is_empty());
    }

    #[tokio::test]
    async fn test_get_partitions_filters_by_table() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let start = Utc::now();
        let end = start + Duration::hours(1);

        // Create partitions for different tables
        let partition1 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(1000);
        let partition2 =
            crate::partition_metadata::PartitionEntry::new("process_activity", "2024-01")
                .with_time_range(start, end)
                .with_row_count(2000);
        let partition3 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-02")
                .with_time_range(start, end)
                .with_row_count(3000);

        index.update_partition(partition1).await.unwrap();
        index.update_partition(partition2).await.unwrap();
        index.update_partition(partition3).await.unwrap();

        // Query for network_activity should return 2 partitions
        let network_partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(network_partitions.len(), 2);

        // Query for process_activity should return 1 partition
        let process_partitions = index.get_partitions("process_activity").await.unwrap();
        assert_eq!(process_partitions.len(), 1);
        assert_eq!(process_partitions[0].row_count, 2000);
    }

    #[tokio::test]
    async fn test_get_partitions_in_range_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let base = Utc::now();

        // Create partitions with different time ranges
        let partition1 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(1000);
        let partition2 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-02")
                .with_time_range(base + Duration::hours(2), base + Duration::hours(4))
                .with_row_count(2000);
        let partition3 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-03")
                .with_time_range(base + Duration::hours(4), base + Duration::hours(6))
                .with_row_count(3000);

        index.update_partition(partition1).await.unwrap();
        index.update_partition(partition2).await.unwrap();
        index.update_partition(partition3).await.unwrap();

        // Query for time range that overlaps partition1 and partition2 (Requirement 4.3)
        let query_start = base + Duration::hours(1);
        let query_end = base + Duration::hours(3);

        let partitions = index
            .get_partitions_in_range("network_activity", query_start, query_end)
            .await
            .unwrap();

        assert_eq!(partitions.len(), 2);

        // Verify the correct partitions are returned
        let partition_keys: Vec<&str> = partitions
            .iter()
            .map(|p| p.partition_key.as_str())
            .collect();
        assert!(partition_keys.contains(&"2024-01"));
        assert!(partition_keys.contains(&"2024-02"));
        assert!(!partition_keys.contains(&"2024-03"));
    }

    #[tokio::test]
    async fn test_get_partitions_in_range_no_overlap() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let base = Utc::now();

        // Create a partition
        let partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(1000);

        index.update_partition(partition).await.unwrap();

        // Query for time range that doesn't overlap
        let query_start = base + Duration::hours(3);
        let query_end = base + Duration::hours(5);

        let partitions = index
            .get_partitions_in_range("network_activity", query_start, query_end)
            .await
            .unwrap();

        assert!(partitions.is_empty());
    }

    #[tokio::test]
    async fn test_get_partitions_in_range_filters_by_table() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let base = Utc::now();

        // Create partitions for different tables with overlapping time ranges
        let partition1 =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(1000);
        let partition2 =
            crate::partition_metadata::PartitionEntry::new("process_activity", "2024-01")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(2000);

        index.update_partition(partition1).await.unwrap();
        index.update_partition(partition2).await.unwrap();

        // Query should only return partitions for the specified table
        let query_start = base;
        let query_end = base + Duration::hours(1);

        let network_partitions = index
            .get_partitions_in_range("network_activity", query_start, query_end)
            .await
            .unwrap();
        assert_eq!(network_partitions.len(), 1);
        assert_eq!(network_partitions[0].table_name, "network_activity");

        let process_partitions = index
            .get_partitions_in_range("process_activity", query_start, query_end)
            .await
            .unwrap();
        assert_eq!(process_partitions.len(), 1);
        assert_eq!(process_partitions[0].table_name, "process_activity");
    }

    #[tokio::test]
    async fn test_get_partitions_in_range_includes_empty_partitions() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let base = Utc::now();

        // Create an empty partition (Requirement 4.4)
        let partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "2024-01")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(0);

        index.update_partition(partition).await.unwrap();

        // Empty partitions should still be returned in range queries
        let query_start = base;
        let query_end = base + Duration::hours(1);

        let partitions = index
            .get_partitions_in_range("network_activity", query_start, query_end)
            .await
            .unwrap();

        assert_eq!(partitions.len(), 1);
        assert!(partitions[0].is_empty);
    }

    #[tokio::test]
    async fn test_partition_metadata_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let base = Utc::now();

        // 1. Create partitions for a table
        for i in 0..3 {
            let start = base + Duration::hours(i * 2);
            let end = start + Duration::hours(2);
            let partition = crate::partition_metadata::PartitionEntry::new(
                "network_activity",
                format!("partition_{}", i),
            )
            .with_time_range(start, end)
            .with_row_count(1000 * (i as u64 + 1))
            .with_size_bytes(500_000 * (i as u64 + 1));

            index.update_partition(partition).await.unwrap();
        }

        // 2. Get all partitions
        let all_partitions = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(all_partitions.len(), 3);

        // 3. Query partitions in a specific time range
        let query_start = base + Duration::hours(1);
        let query_end = base + Duration::hours(3);
        let range_partitions = index
            .get_partitions_in_range("network_activity", query_start, query_end)
            .await
            .unwrap();
        assert_eq!(range_partitions.len(), 2); // partition_0 and partition_1 overlap

        // 4. Update a partition
        let updated_partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "partition_0")
                .with_time_range(base, base + Duration::hours(2))
                .with_row_count(5000)
                .with_size_bytes(2_500_000);

        index.update_partition(updated_partition).await.unwrap();

        // 5. Verify update
        let partitions_after_update = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(partitions_after_update.len(), 3); // Still 3 partitions

        let partition_0 = partitions_after_update
            .iter()
            .find(|p| p.partition_key == "partition_0")
            .unwrap();
        assert_eq!(partition_0.row_count, 5000);
        assert_eq!(partition_0.size_bytes, 2_500_000);

        // 6. Create an empty partition
        let empty_partition =
            crate::partition_metadata::PartitionEntry::new("network_activity", "partition_empty")
                .with_time_range(base + Duration::hours(6), base + Duration::hours(8))
                .with_row_count(0);

        index.update_partition(empty_partition).await.unwrap();

        // 7. Verify empty partition is retained
        let all_partitions_final = index.get_partitions("network_activity").await.unwrap();
        assert_eq!(all_partitions_final.len(), 4);

        let empty = all_partitions_final
            .iter()
            .find(|p| p.partition_key == "partition_empty")
            .unwrap();
        assert!(empty.is_empty);
        assert_eq!(empty.row_count, 0);
    }

    // ========================================================================
    // Statistics Tests
    // ========================================================================

    #[tokio::test]
    async fn test_update_statistics_create_new() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1)
            .add_column(
                crate::table_statistics::ColumnStatistics::new("src_ip")
                    .with_counts(50000, 100, 1_000_000),
            );

        let result = index.update_statistics(stats).await;
        assert!(result.is_ok());

        // Verify statistics were stored
        let retrieved = index.get_statistics("network_activity").await.unwrap();
        assert!(retrieved.is_some());

        let stats = retrieved.unwrap();
        assert_eq!(stats.table_name, "network_activity");
        assert_eq!(stats.total_rows, 1_000_000);
        assert_eq!(stats.sample_rate, 0.1);
        assert_eq!(stats.columns.len(), 1);
    }

    #[tokio::test]
    async fn test_update_statistics_update_existing() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Create initial statistics
        let stats1 = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_sample_rate(0.1);

        index.update_statistics(stats1).await.unwrap();

        // Update with new statistics
        let stats2 = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(2_000_000)
            .with_sample_rate(0.2);

        index.update_statistics(stats2).await.unwrap();

        // Verify statistics were updated
        let retrieved = index.get_statistics("network_activity").await.unwrap();
        assert!(retrieved.is_some());

        let stats = retrieved.unwrap();
        assert_eq!(stats.total_rows, 2_000_000);
        assert_eq!(stats.sample_rate, 0.2);
    }

    #[tokio::test]
    async fn test_update_statistics_with_sample_rate() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Requirement 5.2: Statistics should support sampling with configurable sample rate
        let stats = crate::table_statistics::TableStatistics::new("large_table")
            .with_total_rows(100_000_000)
            .with_sample_rate(0.01); // 1% sample for large table

        index.update_statistics(stats).await.unwrap();

        let retrieved = index.get_statistics("large_table").await.unwrap().unwrap();
        assert_eq!(retrieved.sample_rate, 0.01);
    }

    #[tokio::test]
    async fn test_get_statistics_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(500_000)
            .add_column(
                crate::table_statistics::ColumnStatistics::new("src_ip")
                    .with_counts(10000, 50, 500_000)
                    .with_min_max("10.0.0.1", "192.168.255.255"),
            )
            .add_column(
                crate::table_statistics::ColumnStatistics::new("dst_port")
                    .with_counts(65535, 0, 500_000),
            );

        index.update_statistics(stats).await.unwrap();

        let retrieved = index.get_statistics("network_activity").await.unwrap();
        assert!(retrieved.is_some());

        let stats = retrieved.unwrap();
        assert_eq!(stats.table_name, "network_activity");
        assert_eq!(stats.total_rows, 500_000);
        assert_eq!(stats.columns.len(), 2);

        // Verify column statistics
        let src_ip = stats.get_column("src_ip").unwrap();
        assert_eq!(src_ip.distinct_count, 10000);
        assert_eq!(src_ip.null_count, 50);
        assert_eq!(src_ip.min_value, Some("10.0.0.1".to_string()));

        let dst_port = stats.get_column("dst_port").unwrap();
        assert_eq!(dst_port.distinct_count, 65535);
        assert_eq!(dst_port.null_count, 0);
    }

    #[tokio::test]
    async fn test_get_statistics_not_found() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let result = index.get_statistics("nonexistent_table").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_statistics_with_freshness() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Requirement 5.3 & 5.4: Statistics should have collected_at timestamp for freshness
        let stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(1_000_000);

        index.update_statistics(stats).await.unwrap();

        let retrieved = index
            .get_statistics("network_activity")
            .await
            .unwrap()
            .unwrap();

        // Verify collected_at is set and recent
        let now = Utc::now();
        let age = now.signed_duration_since(retrieved.collected_at);
        assert!(age.num_seconds() < 5); // Should be very recent

        // Verify is_stale works correctly
        assert!(!retrieved.is_stale(3600)); // Not stale within 1 hour
    }

    #[tokio::test]
    async fn test_get_statistics_stale_check() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Create statistics with an old timestamp
        let old_timestamp = Utc::now() - Duration::hours(2);
        let stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .with_collected_at(old_timestamp);

        index.update_statistics(stats).await.unwrap();

        let retrieved = index
            .get_statistics("network_activity")
            .await
            .unwrap()
            .unwrap();

        // Verify staleness check (Requirement 5.4)
        assert!(retrieved.is_stale(3600)); // Stale after 1 hour
        assert!(!retrieved.is_stale(10800)); // Not stale within 3 hours
    }

    #[tokio::test]
    async fn test_get_statistics_multiple_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Create statistics for multiple tables
        let stats1 = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(1_000_000);
        let stats2 = crate::table_statistics::TableStatistics::new("process_activity")
            .with_total_rows(500_000);
        let stats3 =
            crate::table_statistics::TableStatistics::new("file_activity").with_total_rows(250_000);

        index.update_statistics(stats1).await.unwrap();
        index.update_statistics(stats2).await.unwrap();
        index.update_statistics(stats3).await.unwrap();

        // Verify each table's statistics are independent
        let network = index
            .get_statistics("network_activity")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(network.total_rows, 1_000_000);

        let process = index
            .get_statistics("process_activity")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(process.total_rows, 500_000);

        let file = index
            .get_statistics("file_activity")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(file.total_rows, 250_000);
    }

    #[tokio::test]
    async fn test_collect_statistics_stub() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // collect_statistics is a stub that returns an error
        let result = index.collect_statistics("network_activity", 0.1).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IndexError::StatisticsNotFound(msg) => {
                assert!(msg.contains("network_activity"));
                assert!(msg.contains("not yet implemented"));
            }
            other => panic!("Expected StatisticsNotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_statistics_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Initially no statistics
        let initial = index.get_statistics("network_activity").await.unwrap();
        assert!(initial.is_none());

        // 2. Create statistics with sampling (Requirement 5.2)
        let stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(10_000_000)
            .with_sample_rate(0.1) // 10% sample
            .add_column(
                crate::table_statistics::ColumnStatistics::new("src_ip")
                    .with_counts(100000, 500, 1_000_000)
                    .with_min_max("10.0.0.1", "192.168.255.255"),
            )
            .add_column(
                crate::table_statistics::ColumnStatistics::new("severity")
                    .with_counts(5, 0, 1_000_000), // Low cardinality
            );

        index.update_statistics(stats).await.unwrap();

        // 3. Retrieve and verify (Requirement 5.4 - freshness)
        let retrieved = index
            .get_statistics("network_activity")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.total_rows, 10_000_000);
        assert_eq!(retrieved.sample_rate, 0.1);
        assert!(!retrieved.is_stale(3600));

        // 4. Check column statistics
        let src_ip = retrieved.get_column("src_ip").unwrap();
        assert_eq!(src_ip.cardinality(), 0.1); // 100000/1000000

        let severity = retrieved.get_column("severity").unwrap();
        assert_eq!(severity.cardinality(), 0.000005); // 5/1000000 - low cardinality

        // 5. Update statistics (e.g., after data refresh)
        let updated_stats = crate::table_statistics::TableStatistics::new("network_activity")
            .with_total_rows(15_000_000)
            .with_sample_rate(0.05); // Lower sample rate for larger table

        index.update_statistics(updated_stats).await.unwrap();

        // 6. Verify update
        let final_stats = index
            .get_statistics("network_activity")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(final_stats.total_rows, 15_000_000);
        assert_eq!(final_stats.sample_rate, 0.05);
    }

    // ========================================================================
    // QueryCache Tests
    // ========================================================================

    #[test]
    fn test_query_cache_new() {
        let cache = QueryCache::new();
        assert_eq!(cache.max_entries(), 1000);
        assert_eq!(cache.default_ttl_secs(), 3600);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_query_cache_new_with_config() {
        let cache = QueryCache::new_with_config(500, 1800);
        assert_eq!(cache.max_entries(), 500);
        assert_eq!(cache.default_ttl_secs(), 1800);
    }

    #[test]
    fn test_query_cache_from_config() {
        let config = IndexConfig::new()
            .with_cache_max_entries(250)
            .with_cache_default_ttl_secs(900);
        let cache = QueryCache::from_config(&config);
        assert_eq!(cache.max_entries(), 250);
        assert_eq!(cache.default_ttl_secs(), 900);
    }

    #[test]
    fn test_query_cache_insert_and_get() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("test_hash", vec!["table1".to_string()]);
        let result = CachedResult::new(key.clone(), serde_json::json!({"data": "test"}), 3600);

        cache.insert(key.clone(), result);

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());

        let cached = retrieved.unwrap();
        assert_eq!(cached.result, serde_json::json!({"data": "test"}));
        assert_eq!(cached.hit_count, 1); // Hit count incremented on get
    }

    #[test]
    fn test_query_cache_get_miss() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("nonexistent", vec!["table1".to_string()]);
        let result = cache.get(&key);

        assert!(result.is_none());

        // Verify miss was recorded in stats
        let stats = cache.stats();
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_count, 0);
    }

    #[test]
    fn test_query_cache_expired_entry() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("test_hash", vec!["table1".to_string()]);
        // Create with 0 TTL (immediately expired)
        let result = CachedResult::new(key.clone(), serde_json::json!({"data": "test"}), 0);

        cache.insert(key.clone(), result);

        // Entry should be removed on get because it's expired
        let retrieved = cache.get(&key);
        assert!(retrieved.is_none());

        // Verify miss was recorded
        let stats = cache.stats();
        assert_eq!(stats.miss_count, 1);
    }

    #[test]
    fn test_query_cache_lru_eviction() {
        // Create cache with max 3 entries
        let cache = QueryCache::new_with_config(3, 3600);

        // Insert 3 entries
        for i in 0..3 {
            let key = QueryCacheKey::new(format!("hash_{}", i), vec!["table1".to_string()]);
            let result = CachedResult::new(key.clone(), serde_json::json!({"index": i}), 3600);
            cache.insert(key, result);
        }

        assert_eq!(cache.len(), 3);

        // Access entry 0 and 2 to make them more recently used
        let key0 = QueryCacheKey::new("hash_0", vec!["table1".to_string()]);
        let key2 = QueryCacheKey::new("hash_2", vec!["table1".to_string()]);
        cache.get(&key0);
        cache.get(&key2);

        // Insert a 4th entry - should evict the LRU entry (hash_1)
        let key3 = QueryCacheKey::new("hash_3", vec!["table1".to_string()]);
        let result3 = CachedResult::new(key3.clone(), serde_json::json!({"index": 3}), 3600);
        cache.insert(key3.clone(), result3);

        // Cache should still have 3 entries
        assert_eq!(cache.len(), 3);

        // hash_1 should have been evicted (it was least recently used)
        let key1 = QueryCacheKey::new("hash_1", vec!["table1".to_string()]);
        assert!(cache.get(&key1).is_none());

        // hash_0, hash_2, and hash_3 should still be present
        assert!(cache.get(&key0).is_some());
        assert!(cache.get(&key2).is_some());
        assert!(cache.get(&key3).is_some());

        // Verify eviction was recorded
        let stats = cache.stats();
        assert!(stats.eviction_count >= 1);
    }

    #[test]
    fn test_query_cache_invalidate_by_table() {
        let cache = QueryCache::new_with_config(10, 3600);

        // Insert entries for different tables
        let key1 = QueryCacheKey::new("hash_1", vec!["table_a".to_string()]);
        let key2 = QueryCacheKey::new("hash_2", vec!["table_a".to_string(), "table_b".to_string()]);
        let key3 = QueryCacheKey::new("hash_3", vec!["table_b".to_string()]);
        let key4 = QueryCacheKey::new("hash_4", vec!["table_c".to_string()]);

        for key in [&key1, &key2, &key3, &key4] {
            let result = CachedResult::new(
                key.clone(),
                serde_json::json!({"key": key.query_hash.clone()}),
                3600,
            );
            cache.insert(key.clone(), result);
        }

        assert_eq!(cache.len(), 4);

        // Invalidate entries for table_a
        let invalidated = cache.invalidate_by_table("table_a");
        assert_eq!(invalidated, 2); // key1 and key2 should be invalidated

        assert_eq!(cache.len(), 2);

        // key1 and key2 should be gone
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_none());

        // key3 and key4 should still be present
        assert!(cache.get(&key3).is_some());
        assert!(cache.get(&key4).is_some());
    }

    #[test]
    fn test_query_cache_clear() {
        let cache = QueryCache::new_with_config(10, 3600);

        // Insert some entries
        for i in 0..5 {
            let key = QueryCacheKey::new(format!("hash_{}", i), vec!["table1".to_string()]);
            let result = CachedResult::new(key.clone(), serde_json::json!({"index": i}), 3600);
            cache.insert(key, result);
        }

        assert_eq!(cache.len(), 5);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        // Verify evictions were recorded
        let stats = cache.stats();
        assert_eq!(stats.eviction_count, 5);
    }

    #[test]
    fn test_query_cache_stats() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("test_hash", vec!["table1".to_string()]);
        let result = CachedResult::new(key.clone(), serde_json::json!({"data": "test"}), 3600);

        cache.insert(key.clone(), result);

        // Record some hits and misses
        cache.get(&key); // hit
        cache.get(&key); // hit
        cache.get(&QueryCacheKey::new("missing", vec![])); // miss

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hit_count, 2);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_query_cache_hit_count_tracking() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("test_hash", vec!["table1".to_string()]);
        let result = CachedResult::new(key.clone(), serde_json::json!({"data": "test"}), 3600);

        cache.insert(key.clone(), result);

        // Access multiple times
        let r1 = cache.get(&key).unwrap();
        assert_eq!(r1.hit_count, 1);

        let r2 = cache.get(&key).unwrap();
        assert_eq!(r2.hit_count, 2);

        let r3 = cache.get(&key).unwrap();
        assert_eq!(r3.hit_count, 3);
    }

    #[test]
    fn test_query_cache_update_existing_key() {
        let cache = QueryCache::new_with_config(10, 3600);

        let key = QueryCacheKey::new("test_hash", vec!["table1".to_string()]);

        // Insert initial value
        let result1 = CachedResult::new(key.clone(), serde_json::json!({"version": 1}), 3600);
        cache.insert(key.clone(), result1);

        // Insert updated value with same key
        let result2 = CachedResult::new(key.clone(), serde_json::json!({"version": 2}), 3600);
        cache.insert(key.clone(), result2);

        // Should still have only 1 entry
        assert_eq!(cache.len(), 1);

        // Should get the updated value
        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved.result, serde_json::json!({"version": 2}));
    }

    #[tokio::test]
    async fn test_semantic_index_query_cache_initialized_from_config() {
        let config = IndexConfig::new()
            .with_cache_max_entries(500)
            .with_cache_default_ttl_secs(1800);

        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, config).await.unwrap();

        // The query_cache should be initialized with the config values
        // We can't directly access it, but we can verify the config is stored
        assert_eq!(index.config().cache_max_entries, 500);
        assert_eq!(index.config().cache_default_ttl_secs, 1800);
    }

    // ========================================================================
    // SemanticIndex Cache Method Tests
    // ========================================================================

    #[tokio::test]
    async fn test_cache_query_result_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result = serde_json::json!({"rows": [{"src_ip": "192.168.1.1"}]});

        // Cache the result with 1 hour TTL
        let cache_result = index.cache_query_result(&key, &result, 3600).await;
        assert!(cache_result.is_ok());

        // Verify the result was cached
        let stats = index.get_cache_stats().await;
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_query_result_with_custom_ttl() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result = serde_json::json!({"data": "test"});

        // Cache with 30 minute TTL
        index.cache_query_result(&key, &result, 1800).await.unwrap();

        // Retrieve and verify TTL was applied
        let cached = index.get_cached_result(&key).await.unwrap().unwrap();
        assert!(!cached.is_expired());
    }

    #[tokio::test]
    async fn test_get_cached_result_hit() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result =
            serde_json::json!({"rows": [{"src_ip": "192.168.1.1"}, {"src_ip": "10.0.0.1"}]});

        // Cache the result
        index.cache_query_result(&key, &result, 3600).await.unwrap();

        // Retrieve the cached result
        let cached = index.get_cached_result(&key).await.unwrap();
        assert!(cached.is_some());

        let cached_result = cached.unwrap();
        assert_eq!(cached_result.result, result);
        assert_eq!(cached_result.hit_count, 1);
        assert!(!cached_result.is_expired());
    }

    #[tokio::test]
    async fn test_get_cached_result_miss() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("nonexistent", vec!["network_activity".to_string()]);

        // Try to get a non-existent cache entry
        let cached = index.get_cached_result(&key).await.unwrap();
        assert!(cached.is_none());

        // Verify miss was recorded
        let stats = index.get_cache_stats().await;
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_count, 0);
    }

    #[tokio::test]
    async fn test_get_cached_result_expired() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result = serde_json::json!({"data": "test"});

        // Cache with 0 TTL (immediately expired)
        index.cache_query_result(&key, &result, 0).await.unwrap();

        // Try to retrieve - should return None because it's expired (Requirement 6.3)
        let cached = index.get_cached_result(&key).await.unwrap();
        assert!(cached.is_none());

        // Verify miss was recorded
        let stats = index.get_cache_stats().await;
        assert_eq!(stats.miss_count, 1);
    }

    #[tokio::test]
    async fn test_get_cached_result_multiple_hits() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result = serde_json::json!({"data": "test"});

        index.cache_query_result(&key, &result, 3600).await.unwrap();

        // Access multiple times
        let r1 = index.get_cached_result(&key).await.unwrap().unwrap();
        assert_eq!(r1.hit_count, 1);

        let r2 = index.get_cached_result(&key).await.unwrap().unwrap();
        assert_eq!(r2.hit_count, 2);

        let r3 = index.get_cached_result(&key).await.unwrap().unwrap();
        assert_eq!(r3.hit_count, 3);

        // Verify hits were recorded in stats
        let stats = index.get_cache_stats().await;
        assert_eq!(stats.hit_count, 3);
    }

    #[tokio::test]
    async fn test_invalidate_cache_success() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Cache results for different tables
        let key1 = QueryCacheKey::new("hash1", vec!["network_activity".to_string()]);
        let key2 = QueryCacheKey::new("hash2", vec!["network_activity".to_string()]);
        let key3 = QueryCacheKey::new("hash3", vec!["process_activity".to_string()]);

        index
            .cache_query_result(&key1, &serde_json::json!({}), 3600)
            .await
            .unwrap();
        index
            .cache_query_result(&key2, &serde_json::json!({}), 3600)
            .await
            .unwrap();
        index
            .cache_query_result(&key3, &serde_json::json!({}), 3600)
            .await
            .unwrap();

        // Verify all 3 entries are cached
        let stats_before = index.get_cache_stats().await;
        assert_eq!(stats_before.total_entries, 3);

        // Invalidate cache for network_activity
        let invalidated = index.invalidate_cache("network_activity").await.unwrap();
        assert_eq!(invalidated, 2);

        // Verify only process_activity entry remains
        let stats_after = index.get_cache_stats().await;
        assert_eq!(stats_after.total_entries, 1);

        // Verify network_activity entries are gone
        assert!(index.get_cached_result(&key1).await.unwrap().is_none());
        assert!(index.get_cached_result(&key2).await.unwrap().is_none());

        // Verify process_activity entry still exists
        assert!(index.get_cached_result(&key3).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_invalidate_cache_no_matches() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Cache a result for network_activity
        let key = QueryCacheKey::new("hash1", vec!["network_activity".to_string()]);
        index
            .cache_query_result(&key, &serde_json::json!({}), 3600)
            .await
            .unwrap();

        // Invalidate cache for a different table
        let invalidated = index.invalidate_cache("nonexistent_table").await.unwrap();
        assert_eq!(invalidated, 0);

        // Original entry should still exist
        assert!(index.get_cached_result(&key).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_invalidate_cache_multi_table_query() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Cache a result that involves multiple tables
        let key = QueryCacheKey::new(
            "hash1",
            vec![
                "network_activity".to_string(),
                "process_activity".to_string(),
            ],
        );
        index
            .cache_query_result(&key, &serde_json::json!({}), 3600)
            .await
            .unwrap();

        // Invalidating either table should remove the entry
        let invalidated = index.invalidate_cache("network_activity").await.unwrap();
        assert_eq!(invalidated, 1);

        assert!(index.get_cached_result(&key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_cache_stats_empty() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let stats = index.get_cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.eviction_count, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_get_cache_stats_with_activity() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let key = QueryCacheKey::new("test_hash", vec!["network_activity".to_string()]);
        let result = serde_json::json!({"data": "test"});

        // Insert an entry
        index.cache_query_result(&key, &result, 3600).await.unwrap();

        // Generate some hits and misses
        index.get_cached_result(&key).await.unwrap(); // hit
        index.get_cached_result(&key).await.unwrap(); // hit
        index
            .get_cached_result(&QueryCacheKey::new("missing", vec![]))
            .await
            .unwrap(); // miss

        let stats = index.get_cache_stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hit_count, 2);
        assert_eq!(stats.miss_count, 1);
        assert!((stats.hit_rate() - 2.0 / 3.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cache_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Cache a query result
        let key = QueryCacheKey::new("query_hash_123", vec!["network_activity".to_string()]);
        let result = serde_json::json!({
            "rows": [
                {"src_ip": "192.168.1.1", "dst_ip": "10.0.0.1"},
                {"src_ip": "192.168.1.2", "dst_ip": "10.0.0.2"}
            ],
            "total": 2
        });

        index.cache_query_result(&key, &result, 3600).await.unwrap();

        // 2. Retrieve the cached result
        let cached = index.get_cached_result(&key).await.unwrap().unwrap();
        assert_eq!(cached.result["total"], 2);
        assert_eq!(cached.hit_count, 1);

        // 3. Check cache stats
        let stats = index.get_cache_stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hit_count, 1);

        // 4. Simulate data change - invalidate cache
        let invalidated = index.invalidate_cache("network_activity").await.unwrap();
        assert_eq!(invalidated, 1);

        // 5. Verify cache is empty
        let cached_after = index.get_cached_result(&key).await.unwrap();
        assert!(cached_after.is_none());

        // 6. Check final stats
        let final_stats = index.get_cache_stats().await;
        assert_eq!(final_stats.total_entries, 0);
        assert_eq!(final_stats.eviction_count, 1);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction_via_semantic_index() {
        // Create index with small cache
        let config = IndexConfig::default().with_cache_max_entries(3);
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, config).await.unwrap();

        // Insert 3 entries
        for i in 0..3 {
            let key = QueryCacheKey::new(format!("hash_{}", i), vec!["table".to_string()]);
            index
                .cache_query_result(&key, &serde_json::json!({"i": i}), 3600)
                .await
                .unwrap();
        }

        // Access entry 0 and 2 to make them more recently used
        let key0 = QueryCacheKey::new("hash_0", vec!["table".to_string()]);
        let key2 = QueryCacheKey::new("hash_2", vec!["table".to_string()]);
        index.get_cached_result(&key0).await.unwrap();
        index.get_cached_result(&key2).await.unwrap();

        // Insert a 4th entry - should evict LRU (hash_1)
        let key3 = QueryCacheKey::new("hash_3", vec!["table".to_string()]);
        index
            .cache_query_result(&key3, &serde_json::json!({"i": 3}), 3600)
            .await
            .unwrap();

        // Verify hash_1 was evicted
        let key1 = QueryCacheKey::new("hash_1", vec!["table".to_string()]);
        assert!(index.get_cached_result(&key1).await.unwrap().is_none());

        // Verify others still exist
        assert!(index.get_cached_result(&key0).await.unwrap().is_some());
        assert!(index.get_cached_result(&key2).await.unwrap().is_some());
        assert!(index.get_cached_result(&key3).await.unwrap().is_some());

        // Verify eviction was recorded
        let stats = index.get_cache_stats().await;
        assert!(stats.eviction_count >= 1);
    }

    // ========================================================================
    // Query Plan Hints Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_query_plan_hints_empty_index() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // No tables registered, so hints should be empty
        assert!(hints.tables.is_empty());
        assert!(hints.partitions.is_empty());
        assert!(hints.statistics.is_empty());
        assert!(hints.cached_result.is_none());
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_with_matching_table() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table that matches the entity
        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.3.0")
            .with_dialect(WarehouseDialect::Snowflake);
        index.register_table(table).await.unwrap();

        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should have one table hint
        assert_eq!(hints.tables.len(), 1);
        assert_eq!(hints.tables[0].table_name, "network_activity");
        assert_eq!(hints.tables[0].schema_name, Some("ocsf".to_string()));
        assert_eq!(hints.tables[0].ocsf_version, "1.3.0");
        assert_eq!(hints.tables[0].dialect, WarehouseDialect::Snowflake);
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_with_partitions() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table
        let table = TableEntry::new("network_activity", 4001).with_ocsf_version("1.3.0");
        index.register_table(table).await.unwrap();

        // Create partitions
        let now = Utc::now();
        let partition1 = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(now - Duration::days(60), now - Duration::days(30))
            .with_row_count(1_000_000);
        let partition2 = PartitionEntry::new("network_activity", "2024-02")
            .with_time_range(now - Duration::days(30), now)
            .with_row_count(500_000);

        index.update_partition(partition1).await.unwrap();
        index.update_partition(partition2).await.unwrap();

        // Query without time range should return all partitions
        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        assert_eq!(hints.partitions.len(), 2);
        assert_eq!(hints.total_estimated_rows(), 1_500_000);
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_with_time_range_pruning() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table
        let table = TableEntry::new("network_activity", 4001).with_ocsf_version("1.3.0");
        index.register_table(table).await.unwrap();

        // Create partitions with specific time ranges
        let now = Utc::now();
        // Partition 1: 60-30 days ago (should NOT be included in -7d query)
        let partition1 = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(now - Duration::days(60), now - Duration::days(30))
            .with_row_count(1_000_000);
        // Partition 2: 30-10 days ago (should NOT be included in -7d query)
        let partition2 = PartitionEntry::new("network_activity", "2024-02")
            .with_time_range(now - Duration::days(30), now - Duration::days(10))
            .with_row_count(500_000);
        // Partition 3: last 10 days (SHOULD be included in -7d query)
        let partition3 = PartitionEntry::new("network_activity", "2024-03")
            .with_time_range(now - Duration::days(10), now + Duration::days(1))
            .with_row_count(100_000);

        index.update_partition(partition1).await.unwrap();
        index.update_partition(partition2).await.unwrap();
        index.update_partition(partition3).await.unwrap();

        // Query with time range that only overlaps partition3
        // Using relative time "-7d" to "now"
        let query = ocsf_semantic::query::SemanticQuery::new("network_activity")
            .with_time_range(ocsf_semantic::query::TimeRange::new("-7d", "now"));

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should only return partition3 (overlaps with last 7 days)
        assert_eq!(hints.partitions.len(), 1);
        assert_eq!(hints.partitions[0].partition_key, "2024-03");
        assert_eq!(hints.partitions[0].estimated_rows, 100_000);
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_with_statistics() {
        use crate::table_statistics::{ColumnStatistics, TableStatistics};

        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table
        let table = TableEntry::new("network_activity", 4001).with_ocsf_version("1.3.0");
        index.register_table(table).await.unwrap();

        // Add statistics
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(10_000_000)
            .with_sample_rate(0.1)
            .add_column(ColumnStatistics::new("src_ip").with_counts(50000, 100, 10_000_000));
        index.update_statistics(stats).await.unwrap();

        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should have statistics
        assert_eq!(hints.statistics.len(), 1);
        assert_eq!(hints.statistics[0].table_name, "network_activity");
        assert_eq!(hints.statistics[0].total_rows, 10_000_000);
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_with_cached_result() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table
        let table = TableEntry::new("network_activity", 4001).with_ocsf_version("1.3.0");
        index.register_table(table).await.unwrap();

        // Create a query and cache a result for it
        let query = ocsf_semantic::query::SemanticQuery::new("network_activity")
            .add_select("src_ip".to_string());

        let cache_key = QueryCacheKey::from_query(&query);
        let result = serde_json::json!({"rows": [{"src_ip": "192.168.1.1"}]});
        index
            .cache_query_result(&cache_key, &result, 3600)
            .await
            .unwrap();

        // Get hints - should include cached result
        let hints = index.get_query_plan_hints(&query).await.unwrap();

        assert!(hints.cached_result.is_some());
        assert!(hints.has_valid_cache());
        assert_eq!(hints.cached_result.as_ref().unwrap().result, result);
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_excludes_inactive_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables
        let table1 = TableEntry::new("network_activity_active", 4001).with_ocsf_version("1.3.0");
        let table2 = TableEntry::new("network_activity_inactive", 4001).with_ocsf_version("1.3.0");

        index.register_table(table1).await.unwrap();
        let table2_id = index.register_table(table2).await.unwrap();

        // Deactivate one table
        index.deregister_table(table2_id).await.unwrap();

        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should only have the active table
        assert_eq!(hints.tables.len(), 1);
        assert_eq!(hints.tables[0].table_name, "network_activity_active");
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_multiple_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register multiple tables for the same entity (partitioned/sharded)
        let table1 = TableEntry::new("network_activity_2024", 4001)
            .with_ocsf_version("1.3.0")
            .with_dialect(WarehouseDialect::Snowflake);
        let table2 = TableEntry::new("network_activity_2023", 4001)
            .with_ocsf_version("1.3.0")
            .with_dialect(WarehouseDialect::Snowflake);

        index.register_table(table1).await.unwrap();
        index.register_table(table2).await.unwrap();

        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should have both tables
        assert_eq!(hints.tables.len(), 2);
        let table_names: Vec<&str> = hints.tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(table_names.contains(&"network_activity_2024"));
        assert!(table_names.contains(&"network_activity_2023"));
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_case_insensitive_matching() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with different case
        let table = TableEntry::new("Network_Activity", 4001).with_ocsf_version("1.3.0");
        index.register_table(table).await.unwrap();

        // Query with lowercase entity
        let query = ocsf_semantic::query::SemanticQuery::new("network_activity");

        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Should match despite case difference
        assert_eq!(hints.tables.len(), 1);
        assert_eq!(hints.tables[0].table_name, "Network_Activity");
    }

    #[tokio::test]
    async fn test_get_query_plan_hints_full_workflow() {
        use crate::table_statistics::{ColumnStatistics, TableStatistics};

        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // 1. Register a table
        let table = TableEntry::new("network_activity", 4001)
            .with_schema("ocsf")
            .with_ocsf_version("1.3.0")
            .with_dialect(WarehouseDialect::Databricks);
        index.register_table(table).await.unwrap();

        // 2. Create partitions
        let now = Utc::now();
        let partition = PartitionEntry::new("network_activity", "2024-01")
            .with_time_range(now - Duration::days(30), now)
            .with_row_count(1_000_000);
        index.update_partition(partition).await.unwrap();

        // 3. Add statistics
        let stats = TableStatistics::new("network_activity")
            .with_total_rows(1_000_000)
            .add_column(ColumnStatistics::new("src_ip").with_counts(10000, 50, 1_000_000));
        index.update_statistics(stats).await.unwrap();

        // 4. Create and cache a query result
        let query = ocsf_semantic::query::SemanticQuery::new("network_activity")
            .add_select("src_ip".to_string())
            .with_time_range(ocsf_semantic::query::TimeRange::new("-7d", "now"));

        let cache_key = QueryCacheKey::from_query(&query);
        index
            .cache_query_result(&cache_key, &serde_json::json!({"count": 100}), 3600)
            .await
            .unwrap();

        // 5. Get query plan hints
        let hints = index.get_query_plan_hints(&query).await.unwrap();

        // Verify all components are present
        assert_eq!(hints.tables.len(), 1);
        assert_eq!(hints.tables[0].table_name, "network_activity");
        assert_eq!(hints.tables[0].schema_name, Some("ocsf".to_string()));
        assert_eq!(hints.tables[0].dialect, WarehouseDialect::Databricks);

        assert_eq!(hints.partitions.len(), 1);
        assert_eq!(hints.partitions[0].partition_key, "2024-01");
        assert_eq!(hints.partitions[0].estimated_rows, 1_000_000);

        assert_eq!(hints.statistics.len(), 1);
        assert_eq!(hints.statistics[0].total_rows, 1_000_000);

        assert!(hints.cached_result.is_some());
        assert!(hints.has_valid_cache());
    }

    // ========================================================================
    // parse_time_string Tests
    // ========================================================================

    #[test]
    fn test_parse_time_string_now() {
        let result = super::parse_time_string("now");
        assert!(result.is_some());
        // Should be close to current time
        let diff = (Utc::now() - result.unwrap()).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn test_parse_time_string_now_case_insensitive() {
        assert!(super::parse_time_string("NOW").is_some());
        assert!(super::parse_time_string("Now").is_some());
    }

    #[test]
    fn test_parse_time_string_relative_days() {
        let result = super::parse_time_string("-7d");
        assert!(result.is_some());
        let expected = Utc::now() - Duration::days(7);
        let diff = (expected - result.unwrap()).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn test_parse_time_string_relative_hours() {
        let result = super::parse_time_string("-24h");
        assert!(result.is_some());
        let expected = Utc::now() - Duration::hours(24);
        let diff = (expected - result.unwrap()).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn test_parse_time_string_iso8601() {
        let result = super::parse_time_string("2024-01-15T10:30:00Z");
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_time_string_invalid() {
        assert!(super::parse_time_string("invalid").is_none());
        assert!(super::parse_time_string("").is_none());
        assert!(super::parse_time_string("-abc").is_none());
    }

    #[test]
    fn test_parse_time_string_whitespace() {
        let result = super::parse_time_string("  now  ");
        assert!(result.is_some());
    }

    // ========================================================================
    // Detection Coverage Query Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_single_match() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()]);
        let table = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query for a technique that exists
        let tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].table_name, "network_activity");
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_multiple_matches() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register multiple tables with the same technique
        let coverage1 =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        let table1 = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage1);
        index.register_table(table1).await.unwrap();

        let coverage2 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1059.001".to_string()]);
        let table2 = TableEntry::new("process_events", 1001).with_detection_coverage(coverage2);
        index.register_table(table2).await.unwrap();

        // Query for the shared technique
        let tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert_eq!(tables.len(), 2);
        let table_names: Vec<&str> = tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(table_names.contains(&"network_activity"));
        assert!(table_names.contains(&"process_events"));
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_no_match() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        let table = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query for a technique that doesn't exist
        let tables = index.get_tables_by_mitre_technique("T9999").await.unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_no_coverage() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table without detection coverage
        let table = TableEntry::new("network_activity", 4001);
        index.register_table(table).await.unwrap();

        // Query for any technique
        let tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_excludes_inactive() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        let table = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage);
        let table_id = index.register_table(table).await.unwrap();

        // Deactivate the table
        index.deregister_table(table_id).await.unwrap();

        // Query should not return the inactive table
        let tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_case_sensitive() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        let table = TableEntry::new("network_activity", 4001).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query with different case should not match
        let tables = index
            .get_tables_by_mitre_technique("t1071.004")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_technique_empty_index() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query on empty index
        let tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_single_match() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage = DetectionCoverage::new().with_mitre_tactics(vec![
            "credential-access".to_string(),
            "lateral-movement".to_string(),
        ]);
        let table =
            TableEntry::new("authentication_events", 3002).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query for a tactic that exists
        let tables = index
            .get_tables_by_mitre_tactic("credential-access")
            .await
            .unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].table_name, "authentication_events");
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_multiple_matches() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register multiple tables with the same tactic
        let coverage1 =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);
        let table1 =
            TableEntry::new("authentication_events", 3002).with_detection_coverage(coverage1);
        index.register_table(table1).await.unwrap();

        let coverage2 = DetectionCoverage::new().with_mitre_tactics(vec![
            "credential-access".to_string(),
            "persistence".to_string(),
        ]);
        let table2 = TableEntry::new("account_changes", 3001).with_detection_coverage(coverage2);
        index.register_table(table2).await.unwrap();

        // Query for the shared tactic
        let tables = index
            .get_tables_by_mitre_tactic("credential-access")
            .await
            .unwrap();
        assert_eq!(tables.len(), 2);
        let table_names: Vec<&str> = tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(table_names.contains(&"authentication_events"));
        assert!(table_names.contains(&"account_changes"));
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_no_match() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);
        let table =
            TableEntry::new("authentication_events", 3002).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query for a tactic that doesn't exist
        let tables = index
            .get_tables_by_mitre_tactic("unknown-tactic")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_no_coverage() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table without detection coverage
        let table = TableEntry::new("authentication_events", 3002);
        index.register_table(table).await.unwrap();

        // Query for any tactic
        let tables = index
            .get_tables_by_mitre_tactic("credential-access")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_excludes_inactive() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);
        let table =
            TableEntry::new("authentication_events", 3002).with_detection_coverage(coverage);
        let table_id = index.register_table(table).await.unwrap();

        // Deactivate the table
        index.deregister_table(table_id).await.unwrap();

        // Query should not return the inactive table
        let tables = index
            .get_tables_by_mitre_tactic("credential-access")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_case_sensitive() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register a table with detection coverage
        let coverage =
            DetectionCoverage::new().with_mitre_tactics(vec!["credential-access".to_string()]);
        let table =
            TableEntry::new("authentication_events", 3002).with_detection_coverage(coverage);
        index.register_table(table).await.unwrap();

        // Query with different case should not match
        let tables = index
            .get_tables_by_mitre_tactic("Credential-Access")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_get_tables_by_mitre_tactic_empty_index() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Query on empty index
        let tables = index
            .get_tables_by_mitre_tactic("credential-access")
            .await
            .unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_detection_coverage_mixed_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables with various coverage configurations
        // Table 1: Has both technique and tactic
        let coverage1 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_mitre_tactics(vec!["command-and-control".to_string()]);
        let table1 = TableEntry::new("network_c2", 4001).with_detection_coverage(coverage1);
        index.register_table(table1).await.unwrap();

        // Table 2: Has only technique
        let coverage2 =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        let table2 = TableEntry::new("dns_logs", 4003).with_detection_coverage(coverage2);
        index.register_table(table2).await.unwrap();

        // Table 3: Has only tactic
        let coverage3 =
            DetectionCoverage::new().with_mitre_tactics(vec!["command-and-control".to_string()]);
        let table3 = TableEntry::new("beacon_detection", 4001).with_detection_coverage(coverage3);
        index.register_table(table3).await.unwrap();

        // Table 4: No coverage
        let table4 = TableEntry::new("raw_logs", 4001);
        index.register_table(table4).await.unwrap();

        // Query by technique - should return tables 1 and 2
        let technique_tables = index
            .get_tables_by_mitre_technique("T1071.004")
            .await
            .unwrap();
        assert_eq!(technique_tables.len(), 2);
        let technique_names: Vec<&str> = technique_tables
            .iter()
            .map(|t| t.table_name.as_str())
            .collect();
        assert!(technique_names.contains(&"network_c2"));
        assert!(technique_names.contains(&"dns_logs"));

        // Query by tactic - should return tables 1 and 3
        let tactic_tables = index
            .get_tables_by_mitre_tactic("command-and-control")
            .await
            .unwrap();
        assert_eq!(tactic_tables.len(), 2);
        let tactic_names: Vec<&str> = tactic_tables
            .iter()
            .map(|t| t.table_name.as_str())
            .collect();
        assert!(tactic_names.contains(&"network_c2"));
        assert!(tactic_names.contains(&"beacon_detection"));
    }

    // ========================================================================
    // Detection Coverage Summary Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_detection_coverage_summary_empty_index() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        assert_eq!(summary.tables_with_coverage, 0);
        assert!(summary.mitre_techniques.is_empty());
        assert!(summary.mitre_tactics.is_empty());
        assert!(summary.data_sources.is_empty());
        assert!(summary.kill_chain_coverage.is_empty());
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_no_coverage_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables without detection coverage
        index
            .register_table(TableEntry::new("table1", 4001))
            .await
            .unwrap();
        index
            .register_table(TableEntry::new("table2", 4002))
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        assert_eq!(summary.tables_with_coverage, 0);
        assert!(summary.mitre_techniques.is_empty());
        assert!(summary.mitre_tactics.is_empty());
        assert!(summary.data_sources.is_empty());
        assert!(summary.kill_chain_coverage.is_empty());
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_single_table() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()])
            .with_mitre_tactics(vec!["command-and-control".to_string()])
            .with_data_sources(vec!["network_connection".to_string()])
            .with_kill_chain_phases(vec!["delivery".to_string()]);

        index
            .register_table(
                TableEntry::new("network_activity", 4001).with_detection_coverage(coverage),
            )
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        assert_eq!(summary.tables_with_coverage, 1);
        assert_eq!(summary.mitre_techniques.len(), 2);
        assert_eq!(summary.mitre_tactics.len(), 1);
        assert_eq!(summary.data_sources.len(), 1);
        assert_eq!(summary.kill_chain_coverage.len(), 1);

        // Check technique details
        let t1071 = summary
            .mitre_techniques
            .iter()
            .find(|t| t.technique_id == "T1071.004")
            .unwrap();
        assert_eq!(t1071.table_count, 1);
        assert_eq!(t1071.tables, vec!["network_activity".to_string()]);

        // Check tactic details
        let c2_tactic = summary
            .mitre_tactics
            .iter()
            .find(|t| t.tactic == "command-and-control")
            .unwrap();
        assert_eq!(c2_tactic.table_count, 1);
        assert_eq!(c2_tactic.technique_count, 2); // Both techniques from this table

        // Check data source details
        let network_ds = summary
            .data_sources
            .iter()
            .find(|d| d.data_source == "network_connection")
            .unwrap();
        assert_eq!(network_ds.table_count, 1);
        assert_eq!(network_ds.tables, vec!["network_activity".to_string()]);

        // Check kill chain details
        let delivery = summary
            .kill_chain_coverage
            .iter()
            .find(|k| k.phase == "delivery")
            .unwrap();
        assert_eq!(delivery.table_count, 1);
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_multiple_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Table 1: Network activity
        let coverage1 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string(), "T1110.003".to_string()])
            .with_mitre_tactics(vec!["command-and-control".to_string()])
            .with_data_sources(vec!["network_connection".to_string()])
            .with_kill_chain_phases(vec!["delivery".to_string()]);
        index
            .register_table(
                TableEntry::new("network_activity", 4001).with_detection_coverage(coverage1),
            )
            .await
            .unwrap();

        // Table 2: Auth events (shares T1071.004 technique)
        let coverage2 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_mitre_tactics(vec!["credential-access".to_string()])
            .with_data_sources(vec!["authentication_log".to_string()])
            .with_kill_chain_phases(vec!["exploitation".to_string()]);
        index
            .register_table(TableEntry::new("auth_events", 3002).with_detection_coverage(coverage2))
            .await
            .unwrap();

        // Table 3: DNS logs (shares network_connection data source)
        let coverage3 = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.001".to_string()])
            .with_mitre_tactics(vec!["command-and-control".to_string()])
            .with_data_sources(vec![
                "network_connection".to_string(),
                "dns_query".to_string(),
            ])
            .with_kill_chain_phases(vec!["delivery".to_string()]);
        index
            .register_table(TableEntry::new("dns_logs", 4003).with_detection_coverage(coverage3))
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        assert_eq!(summary.tables_with_coverage, 3);

        // Check techniques: T1071.004 should be covered by 2 tables
        assert_eq!(summary.mitre_techniques.len(), 3); // T1071.004, T1110.003, T1071.001
        let t1071_004 = summary
            .mitre_techniques
            .iter()
            .find(|t| t.technique_id == "T1071.004")
            .unwrap();
        assert_eq!(t1071_004.table_count, 2);
        assert!(t1071_004.tables.contains(&"network_activity".to_string()));
        assert!(t1071_004.tables.contains(&"auth_events".to_string()));

        // Check tactics: command-and-control should be covered by 2 tables
        assert_eq!(summary.mitre_tactics.len(), 2); // command-and-control, credential-access
        let c2_tactic = summary
            .mitre_tactics
            .iter()
            .find(|t| t.tactic == "command-and-control")
            .unwrap();
        assert_eq!(c2_tactic.table_count, 2);
        // Technique count should include techniques from both tables with this tactic
        assert_eq!(c2_tactic.technique_count, 3); // T1071.004, T1110.003, T1071.001

        // Check data sources: network_connection should be covered by 2 tables
        assert_eq!(summary.data_sources.len(), 3); // network_connection, authentication_log, dns_query
        let network_ds = summary
            .data_sources
            .iter()
            .find(|d| d.data_source == "network_connection")
            .unwrap();
        assert_eq!(network_ds.table_count, 2);
        assert!(network_ds.tables.contains(&"network_activity".to_string()));
        assert!(network_ds.tables.contains(&"dns_logs".to_string()));

        // Check kill chain: delivery should be covered by 2 tables
        assert_eq!(summary.kill_chain_coverage.len(), 2); // delivery, exploitation
        let delivery = summary
            .kill_chain_coverage
            .iter()
            .find(|k| k.phase == "delivery")
            .unwrap();
        assert_eq!(delivery.table_count, 2);
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_excludes_inactive_tables() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register an active table
        let coverage1 =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1071.004".to_string()]);
        index
            .register_table(
                TableEntry::new("active_table", 4001).with_detection_coverage(coverage1),
            )
            .await
            .unwrap();

        // Register and deactivate another table
        let coverage2 =
            DetectionCoverage::new().with_mitre_techniques(vec!["T1110.003".to_string()]);
        let table_id = index
            .register_table(
                TableEntry::new("inactive_table", 4002).with_detection_coverage(coverage2),
            )
            .await
            .unwrap();
        index.deregister_table(table_id).await.unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        // Only the active table should be counted
        assert_eq!(summary.tables_with_coverage, 1);
        assert_eq!(summary.mitre_techniques.len(), 1);
        assert_eq!(summary.mitre_techniques[0].technique_id, "T1071.004");
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_sorted_output() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        // Register tables with coverage in non-alphabetical order
        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec![
                "T1110.003".to_string(),
                "T1071.004".to_string(),
                "T1059.001".to_string(),
            ])
            .with_mitre_tactics(vec![
                "lateral-movement".to_string(),
                "credential-access".to_string(),
            ])
            .with_data_sources(vec![
                "process_creation".to_string(),
                "authentication_log".to_string(),
            ])
            .with_kill_chain_phases(vec!["exploitation".to_string(), "delivery".to_string()]);
        index
            .register_table(TableEntry::new("test_table", 4001).with_detection_coverage(coverage))
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        // Verify techniques are sorted
        let technique_ids: Vec<&str> = summary
            .mitre_techniques
            .iter()
            .map(|t| t.technique_id.as_str())
            .collect();
        assert_eq!(technique_ids, vec!["T1059.001", "T1071.004", "T1110.003"]);

        // Verify tactics are sorted
        let tactics: Vec<&str> = summary
            .mitre_tactics
            .iter()
            .map(|t| t.tactic.as_str())
            .collect();
        assert_eq!(tactics, vec!["credential-access", "lateral-movement"]);

        // Verify data sources are sorted
        let data_sources: Vec<&str> = summary
            .data_sources
            .iter()
            .map(|d| d.data_source.as_str())
            .collect();
        assert_eq!(data_sources, vec!["authentication_log", "process_creation"]);

        // Verify kill chain phases are sorted
        let phases: Vec<&str> = summary
            .kill_chain_coverage
            .iter()
            .map(|k| k.phase.as_str())
            .collect();
        assert_eq!(phases, vec!["delivery", "exploitation"]);
    }

    #[tokio::test]
    async fn test_get_detection_coverage_summary_json_serialization() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let coverage = DetectionCoverage::new()
            .with_mitre_techniques(vec!["T1071.004".to_string()])
            .with_mitre_tactics(vec!["command-and-control".to_string()])
            .with_data_sources(vec!["network_connection".to_string()])
            .with_kill_chain_phases(vec!["delivery".to_string()]);
        index
            .register_table(
                TableEntry::new("network_activity", 4001).with_detection_coverage(coverage),
            )
            .await
            .unwrap();

        let summary = index.get_detection_coverage_summary().await.unwrap();

        // Verify JSON serialization works
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("tables_with_coverage"));
        assert!(json.contains("T1071.004"));
        assert!(json.contains("command-and-control"));
        assert!(json.contains("network_connection"));
        assert!(json.contains("delivery"));

        // Verify round-trip
        let deserialized: crate::detection_coverage::DetectionCoverageSummary =
            serde_json::from_str(&json).unwrap();
        assert_eq!(summary, deserialized);
    }

    // ========================================================================
    // Filtered Lineage Query Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_source_lineage_filtered_basic() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record lineage with different timestamps
        for i in 0..5 {
            let lineage =
                SourceLineageRecord::new("splunk", format!("logs_{}", i), "network_activity")
                    .with_ingestion_timestamp(now - Duration::hours(i as i64));
            index.record_source_lineage(lineage).await.unwrap();
        }

        // Query with time range that includes all records
        let start = now - Duration::hours(10);
        let end = now + Duration::hours(1);
        let result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        assert_eq!(result.edges.len(), 5);
        assert_eq!(result.total_count, 5);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_get_source_lineage_filtered_time_range() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record lineage with different timestamps
        for i in 0..10 {
            let lineage =
                SourceLineageRecord::new("splunk", format!("logs_{}", i), "network_activity")
                    .with_ingestion_timestamp(now - Duration::hours(i as i64));
            index.record_source_lineage(lineage).await.unwrap();
        }

        // Query with time range that includes only recent records (last 3 hours)
        let start = now - Duration::hours(3);
        let end = now + Duration::hours(1);
        let result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        // Should include records from hours 0, 1, 2, 3 (4 records)
        assert_eq!(result.total_count, 4);
        assert_eq!(result.edges.len(), 4);
        assert!(!result.has_more);

        // Verify all returned edges are within the time range
        for edge in &result.edges {
            assert!(edge.timestamp >= start);
            assert!(edge.timestamp <= end);
        }
    }

    #[tokio::test]
    async fn test_get_source_lineage_filtered_pagination() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record 10 lineage records
        for i in 0..10 {
            let lineage =
                SourceLineageRecord::new("splunk", format!("logs_{}", i), "network_activity")
                    .with_ingestion_timestamp(now - Duration::hours(i as i64));
            index.record_source_lineage(lineage).await.unwrap();
        }

        let start = now - Duration::hours(20);
        let end = now + Duration::hours(1);

        // First page
        let page1 = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 3)
            .await
            .unwrap();
        assert_eq!(page1.edges.len(), 3);
        assert_eq!(page1.total_count, 10);
        assert!(page1.has_more);

        // Second page
        let page2 = index
            .get_source_lineage_filtered("network_activity", start, end, 3, 3)
            .await
            .unwrap();
        assert_eq!(page2.edges.len(), 3);
        assert_eq!(page2.total_count, 10);
        assert!(page2.has_more);

        // Third page
        let page3 = index
            .get_source_lineage_filtered("network_activity", start, end, 6, 3)
            .await
            .unwrap();
        assert_eq!(page3.edges.len(), 3);
        assert_eq!(page3.total_count, 10);
        assert!(page3.has_more);

        // Fourth page (last)
        let page4 = index
            .get_source_lineage_filtered("network_activity", start, end, 9, 3)
            .await
            .unwrap();
        assert_eq!(page4.edges.len(), 1);
        assert_eq!(page4.total_count, 10);
        assert!(!page4.has_more);
    }

    #[tokio::test]
    async fn test_get_source_lineage_filtered_empty_result() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record lineage for a different table
        let lineage = SourceLineageRecord::new("splunk", "logs", "process_activity")
            .with_ingestion_timestamp(now);
        index.record_source_lineage(lineage).await.unwrap();

        // Query for network_activity (no records)
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);
        let result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_get_source_lineage_filtered_time_range_no_match() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record lineage at current time
        let lineage = SourceLineageRecord::new("splunk", "logs", "network_activity")
            .with_ingestion_timestamp(now);
        index.record_source_lineage(lineage).await.unwrap();

        // Query for a time range that doesn't include the record
        let start = now - Duration::hours(10);
        let end = now - Duration::hours(5);
        let result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        assert!(result.edges.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_get_source_lineage_filtered_ordered_by_timestamp() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record lineage in non-chronological order
        let lineage3 = SourceLineageRecord::new("splunk", "logs_3", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(3));
        let lineage1 = SourceLineageRecord::new("splunk", "logs_1", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(1));
        let lineage2 = SourceLineageRecord::new("splunk", "logs_2", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(2));

        index.record_source_lineage(lineage3).await.unwrap();
        index.record_source_lineage(lineage1).await.unwrap();
        index.record_source_lineage(lineage2).await.unwrap();

        let start = now - Duration::hours(10);
        let end = now + Duration::hours(1);
        let result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        // Should be ordered by timestamp ascending
        assert_eq!(result.edges.len(), 3);
        assert!(result.edges[0].timestamp <= result.edges[1].timestamp);
        assert!(result.edges[1].timestamp <= result.edges[2].timestamp);
    }

    #[tokio::test]
    async fn test_get_field_lineage_filtered_basic() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record source lineage first
        let source_lineage = SourceLineageRecord::new("splunk", "raw_logs", "network_activity")
            .with_ingestion_timestamp(now);
        let lineage_id = index.record_source_lineage(source_lineage).await.unwrap();

        // Record field lineage
        let field_lineage = FieldLineageRecord::new(lineage_id, "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");
        index.record_field_lineage(field_lineage).await.unwrap();

        // Query with time range
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);
        let result = index
            .get_field_lineage_filtered("src_endpoint.ip", start, end, 0, 10)
            .await
            .unwrap();

        assert_eq!(result.mappings.len(), 1);
        assert_eq!(result.total_count, 1);
        assert!(!result.has_more);
        assert_eq!(result.mappings[0].source_path, "src_ip");
        assert_eq!(result.mappings[0].target_path, "src_endpoint.ip");
        assert_eq!(
            result.mappings[0].transformation,
            Some("LOWER(src_ip)".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_field_lineage_filtered_time_range() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record source lineage at different times
        let source1 = SourceLineageRecord::new("splunk", "logs_old", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(10));
        let id1 = index.record_source_lineage(source1).await.unwrap();

        let source2 = SourceLineageRecord::new("splunk", "logs_recent", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(1));
        let id2 = index.record_source_lineage(source2).await.unwrap();

        // Record field lineage for both
        let field1 = FieldLineageRecord::new(id1, "old_ip", "src_endpoint.ip");
        let field2 = FieldLineageRecord::new(id2, "new_ip", "src_endpoint.ip");
        index.record_field_lineage(field1).await.unwrap();
        index.record_field_lineage(field2).await.unwrap();

        // Query with time range that only includes recent record
        let start = now - Duration::hours(2);
        let end = now + Duration::hours(1);
        let result = index
            .get_field_lineage_filtered("src_endpoint.ip", start, end, 0, 10)
            .await
            .unwrap();

        assert_eq!(result.mappings.len(), 1);
        assert_eq!(result.mappings[0].source_path, "new_ip");
    }

    #[tokio::test]
    async fn test_get_field_lineage_filtered_pagination() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Record source lineage
        let source = SourceLineageRecord::new("splunk", "logs", "network_activity")
            .with_ingestion_timestamp(now);
        let lineage_id = index.record_source_lineage(source).await.unwrap();

        // Record multiple field lineages
        for i in 0..10 {
            let field = FieldLineageRecord::new(lineage_id, format!("field_{}", i), "target_field");
            index.record_field_lineage(field).await.unwrap();
        }

        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);

        // First page
        let page1 = index
            .get_field_lineage_filtered("target_field", start, end, 0, 3)
            .await
            .unwrap();
        assert_eq!(page1.mappings.len(), 3);
        assert_eq!(page1.total_count, 10);
        assert!(page1.has_more);

        // Last page
        let page4 = index
            .get_field_lineage_filtered("target_field", start, end, 9, 3)
            .await
            .unwrap();
        assert_eq!(page4.mappings.len(), 1);
        assert_eq!(page4.total_count, 10);
        assert!(!page4.has_more);
    }

    #[tokio::test]
    async fn test_get_field_lineage_filtered_empty_result() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // Query for a field with no lineage
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);
        let result = index
            .get_field_lineage_filtered("nonexistent.field", start, end, 0, 10)
            .await
            .unwrap();

        assert!(result.mappings.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_filtered_lineage_workflow() {
        let backend = InMemoryBackend::new();
        let index = SemanticIndex::new(backend, IndexConfig::default())
            .await
            .unwrap();

        let now = Utc::now();

        // 1. Record source lineage at different times
        let source1 = SourceLineageRecord::new("splunk", "old_logs", "network_activity")
            .with_ingestion_timestamp(now - Duration::days(7))
            .with_record_count(1000);
        let id1 = index.record_source_lineage(source1).await.unwrap();

        let source2 = SourceLineageRecord::new("kafka", "recent_events", "network_activity")
            .with_ingestion_timestamp(now - Duration::hours(1))
            .with_record_count(500);
        let id2 = index.record_source_lineage(source2).await.unwrap();

        // 2. Record field lineage for both sources
        let field1 = FieldLineageRecord::new(id1, "old_src_ip", "src_endpoint.ip");
        let field2 = FieldLineageRecord::new(id2, "new_src_ip", "src_endpoint.ip");
        index.record_field_lineage(field1).await.unwrap();
        index.record_field_lineage(field2).await.unwrap();

        // 3. Query source lineage for last 24 hours
        let start = now - Duration::hours(24);
        let end = now + Duration::hours(1);
        let source_result = index
            .get_source_lineage_filtered("network_activity", start, end, 0, 10)
            .await
            .unwrap();

        assert_eq!(source_result.total_count, 1);
        assert_eq!(source_result.edges[0].source, "kafka:recent_events");

        // 4. Query field lineage for last 24 hours
        let field_result = index
            .get_field_lineage_filtered("src_endpoint.ip", start, end, 0, 10)
            .await
            .unwrap();

        assert_eq!(field_result.total_count, 1);
        assert_eq!(field_result.mappings[0].source_path, "new_src_ip");

        // 5. Query for all time (should include both)
        let all_start = now - Duration::days(30);
        let all_source_result = index
            .get_source_lineage_filtered("network_activity", all_start, end, 0, 10)
            .await
            .unwrap();
        assert_eq!(all_source_result.total_count, 2);

        let all_field_result = index
            .get_field_lineage_filtered("src_endpoint.ip", all_start, end, 0, 10)
            .await
            .unwrap();
        assert_eq!(all_field_result.total_count, 2);
    }
}
