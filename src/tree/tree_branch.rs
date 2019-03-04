#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))]
use std::error::Error;

use crate::traits::Branch;

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
pub struct TreeBranch {
    count: u64,
    zero: Vec<u8>,
    one: Vec<u8>,
    split_index: u32,
    key: Vec<u8>,
}

impl TreeBranch {
    fn new() -> Self {
        Self {
            count: 0,
            zero: vec![],
            one: vec![],
            split_index: 0,
            key: vec![],
        }
    }

    fn get_count(&self) -> u64 {
        self.count
    }
    fn get_zero(&self) -> &[u8] {
        self.zero.as_ref()
    }
    fn get_one(&self) -> &[u8] {
        self.one.as_ref()
    }
    fn get_split_index(&self) -> u32 {
        self.split_index
    }
    fn get_key(&self) -> Option<&[u8]> { Some(&self.key) }

    fn set_count(&mut self, count: u64) {
        self.count = count;
    }
    fn set_zero(&mut self, zero: Vec<u8>) {
        self.zero = zero;
    }
    fn set_one(&mut self, one: Vec<u8>) {
        self.one = one;
    }
    fn set_split_index(&mut self, split_index: u32) {
        self.split_index = split_index;
    }
    fn set_key(&mut self, key: Vec<u8>) { self.key = key; }
}

impl Branch for TreeBranch {
    fn new() -> Self { Self::new() }

    fn get_count(&self) -> u64 { Self::get_count(&self) }
    fn get_zero(&self) -> &[u8] { Self::get_zero(&self) }
    fn get_one(&self) -> &[u8] { Self::get_one(&self) }
    fn get_split_index(&self) -> u32 { Self::get_split_index(&self) }
    fn get_key(&self) -> Option<&[u8]> { Self::get_key(&self) }

    fn set_count(&mut self, count: u64) { Self::set_count(self, count) }
    fn set_zero(&mut self, zero: &[u8]) { Self::set_zero(self, zero.to_vec()) }
    fn set_one(&mut self, one: &[u8]) { Self::set_one(self, one.to_vec()) }
    fn set_split_index(&mut self, index: u32) { Self::set_split_index(self, index) }
    fn set_key(&mut self, key: &[u8]) { Self::set_key(self, key.to_vec()) }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeBranch {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeBranch {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}