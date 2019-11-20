#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::{Array, Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::HashTreeDB;
use crate::tree_hasher::TreeHasher;

/// Internal type alias for the underlying tree.
type Tree<ArrayType, ValueType> = MerkleBIT<
    HashTreeDB<ArrayType>,
    TreeBranch<ArrayType>,
    TreeLeaf<ArrayType>,
    TreeData,
    TreeNode<ArrayType>,
    TreeHasher,
    ValueType,
    ArrayType,
>;

/// A `MerkleBIT` implemented with a `HashMap`.  Can be used for quickly storing items in memory, though
/// larger sets of items should be stored on disk or over the network in a real database.
pub struct HashTree<ArrayType = [u8; 32], ValueType = Vec<u8>>
where
    ValueType: Encode + Decode,
    ArrayType: Array,
{
    /// The underlying tree.  The type requirements have already been implemented for easy use.
    tree: Tree<ArrayType, ValueType>,
}

impl<ValueType, ArrayType> HashTree<ArrayType, ValueType>
where
    ValueType: Encode + Decode,
    ArrayType: Array,
{
    /// Creates a new `HashTree`.  `depth` indicates the maximum depth of the tree.
    #[inline]
    pub fn new(depth: usize) -> BinaryMerkleTreeResult<Self> {
        let path = PathBuf::new();
        let tree = MerkleBIT::new(&path, depth)?;
        Ok(Self { tree })
    }

    /// Creates a new `HashTree`.  This method exists for conforming with the general API for the `MerkleBIT`
    /// and does not need to be used (except for compatibility).  Prefer `new` when possible.
    #[inline]
    pub fn open(path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::new(path, depth)?;
        Ok(Self { tree })
    }

    /// Gets the values associated with `keys` from the tree.
    #[inline]
    pub fn get(
        &self,
        root_hash: &ArrayType,
        keys: &mut [ArrayType],
    ) -> BinaryMerkleTreeResult<HashMap<ArrayType, Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    /// Inserts elements into the tree.  Using `previous_root` specifies that the insert depends on
    /// the state from the previous root, and will update references accordingly.
    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&ArrayType>,
        keys: &mut [ArrayType],
        values: &[ValueType],
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert(previous_root, keys, values)
    }

    /// Removes a root from the tree.  This will remove all elements with less than two references
    /// under the given root.
    #[inline]
    pub fn remove(&mut self, root_hash: &ArrayType) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    /// Generates an inclusion proof for the given key at the specified root.
    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
    ) -> BinaryMerkleTreeResult<Vec<(ArrayType, bool)>> {
        self.tree.generate_inclusion_proof(root, key)
    }

    /// Verifies an inclusion proof with the given root, key, and value.
    #[inline]
    pub fn verify_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
        value: &ValueType,
        proof: &[(ArrayType, bool)],
    ) -> BinaryMerkleTreeResult<()> {
        self.tree.verify_inclusion_proof(root, key, value, proof)
    }

    /// Gets a single item out of the tree.
    #[inline]
    pub fn get_one(
        &self,
        root: &ArrayType,
        key: &ArrayType,
    ) -> BinaryMerkleTreeResult<Option<ValueType>> {
        self.tree.get_one(root, key)
    }

    /// Inserts a single item into the tree.
    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&ArrayType>,
        key: &ArrayType,
        value: &ValueType,
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert_one(previous_root, key, value)
    }
}
