//! OCSF Semantic Index Layer.
//!
//! This crate provides a metadata management system that bridges the ETL pipeline
//! and query engine. It tracks:
//!
//! - **Physical table locations**: Which tables exist for each OCSF event class
//! - **Source-to-OCSF lineage**: Which raw data sources contributed to each table
//! - **Field lineage**: How raw source fields map to OCSF fields with transformations
//! - **Partition metadata**: Time bounds and row counts for query optimization
//! - **Column statistics**: Cardinality and null rates for query planning
//! - **Query result caching**: TTL-based caching with LRU eviction
//!
//! # Architecture
//!
//! The index uses a trait-based storage abstraction (`IndexBackend`) that supports:
//! - SQLite for embedded and single-node deployments
//! - PostgreSQL for production deployments
//! - In-memory backend for testing
//!
//! # Example
//!
//! ```rust,ignore
//! use ocsf_index::{SemanticIndex, IndexConfig, TableEntry};
//! use ocsf_index::backend::InMemoryBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let backend = InMemoryBackend::new();
//!     let config = IndexConfig::default();
//!     let index = SemanticIndex::new(backend, config).await?;
//!
//!     // Register a table
//!     let table = TableEntry::new("network_activity", 4001)
//!         .with_schema("ocsf")
//!         .with_ocsf_version("1.3.0");
//!     let table_id = index.register_table(table).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod detection_coverage;
pub mod error;
pub mod field_lineage;
pub mod index;
pub mod lineage_capture;
pub mod lineage_query;
pub mod partition_metadata;
pub mod query_cache;
pub mod query_hints;
pub mod source_lineage;
pub mod table_registry;
pub mod table_statistics;
pub mod types;

// Backend implementations
pub mod backend;

// Re-export main types for convenience
pub use detection_coverage::{
    DataSourceCoverage, DetectionCoverageSummary, KillChainCoverage, MitreTacticCoverage,
    MitreTechniqueCoverage,
};
pub use error::{BackendError, IndexError, IndexResult};
pub use field_lineage::{FieldLineageRecord, FieldMapping};
pub use index::SemanticIndex;
pub use lineage_capture::LineageCapture;
pub use lineage_query::{FieldLineageQueryResult, LineageQueryResult};
pub use partition_metadata::PartitionEntry;
pub use query_cache::{CacheStats, CachedResult, QueryCacheKey};
pub use query_hints::{PartitionHint, QueryPlanHints, TableHint};
pub use source_lineage::{LineageEdge, SourceLineageRecord};
pub use table_registry::{DetectionCoverage, TableEntry};
pub use table_statistics::{ColumnStatistics, StatisticsId, TableStatistics};
pub use types::*;
