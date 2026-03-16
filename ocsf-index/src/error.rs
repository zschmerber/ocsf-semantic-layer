//! Error types for the OCSF Semantic Index.
//!
//! This module provides comprehensive error types for all index operations,
//! following the project's error handling conventions with `thiserror`.

use thiserror::Error;

use crate::types::{LineageId, TableId};

/// Index error types.
#[derive(Debug, Error)]
pub enum IndexError {
    /// Table not found in the registry.
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Invalid class UID provided.
    #[error("Invalid class UID: {0}. Must be a positive integer.")]
    InvalidClassUid(u32),

    /// Lineage record not found.
    #[error("Lineage record not found: {0}")]
    LineageNotFound(LineageId),

    /// Partition not found.
    #[error("Partition not found: {table_name}/{partition_key}")]
    PartitionNotFound {
        table_name: String,
        partition_key: String,
    },

    /// Statistics not found for table.
    #[error("Statistics not found for table: {0}")]
    StatisticsNotFound(String),

    /// Cache key not found.
    #[error("Cache key not found: {0}")]
    CacheKeyNotFound(String),

    /// Backend storage error.
    #[error("Backend error: {0}")]
    BackendError(#[from] BackendError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid time range provided.
    #[error("Invalid time range: start ({start}) must be before end ({end})")]
    InvalidTimeRange { start: String, end: String },

    /// Duplicate table registration.
    #[error("Duplicate table registration: {0}")]
    DuplicateTable(String),

    /// Referential integrity violation.
    #[error("Referential integrity violation: {0}")]
    ReferentialIntegrity(String),

    /// Table ID not found.
    #[error("Table ID not found: {0}")]
    TableIdNotFound(TableId),
}

/// Backend-specific error types.
#[derive(Debug, Error)]
pub enum BackendError {
    /// Connection error.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Query execution error.
    #[error("Query error: {0}")]
    QueryError(String),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// SQLite-specific error.
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    /// PostgreSQL-specific error.
    #[error("PostgreSQL error: {0}")]
    PostgresError(String),

    /// Record not found.
    #[error("Record not found: {0}")]
    RecordNotFound(String),

    /// Serialization error within backend.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type alias for index operations.
pub type IndexResult<T> = Result<T, IndexError>;

impl IndexError {
    /// Creates a table not found error.
    pub fn table_not_found(name: impl Into<String>) -> Self {
        Self::TableNotFound(name.into())
    }

    /// Creates an invalid class UID error.
    pub fn invalid_class_uid(uid: u32) -> Self {
        Self::InvalidClassUid(uid)
    }

    /// Creates a lineage not found error.
    pub fn lineage_not_found(id: LineageId) -> Self {
        Self::LineageNotFound(id)
    }

    /// Creates a partition not found error.
    pub fn partition_not_found(
        table_name: impl Into<String>,
        partition_key: impl Into<String>,
    ) -> Self {
        Self::PartitionNotFound {
            table_name: table_name.into(),
            partition_key: partition_key.into(),
        }
    }

    /// Creates a statistics not found error.
    pub fn statistics_not_found(table_name: impl Into<String>) -> Self {
        Self::StatisticsNotFound(table_name.into())
    }

    /// Creates a cache key not found error.
    pub fn cache_key_not_found(key: impl Into<String>) -> Self {
        Self::CacheKeyNotFound(key.into())
    }

    /// Creates an invalid time range error.
    pub fn invalid_time_range(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self::InvalidTimeRange {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Creates a duplicate table error.
    pub fn duplicate_table(name: impl Into<String>) -> Self {
        Self::DuplicateTable(name.into())
    }

    /// Creates a referential integrity error.
    pub fn referential_integrity(message: impl Into<String>) -> Self {
        Self::ReferentialIntegrity(message.into())
    }
}

impl BackendError {
    /// Creates a connection error.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::ConnectionError(message.into())
    }

    /// Creates a query error.
    pub fn query(message: impl Into<String>) -> Self {
        Self::QueryError(message.into())
    }

    /// Creates a transaction error.
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::TransactionError(message.into())
    }

    /// Creates a PostgreSQL error.
    pub fn postgres(message: impl Into<String>) -> Self {
        Self::PostgresError(message.into())
    }

    /// Creates a record not found error.
    pub fn record_not_found(message: impl Into<String>) -> Self {
        Self::RecordNotFound(message.into())
    }

    /// Creates a serialization error.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }
}
