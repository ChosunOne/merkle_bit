use crate::traits::Array;
use std::cmp::Ordering;

/// A reference to a node in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TreeRef<ArrayType: Array> {
    /// The associated key with this `TreeRef`.
    pub key: ArrayType,
    /// The location of the `TreeRef` in the tree.
    pub location: ArrayType,
    /// The total number of elements underneath this `TreeRef`.  This represents the total number of nodes
    /// under this node in the tree.
    pub node_count: u64,
    /// The number of nodes underneath this `TreeRef` when building the tree.  This value is used in the tree building process
    /// on `insert`, and does not consider the total number of nodes in the tree.
    pub count: u32,
}

impl<ArrayType: Array> TreeRef<ArrayType> {
    /// Creates a new `TreeRef`.
    #[inline]
    pub const fn new(key: ArrayType, location: ArrayType, node_count: u64, count: u32) -> Self {
        Self {
            key,
            location,
            node_count,
            count,
        }
    }
}

impl<ArrayType: Array> PartialOrd for TreeRef<ArrayType> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<ArrayType: Array> Ord for TreeRef<ArrayType> {
    #[inline]
    fn cmp(&self, other_ref: &Self) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}
