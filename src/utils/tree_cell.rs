use crate::constants::KEY_LEN;
use crate::traits::{Branch, Data, Leaf};

pub struct TreeCell<'a, NodeType> {
    pub location: [u8; KEY_LEN],
    pub keys: &'a [&'a [u8; KEY_LEN]],
    pub node: NodeType,
    pub depth: usize,
}

impl<'a, 'b, NodeType> TreeCell<'a, NodeType> {
    pub fn new<BranchType, LeafType, DataType>(
        location: [u8; KEY_LEN],
        keys: &'a [&'a [u8; KEY_LEN]],
        node: NodeType,
        depth: usize,
    ) -> TreeCell<'a, NodeType>
    where
        BranchType: Branch,
        LeafType: Leaf,
        DataType: Data,
    {
        TreeCell { location, keys, node, depth }
    }
}
