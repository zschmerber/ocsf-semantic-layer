//! Storage backend implementations for the semantic index.
//!
//! This module provides the `IndexBackend` trait and implementations:
//!
//! - `InMemoryBackend`: For testing and development
//! - `SqliteBackend`: For embedded and single-node deployments
//! - `PostgresBackend`: For production deployments (future)
//!
//! # Example
//!
//! ```rust,ignore
//! use ocsf_index::backend::{IndexBackend, InMemoryBackend};
//!
//! let backend = InMemoryBackend::new();
//! // Use backend with SemanticIndex
//! ```

pub mod memory;
pub mod sqlite;

pub use memory::InMemoryBackend;
pub use sqlite::SqliteBackend;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::BackendError;
use crate::types::RecordId;

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, BackendError>;

/// Trait for records that can be stored in the index backend.
///
/// All index records must implement this trait to be stored and retrieved
/// from the backend storage.
pub trait IndexRecord: Serialize + DeserializeOwned + Send + Sync + Clone {
    /// Returns the record type identifier.
    fn record_type() -> &'static str;

    /// Returns the record's ID if it has been persisted.
    fn id(&self) -> Option<RecordId>;

    /// Sets the record's ID after persistence.
    fn set_id(&mut self, id: RecordId);
}

/// Filter criteria for listing records.
#[derive(Debug, Clone, Default)]
pub struct RecordFilter {
    /// Filter by record type.
    pub record_type: Option<String>,

    /// Filter by table name.
    pub table_name: Option<String>,

    /// Filter by time range (start, end).
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,

    /// Maximum number of records to return.
    pub limit: Option<usize>,

    /// Number of records to skip.
    pub offset: Option<usize>,
}

impl RecordFilter {
    /// Creates a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by table name.
    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table_name = Some(table.into());
        self
    }

    /// Filters by time range.
    pub fn with_time_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.time_range = Some((start, end));
        self
    }

    /// Sets pagination parameters.
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }

    /// Sets the limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Async trait for index storage backends.
///
/// This trait defines the storage operations that all backends must implement.
/// It provides generic CRUD operations for any type implementing `IndexRecord`.
#[allow(async_fn_in_trait)]
pub trait IndexBackend: Send + Sync {
    /// Creates a new record and returns its ID.
    async fn create<T: IndexRecord>(&self, record: &T) -> BackendResult<RecordId>;

    /// Reads a record by ID.
    async fn read<T: IndexRecord>(&self, id: RecordId) -> BackendResult<Option<T>>;

    /// Updates an existing record.
    async fn update<T: IndexRecord>(&self, id: RecordId, record: &T) -> BackendResult<()>;

    /// Deletes a record by ID.
    async fn delete(&self, record_type: &str, id: RecordId) -> BackendResult<()>;

    /// Lists records matching the filter criteria.
    async fn list<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<Vec<T>>;

    /// Creates multiple records in a batch.
    async fn batch_create<T: IndexRecord>(&self, records: &[T]) -> BackendResult<Vec<RecordId>>;

    /// Deletes multiple records by ID.
    async fn batch_delete(&self, record_type: &str, ids: &[RecordId]) -> BackendResult<u64>;

    /// Counts records matching the filter criteria.
    async fn count<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<u64>;
}
