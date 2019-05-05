use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};

use crate::constants::KEY_LEN;

pub trait Hasher {
    type HashType;
    fn new(size: usize) -> Self::HashType;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> [u8; KEY_LEN];
}

pub trait Branch {
    fn new() -> Self;
    fn get_count(&self) -> u64;
    fn get_zero(&self) -> &[u8; KEY_LEN];
    fn get_one(&self) -> &[u8; KEY_LEN];
    fn get_split_index(&self) -> u8;
    fn get_key(&self) -> &[u8; KEY_LEN];
    fn set_count(&mut self, count: u64);
    fn set_zero(&mut self, zero: [u8; KEY_LEN]);
    fn set_one(&mut self, one: [u8; KEY_LEN]);
    fn set_split_index(&mut self, index: u8);
    fn set_key(&mut self, key: [u8; KEY_LEN]);
    fn deconstruct(self) -> (u64, [u8; KEY_LEN], [u8; KEY_LEN], u8, [u8; KEY_LEN]);
}

pub trait Leaf {
    fn new() -> Self;
    fn get_key(&self) -> &[u8; KEY_LEN];
    fn get_data(&self) -> &[u8; KEY_LEN];
    fn set_key(&mut self, key: [u8; KEY_LEN]);
    fn set_data(&mut self, data: [u8; KEY_LEN]);
    fn deconstruct(self) -> ([u8; KEY_LEN], [u8; KEY_LEN]);
}

pub trait Data {
    fn new() -> Self;
    fn get_value(&self) -> &[u8];
    fn set_value(&mut self, value: &[u8]);
}

pub trait Node<BranchType, LeafType, DataType>
where
    BranchType: Branch,
    LeafType: Leaf,
    DataType: Data,
{
    fn new(node_variant: NodeVariant<BranchType, LeafType, DataType>) -> Self;
    fn get_references(&self) -> u64;
    fn get_variant(self) -> NodeVariant<BranchType, LeafType, DataType>;
    fn set_references(&mut self, references: u64);
    fn set_branch(&mut self, branch: BranchType);
    fn set_leaf(&mut self, leaf: LeafType);
    fn set_data(&mut self, data: DataType);
}

/// Contains the distinguishing data from the node
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "use_serde",), derive(Serialize, Deserialize))]
pub enum NodeVariant<BranchType, LeafType, DataType>
where
    BranchType: Branch,
    LeafType: Leaf,
    DataType: Data,
{
    Branch(BranchType),
    Leaf(LeafType),
    Data(DataType),
}

pub trait Database {
    type NodeType;
    type EntryType;
    fn open(path: &PathBuf) -> Result<Self, Exception>
    where
        Self: Sized;
    fn get_node(&self, key: &[u8; KEY_LEN]) -> Result<Option<Self::NodeType>, Exception>;
    fn insert(&mut self, key: [u8; KEY_LEN], node: Self::NodeType) -> Result<(), Exception>;
    fn remove(&mut self, key: &[u8; KEY_LEN]) -> Result<(), Exception>;
    fn batch_write(&mut self) -> Result<(), Exception>;
}

pub trait Encode {
    fn encode(&self) -> Result<Vec<u8>, Exception>;
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode(&self) -> Result<Self, Exception> {
        Ok(self.clone())
    }
}

pub trait Decode {
    fn decode(buffer: &[u8]) -> Result<Self, Exception>
    where
        Self: Sized;
}

impl Decode for Vec<u8> {
    #[inline]
    fn decode(buffer: &[u8]) -> Result<Self, Exception> {
        Ok(buffer.to_vec())
    }
}

#[derive(Debug)]
pub struct Exception {
    details: String,
}

impl Exception {
    #[inline]
    pub fn new(details: &str) -> Self {
        Self {
            details: details.to_string(),
        }
    }
}

impl Display for Exception {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.details)
    }
}

impl Error for Exception {
    #[inline]
    fn description(&self) -> &str {
        &self.details
    }
}
