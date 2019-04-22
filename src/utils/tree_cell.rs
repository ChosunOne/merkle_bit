use crate::constants::KEY_LEN;
use crate::traits::{Branch, Leaf, Data};

pub struct TreeCell<'a, NodeType> {
    pub keys: &'a [&'a [u8; KEY_LEN]],
    pub node: NodeType,
    pub depth: usize,
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(
        keys: &'a [&'a [u8; KEY_LEN]],
        node: NodeType,
        depth: usize,
    ) -> TreeCell<'a, NodeType>
        where
            BranchType: Branch,
            LeafType: Leaf,
            DataType: Data,
    {
        TreeCell { keys, node, depth }
    }
}