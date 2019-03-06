use std::path::PathBuf;

use crate::traits::{Encode, Decode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::HashTreeDB;
use crate::tree_hasher::TreeHasher;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};

pub struct HashTree<ValueType>
    where ValueType: Encode + Decode {
    tree: MerkleBIT<HashTreeDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, ValueType>
}

impl<ValueType> HashTree<ValueType>
    where ValueType: Encode + Decode {
    pub fn new(depth: usize) -> BinaryMerkleTreeResult<Self> {
        let path = PathBuf::new();
        let tree = MerkleBIT::new(&path, depth)?;
        Ok(Self {
            tree
        })
    }

    pub fn open(_path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::new(&_path, depth)?;
        Ok(Self {
            tree
        })
    }

    pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&[u8]], values: &mut [&ValueType]) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}