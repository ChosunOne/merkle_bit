use crate::Array;
use std::cmp::Ordering;

/// A reference to a node in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TreeRef<const N: usize> {
    /// The associated key with this `TreeRef`.
    pub key: Array<N>,
    /// The location of the `TreeRef` in the tree.
    pub location: Array<N>,
    /// The total number of elements underneath this `TreeRef`.  This represents the total number of nodes
    /// under this node in the tree.
    pub node_count: u64,
    /// The number of nodes underneath this `TreeRef` when building the tree.  This value is used in the tree building process
    /// on `insert`, and does not consider the total number of nodes in the tree.
    pub count: u32,
}

impl<const N: usize> TreeRef<N> {
    /// Creates a new `TreeRef`.
    #[inline]
    #[must_use]
    pub const fn new(key: Array<N>, location: Array<N>, node_count: u64, count: u32) -> Self {
        Self {
            key,
            location,
            node_count,
            count,
        }
    }
}

impl<const N: usize> PartialOrd for TreeRef<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<const N: usize> Ord for TreeRef<N> {
    #[inline]
    fn cmp(&self, other_ref: &Self) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}
