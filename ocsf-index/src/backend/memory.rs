//! In-memory backend implementation for testing and development.
//!
//! This backend stores all records in memory using thread-safe data structures.
//! It's primarily intended for testing and development scenarios.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use super::{BackendResult, IndexBackend, IndexRecord, RecordFilter};
use crate::error::BackendError;
use crate::types::RecordId;

/// In-memory storage backend.
///
/// Uses a HashMap with RwLock for thread-safe access. Records are stored
/// as JSON strings keyed by (record_type, record_id).
pub struct InMemoryBackend {
    /// Storage: (record_type, record_id) -> JSON string
    storage: RwLock<HashMap<(String, u64), String>>,
    /// Next ID counter
    next_id: AtomicU64,
}

impl InMemoryBackend {
    /// Creates a new in-memory backend.
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Returns the number of records stored.
    pub fn len(&self) -> usize {
        self.storage.read().unwrap().len()
    }

    /// Returns true if the backend is empty.
    pub fn is_empty(&self) -> bool {
        self.storage.read().unwrap().is_empty()
    }

    /// Clears all records from the backend.
    pub fn clear(&self) {
        self.storage.write().unwrap().clear();
    }

    /// Generates the next unique ID.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexBackend for InMemoryBackend {
    async fn create<T: IndexRecord>(&self, record: &T) -> BackendResult<RecordId> {
        let id = self.next_id();
        let record_type = T::record_type().to_string();

        // Clone and set ID
        let mut record = record.clone();
        record.set_id(RecordId(id));

        let json = serde_json::to_string(&record)
            .map_err(|e| BackendError::serialization(e.to_string()))?;

        self.storage
            .write()
            .unwrap()
            .insert((record_type, id), json);

        Ok(RecordId(id))
    }

    async fn read<T: IndexRecord>(&self, id: RecordId) -> BackendResult<Option<T>> {
        let record_type = T::record_type().to_string();
        let storage = self.storage.read().unwrap();

        match storage.get(&(record_type, id.0)) {
            Some(json) => {
                let record: T = serde_json::from_str(json)
                    .map_err(|e| BackendError::serialization(e.to_string()))?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    async fn update<T: IndexRecord>(&self, id: RecordId, record: &T) -> BackendResult<()> {
        let record_type = T::record_type().to_string();
        let key = (record_type.clone(), id.0);

        let mut storage = self.storage.write().unwrap();

        if !storage.contains_key(&key) {
            return Err(BackendError::record_not_found(format!(
                "{}:{}",
                record_type, id.0
            )));
        }

        // Clone and ensure ID is set
        let mut record = record.clone();
        record.set_id(id);

        let json = serde_json::to_string(&record)
            .map_err(|e| BackendError::serialization(e.to_string()))?;

        storage.insert(key, json);
        Ok(())
    }

    async fn delete(&self, record_type: &str, id: RecordId) -> BackendResult<()> {
        let key = (record_type.to_string(), id.0);
        let mut storage = self.storage.write().unwrap();

        if storage.remove(&key).is_none() {
            return Err(BackendError::record_not_found(format!(
                "{}:{}",
                record_type, id.0
            )));
        }

        Ok(())
    }

    async fn list<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<Vec<T>> {
        let record_type = T::record_type().to_string();
        let storage = self.storage.read().unwrap();

        let mut records: Vec<T> = storage
            .iter()
            .filter(|((rt, _), _)| rt == &record_type)
            .filter_map(|(_, json)| serde_json::from_str(json).ok())
            .collect();

        // Apply offset
        if let Some(offset) = filter.offset {
            if offset < records.len() {
                records = records.into_iter().skip(offset).collect();
            } else {
                records.clear();
            }
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            records.truncate(limit);
        }

        Ok(records)
    }

    async fn batch_create<T: IndexRecord>(&self, records: &[T]) -> BackendResult<Vec<RecordId>> {
        let mut ids = Vec::with_capacity(records.len());

        for record in records {
            let id = self.create(record).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    async fn batch_delete(&self, record_type: &str, ids: &[RecordId]) -> BackendResult<u64> {
        let mut storage = self.storage.write().unwrap();
        let mut deleted = 0u64;

        for id in ids {
            let key = (record_type.to_string(), id.0);
            if storage.remove(&key).is_some() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    async fn count<T: IndexRecord>(&self, _filter: &RecordFilter) -> BackendResult<u64> {
        let record_type = T::record_type().to_string();
        let storage = self.storage.read().unwrap();

        let count = storage.keys().filter(|(rt, _)| rt == &record_type).count() as u64;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestRecord {
        id: Option<RecordId>,
        name: String,
        value: i32,
    }

    impl IndexRecord for TestRecord {
        fn record_type() -> &'static str {
            "test_record"
        }

        fn id(&self) -> Option<RecordId> {
            self.id
        }

        fn set_id(&mut self, id: RecordId) {
            self.id = Some(id);
        }
    }

    #[tokio::test]
    async fn test_create_and_read() {
        let backend = InMemoryBackend::new();
        let record = TestRecord {
            id: None,
            name: "test".to_string(),
            value: 42,
        };

        let id = backend.create(&record).await.unwrap();
        assert_eq!(id.0, 1);

        let retrieved: Option<TestRecord> = backend.read(id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.value, 42);
        assert_eq!(retrieved.id, Some(id));
    }

    #[tokio::test]
    async fn test_update() {
        let backend = InMemoryBackend::new();
        let record = TestRecord {
            id: None,
            name: "original".to_string(),
            value: 1,
        };

        let id = backend.create(&record).await.unwrap();

        let updated = TestRecord {
            id: Some(id),
            name: "updated".to_string(),
            value: 2,
        };

        backend.update(id, &updated).await.unwrap();

        let retrieved: Option<TestRecord> = backend.read(id).await.unwrap();
        assert_eq!(retrieved.unwrap().name, "updated");
    }

    #[tokio::test]
    async fn test_delete() {
        let backend = InMemoryBackend::new();
        let record = TestRecord {
            id: None,
            name: "to_delete".to_string(),
            value: 0,
        };

        let id = backend.create(&record).await.unwrap();
        assert_eq!(backend.len(), 1);

        backend.delete("test_record", id).await.unwrap();
        assert_eq!(backend.len(), 0);

        let retrieved: Option<TestRecord> = backend.read(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_with_pagination() {
        let backend = InMemoryBackend::new();

        for i in 0..10 {
            let record = TestRecord {
                id: None,
                name: format!("record_{}", i),
                value: i,
            };
            backend.create(&record).await.unwrap();
        }

        let filter = RecordFilter::new().with_pagination(3, 2);
        let records: Vec<TestRecord> = backend.list(&filter).await.unwrap();

        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_batch_create() {
        let backend = InMemoryBackend::new();
        let records: Vec<TestRecord> = (0..5)
            .map(|i| TestRecord {
                id: None,
                name: format!("batch_{}", i),
                value: i,
            })
            .collect();

        let ids = backend.batch_create(&records).await.unwrap();
        assert_eq!(ids.len(), 5);
        assert_eq!(backend.len(), 5);
    }

    #[tokio::test]
    async fn test_batch_delete() {
        let backend = InMemoryBackend::new();
        let records: Vec<TestRecord> = (0..5)
            .map(|i| TestRecord {
                id: None,
                name: format!("batch_{}", i),
                value: i,
            })
            .collect();

        let ids = backend.batch_create(&records).await.unwrap();
        let deleted = backend
            .batch_delete("test_record", &ids[0..3])
            .await
            .unwrap();

        assert_eq!(deleted, 3);
        assert_eq!(backend.len(), 2);
    }

    #[tokio::test]
    async fn test_count() {
        let backend = InMemoryBackend::new();

        for i in 0..7 {
            let record = TestRecord {
                id: None,
                name: format!("record_{}", i),
                value: i,
            };
            backend.create(&record).await.unwrap();
        }

        let count = backend
            .count::<TestRecord>(&RecordFilter::new())
            .await
            .unwrap();
        assert_eq!(count, 7);
    }
}
