use std::collections::hash_map::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_node::TreeNode;
use crate::tree_hasher::TreeHasher;
use crate::tree_hasher::TreeHashResult;


use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::*;

#[cfg(not(any(feature = "use_hashbrown")))]
struct HashDB {
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

pub struct HashTree {
    tree: MerkleBIT<HashDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, TreeHashResult, Vec<u8>>
}

impl HashTree {
    pub fn new(depth: usize) -> Self {
        let path = PathBuf::new();
        let tree = MerkleBIT::new(&path, depth).expect("Creating a HashTree should not fail");
        Self {
            tree
        }
    }

    pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&[u8]], values: &mut Vec<&Vec<u8>>) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}