use std::error::Error;
use std::path::PathBuf;

use rocksdb::{DB, WriteBatch};

use crate::constants::KEY_LEN;
use crate::traits::{Database, Decode, Encode, Exception};
use crate::tree::tree_node::TreeNode;

impl From<rocksdb::Error> for Exception {
    fn from(error: rocksdb::Error) -> Self {
        Exception::new(error.description())
    }
}

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
    type EntryType = (usize, usize);

    fn open(path: &PathBuf) -> Result<Self, Exception> {
        Ok(RocksDB::new(DB::open_default(path)?))
    }

    fn get_node(&self, key: &[u8; KEY_LEN]) -> Result<Option<Self::NodeType>, Exception> {
        if let Some(buffer) = self.db.get(key)? {
            Ok(Some(Self::NodeType::decode(buffer.as_ref())?))
        } else {
            Ok(None)
        }
    }

    fn insert(&mut self, key: [u8; KEY_LEN], value: Self::NodeType) -> Result<(), Exception> {
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

    fn remove(&mut self, key: &[u8; KEY_LEN]) -> Result<(), Exception> {
        Ok(self.db.delete(key)?)
    }

    fn batch_write(&mut self) -> Result<(), Exception> {
        if let Some(wb) = self.pending_inserts.replace(WriteBatch::default()) {
            self.db.write(wb)?;
        }
        self.pending_inserts = None;
        Ok(())
    }
}
