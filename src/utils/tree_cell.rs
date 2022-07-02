use crate::traits::{Array, Branch, Data, Leaf};

/// Represents a position in the tree during tree traversal.
#[non_exhaustive]
pub struct TreeCell<'keys, NodeType, ArrayType: Array> {
    /// The location of the node being traversed.
    pub location: ArrayType,
    /// The keys traversing this part of the tree.
    pub keys: &'keys [ArrayType],
    /// The node currently being traversed.
    pub node: NodeType,
    /// The depth of the traversal in the tree.
    pub depth: usize,
}

impl<'keys, NodeType, ArrayType: Array> TreeCell<'keys, NodeType, ArrayType> {
    /// Creates a new `TreeCell`.
    #[inline]
    pub const fn new<BranchType: Branch<ArrayType>, LeafType: Leaf<ArrayType>, DataType: Data>(
        location: ArrayType,
        keys: &'keys [ArrayType],
        node: NodeType,
        depth: usize,
    ) -> Self {
        Self {
            location,
            keys,
            node,
            depth,
        }
    }
}
