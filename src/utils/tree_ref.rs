use std::cmp::Ordering;

use crate::constants::KEY_LEN;

/// A reference to a node in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct TreeRef {
    /// The associated key with this `TreeRef`.
    pub key: [u8; KEY_LEN],
    /// The location of the `TreeRef` in the tree.
    pub location: [u8; KEY_LEN],
    /// The total number of elements underneath this `TreeRef`.  This represents the total number of nodes
    /// under this node in the tree.
    pub node_count: u64,
    /// The number of nodes underneath this `TreeRef` when building the tree.  This value is used in the tree building process
    /// on `insert`, and does not consider the total number of nodes in the tree.
    pub count: u32,
}

impl TreeRef {
    /// Creates a new TreeRef.
    #[inline]
    pub const fn new(
        key: [u8; KEY_LEN],
        location: [u8; KEY_LEN],
        node_count: u64,
        count: u32,
    ) -> Self {
        Self {
            key,
            location,
            node_count,
            count,
        }
    }
}

impl Ord for TreeRef {
    #[inline]
    fn cmp(&self, other_ref: &Self) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}
