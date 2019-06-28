#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::{Array, Database, Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::rocksdb::RocksDB;
use crate::tree_hasher::TreeHasher;
#[cfg(feature = "use_serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "use_serde")]
use serde::Serialize;

pub struct RocksTree<ArrayType = [u8; 32], ValueType = Vec<u8>>
where
    ArrayType: Array + Serialize + DeserializeOwned,
    ValueType: Encode + Decode,
{
    tree: MerkleBIT<
        RocksDB<ArrayType>,
        TreeBranch<ArrayType>,
        TreeLeaf<ArrayType>,
        TreeData,
        TreeNode<ArrayType>,
        TreeHasher,
        ValueType,
        ArrayType,
    >,
}

impl<ArrayType, ValueType> RocksTree<ArrayType, ValueType>
where
    ArrayType: Array + Serialize + DeserializeOwned,
    ValueType: Encode + Decode,
{
    #[inline]
    pub fn open(path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = RocksDB::open(path)?;
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn from_db(db: RocksDB<ArrayType>, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn get(
        &self,
        root_hash: &ArrayType,
        keys: &mut [ArrayType],
    ) -> BinaryMerkleTreeResult<HashMap<ArrayType, Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    #[inline]
    pub fn get_one(
        &self,
        root: &ArrayType,
        key: &ArrayType,
    ) -> BinaryMerkleTreeResult<Option<ValueType>> {
        self.tree.get_one(&root, &key)
    }

    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&ArrayType>,
        keys: &mut [ArrayType],
        values: &[ValueType],
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert(previous_root, keys, values)
    }

    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&ArrayType>,
        key: &ArrayType,
        value: &ValueType,
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert_one(previous_root, key, value)
    }

    #[inline]
    pub fn remove(&mut self, root_hash: &ArrayType) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
    ) -> BinaryMerkleTreeResult<Vec<(ArrayType, bool)>> {
        self.tree.generate_inclusion_proof(root, key)
    }

    #[inline]
    pub fn verify_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
        value: &ValueType,
        proof: &Vec<(ArrayType, bool)>,
    ) -> BinaryMerkleTreeResult<()> {
        self.tree.verify_inclusion_proof(root, key, value, proof)
    }
}
