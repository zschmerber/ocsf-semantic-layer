//! Sidecar file format: fixed-size header + bincode-encoded entry data section.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crc32fast::Hasher as Crc32Hasher;
use memmap2::Mmap;

use crate::error::{CatalogError, CatalogResult};
use crate::models::{CatalogEntry, CatalogEntryId};
use ocsf_vector::VectorStore;

/// Magic number identifying an OCSF sidecar file.
pub const MAGIC: [u8; 4] = *b"OCSF";
/// Current format version.
pub const FORMAT_VERSION: u32 = 1;
/// Fixed header size in bytes: 4 + 4 + 8 + 32 + 8 + 4 = 60.
pub const HEADER_SIZE: usize = 60;

/// Fixed-size 60-byte header for the sidecar binary file.
///
/// Layout (little-endian):
///   [0..4]   magic: b"OCSF"
///   [4..8]   format_version: u32
///   [8..16]  catalog_version: u64
///   [16..48] ocsf_version: [u8; 32]
///   [48..56] entry_count: u64
///   [56..60] data_checksum: u32
#[derive(Debug, Clone, PartialEq)]
pub struct SidecarHeader {
    /// Magic number: b"OCSF".
    pub magic: [u8; 4],
    /// Format version (currently 1).
    pub format_version: u32,
    /// Monotonically increasing catalog version.
    pub catalog_version: u64,
    /// Primary OCSF schema version string (up to 32 bytes, null-padded).
    pub ocsf_version: [u8; 32],
    /// Number of entries in the data section.
    pub entry_count: u64,
    /// CRC32 checksum of the data section.
    pub data_checksum: u32,
}

impl Default for SidecarHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl SidecarHeader {
    /// Create a new header with default values.
    pub fn new() -> Self {
        Self {
            magic: MAGIC,
            format_version: FORMAT_VERSION,
            catalog_version: 0,
            ocsf_version: [0u8; 32],
            entry_count: 0,
            data_checksum: 0,
        }
    }

    /// Serialize to exactly HEADER_SIZE bytes (little-endian).
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.format_version.to_le_bytes());
        buf[8..16].copy_from_slice(&self.catalog_version.to_le_bytes());
        buf[16..48].copy_from_slice(&self.ocsf_version);
        buf[48..56].copy_from_slice(&self.entry_count.to_le_bytes());
        buf[56..60].copy_from_slice(&self.data_checksum.to_le_bytes());
        buf
    }

    /// Deserialize from exactly HEADER_SIZE bytes.
    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> Self {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        let format_version = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let catalog_version = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        let mut ocsf_version = [0u8; 32];
        ocsf_version.copy_from_slice(&buf[16..48]);
        let entry_count = u64::from_le_bytes(buf[48..56].try_into().unwrap());
        let data_checksum = u32::from_le_bytes(buf[56..60].try_into().unwrap());
        Self {
            magic,
            format_version,
            catalog_version,
            ocsf_version,
            entry_count,
            data_checksum,
        }
    }
}

/// Compute CRC32 checksum of a byte slice.
pub fn compute_checksum(data: &[u8]) -> u32 {
    let mut hasher = Crc32Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// File-backed catalog using bincode serialization and memory-mapped reads.
pub struct SidecarCatalog {
    pub(crate) path: PathBuf,
    /// In-memory index: (entity_name, ocsf_version) -> CatalogEntryId
    pub(crate) index: RwLock<HashMap<(String, String), CatalogEntryId>>,
    /// In-memory entries: id -> CatalogEntry
    pub(crate) entries: RwLock<HashMap<u64, CatalogEntry>>,
    /// Current header state
    pub(crate) header: RwLock<SidecarHeader>,
    /// Next available ID
    pub(crate) next_id: RwLock<u64>,
    /// Vector store for semantic search (wrapped in Mutex for interior mutability)
    pub(crate) vector_store: tokio::sync::Mutex<ocsf_vector::InMemoryVectorStore>,
    /// Optional embedding generator
    pub(crate) embedding_generator: Option<ocsf_vector::AnyEmbeddingGenerator>,
}

impl SidecarCatalog {
    /// Create a new sidecar file at the given path.
    pub fn create(path: impl AsRef<Path>) -> CatalogResult<Self> {
        let path = path.as_ref().to_path_buf();
        let mut header = SidecarHeader::new();

        // Encode empty data section and compute its checksum before writing the header
        let empty_data: Vec<u8> = bincode::serialize(&Vec::<CatalogEntry>::new())
            .map_err(|e| CatalogError::Serialization(e.to_string()))?;
        header.data_checksum = compute_checksum(&empty_data);

        // Write header then data section
        let mut file = File::create(&path)?;
        file.write_all(&header.to_bytes())?;
        file.write_all(&empty_data)?;
        file.flush()?;

        Ok(Self {
            path,
            index: RwLock::new(HashMap::new()),
            entries: RwLock::new(HashMap::new()),
            header: RwLock::new(header),
            next_id: RwLock::new(1),
            vector_store: tokio::sync::Mutex::new(ocsf_vector::InMemoryVectorStore::new(384)),
            embedding_generator: None,
        })
    }

    /// Open an existing sidecar file, validating magic, version, and checksum.
    pub fn open(path: impl AsRef<Path>) -> CatalogResult<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;

        // Read header
        let mut header_buf = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_buf)?;
        let header = SidecarHeader::from_bytes(&header_buf);

        // Validate magic
        if header.magic != MAGIC {
            return Err(CatalogError::Corruption(
                "Invalid magic number — not an OCSF sidecar file".to_string(),
            ));
        }

        // Validate format version
        if header.format_version != FORMAT_VERSION {
            return Err(CatalogError::Corruption(format!(
                "Unsupported format version: {} (expected {})",
                header.format_version, FORMAT_VERSION
            )));
        }

        // Read data section
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Validate checksum
        let actual_checksum = compute_checksum(&data);
        if actual_checksum != header.data_checksum {
            return Err(CatalogError::Corruption(format!(
                "Checksum mismatch: expected {}, got {}",
                header.data_checksum, actual_checksum
            )));
        }

        // Deserialize entries
        let catalog_entries: Vec<CatalogEntry> = bincode::deserialize(&data)
            .map_err(|e| CatalogError::Serialization(e.to_string()))?;

        // Build in-memory index
        let mut index = HashMap::new();
        let mut entries_map = HashMap::new();
        let mut max_id = 0u64;

        for entry in catalog_entries {
            if let Some(id) = entry.id {
                index.insert((entry.entity_name.clone(), entry.ocsf_version.clone()), id);
                if id.0 > max_id {
                    max_id = id.0;
                }
                entries_map.insert(id.0, entry);
            }
        }

        Ok(Self {
            path,
            index: RwLock::new(index),
            entries: RwLock::new(entries_map),
            header: RwLock::new(header),
            next_id: RwLock::new(max_id + 1),
            vector_store: tokio::sync::Mutex::new(ocsf_vector::InMemoryVectorStore::new(384)),
            embedding_generator: None,
        })
    }

    /// Configure an embedding generator for semantic search.
    pub fn with_embedding_generator(mut self, gen: ocsf_vector::AnyEmbeddingGenerator) -> Self {
        self.embedding_generator = Some(gen);
        self
    }

    /// Generate and store an embedding for a catalog entry (no-op if no generator configured).
    pub(crate) async fn index_entry_embedding(&self, entry: &CatalogEntry) {
        let gen = match &self.embedding_generator {
            Some(g) => g,
            None => return,
        };
        let text = format!(
            "{} {} {}",
            entry.entity_name,
            entry.description,
            entry.attributes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(" ")
        );
        if let Ok(vector) = gen.embed(&text).await {
            use ocsf_vector::{Embedding, EmbeddingMetadata};
            let id_str = entry.id.map(|id| id.0.to_string()).unwrap_or_default();
            let metadata = EmbeddingMetadata::semantic_entity(
                entry.entity_name.clone(),
                entry.description.clone(),
            );
            let embedding = Embedding::new(id_str, vector, metadata);
            let mut store = self.vector_store.lock().await;
            let _ = store.store_one(embedding).await;
        }
    }

    /// Persist all entries to disk, updating header checksum and entry count.
    pub(crate) fn flush(&self) -> CatalogResult<()> {
        let entries = self.entries.read().unwrap();
        let mut header = self.header.write().unwrap();

        let all_entries: Vec<CatalogEntry> = entries.values().cloned().collect();
        let data = bincode::serialize(&all_entries)
            .map_err(|e| CatalogError::Serialization(e.to_string()))?;

        header.data_checksum = compute_checksum(&data);
        header.entry_count = entries.len() as u64;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        file.write_all(&header.to_bytes())?;
        file.write_all(&data)?;
        file.flush()?;

        Ok(())
    }

    /// Memory-mapped read of the full file (for read-only access to the data section).
    ///
    /// The data section starts at byte offset `HEADER_SIZE`.
    pub fn mmap_data(&self) -> CatalogResult<Mmap> {
        let file = File::open(&self.path)?;
        // SAFETY: the file is opened read-only; we do not mutate it during the mmap lifetime.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(mmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn header_round_trip() {
        let mut h = SidecarHeader::new();
        h.catalog_version = 42;
        h.entry_count = 7;
        h.data_checksum = 0xDEAD_BEEF;
        let bytes = h.to_bytes();
        let h2 = SidecarHeader::from_bytes(&bytes);
        assert_eq!(h, h2);
    }

    #[test]
    fn header_size_is_correct() {
        assert_eq!(HEADER_SIZE, 60);
        let h = SidecarHeader::new();
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);
    }

    #[test]
    fn create_and_open_empty_catalog() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        SidecarCatalog::create(path).unwrap();
        let catalog = SidecarCatalog::open(path).unwrap();

        let header = catalog.header.read().unwrap();
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.format_version, FORMAT_VERSION);
        assert_eq!(header.entry_count, 0);
    }

    #[test]
    fn open_rejects_bad_magic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Write a file with wrong magic
        let mut file = File::create(path).unwrap();
        let mut bad_header = SidecarHeader::new();
        bad_header.magic = *b"XXXX";
        file.write_all(&bad_header.to_bytes()).unwrap();
        // Write empty data section with correct checksum
        let empty_data = bincode::serialize(&Vec::<CatalogEntry>::new()).unwrap();
        file.write_all(&empty_data).unwrap();
        drop(file);

        let result = SidecarCatalog::open(path);
        assert!(matches!(result, Err(CatalogError::Corruption(_))));
    }

    #[test]
    fn open_rejects_bad_format_version() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let mut file = File::create(path).unwrap();
        let mut bad_header = SidecarHeader::new();
        bad_header.format_version = 99;
        file.write_all(&bad_header.to_bytes()).unwrap();
        let empty_data = bincode::serialize(&Vec::<CatalogEntry>::new()).unwrap();
        file.write_all(&empty_data).unwrap();
        drop(file);

        let result = SidecarCatalog::open(path);
        assert!(matches!(result, Err(CatalogError::Corruption(_))));
    }

    #[test]
    fn open_rejects_checksum_mismatch() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create a valid catalog first
        SidecarCatalog::create(path).unwrap();

        // Corrupt the data section by appending a byte
        let mut file = OpenOptions::new().append(true).open(path).unwrap();
        file.write_all(&[0xFF]).unwrap();
        drop(file);

        let result = SidecarCatalog::open(path);
        assert!(matches!(result, Err(CatalogError::Corruption(_))));
    }

    #[test]
    fn mmap_data_returns_valid_mapping() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        SidecarCatalog::create(path).unwrap();
        let catalog = SidecarCatalog::open(path).unwrap();
        let mmap = catalog.mmap_data().unwrap();

        // The mmap should at least contain the header
        assert!(mmap.len() >= HEADER_SIZE);
        // First 4 bytes should be the magic number
        assert_eq!(&mmap[0..4], b"OCSF");
    }
}
