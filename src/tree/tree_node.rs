#[cfg(feature = "use_bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "use_ron")]
use ron;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "use_cbor")]
use serde_cbor;
#[cfg(feature = "use_json")]
use serde_json;
#[cfg(feature = "use_pickle")]
use serde_pickle;
#[cfg(feature = "use_yaml")]
use serde_yaml;

#[cfg(feature = "use_serde")]
use crate::merkle_bit::BinaryMerkleTreeResult;
#[cfg(feature = "use_serialization")]
use crate::traits::{Decode, Encode};
use crate::traits::{Node, NodeVariant};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;

#[derive(Clone, Debug)]
#[cfg_attr(any(feature = "use_serde"), derive(Serialize, Deserialize))]
pub struct TreeNode {
    pub references: u64,
    pub node: NodeVariant<TreeBranch, TreeLeaf, TreeData>,
}

impl TreeNode {
    pub fn new(node_variant: NodeVariant<TreeBranch, TreeLeaf, TreeData>) -> Self {
        Self {
            references: 0,
            node: node_variant,
        }
    }

    fn get_references(&self) -> u64 {
        self.references
    }

    fn set_references(&mut self, references: u64) {
        self.references = references;
    }
    fn set_branch(&mut self, branch: TreeBranch) {
        self.node = NodeVariant::Branch(branch);
    }

    fn set_leaf(&mut self, leaf: TreeLeaf) {
        self.node = NodeVariant::Leaf(leaf);
    }
    fn set_data(&mut self, data: TreeData) {
        self.node = NodeVariant::Data(data);
    }
}

impl Node<TreeBranch, TreeLeaf, TreeData> for TreeNode {
    fn new(node_variant: NodeVariant<TreeBranch, TreeLeaf, TreeData>) -> Self {
        Self::new(node_variant)
    }

    fn get_references(&self) -> u64 {
        Self::get_references(&self)
    }
    fn get_variant(self) -> NodeVariant<TreeBranch, TreeLeaf, TreeData> {
        self.node
    }

    fn set_references(&mut self, references: u64) {
        Self::set_references(self, references)
    }
    fn set_branch(&mut self, branch: TreeBranch) {
        Self::set_branch(self, branch)
    }
    fn set_leaf(&mut self, leaf: TreeLeaf) {
        Self::set_leaf(self, leaf)
    }
    fn set_data(&mut self, data: TreeData) {
        Self::set_data(self, data)
    }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeNode {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
