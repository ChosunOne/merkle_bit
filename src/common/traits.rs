use std::error::Error;
use std::path::PathBuf;

use common::binary_merkle_tree::BinaryMerkleTreeResult;
use common::binary_merkle_tree::NodeVariant;
use common::{Decode, Encode};

pub trait Hasher {
    type HashType;
    type HashResultType;
    fn new(size: usize) -> Self::HashType;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> Self::HashResultType;
}

pub trait Branch {
    fn new() -> Self;
    fn get_count(&self) -> u64;
    fn get_zero(&self) -> &[u8];
    fn get_one(&self) -> &[u8];
    fn get_split_index(&self) -> u32;
    fn set_count(&mut self, count: u64);
    fn set_zero(&mut self, zero: &[u8]);
    fn set_one(&mut self, one: &[u8]);
    fn set_split_index(&mut self, index: u32);
}

pub trait Leaf {
    fn new() -> Self;
    fn get_key(&self) -> &[u8];
    fn get_data(&self) -> &[u8];
    fn set_key(&mut self, key: &[u8]);
    fn set_data(&mut self, data: &[u8]);
}

pub trait Data {
    fn new() -> Self;
    fn get_value(&self) -> &[u8];
    fn set_value(&mut self, value: &[u8]);
}

pub trait Node<BranchType, LeafType, DataType, ValueType>
    where BranchType: Branch,
          LeafType: Leaf,
          DataType: Data,
          ValueType: Decode + Encode {
    fn new() -> Self;
    fn get_references(&self) -> u64;
    fn get_variant(&self) -> BinaryMerkleTreeResult<NodeVariant<BranchType, LeafType, DataType>>;
    fn set_references(&mut self, references: u64);
    fn set_branch(&mut self, branch: BranchType);
    fn set_leaf(&mut self, leaf: LeafType);
    fn set_data(&mut self, data: DataType);
}

pub trait IDB {
    type NodeType;
    type EntryType;
    fn open(path: PathBuf) -> Result<Self, Box<Error>> where Self: Sized;
    fn get_node(&self, key: &[u8]) -> Result<Option<Self::NodeType>, Box<Error>>;
    fn insert(&mut self, key: &[u8], node: &Self::NodeType) -> Result<(), Box<Error>>;
    fn remove(&mut self, key: &[u8]) -> Result<(), Box<Error>>;
    fn batch_write(&mut self) -> Result<(), Box<Error>>;
}