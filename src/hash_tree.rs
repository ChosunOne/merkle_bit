#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use crate::constants::KEY_LEN;
use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::{Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_node::TreeNode;
use crate::tree_db::HashTreeDB;
use crate::tree_hasher::TreeHasher;

/// A `MerkleBIT` implemented with a `HashMap`.  Can be used for quickly storing items in memory, though
/// larger sets of items should be stored on disk or over the network in a real database.
pub struct HashTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
{
    /// The underlying tree.  The type requirements have already been implemented for easy use.
    tree: MerkleBIT<HashTreeDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, ValueType>,
}

impl<ValueType> HashTree<ValueType>
where
    ValueType: Encode + Decode + Sync + Send,
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
    pub fn get<'a>(
        &self,
        root_hash: &[u8; KEY_LEN],
        keys: &mut [&'a [u8; KEY_LEN]],
    ) -> BinaryMerkleTreeResult<HashMap<&'a [u8; KEY_LEN], Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    /// Inserts elements into the tree.  Using `previous_root` specifies that the insert depends on
    /// the state from the previous root, and will update references accordingly.
    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&[u8; KEY_LEN]>,
        keys: &mut [&[u8; KEY_LEN]],
        values: &mut [&ValueType],
    ) -> BinaryMerkleTreeResult<[u8; KEY_LEN]> {
        self.tree.insert(previous_root, keys, values)
    }

    /// Removes a root from the tree.  This will remove all elements with less than two references
    /// under the given root.
    #[inline]
    pub fn remove(&mut self, root_hash: &[u8; KEY_LEN]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}
