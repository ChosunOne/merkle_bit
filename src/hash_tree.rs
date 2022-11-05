use core::marker::PhantomData;
#[cfg(not(any(feature = "hashbrown")))]
use std::collections::HashMap;
use std::path::Path;

use crate::Array;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT, MerkleTree};
use crate::traits::{Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::HashTreeDB;
use crate::tree_hasher::TreeHasher;

/// Internal type alias for the underlying tree.
type Tree<const N: usize, Value = Vec<u8>> = MerkleBIT<HashTree<N, Value>, N>;

/// A `MerkleBIT` implemented with a `HashMap`.  Can be used for quickly storing items in memory, though
/// larger sets of items should be stored on disk or over the network in a real database.
pub struct HashTree<const N: usize = 32, Value: Encode + Decode = Vec<u8>> {
    /// The underlying tree.  The type requirements have already been implemented for easy use.
    tree: Tree<N>,
    /// Marker for `Value`
    _value: PhantomData<Value>,
}

impl<const N: usize, Value: Encode + Decode> MerkleTree<N> for HashTree<N, Value> {
    type Database = HashTreeDB<N>;
    type Branch = TreeBranch<N>;
    type Leaf = TreeLeaf<N>;
    type Data = TreeData;
    type Node = TreeNode<N>;
    type Hasher = TreeHasher;
    type Value = Value;
}

impl<const N: usize> HashTree<N> {
    /// Creates a new `HashTree`.  `depth` indicates the maximum depth of the tree.
    /// # Errors
    /// None.
    #[inline]
    pub fn new(depth: usize) -> BinaryMerkleTreeResult<Self> {
        let path = Path::new("");
        let tree = MerkleBIT::new(path, depth)?;
        Ok(Self {
            tree,
            _value: PhantomData::default(),
        })
    }

    /// Creates a new `HashTree`.  This method exists for conforming with the general API for the `MerkleBIT`
    /// and does not need to be used (except for compatibility).  Prefer `new` when possible.
    /// # Errors
    /// None.
    #[inline]
    pub fn open(path: &Path, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::new(path, depth)?;
        Ok(Self {
            tree,
            _value: PhantomData::default(),
        })
    }

    /// Gets the values associated with `keys` from the tree.
    /// # Errors
    /// `Exception` generated if the `get` encounters an invalid state during tree traversal.
    #[inline]
    pub fn get(
        &self,
        root_hash: &Array<N>,
        keys: &mut [Array<N>],
    ) -> BinaryMerkleTreeResult<HashMap<Array<N>, Option<<Self as MerkleTree<N>>::Value>>> {
        self.tree.get(root_hash, keys)
    }

    /// Inserts elements into the tree.  Using `previous_root` specifies that the insert depends on
    /// the state from the previous root, and will update references accordingly.
    /// # Errors
    /// `Exception` generated if the `insert` encounters an invalid state during tree traversal.
    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&Array<N>>,
        keys: &mut [Array<N>],
        values: &[<Self as MerkleTree<N>>::Value],
    ) -> BinaryMerkleTreeResult<Array<N>> {
        self.tree.insert(previous_root, keys, values)
    }

    /// Removes a root from the tree.  This will remove all elements with less than two references
    /// under the given root.
    /// # Errors
    /// `Exception` generated if the `remove` encounters an invalid state during tree traversal.
    #[inline]
    pub fn remove(&mut self, root_hash: &Array<N>) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    /// Generates an inclusion proof for the given key at the specified root.
    /// # Errors
    /// `Exception` generated if an invalid state is encountered during tree traversal
    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &Array<N>,
        key: Array<N>,
    ) -> BinaryMerkleTreeResult<Vec<(Array<N>, bool)>> {
        self.tree.generate_inclusion_proof(root, key)
    }

    /// Verifies an inclusion proof with the given root, key, and value.
    /// # Errors
    /// `Exception` generated if the given proof is invalid.
    #[inline]
    pub fn verify_inclusion_proof(
        root: &Array<N>,
        key: Array<N>,
        value: &<Self as MerkleTree<N>>::Value,
        proof: &[(Array<N>, bool)],
    ) -> BinaryMerkleTreeResult<()> {
        Tree::verify_inclusion_proof(root, key, value, proof)
    }

    /// Gets a single item out of the tree.
    /// # Errors
    /// `Exception` generated if the `get_one` encounters an invalid state during tree traversal.
    #[inline]
    pub fn get_one(
        &self,
        root: &Array<N>,
        key: &Array<N>,
    ) -> BinaryMerkleTreeResult<Option<<Self as MerkleTree<N>>::Value>> {
        self.tree.get_one(root, key)
    }

    /// Inserts a single item into the tree.
    /// # Errors
    /// `Exception` generated if the `insert_one` encounters an invalid state during tree traversal.
    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&Array<N>>,
        key: &Array<N>,
        value: &<Self as MerkleTree<N>>::Value,
    ) -> BinaryMerkleTreeResult<Array<N>> {
        self.tree.insert_one(previous_root, key, value)
    }

    #[inline]
    #[must_use]
    /// Decomposes the tree into the its DB and size
    pub fn decompose(self) -> (HashTreeDB<N>, usize) {
        self.tree.decompose()
    }
}
