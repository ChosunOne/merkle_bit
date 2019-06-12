use crate::traits::{Array, Branch, Data, Leaf};

/// Represents a position in the tree during tree traversal.
pub struct TreeCell<'a, NodeType, ArrayType>
where
    ArrayType: Array,
{
    /// The location of the node being traversed.
    pub location: ArrayType,
    /// The keys traversing this part of the tree.
    pub keys: &'a [ArrayType],
    /// The node currently being traversed.
    pub node: NodeType,
    /// The depth of the traversal in the tree.
    pub depth: usize,
}

impl<'a, NodeType, ArrayType> TreeCell<'a, NodeType, ArrayType>
where
    ArrayType: Array,
{
    /// Creates a new `TreeCell`.
    #[inline]
    pub fn new<BranchType, LeafType, DataType>(
        location: ArrayType,
        keys: &'a [ArrayType],
        node: NodeType,
        depth: usize,
    ) -> Self
    where
        BranchType: Branch<ArrayType>,
        LeafType: Leaf<ArrayType>,
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
