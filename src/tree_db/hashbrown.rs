use std::path::PathBuf;

use hashbrown::HashMap;

use crate::traits::{Array, Database, Exception};
use crate::tree::tree_node::TreeNode;

pub struct HashDB<ArrayType: Array> {
    map: HashMap<ArrayType, TreeNode<ArrayType>>,
}

impl<ArrayType: Array> HashDB<ArrayType> {
    #[inline]
    pub fn new(map: HashMap<ArrayType, TreeNode<ArrayType>>) -> Self {
        Self { map }
    }
}

impl<ArrayType: Array> Database<ArrayType> for HashDB<ArrayType> {
    type NodeType = TreeNode<ArrayType>;
    type EntryType = (Vec<u8>, TreeNode<ArrayType>);

    #[inline]
    fn open(_path: &PathBuf) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    #[inline]
    fn get_node(&self, key: ArrayType) -> Result<Option<Self::NodeType>, Exception> {
        if let Some(m) = self.map.get(&key) {
            let node = m.clone();
            Ok(Some(node))
        } else {
            Ok(None)
        }
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
