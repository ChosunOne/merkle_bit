use std::error::Error;
use std::path::PathBuf;

use crate::traits::{Database, Decode, Encode};
use crate::tree::tree_node::TreeNode;

use rocksdb::{WriteBatch, DB};

pub struct RocksDB {
    db: DB,
    pending_inserts: Option<WriteBatch>,
}

impl RocksDB {
    pub fn new(db: DB) -> Self {
        Self {
            db,
            pending_inserts: Some(WriteBatch::default()),
        }
    }
}

impl Database for RocksDB {
    type NodeType = TreeNode;
    type EntryType = (Vec<u8>, Vec<u8>);

    fn open(path: &PathBuf) -> Result<Self, Box<Error>> {
        Ok(RocksDB::new(DB::open_default(path)?))
    }

    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
        if let Some(buffer) = self.db.get(key)? {
            Ok(Some(Self::NodeType::decode(buffer.as_ref())?))
        } else {
            Ok(None)
        }
    }

    fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
        let serialized = value.encode()?;
        if let Some(wb) = &mut self.pending_inserts {
            wb.put(key, serialized)?;
        } else {
            let mut wb = WriteBatch::default();
            wb.put(key, serialized)?;
            self.pending_inserts = Some(wb);
        }
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        Ok(self.db.delete(key)?)
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        if let Some(wb) = self.pending_inserts.replace(WriteBatch::default()) {
            self.db.write(wb)?;
        }
        self.pending_inserts = None;
        Ok(())
    }
}
