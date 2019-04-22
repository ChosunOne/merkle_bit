use std::collections::hash_map::HashMap;
use std::path::PathBuf;

use crate::constants::KEY_LEN;
use crate::traits::{Database, Exception};
use crate::tree::tree_node::TreeNode;

pub struct HashDB {
    map: HashMap<[u8; KEY_LEN], TreeNode>,
}

impl HashDB {
    pub fn new(map: HashMap<[u8; KEY_LEN], TreeNode>) -> Self {
        Self { map }
    }
}

impl Database for HashDB {
    type NodeType = TreeNode;
    type EntryType = ([u8; KEY_LEN], Vec<u8>);

    fn open(_path: &PathBuf) -> Result<Self, Exception> {
        Ok(Self::new(HashMap::new()))
    }

    fn get_node(&self, key: &[u8; KEY_LEN]) -> Result<Option<Self::NodeType>, Exception> {
        if let Some(m) = self.map.get(key) {
            let node = m.clone();
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    fn insert(&mut self, key: [u8; KEY_LEN], value: Self::NodeType) -> Result<(), Exception> {
        self.map.insert(key, value);
        Ok(())
    }

    fn remove(&mut self, key: &[u8; KEY_LEN]) -> Result<(), Exception> {
        self.map.remove(key);
        Ok(())
    }

    fn batch_write(&mut self) -> Result<(), Exception> {
        Ok(())
    }
}
