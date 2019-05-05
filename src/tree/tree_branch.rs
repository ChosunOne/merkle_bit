#[cfg(feature = "use_serde")]
use std::error::Error;
#[cfg(feature = "use_json")]
use std::string::FromUtf8Error;

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

use crate::constants::KEY_LEN;
#[cfg(feature = "use_serde")]
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Branch;
#[cfg(feature = "use_serde")]
use crate::traits::{Decode, Encode, Exception};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "use_serde"), derive(Serialize, Deserialize))]
pub struct TreeBranch {
    count: u64,
    zero: [u8; KEY_LEN],
    one: [u8; KEY_LEN],
    split_index: u8,
    key: [u8; KEY_LEN],
}

impl TreeBranch {
    const fn new() -> Self {
        Self {
            count: 0,
            zero: [0; KEY_LEN],
            one: [0; KEY_LEN],
            split_index: 0,
            key: [0; KEY_LEN],
        }
    }

    const fn get_count(&self) -> u64 {
        self.count
    }
    const fn get_zero(&self) -> &[u8; KEY_LEN] {
        &self.zero
    }
    const fn get_one(&self) -> &[u8; KEY_LEN] {
        &self.one
    }
    const fn get_split_index(&self) -> u8 {
        self.split_index
    }
    const fn get_key(&self) -> &[u8; KEY_LEN] {
        &self.key
    }

    fn set_count(&mut self, count: u64) {
        self.count = count;
    }
    fn set_zero(&mut self, zero: [u8; KEY_LEN]) {
        self.zero = zero;
    }
    fn set_one(&mut self, one: [u8; KEY_LEN]) {
        self.one = one;;
    }
    fn set_split_index(&mut self, split_index: u8) {
        self.split_index = split_index;
    }
    fn set_key(&mut self, key: [u8; KEY_LEN]) {
        self.key = key;
    }

    const fn deconstruct(self) -> (u64, [u8; KEY_LEN], [u8; KEY_LEN], u8, [u8; KEY_LEN]) {
        (self.count, self.zero, self.one, self.split_index, self.key)
    }
}

impl Branch for TreeBranch {
    #[inline]
    fn new() -> Self {
        Self::new()
    }

    #[inline]
    fn get_count(&self) -> u64 {
        Self::get_count(self)
    }
    #[inline]
    fn get_zero(&self) -> &[u8; KEY_LEN] {
        Self::get_zero(self)
    }
    #[inline]
    fn get_one(&self) -> &[u8; KEY_LEN] {
        Self::get_one(self)
    }
    #[inline]
    fn get_split_index(&self) -> u8 {
        Self::get_split_index(self)
    }
    #[inline]
    fn get_key(&self) -> &[u8; KEY_LEN] {
        Self::get_key(self)
    }

    #[inline]
    fn set_count(&mut self, count: u64) {
        Self::set_count(self, count)
    }
    #[inline]
    fn set_zero(&mut self, zero: [u8; KEY_LEN]) {
        Self::set_zero(self, zero)
    }
    #[inline]
    fn set_one(&mut self, one: [u8; KEY_LEN]) {
        Self::set_one(self, one)
    }
    #[inline]
    fn set_split_index(&mut self, index: u8) {
        Self::set_split_index(self, index)
    }
    #[inline]
    fn set_key(&mut self, key: [u8; KEY_LEN]) {
        Self::set_key(self, key)
    }

    #[inline]
    fn deconstruct(self) -> (u64, [u8; KEY_LEN], [u8; KEY_LEN], u8, [u8; KEY_LEN]) {
        Self::deconstruct(self)
    }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeBranch {
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
impl Encode for TreeBranch {
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
impl Encode for TreeBranch {
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
impl Encode for TreeBranch {
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
impl Encode for TreeBranch {
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
impl Encode for TreeBranch {
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
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeBranch {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
