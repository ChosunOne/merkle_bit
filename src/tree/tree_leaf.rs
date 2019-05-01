#[cfg(feature = "use_bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "use_ron")]
use ron;
#[cfg(feature = "use_serialization")]
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
#[cfg(feature = "use_serialization")]
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Leaf;
#[cfg(feature = "use_serialization")]
use crate::traits::{Decode, Encode};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct TreeLeaf {
    key: [u8; KEY_LEN],
    data: [u8; KEY_LEN],
}

impl TreeLeaf {
    pub fn new() -> Self {
        Self {
            key: [0; KEY_LEN],
            data: [0; KEY_LEN],
        }
    }

    fn get_key(&self) -> &[u8; KEY_LEN] {
        &self.key
    }
    fn get_data(&self) -> &[u8; KEY_LEN] {
        &self.data
    }

    fn set_key(&mut self, key: [u8; KEY_LEN]) {
        self.key = key;
    }
    fn set_data(&mut self, data: [u8; KEY_LEN]) {
        self.data = data;
    }

    fn deconstruct(self) -> ([u8; KEY_LEN], [u8; KEY_LEN]) {
        (self.key, self.data)
    }
}

impl Leaf for TreeLeaf {
    fn new() -> Self {
        Self::new()
    }

    fn get_key(&self) -> &[u8; KEY_LEN] {
        Self::get_key(&self)
    }
    fn get_data(&self) -> &[u8; KEY_LEN] {
        Self::get_data(&self)
    }

    fn set_key(&mut self, key: [u8; KEY_LEN]) {
        Self::set_key(self, key)
    }
    fn set_data(&mut self, data: [u8; KEY_LEN]) {
        Self::set_data(self, data)
    }

    fn deconstruct(self) -> ([u8; KEY_LEN], [u8; KEY_LEN]) {
        Self::deconstruct(self)
    }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeLeaf {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
