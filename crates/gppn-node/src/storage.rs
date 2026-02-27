//! RocksDB storage backend for the GPPN node.

use anyhow::Result;
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::path::Path;

/// Column family names for different data types.
const CF_PAYMENTS: &str = "payments";
const CF_ROUTING: &str = "routing";
const CF_IDENTITY: &str = "identity";
const CF_STATE: &str = "state";
const CF_PEERS: &str = "peers";

/// RocksDB-backed storage for the GPPN node.
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
            ColumnFamilyDescriptor::new(CF_PAYMENTS, Options::default()),
            ColumnFamilyDescriptor::new(CF_ROUTING, Options::default()),
            ColumnFamilyDescriptor::new(CF_IDENTITY, Options::default()),
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
            ColumnFamilyDescriptor::new(CF_PEERS, Options::default()),
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

    /// Store a payment record.
    pub fn put_payment(&self, id: &str, data: &[u8]) -> Result<()> {
        self.put(CF_PAYMENTS, id.as_bytes(), data)
    }

    /// Get a payment record.
    pub fn get_payment(&self, id: &str) -> Result<Option<Vec<u8>>> {
        self.get(CF_PAYMENTS, id.as_bytes())
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
        let dir = std::env::temp_dir().join(format!("gppn-test-{}", uuid::Uuid::now_v7()));
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
    fn test_put_get_payment() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_payment("pm-001", b"test data").unwrap();
        let result = storage.get_payment("pm-001").unwrap();
        assert_eq!(result, Some(b"test data".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_get_nonexistent() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        let result = storage.get_payment("nonexistent").unwrap();
        assert!(result.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_delete() {
        let dir = temp_dir();
        let storage = Storage::open(&dir).unwrap();

        storage.put_payment("pm-002", b"delete me").unwrap();
        storage.delete("payments", b"pm-002").unwrap();
        let result = storage.get_payment("pm-002").unwrap();
        assert!(result.is_none());

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

        storage.put_state("last_block", b"12345").unwrap();
        let result = storage.get_state("last_block").unwrap();
        assert_eq!(result, Some(b"12345".to_vec()));

        std::fs::remove_dir_all(&dir).ok();
    }
}
