#[cfg(feature = "use_serde")]
use std::error::Error;
#[cfg(feature = "use_json")]
use std::string::FromUtf8Error;

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
use crate::traits::{Array, Branch};
#[cfg(feature = "use_serde")]
use crate::traits::{Decode, Encode, Exception};

/// A struct representing a branch in the tree.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "use_serde"), derive(Serialize, Deserialize))]
pub struct TreeBranch<ArrayType>
where
    ArrayType: Array,
{
    /// The number of leaf nodes under this branch.
    count: u64,
    /// The location of the next node when traversing the zero branch.
    zero: ArrayType,
    /// The location of the next node when traversing the one branch.
    one: ArrayType,
    /// The index bit of the associated key on which to make a decision to go down the zero or one branch.
    split_index: usize,
    /// The associated key with this branch.
    key: ArrayType,
}

impl<ArrayType> TreeBranch<ArrayType>
where
    ArrayType: Array,
{
    /// Create a new `TreeBranch`
    fn new() -> Self {
        Self {
            count: 0,
            zero: ArrayType::default(),
            one: ArrayType::default(),
            split_index: 0,
            key: ArrayType::default(),
        }
    }

    /// Get the count of leaf nodes under this branch.
    fn get_count(&self) -> u64 {
        self.count
    }

    /// Get the location of the next node when going down the zero side.
    fn get_zero(&self) -> &ArrayType {
        &self.zero
    }

    /// Get the location of the next node when going down the one side.
    fn get_one(&self) -> &ArrayType {
        &self.one
    }

    /// Get the index to split on when deciding which child to traverse.
    fn get_split_index(&self) -> usize {
        self.split_index
    }

    /// Get the associated key with this branch.
    fn get_key(&self) -> &ArrayType {
        &self.key
    }

    /// Set the number of leaf nodes under this branch.
    fn set_count(&mut self, count: u64) {
        self.count = count;
    }

    /// Set the location of the next node to traverse when going down the zero side.
    fn set_zero(&mut self, zero: ArrayType) {
        self.zero = zero;
    }

    /// Set the location of the next node to traverse when going down the one side.
    fn set_one(&mut self, one: ArrayType) {
        self.one = one;
    }

    /// Sets the index of the key to split on when deciding which child to traverse.
    fn set_split_index(&mut self, split_index: usize) {
        self.split_index = split_index;
    }

    /// Sets the associated key for this node.
    fn set_key(&mut self, key: ArrayType) {
        self.key = key;
    }

    /// Decomposes the `TreeBranch` into its constituent parts.
    fn decompose(self) -> (u64, ArrayType, ArrayType, usize, ArrayType) {
        (self.count, self.zero, self.one, self.split_index, self.key)
    }
}

impl<ArrayType> Branch<ArrayType> for TreeBranch<ArrayType>
where
    ArrayType: Array,
{
    #[inline]
    fn new() -> Self {
        Self::new()
    }

    #[inline]
    fn get_count(&self) -> u64 {
        Self::get_count(self)
    }
    #[inline]
    fn get_zero(&self) -> &ArrayType {
        Self::get_zero(self)
    }
    #[inline]
    fn get_one(&self) -> &ArrayType {
        Self::get_one(self)
    }
    #[inline]
    fn get_split_index(&self) -> usize {
        Self::get_split_index(self)
    }
    #[inline]
    fn get_key(&self) -> &ArrayType {
        Self::get_key(self)
    }

    #[inline]
    fn set_count(&mut self, count: u64) {
        Self::set_count(self, count)
    }
    #[inline]
    fn set_zero(&mut self, zero: ArrayType) {
        Self::set_zero(self, zero)
    }
    #[inline]
    fn set_one(&mut self, one: ArrayType) {
        Self::set_one(self, one)
    }
    #[inline]
    fn set_split_index(&mut self, index: usize) {
        Self::set_split_index(self, index)
    }
    #[inline]
    fn set_key(&mut self, key: ArrayType) {
        Self::set_key(self, key)
    }

    #[inline]
    fn decompose(self) -> (u64, ArrayType, ArrayType, usize, ArrayType) {
        Self::decompose(self)
    }
}

#[cfg(feature = "use_bincode")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_bincode")]
impl From<Box<bincode::ErrorKind>> for Exception {
    #[inline]
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_json")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_json")]
impl From<serde_json::Error> for Exception {
    #[inline]
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_json")]
impl From<FromUtf8Error> for Exception {
    #[inline]
    fn from(error: FromUtf8Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_cbor")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_cbor")]
impl From<serde_cbor::error::Error> for Exception {
    #[inline]
    fn from(error: serde_cbor::error::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_yaml")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl From<serde_yaml::Error> for Exception {
    #[inline]
    fn from(error: serde_yaml::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_pickle")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_pickle")]
impl From<serde_pickle::Error> for Exception {
    #[inline]
    fn from(error: serde_pickle::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_ron")]
impl<ArrayType> Encode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize,
{
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_ron")]
impl From<ron::ser::Error> for Exception {
    #[inline]
    fn from(error: ron::ser::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_ron")]
impl From<ron::de::Error> for Exception {
    #[inline]
    fn from(error: ron::de::Error) -> Self {
        Self::new(error.description())
    }
}

#[cfg(feature = "use_bincode")]
impl<ArrayType> Decode for TreeBranch<ArrayType>
where
    ArrayType: Array + Serialize + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let a = deserialize(buffer)?;
        Ok(a)
    }
}

#[cfg(feature = "use_json")]
impl<ArrayType> Decode for TreeBranch<ArrayType>
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
impl<ArrayType> Decode for TreeBranch<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl<ArrayType> Decode for TreeBranch<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl<ArrayType> Decode for TreeBranch<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl<ArrayType> Decode for TreeBranch<ArrayType>
where
    ArrayType: Array + DeserializeOwned,
{
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
