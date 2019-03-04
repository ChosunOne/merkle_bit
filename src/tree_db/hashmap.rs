use std::collections::hash_map::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::traits::Database;
use crate::tree::tree_node::TreeNode;

#[cfg(not(any(feature = "use_hashbrown")))]
pub struct HashDB {
    map: HashMap<Vec<u8>, TreeNode>
}

#[cfg(not(any(feature = "use_hashbrown")))]
impl HashDB {
    pub fn new(map: HashMap<Vec<u8>, TreeNode>) -> Self {
        Self {
            map
        }
    }
}

impl Database for HashDB {
    type NodeType = TreeNode;
    type EntryType = (Vec<u8>, TreeNode);

    fn open(_path: &PathBuf) -> Result<Self, Box<Error>> { Ok(Self::new(HashMap::new())) }

    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
        if let Some(m) = self.map.get(key) {
            let node = m.clone();
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
        self.map.insert(key.to_vec(), value.clone());
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        self.map.remove(key);
        Ok(())
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        Ok(())
    }
}