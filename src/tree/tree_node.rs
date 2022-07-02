#[cfg(feature = "bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "ron")]
use ron;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "cbor")]
use serde_cbor;
#[cfg(feature = "json")]
use serde_json;
#[cfg(feature = "pickle")]
use serde_pickle;
#[cfg(feature = "yaml")]
use serde_yaml;

#[cfg(feature = "serde")]
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::{Array, Node, NodeVariant};
#[cfg(feature = "serde")]
use crate::traits::{Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;

/// A node in the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "serde"), derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct TreeNode<ArrayType: Array> {
    /// The number of references to this node.
    pub references: u64,
    /// The `NodeVariant` of the node.
    pub node: NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>,
}

impl<ArrayType: Array> Node<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>
    for TreeNode<ArrayType>
{
    #[inline]
    fn new(
        node_variant: NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>,
    ) -> Self {
        Self {
            references: 0,
            node: node_variant,
        }
    }

    #[inline]
    fn get_references(&self) -> u64 {
        self.references
    }
    #[inline]
    fn get_variant(
        self,
    ) -> NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType> {
        self.node
    }

    #[inline]
    fn set_references(&mut self, references: u64) {
        self.references = references;
    }
    #[inline]
    fn set_branch(&mut self, branch: TreeBranch<ArrayType>) {
        self.node = NodeVariant::Branch(branch);
    }
    #[inline]
    fn set_leaf(&mut self, leaf: TreeLeaf<ArrayType>) {
        self.node = NodeVariant::Leaf(leaf);
    }
    #[inline]
    fn set_data(&mut self, data: TreeData) {
        self.node = NodeVariant::Data(data);
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, Default::default())?)
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + Serialize> Encode for TreeNode<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer, Default::default())?)
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeNode<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
