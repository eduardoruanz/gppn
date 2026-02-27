//! RocksDB storage backend for the Veritas node.

use anyhow::Result;
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::path::Path;

/// Column family names for different data types.
const CF_CREDENTIALS: &str = "credentials";
const CF_SCHEMAS: &str = "schemas";
const CF_IDENTITY: &str = "identity";
const CF_STATE: &str = "state";
const CF_PEERS: &str = "peers";
const CF_PROOFS: &str = "proofs";

/// RocksDB-backed storage for the Veritas node.
pub struct Storage {
    db: DB,
}

impl Storage {
    /// Open or create a RocksDB database at the given path with column families.
    pub fn open(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path)?;

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new(CF_CREDENTIALS, Options::default()),
            ColumnFamilyDescriptor::new(CF_SCHEMAS, Options::default()),
            ColumnFamilyDescriptor::new(CF_IDENTITY, Options::default()),
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
            ColumnFamilyDescriptor::new(CF_PEERS, Options::default()),
            ColumnFamilyDescriptor::new(CF_PROOFS, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;

        Ok(Self { db })
    }

    /// Put a value into a column family.
    pub fn put(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| anyhow::anyhow!("column family '{}' not found", cf_name))?;
        self.db.put_cf(&cf, key, value)?;
        Ok(())
    }

    /// Get a value from a column family.
    pub fn get(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| anyhow::anyhow!("column family '{}' not found", cf_name))?;
        let value = self.db.get_cf(&cf, key)?;
        Ok(value)
    }

    /// Delete a key from a column family.
    pub fn delete(&self, cf_name: &str, key: &[u8]) -> Result<()> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| anyhow::anyhow!("column family '{}' not found", cf_name))?;
        self.db.delete_cf(&cf, key)?;
        Ok(())
    }

    /// Store a credential record.
    pub fn put_credential(&self, id: &str, data: &[u8]) -> Result<()> {
        self.put(CF_CREDENTIALS, id.as_bytes(), data)
    }

    /// Get a credential record.
    pub fn get_credential(&self, id: &str) -> Result<Option<Vec<u8>>> {
        self.get(CF_CREDENTIALS, id.as_bytes())
    }

    /// Store a proof record.
    pub fn put_proof(&self, id: &str, data: &[u8]) -> Result<()> {
        self.put(CF_PROOFS, id.as_bytes(), data)
    }

    /// Get a proof record.
    pub fn get_proof(&self, id: &str) -> Result<Option<Vec<u8>>> {
        self.get(CF_PROOFS, id.as_bytes())
    }

    /// Store peer information.
    pub fn put_peer(&self, peer_id: &str, data: &[u8]) -> Result<()> {
        self.put(CF_PEERS, peer_id.as_bytes(), data)
    }

    /// Get peer information.
    pub fn get_peer(&self, peer_id: &str) -> Result<Option<Vec<u8>>> {
        self.get(CF_PEERS, peer_id.as_bytes())
    }

    /// Store node state.
    pub fn put_state(&self, key: &str, data: &[u8]) -> Result<()> {
        self.put(CF_STATE, key.as_bytes(), data)
    }

    /// Get node state.
    pub fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.get(CF_STATE, key.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("veritas-test-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_open_storage() {
        let dir = temp_dir();
        let storage = Storage::open(&dir);
        assert!(storage.is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_put_get_credential() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_credential("vc-001", b"test data").unwrap();
        let result = storage.get_credential("vc-001").unwrap();
        assert_eq!(result, Some(b"test data".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_get_nonexistent() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        let result = storage.get_credential("nonexistent").unwrap();
        assert!(result.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_delete() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_credential("vc-002", b"delete me").unwrap();
        storage.delete("credentials", b"vc-002").unwrap();
        let result = storage.get_credential("vc-002").unwrap();
        assert!(result.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_put_get_proof() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_proof("proof-001", b"proof data").unwrap();
        let result = storage.get_proof("proof-001").unwrap();
        assert_eq!(result, Some(b"proof data".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_put_get_peer() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_peer("peer-abc", b"peer info").unwrap();
        let result = storage.get_peer("peer-abc").unwrap();
        assert_eq!(result, Some(b"peer info".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_put_get_state() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage
            .put_state("node_did", b"did:veritas:key:abc")
            .unwrap();
        let result = storage.get_state("node_did").unwrap();
        assert_eq!(result, Some(b"did:veritas:key:abc".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }
}
