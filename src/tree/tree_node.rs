#[cfg(feature = "use_bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "use_ron")]
use ron;
#[cfg(feature = "use_serde")]
use serde::de::DeserializeOwned;
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
use crate::traits::{Array, Node, NodeVariant};
#[cfg(feature = "use_serialization")]
use crate::traits::{Decode, Encode};
use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_leaf::TreeLeaf;
#[cfg(feature = "use_evmap")]
use evmap::ShallowCopy;

/// A node in the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "use_serde"), derive(Serialize, Deserialize))]
pub struct TreeNode<ArrayType>
where
    ArrayType: Array,
{
    /// The number of references to this node.
    pub references: u64,
    /// The `NodeVariant` of the node.
    pub node: NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>,
}

impl<ArrayType> TreeNode<ArrayType>
where
    ArrayType: Array,
{
    /// Creates a new `TreeNode`.
    #[inline]
    pub fn new(
        node_variant: NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>,
    ) -> Self {
        Self {
            references: 0,
            node: node_variant,
        }
    }

    /// Gets the number of references to the node.
    fn get_references(&self) -> u64 {
        self.references
    }

    /// Sets the number of references to the node.
    fn set_references(&mut self, references: u64) {
        self.references = references;
    }

    /// Sets the node as a `NodeVariant::Branch`.
    fn set_branch(&mut self, branch: TreeBranch<ArrayType>) {
        self.node = NodeVariant::Branch(branch);
    }

    /// Sets the node as a `NodeVariant::Leaf`.
    fn set_leaf(&mut self, leaf: TreeLeaf<ArrayType>) {
        self.node = NodeVariant::Leaf(leaf);
    }

    /// Sets the node as a `NodeVariant::Data`.
    fn set_data(&mut self, data: TreeData) {
        self.node = NodeVariant::Data(data);
    }
}

impl<ArrayType> Node<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>
    for TreeNode<ArrayType>
where
    ArrayType: Array,
{
    #[inline]
    fn new(
        node_variant: NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType>,
    ) -> Self {
        Self::new(node_variant)
    }

    #[inline]
    fn get_references(&self) -> u64 {
        Self::get_references(self)
    }
    #[inline]
    fn get_variant(
        self,
    ) -> NodeVariant<TreeBranch<ArrayType>, TreeLeaf<ArrayType>, TreeData, ArrayType> {
        self.node
    }

    #[inline]
    fn set_references(&mut self, references: u64) {
        Self::set_references(self, references)
    }
    #[inline]
    fn set_branch(&mut self, branch: TreeBranch<ArrayType>) {
        Self::set_branch(self, branch)
    }
    #[inline]
    fn set_leaf(&mut self, leaf: TreeLeaf<ArrayType>) {
        Self::set_leaf(self, leaf)
    }
    #[inline]
    fn set_data(&mut self, data: TreeData) {
        Self::set_data(self, data)
    }
}

#[cfg(feature = "use_bincode")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl<ArrayType> Encode for TreeNode<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl<ArrayType> Decode for TreeNode<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
