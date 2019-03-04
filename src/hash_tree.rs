use std::collections::hash_map::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::tree::tree_branch::TreeBranch;
use crate::tree::tree_leaf::TreeLeaf;
use crate::tree::tree_data::TreeData;
use crate::tree::tree_node::TreeNode;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
use std::hash::Hasher;

#[cfg(feature = "use_blake2b")]
use std::cmp::Ordering;

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


use crate::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use crate::traits::*;

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

#[cfg(not(any(feature = "use_hashbrown")))]
struct HashDB {
    map: HashMap<Vec<u8>, TreeNode>
}

#[cfg(not(any(feature = "use_hashbrown")))]
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
        let tree = MerkleBIT::new(&path, depth).expect("Creating a HashTree should not fail");
        Self {
            tree
        }
    }

    pub fn get(&self, root_hash: &[u8], keys: &mut [&[u8]]) -> BinaryMerkleTreeResult<Vec<Option<Vec<u8>>>> {
        self.tree.get(root_hash, keys)
    }

    pub fn insert(&mut self, previous_root: Option<&[u8]>, keys: &mut [&[u8]], values: &mut Vec<&Vec<u8>>) -> BinaryMerkleTreeResult<Vec<u8>> {
        self.tree.insert(previous_root, keys, values)
    }

    pub fn remove(&mut self, root_hash: &[u8]) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }
}