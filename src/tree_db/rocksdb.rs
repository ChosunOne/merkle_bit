use std::error::Error;
use std::path::Path;

use crate::traits::{Database, Decode, Encode, MerkleBitError};
use crate::tree::tree_node::TreeNode;
use crate::Array;
use rocksdb::{WriteBatch, DB};
use std::marker::PhantomData;

impl From<rocksdb::Error> for MerkleBitError {
    #[inline]
    fn from(error: rocksdb::Error) -> Self {
        Self::new(&error.to_string())
    }
}

pub struct RocksDB<const N: usize> {
    db: DB,
    pending_inserts: Option<WriteBatch>,
}

impl<const N: usize> RocksDB<N> {
    #[inline]
    pub fn new(db: DB) -> Self {
        Self {
            db,
            pending_inserts: Some(WriteBatch::default()),
        }
    }

    #[inline]
    pub fn decompose(self) -> DB {
        self.db
    }
}

impl<const N: usize> Database<N, TreeNode<N>> for RocksDB<N> {
    #[inline]
    fn open(path: &Path) -> Result<Self, MerkleBitError> {
        Ok(Self::new(DB::open_default(path)?))
    }

    #[inline]
    fn get_node(&self, key: Array<N>) -> Result<Option<TreeNode<N>>, MerkleBitError> {
        if let Some(buffer) = self.db.get(&key)? {
            Ok(Some(TreeNode::decode(buffer.as_ref())?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn insert(&mut self, key: Array<N>, value: TreeNode<N>) -> Result<(), MerkleBitError> {
        let serialized = value.encode()?;
        if let Some(wb) = &mut self.pending_inserts {
            wb.put(key, serialized);
        } else {
            let mut wb = WriteBatch::default();
            wb.put(key, serialized);
            self.pending_inserts = Some(wb);
        }
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &Array<N>) -> Result<(), MerkleBitError> {
        Ok(self.db.delete(key)?)
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), MerkleBitError> {
        if let Some(wb) = self.pending_inserts.replace(WriteBatch::default()) {
            self.db.write(wb)?;
        }
        self.pending_inserts = None;
        Ok(())
    }
}
