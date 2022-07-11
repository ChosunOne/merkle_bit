#[cfg(feature = "bincode")]
use bincode::{deserialize, serialize};
#[cfg(feature = "cbor")]
use ciborium::de::from_reader;
#[cfg(feature = "cbor")]
use ciborium::ser::into_writer;
#[cfg(feature = "ron")]
use ron;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use serde_json;
#[cfg(feature = "pickle")]
use serde_pickle;
#[cfg(feature = "yaml")]
use serde_yaml;

#[cfg(feature = "serde")]
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Data;
#[cfg(feature = "serde")]
use crate::traits::{Decode, Encode};

/// `TreeData` represents the data to be stored in the tree for a given key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(any(feature = "serde"), derive(Serialize, Deserialize))]
pub struct TreeData {
    /// The value to be stored in the tree.
    value: Vec<u8>,
}

impl Data for TreeData {
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn get_value(&self) -> &[u8] {
        &self.value
    }

    #[inline]
    fn set_value(&mut self, value: &[u8]) {
        self.value = value.to_vec();
    }
}

#[cfg(feature = "bincode")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "json")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "cbor")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let mut buf = Vec::new();
        into_writer(&self, &mut buf)?;
        Ok(buf)
    }
}

#[cfg(feature = "yaml")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "pickle")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, Default::default())?)
    }
}

#[cfg(feature = "ron")]
impl Encode for TreeData {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "bincode")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "json")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "cbor")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(from_reader(buffer)?)
    }
}

#[cfg(feature = "yaml")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "pickle")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer, Default::default())?)
    }
}

#[cfg(feature = "ron")]
impl Decode for TreeData {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
