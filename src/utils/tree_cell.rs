use crate::constants::KEY_LEN;
use crate::traits::{Branch, Data, Leaf};

/// Represents a position in the tree during tree traversal.
pub struct TreeCell<'a, NodeType> {
    /// The location of the node being traversed.
    pub location: [u8; KEY_LEN],
    /// The keys traversing this part of the tree.
    pub keys: &'a [&'a [u8; KEY_LEN]],
    /// The node currently being traversed.
    pub node: NodeType,
    /// The depth of the traversal in the tree.
    pub depth: usize,
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    /// Creates a new `TreeCell`.
    #[inline]
    pub fn new<BranchType, LeafType, DataType>(
        location: [u8; KEY_LEN],
        keys: &'a [&'a [u8; KEY_LEN]],
        node: NodeType,
        depth: usize,
    ) -> Self
    where
        BranchType: Branch,
        LeafType: Leaf,
        DataType: Data,
    {
        Self {
            location,
            keys,
            node,
            depth,
        }
    }
}
