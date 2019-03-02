use std::collections::hash_map::HashMap;
use std::error::Error;
use std::path::PathBuf;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
use std::hash::Hasher;

#[cfg(feature = "use_blake2b")]
use std::cmp::Ordering;

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

#[cfg(feature = "use_blake2b")]
use blake2_rfc;
#[cfg(feature = "use_groestl")]
use groestl::{Digest, Groestl256};
#[cfg(feature = "use_sha2")]
use openssl::sha::Sha256;
#[cfg(any(feature = "use_keccak", feature = "use_sha3"))]
use tiny_keccak::Keccak;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
use std::collections::hash_map::DefaultHasher;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
pub type TreeHasher = DefaultHasher;
#[cfg(not(any(feature = "use_blake2b")))]
pub type TreeHashResult = Vec<u8>;

#[cfg(feature = "use_blake2b")] pub type TreeHasher = Blake2bHasher;
#[cfg(feature = "use_blake2b")] pub type TreeHashResult = Blake2bHashResult;

#[cfg(feature = "use_groestl")] pub type TreeHasher = GroestlHasher;

#[cfg(feature = "use_sha2")] pub type TreeHasher = Sha256Hasher;

#[cfg(feature = "use_sha3")] pub type TreeHasher = Sha3Hasher;

#[cfg(feature = "use_keccak")] pub type TreeHasher = KeccakHasher;


use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT, NodeVariant};
use crate::traits::*;

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
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeData {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeData {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"), derive(Serialize, Deserialize))]
pub struct TreeNode {
    references: u64,
    node: Option<NodeVariant<TreeBranch, TreeLeaf, TreeData>>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            references: 0,
            node: None,
        }
    }

    fn get_references(&self) -> u64 {
        self.references
    }

    fn set_references(&mut self, references: u64) {
        self.references = references;
    }
    fn set_branch(&mut self, branch: TreeBranch) {
        self.node = Some(NodeVariant::Branch(branch));
    }

    fn set_leaf(&mut self, leaf: TreeLeaf) {
        self.node = Some(NodeVariant::Leaf(leaf));
    }
    fn set_data(&mut self, data: TreeData) {
        self.node = Some(NodeVariant::Data(data));
    }
}

#[cfg(feature = "use_bincode")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serialize(self)?)
    }
}

#[cfg(feature = "use_json")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let encoded = serde_json::to_string(&self)?;
        Ok(encoded.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_cbor")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_yaml::to_vec(&self)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(serde_pickle::to_vec(&self, true)?)
    }
}

#[cfg(feature = "use_ron")]
impl Encode for TreeNode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        Ok(ron::ser::to_string(&self)?.as_bytes().to_vec())
    }
}

#[cfg(feature = "use_bincode")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(deserialize(buffer)?)
    }
}

#[cfg(feature = "use_json")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let decoded_string = String::from_utf8(buffer.to_vec())?;
        let decoded = serde_json::from_str(&decoded_string)?;
        Ok(decoded)
    }
}

#[cfg(feature = "use_cbor")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_cbor::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_yaml")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_yaml::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_pickle")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(serde_pickle::from_slice(buffer)?)
    }
}

#[cfg(feature = "use_ron")]
impl Decode for TreeNode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>> {
        Ok(ron::de::from_bytes(buffer)?)
    }
}

impl Node<TreeBranch, TreeLeaf, TreeData, Vec<u8>> for TreeNode {
    fn new() -> Self { Self::new() }

    fn get_references(&self) -> u64 { Self::get_references(&self) }
    fn get_variant(&self) -> BinaryMerkleTreeResult<NodeVariant<TreeBranch, TreeLeaf, TreeData>> {
        match self.node {
            Some(ref node_type) => {
                match node_type {
                    NodeVariant::Branch(branch) => Ok(NodeVariant::Branch(branch.clone())),
                    NodeVariant::Data(data) => Ok(NodeVariant::Data(data.clone())),
                    NodeVariant::Leaf(leaf) => Ok(NodeVariant::Leaf(leaf.clone()))
                }
            }
            None => Err(Box::new(Exception::new("Failed to distinguish node type")))
        }
    }

    fn set_references(&mut self, references: u64) { Self::set_references(self, references) }
    fn set_branch(&mut self, branch: TreeBranch) { Self::set_branch(self, branch) }
    fn set_leaf(&mut self, leaf: TreeLeaf) { Self::set_leaf(self, leaf) }
    fn set_data(&mut self, data: TreeData) { Self::set_data(self, data) }
}

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self { Self::new() }
    fn update(&mut self, data: &[u8]) { Self::write(self, data) }
    fn finalize(self) -> Self::HashResultType { Self::finish(&self).to_le_bytes().to_vec() }
}

#[cfg(feature = "use_blake2b")]
#[derive(Clone)]
pub struct Blake2bHasher(blake2_rfc::blake2b::Blake2b);

#[cfg(feature = "use_blake2b")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blake2bHashResult(blake2_rfc::blake2b::Blake2bResult);

#[cfg(feature = "use_blake2b")]
impl PartialOrd for Blake2bHashResult {
    fn partial_cmp(&self, other: &Blake2bHashResult) -> Option<Ordering> {
        Some(self.0.as_ref().cmp(&other.0.as_ref()))
    }
}

#[cfg(feature = "use_blake2b")]
impl AsRef<[u8]> for Blake2bHashResult {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "use_blake2b")]
impl crate::traits::Hasher for Blake2bHasher {
    type HashType = Self;
    type HashResultType = Blake2bHashResult;

    fn new(size: usize) -> Self {
        let hasher = blake2_rfc::blake2b::Blake2b::new(size);
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> Self::HashResultType { Blake2bHashResult(self.0.finalize()) }
}

#[cfg(feature = "use_groestl")]
pub struct GroestlHasher(Groestl256);

#[cfg(feature = "use_groestl")]
impl crate::traits::Hasher for GroestlHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Groestl256::new();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.input(data); }
    fn finalize(self) -> Self::HashResultType { self.0.result().to_vec() }
}

#[cfg(feature = "use_sha2")]
pub struct Sha256Hasher(Sha256);

#[cfg(feature = "use_sha2")]
impl crate::traits::Hasher for Sha256Hasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Sha256::new();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data) }
    fn finalize(self) -> Self::HashResultType { self.0.finish().to_vec() }
}

#[cfg(feature = "use_sha3")]
pub struct Sha3Hasher(Keccak);

#[cfg(feature = "use_sha3")]
impl crate::traits::Hasher for Sha3Hasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Keccak::new_sha3_256();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> Self::HashResultType {
        let mut res = vec![0; 32];
        self.0.finalize(&mut res);
        res
    }
}

#[cfg(feature = "use_keccak")]
pub struct KeccakHasher(Keccak);

#[cfg(feature = "use_keccak")]
impl crate::traits::Hasher for KeccakHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Keccak::new_keccak256();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> Self::HashResultType {
        let mut res = vec![0u8; 32];
        self.0.finalize(&mut res);
        res
    }
}

struct HashDB {
    map: HashMap<Vec<u8>, TreeNode>
}

impl HashDB {
    pub fn new(map: HashMap<Vec<u8>, TreeNode>) -> Self {
        Self {
            map
        }
    }
}

impl Database for HashDB {
    type NodeType = TreeNode;
    type EntryType = (Vec<u8>, TreeNode);

    fn open(_path: &PathBuf) -> Result<Self, Box<Error>> { Ok(Self::new(HashMap::new())) }

    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>> {
        if let Some(m) = self.map.get(key) {
            let node = m.clone();
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    fn insert(&mut self, key: &[u8], value: &Self::NodeType) -> Result<(), Box<Error>> {
        self.map.insert(key.to_vec(), value.clone());
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>> {
        self.map.remove(key);
        Ok(())
    }

    fn batch_write(&mut self) -> Result<(), Box<Error>> {
        Ok(())
    }
}

pub struct HashTree {
    tree: MerkleBIT<HashDB, TreeBranch, TreeLeaf, TreeData, TreeNode, TreeHasher, TreeHashResult, Vec<u8>>
}

impl HashTree {
    pub fn new(depth: usize) -> Self {
        let path = PathBuf::new();
        Self {
            tree: MerkleBIT::new(&path, depth).unwrap()
        }
    }

    pub fn get(&self, root_hash: &[u8], keys: &mut Vec<&Vec<u8>>) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&Vec<u8>], values: &mut Vec<&Vec<u8>>) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}