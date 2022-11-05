use crate::Array;
use std::collections::hash_map::HashMap;
use std::path::Path;

use crate::traits::{Database, MerkleBitError};
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

    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    #[must_use]
    /// Decomposes the `HashDB` into its underlying `HashMap`.
    pub fn decompose(self) -> HashMap<Array<N>, TreeNode<N>> {
        self.map
    }
}

impl<const N: usize> Database<N, TreeNode<N>> for HashDB<N> {
    #[inline]
    fn open(_path: &Path) -> Result<Self, MerkleBitError> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: Array<N>) -> Result<Option<TreeNode<N>>, MerkleBitError> {
        self.map.get(&key).map_or(Ok(None), |m| {
            let node = m.clone();
            Ok(Some(node))
        })
    }

    #[inline]
    fn insert(&mut self, key: Array<N>, value: TreeNode<N>) -> Result<(), MerkleBitError> {
        self.map.insert(key, value);
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &Array<N>) -> Result<(), MerkleBitError> {
        self.map.remove(key);
        Ok(())
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), MerkleBitError> {
        Ok(())
    }
}
