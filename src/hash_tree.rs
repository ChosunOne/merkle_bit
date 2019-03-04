use std::path::PathBuf;

use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::hashmap::HashDB;
use crate::tree_hasher::TreeHasher;
use crate::tree_hasher::TreeHashResult;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};

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