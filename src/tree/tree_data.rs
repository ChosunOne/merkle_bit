#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use crate::merkle_bit::BinaryMerkleTreeResult;
use crate::traits::Data;

#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
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
pub struct TreeData {
    value: Vec<u8>
}

impl TreeData {
    fn new() -> Self {
        Self {
            value: vec![]
        }
    }

    fn get_value(&self) -> &[u8] { self.value.as_ref() }

    fn set_value(&mut self, value: Vec<u8>) { self.value = value }
}

impl Data for TreeData {
    fn new() -> Self { Self::new() }

    fn get_value(&self) -> &[u8] { Self::get_value(&self) }

    fn set_value(&mut self, value: &[u8]) { Self::set_value(self, value.to_vec()) }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeData {
    fn encode(&self) -> BinaryMerkleTreeResult<Vec<u8>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> BinaryMerkleTreeResult<Self> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}