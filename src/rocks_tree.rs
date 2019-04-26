use std::path::PathBuf;

#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::constants::KEY_LEN;
use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::{Database, Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::rocksdb::RocksDB;
use crate::tree_hasher::TreeHasher;

pub struct RocksTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
{
    tree: MerkleBIT<RocksDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, ValueType>,
}

impl<ValueType> RocksTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
{
    pub fn open(path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = RocksDB::open(path)?;
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(RocksTree { tree })
    }

    pub fn from_db(db: RocksDB, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(RocksTree { tree })
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
