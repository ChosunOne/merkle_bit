use crate::traits::{Branch, Data, Leaf};
use crate::Array;

/// Represents a position in the tree during tree traversal.
#[non_exhaustive]
pub struct TreeCell<'keys, NodeType, const N: usize> {
    /// The location of the node being traversed.
    pub location: Array<N>,
    /// The keys traversing this part of the tree.
    pub keys: &'keys [Array<N>],
    /// The node currently being traversed.
    pub node: NodeType,
    /// The depth of the traversal in the tree.
    pub depth: usize,
}

impl<'keys, NodeType, const N: usize> TreeCell<'keys, NodeType, N> {
    /// Creates a new `TreeCell`.
    #[inline]
    pub const fn new<BranchType: Branch<N>, LeafType: Leaf<N>, DataType: Data>(
        location: Array<N>,
        keys: &'keys [Array<N>],
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
