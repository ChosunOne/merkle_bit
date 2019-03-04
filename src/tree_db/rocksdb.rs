use std::error::Error;
use std::path::PathBuf;

use crate::traits::{Database, Decode, Encode};
use crate::tree::tree_node::TreeNode;

use rocksdb::{DB, WriteBatch};

pub struct RocksDB {
    db: DB,
    pending_inserts: Vec<(Vec<u8>, Vec<u8>)>
}

impl RocksDB {
    pub fn new(db: DB) -> Self {
        Self {
            db,
            pending_inserts: Vec::with_capacity(64)
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
        self.pending_inserts.push((key.to_vec(), serialized));
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        Ok(self.db.delete(key)?)
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        let mut batch = WriteBatch::default();
        for pair in &self.pending_inserts {
            batch.put(&pair.0, &pair.1)?;
        }
        self.db.write(batch)?;
        self.pending_inserts.clear();
        Ok(())
    }
}