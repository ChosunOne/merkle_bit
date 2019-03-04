#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use std::error::Error;

use crate::traits::Leaf;

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

#[derive(Clone, Debug, Default)]
#[cfg_attr(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"), derive(Serialize, Deserialize))]
pub struct TreeLeaf {
    key: Vec<u8>,
    data: Vec<u8>,
}

impl TreeLeaf {
    pub fn new() -> Self {
        Self {
            key: vec![],
            data: vec![],
        }
    }

    fn get_key(&self) -> &[u8] {
        self.key.as_ref()
    }
    fn get_data(&self) -> &[u8] {
        self.data.as_ref()
    }

    fn set_key(&mut self, key: Vec<u8>) {
        self.key = key;
    }
    fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

impl Leaf for TreeLeaf {
    fn new() -> Self { Self::new() }

    fn get_key(&self) -> &[u8] { Self::get_key(&self) }
    fn get_data(&self) -> &[u8] { Self::get_data(&self) }

    fn set_key(&mut self, key: &[u8]) { Self::set_key(self, key.to_vec()) }
    fn set_data(&mut self, data: &[u8]) { Self::set_data(self, data.to_vec()) }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeLeaf {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeLeaf {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}