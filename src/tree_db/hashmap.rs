use std::collections::hash_map::HashMap;
use std::path::PathBuf;

use crate::constants::KEY_LEN;
use crate::traits::{Database, Exception};
use crate::tree::tree_node::TreeNode;

/// A database consisting of a `HashMap`.
pub struct HashDB {
    /// The internal `HashMap` for storing nodes.
    map: HashMap<[u8; KEY_LEN], TreeNode>,
}

impl HashDB {
    /// Creates a new `HashDB`.
    #[inline]
    pub const fn new(map: HashMap<[u8; KEY_LEN], TreeNode>) -> Self {
        Self { map }
    }
}

impl Database for HashDB {
    type NodeType = TreeNode;
    type EntryType = ([u8; KEY_LEN], Vec<u8>);

    #[inline]
    fn open(_path: &PathBuf) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: &[u8; KEY_LEN]) -> Result<Option<Self::NodeType>, Exception> {
        if let Some(m) = self.map.get(key) {
            let node = m.clone();
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    #[inline]
    fn insert(&mut self, key: [u8; KEY_LEN], value: Self::NodeType) -> Result<(), Exception> {
        self.map.insert(key, value);
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &[u8; KEY_LEN]) -> Result<(), Exception> {
        self.map.remove(key);
        Ok(())
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), Exception> {
        Ok(())
    }
}
