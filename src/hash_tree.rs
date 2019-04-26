use std::path::PathBuf;

#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::constants::KEY_LEN;
use crate::traits::{Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::HashTreeDB;
use crate::tree_hasher::TreeHasher;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};

pub struct HashTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
{
    tree: MerkleBIT<HashTreeDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, ValueType>,
}

impl<ValueType> HashTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
{
    pub fn new(depth: usize) -> BinaryMerkleTreeResult<Self> {
        let path = PathBuf::new();
        let tree = MerkleBIT::new(&path, depth)?;
        Ok(Self { tree })
    }

    pub fn open(_path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::new(&_path, depth)?;
        Ok(Self { tree })
    }

    pub fn get<'a>(
        &self,
        root_hash: &[u8; KEY_LEN],
        keys: &mut [&'a [u8; KEY_LEN]],
    ) -> BinaryMerkleTreeResult<HashMap<&'a [u8; KEY_LEN], Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(
        &mut self,
        previous_root: Option<&[u8; KEY_LEN]>,
        keys: &mut [&[u8; KEY_LEN]],
        values: &mut [&ValueType],
    ) -> BinaryMerkleTreeResult<[u8; KEY_LEN]> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8; KEY_LEN]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}
