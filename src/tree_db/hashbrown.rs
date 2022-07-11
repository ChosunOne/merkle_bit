use std::path::Path;

use hashbrown::HashMap;

use crate::traits::{Database, Exception};
use crate::tree::tree_node::TreeNode;
use crate::Array;

pub struct HashDB<const N: usize> {
    map: HashMap<Array<N>, TreeNode<N>>,
}

impl<const N: usize> HashDB<N> {
    #[inline]
    pub fn new(map: HashMap<Array<N>, TreeNode<N>>) -> Self {
        Self { map }
    }
    #[inline]
    #[must_use]
    pub fn decompose(self) -> HashMap<Array<N>, TreeNode<N>> {
        self.map
    }
}

impl<const N: usize> Database<N, TreeNode<N>> for HashDB<N> {
    type EntryType = (Vec<u8>, TreeNode<N>);

    #[inline]
    fn open(_path: &Path) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: Array<N>) -> Result<Option<TreeNode<N>>, Exception> {
        if let Some(m) = self.map.get(&key) {
            let node = m.clone();
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn insert(&mut self, key: Array<N>, value: TreeNode<N>) -> Result<(), Exception> {
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
