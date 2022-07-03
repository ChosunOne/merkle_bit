use crate::Array;
use std::collections::hash_map::HashMap;
use std::path::Path;

use crate::traits::{Database, Exception};
use crate::tree::tree_node::TreeNode;

/// A database consisting of a `HashMap`.
pub struct HashDB<const N: usize> {
    /// The internal `HashMap` for storing nodes.
    map: HashMap<Array<N>, TreeNode<N>>,
}

impl<const N: usize> HashDB<N> {
    /// Creates a new `HashDB`.
    #[inline]
    #[must_use]
    pub const fn new(map: HashMap<Array<N>, TreeNode<N>>) -> Self {
        Self { map }
    }
}

impl<const N: usize> Database<N> for HashDB<N> {
    type NodeType = TreeNode<N>;
    type EntryType = (Array<N>, Vec<u8>);

    #[inline]
    fn open(_path: &Path) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: Array<N>) -> Result<Option<Self::NodeType>, Exception> {
        self.map.get(&key).map_or(Ok(None), |m| {
            let node = m.clone();
            Ok(Some(node))
        })
    }

    #[inline]
    fn insert(&mut self, key: Array<N>, value: Self::NodeType) -> Result<(), Exception> {
        self.map.insert(key, value);
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &Array<N>) -> Result<(), Exception> {
        self.map.remove(key);
        Ok(())
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), Exception> {
        Ok(())
    }
}
