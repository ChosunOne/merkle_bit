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
use crate::traits::{Array, Leaf};
#[cfg(feature = "serde")]
use crate::traits::{Decode, Encode};

/// Represents a leaf of the tree.  Holds a pointer to the location of the underlying `Data` node.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TreeLeaf<ArrayType: Array> {
    /// The associated key with this node.
    key: ArrayType,
    /// The location of the `Data` node in the tree.
    data: ArrayType,
}

impl<ArrayType: Array> Leaf<ArrayType> for TreeLeaf<ArrayType> {
    /// Creates a new `TreeLeaf`
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    /// Gets the associated key with this node.
    #[inline]
    fn get_key(&self) -> &ArrayType {
        &self.key
    }

    /// Gets the location of the `Data` node.
    #[inline]
    fn get_data(&self) -> &ArrayType {
        &self.data
    }

    /// Sets the associated key with this node.
    #[inline]
    fn set_key(&mut self, key: ArrayType) {
        self.key = key;
    }

    /// Sets the location for the `Data` node.
    #[inline]
    fn set_data(&mut self, data: ArrayType) {
        self.data = data;
    }

    /// Decomposes the struct into its constituent parts.
    #[inline]
    fn decompose(self) -> (ArrayType, ArrayType) {
        (self.key, self.data)
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + Serialize> Encode for TreeLeaf<ArrayType> {
    #[inline]
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "bincode")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "json")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "cbor")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "yaml")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "pickle")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "ron")]
impl<ArrayType: Array + DeserializeOwned> Decode for TreeLeaf<ArrayType> {
    #[inline]
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}
