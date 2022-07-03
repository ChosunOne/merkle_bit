#[cfg(not(any(feature = "hashbrown")))]
use std::collections::HashMap;
use std::path::Path;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::{Database, Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::rocksdb::RocksDB;
use crate::tree_hasher::TreeHasher;
use crate::Array;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Internal type alias for the underlying tree.
type Tree<const N: usize, ValueType> = MerkleBIT<
    RocksDB<N>,
    TreeBranch<N>,
    TreeLeaf<N>,
    TreeData,
    TreeNode<N>,
    TreeHasher,
    ValueType,
    N,
>;

pub struct RocksTree<const N: usize = 32, ValueType: Encode + Decode = Vec<u8>> {
    tree: Tree<N, ValueType>,
}

impl<const N: usize, ValueType: Encode + Decode> RocksTree<N, ValueType> {
    #[inline]
    pub fn open(path: &Path, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = RocksDB::open(path)?;
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn from_db(db: RocksDB<N>, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn get(
        &self,
        root_hash: &Array<N>,
        keys: &mut [Array<N>],
    ) -> BinaryMerkleTreeResult<HashMap<Array<N>, Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    #[inline]
    pub fn get_one(
        &self,
        root: &Array<N>,
        key: &Array<N>,
    ) -> BinaryMerkleTreeResult<Option<ValueType>> {
        self.tree.get_one(&root, &key)
    }

    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&Array<N>>,
        keys: &mut [Array<N>],
        values: &[ValueType],
    ) -> BinaryMerkleTreeResult<Array<N>> {
        self.tree.insert(previous_root, keys, values)
    }

    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&Array<N>>,
        key: &Array<N>,
        value: &ValueType,
    ) -> BinaryMerkleTreeResult<Array<N>> {
        self.tree.insert_one(previous_root, key, value)
    }

    #[inline]
    pub fn remove(&mut self, root_hash: &Array<N>) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &Array<N>,
        key: Array<N>,
    ) -> BinaryMerkleTreeResult<Vec<(Array<N>, bool)>> {
        self.tree.generate_inclusion_proof(root, key)
    }

    #[inline]
    pub fn verify_inclusion_proof(
        root: &Array<N>,
        key: Array<N>,
        value: &ValueType,
        proof: &Vec<(Array<N>, bool)>,
    ) -> BinaryMerkleTreeResult<()> {
        Tree::verify_inclusion_proof(root, key, value, proof)
    }
}
