use std::collections::hash_map::HashMap;
use std::path::Path;

use crate::constants::KEY_LEN;
use crate::traits::{Array, Database, Exception};
use crate::tree::tree_node::TreeNode;

/// A database consisting of a `HashMap`.
pub struct HashDB<ArrayType: Array> {
    /// The internal `HashMap` for storing nodes.
    map: HashMap<ArrayType, TreeNode<ArrayType>>,
}

impl<ArrayType: Array> HashDB<ArrayType> {
    /// Creates a new `HashDB`.
    #[inline]
    #[must_use]
    pub const fn new(map: HashMap<ArrayType, TreeNode<ArrayType>>) -> Self {
        Self { map }
    }
}

impl<ArrayType: Array> Database<ArrayType> for HashDB<ArrayType> {
    type NodeType = TreeNode<ArrayType>;
    type EntryType = ([u8; KEY_LEN], Vec<u8>);

    #[inline]
    fn open(_path: &Path) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: ArrayType) -> Result<Option<Self::NodeType>, Exception> {
        self.map.get(&key).map_or(Ok(None), |m| {
            let node = m.clone();
            Ok(Some(node))
        })
    }

    #[inline]
    fn insert(&mut self, key: ArrayType, value: Self::NodeType) -> Result<(), Exception> {
        self.map.insert(key, value);
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &ArrayType) -> Result<(), Exception> {
        self.map.remove(key);
        Ok(())
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), Exception> {
        Ok(())
    }
}
