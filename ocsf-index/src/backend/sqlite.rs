//! SQLite backend implementation for embedded deployments.
//!
//! This backend uses SQLite for persistent storage, suitable for
//! single-node deployments and development environments.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use super::{BackendResult, IndexBackend, IndexRecord, RecordFilter};
use crate::error::BackendError;
use crate::types::RecordId;

/// SQLite storage backend.
///
/// Uses a single SQLite connection with mutex for thread safety.
/// For production use with high concurrency, consider using a connection pool.
pub struct SqliteBackend {
    /// SQLite connection wrapped in mutex for thread safety.
    conn: Mutex<Connection>,
}

impl SqliteBackend {
    /// Creates a new SQLite backend with an in-memory database.
    pub fn new_in_memory() -> BackendResult<Self> {
        let conn = Connection::open_in_memory().map_err(BackendError::from)?;

        let backend = Self {
            conn: Mutex::new(conn),
        };

        backend.initialize_schema()?;
        Ok(backend)
    }

    /// Creates a new SQLite backend with a file-based database.
    pub fn new_file(path: impl AsRef<Path>) -> BackendResult<Self> {
        let conn = Connection::open(path.as_ref()).map_err(BackendError::from)?;

        let backend = Self {
            conn: Mutex::new(conn),
        };

        backend.initialize_schema()?;
        Ok(backend)
    }

    /// Initializes the database schema.
    fn initialize_schema(&self) -> BackendResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                record_type TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_records_type ON records(record_type);
            "#,
        )
        .map_err(BackendError::from)?;

        Ok(())
    }
}

impl IndexBackend for SqliteBackend {
    async fn create<T: IndexRecord>(&self, record: &T) -> BackendResult<RecordId> {
        let record_type = T::record_type();
        let json = serde_json::to_string(record)
            .map_err(|e| BackendError::serialization(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO records (record_type, data) VALUES (?1, ?2)",
            [record_type, &json],
        )
        .map_err(BackendError::from)?;

        let id = conn.last_insert_rowid() as u64;
        Ok(RecordId(id))
    }

    async fn read<T: IndexRecord>(&self, id: RecordId) -> BackendResult<Option<T>> {
        let record_type = T::record_type();
        let conn = self.conn.lock().unwrap();

        let result: Result<String, _> = conn.query_row(
            "SELECT data FROM records WHERE id = ?1 AND record_type = ?2",
            [&id.0.to_string(), record_type],
            |row| row.get(0),
        );

        match result {
            Ok(json) => {
                let mut record: T = serde_json::from_str(&json)
                    .map_err(|e| BackendError::serialization(e.to_string()))?;
                record.set_id(id);
                Ok(Some(record))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BackendError::from(e)),
        }
    }

    async fn update<T: IndexRecord>(&self, id: RecordId, record: &T) -> BackendResult<()> {
        let record_type = T::record_type();
        let json = serde_json::to_string(record)
            .map_err(|e| BackendError::serialization(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        let rows_affected = conn
            .execute(
                "UPDATE records SET data = ?1 WHERE id = ?2 AND record_type = ?3",
                [&json, &id.0.to_string(), record_type],
            )
            .map_err(BackendError::from)?;

        if rows_affected == 0 {
            return Err(BackendError::record_not_found(format!(
                "{}:{}",
                record_type, id.0
            )));
        }

        Ok(())
    }

    async fn delete(&self, record_type: &str, id: RecordId) -> BackendResult<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn
            .execute(
                "DELETE FROM records WHERE id = ?1 AND record_type = ?2",
                [&id.0.to_string(), record_type],
            )
            .map_err(BackendError::from)?;

        if rows_affected == 0 {
            return Err(BackendError::record_not_found(format!(
                "{}:{}",
                record_type, id.0
            )));
        }

        Ok(())
    }

    async fn list<T: IndexRecord>(&self, filter: &RecordFilter) -> BackendResult<Vec<T>> {
        let record_type = T::record_type();
        let conn = self.conn.lock().unwrap();

        let limit = filter.limit.unwrap_or(1000);
        let offset = filter.offset.unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT id, data FROM records WHERE record_type = ?1 ORDER BY id LIMIT ?2 OFFSET ?3",
            )
            .map_err(BackendError::from)?;

        let records = stmt
            .query_map(
                [record_type, &limit.to_string(), &offset.to_string()],
                |row| {
                    let id: i64 = row.get(0)?;
                    let json: String = row.get(1)?;
                    Ok((id, json))
                },
            )
            .map_err(BackendError::from)?
            .filter_map(|r| r.ok())
            .filter_map(|(id, json)| {
                let mut record: T = serde_json::from_str(&json).ok()?;
                record.set_id(RecordId(id as u64));
                Some(record)
            })
            .collect();

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
        let conn = self.conn.lock().unwrap();
        let mut deleted = 0u64;

        for id in ids {
            let rows_affected = conn
                .execute(
                    "DELETE FROM records WHERE id = ?1 AND record_type = ?2",
                    [&id.0.to_string(), record_type],
                )
                .map_err(BackendError::from)?;
            deleted += rows_affected as u64;
        }

        Ok(deleted)
    }

    async fn count<T: IndexRecord>(&self, _filter: &RecordFilter) -> BackendResult<u64> {
        let record_type = T::record_type();
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM records WHERE record_type = ?1",
                [record_type],
                |row| row.get(0),
            )
            .map_err(BackendError::from)?;

        Ok(count as u64)
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
    async fn test_sqlite_create_and_read() {
        let backend = SqliteBackend::new_in_memory().unwrap();
        let record = TestRecord {
            id: None,
            name: "test".to_string(),
            value: 42,
        };

        let id = backend.create(&record).await.unwrap();

        let retrieved: Option<TestRecord> = backend.read(id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.value, 42);
    }

    #[tokio::test]
    async fn test_sqlite_update() {
        let backend = SqliteBackend::new_in_memory().unwrap();
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
    async fn test_sqlite_delete() {
        let backend = SqliteBackend::new_in_memory().unwrap();
        let record = TestRecord {
            id: None,
            name: "to_delete".to_string(),
            value: 0,
        };

        let id = backend.create(&record).await.unwrap();

        backend.delete("test_record", id).await.unwrap();

        let retrieved: Option<TestRecord> = backend.read(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_sqlite_list() {
        let backend = SqliteBackend::new_in_memory().unwrap();

        for i in 0..5 {
            let record = TestRecord {
                id: None,
                name: format!("record_{}", i),
                value: i,
            };
            backend.create(&record).await.unwrap();
        }

        let filter = RecordFilter::new().with_limit(3);
        let records: Vec<TestRecord> = backend.list(&filter).await.unwrap();

        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_sqlite_count() {
        let backend = SqliteBackend::new_in_memory().unwrap();

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
