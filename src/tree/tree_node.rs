#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use std::error::Error;

use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::merkle_bit::NodeVariant;
use crate::traits::{Exception, Node};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_data::TreeData;

use crate::traits::{Decode, Encode};

#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use serde::{Serialize, Deserialize};

#[cfg(feature = "use_bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "use_json")]
use serde_json;
#[cfg(feature = "use_cbor")]
use serde_cbor;
#[cfg(feature = "use_yaml")]
use serde_yaml;
#[cfg(feature = "use_pickle")]
use serde_pickle;
#[cfg(feature = "use_ron")]
use ron;

#[derive(Clone, Debug)]
#[cfg_attr(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"), derive(Serialize, Deserialize))]
pub struct TreeNode {
    references: u64,
    node: Option<NodeVariant<TreeBranch, TreeLeaf, TreeData>>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            references: 0,
            node: None,
        }
    }

    fn get_references(&self) -> u64 {
        self.references
    }

    fn set_references(&mut self, references: u64) {
        self.references = references;
    }
    fn set_branch(&mut self, branch: TreeBranch) {
        self.node = Some(NodeVariant::Branch(branch));
    }

    fn set_leaf(&mut self, leaf: TreeLeaf) {
        self.node = Some(NodeVariant::Leaf(leaf));
    }
    fn set_data(&mut self, data: TreeData) {
        self.node = Some(NodeVariant::Data(data));
    }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}

impl<ValueType> Node<TreeBranch, TreeLeaf, TreeData, ValueType> for TreeNode
    where ValueType: Encode + Decode {
    fn new() -> Self { Self::new() }

    fn get_references(&self) -> u64 { Self::get_references(&self) }
    fn get_variant(&self) -> BinaryMerkleTreeResult<NodeVariant<TreeBranch, TreeLeaf, TreeData>> {
        match self.node {
            Some(ref node_type) => {
                match node_type {
                    NodeVariant::Branch(branch) => Ok(NodeVariant::Branch(branch.clone())),
                    NodeVariant::Data(data) => Ok(NodeVariant::Data(data.clone())),
                    NodeVariant::Leaf(leaf) => Ok(NodeVariant::Leaf(leaf.clone()))
                }
            }
            None => Err(Box::new(Exception::new("Failed to distinguish node type")))
        }
    }

    fn set_references(&mut self, references: u64) { Self::set_references(self, references) }
    fn set_branch(&mut self, branch: TreeBranch) { Self::set_branch(self, branch) }
    fn set_leaf(&mut self, leaf: TreeLeaf) { Self::set_leaf(self, leaf) }
    fn set_data(&mut self, data: TreeData) { Self::set_data(self, data) }
}