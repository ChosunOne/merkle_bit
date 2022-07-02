#[cfg(feature = "json")]
use std::string::FromUtf8Error;

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
use crate::traits::{Array, Branch};
#[cfg(feature = "serde")]
use crate::traits::{Decode, Encode, Exception};

/// A struct representing a branch in the tree.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(any(feature = "serde"), derive(Serialize, Deserialize))]
pub struct TreeBranch<ArrayType: Array> {
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

impl<ArrayType: Array> Branch<ArrayType> for TreeBranch<ArrayType> {
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn get_count(&self) -> u64 {
        self.count
    }
    #[inline]
    fn get_zero(&self) -> &ArrayType {
        &self.zero
    }
    #[inline]
    fn get_one(&self) -> &ArrayType {
        &self.one
    }
    #[inline]
    fn get_split_index(&self) -> usize {
        self.split_index
    }
    #[inline]
    fn get_key(&self) -> &ArrayType {
        &self.key
    }

    #[inline]
    fn set_count(&mut self, count: u64) {
        self.count = count;
    }
    #[inline]
    fn set_zero(&mut self, zero: ArrayType) {
        self.zero = zero;
    }
    #[inline]
    fn set_one(&mut self, one: ArrayType) {
        self.one = one;
    }
    #[inline]
    fn set_split_index(&mut self, index: usize) {
        self.split_index = index;
    }
    #[inline]
    fn set_key(&mut self, key: ArrayType) {
        self.key = key;
    }

    #[inline]
    fn decompose(self) -> (u64, ArrayType, ArrayType, usize, ArrayType) {
        (self.count, self.zero, self.one, self.split_index, self.key)
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "bincode")]
impl From<Box<bincode::ErrorKind>> for Exception {
    #[inline]
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Exception {
    #[inline]
    fn from(error: serde_json::Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "json")]
impl From<FromUtf8Error> for Exception {
    #[inline]
    fn from(error: FromUtf8Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "cbor")]
impl From<serde_cbor::error::Error> for Exception {
    #[inline]
    fn from(error: serde_cbor::error::Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for Exception {
    #[inline]
    fn from(error: serde_yaml::Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, Default::default())?)
    }
}

#[cfg(feature = "pickle")]
impl From<serde_pickle::Error> for Exception {
    #[inline]
    fn from(error: serde_pickle::Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + Serialize> Encode for TreeBranch<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "ron")]
impl From<ron::error::Error> for Exception {
    #[inline]
    fn from(error: ron::error::Error) -> Self {
        Self::new(&error.to_string())
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + Serialize + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let a = deserialize(buffer)?;
        Ok(a)
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer, Default::default())?)
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeBranch<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
