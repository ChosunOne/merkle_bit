use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "use_digest")]
use digest::Digest;

use crate::constants::KEY_LEN;

/// The required interface for structs representing a hasher.
pub trait Hasher {
    /// The type of hasher.
    type HashType;
    /// Creates a new `HashType`.
    fn new(size: usize) -> Self::HashType;
    /// Adds data to be hashed.
    fn update(&mut self, data: &[u8]);
    /// Outputs the hash from updated data.
    fn finalize(self) -> [u8; KEY_LEN];
}

#[cfg(feature = "use_digest")]
impl<T> Hasher for T
    where T: Digest {
    type HashType = T;

    fn new(_size: usize) -> Self::HashType {
        Self::HashType::new()
    }

    fn update(&mut self, data: &[u8]) {
        self.input(data);
    }

    fn finalize(self) -> [u8; KEY_LEN] {
        let mut finalized = [0u8; KEY_LEN];
        let result = self.result();
        if result.len() < KEY_LEN {
            finalized[0..result.len()].copy_from_slice(&result[0..result.len()])
        } else {
            finalized.copy_from_slice(&result[0..KEY_LEN]);
        }
        finalized
    }
}

/// The required interface for structs representing branches in the tree.
pub trait Branch {
    /// Creates a new `Branch`.
    fn new() -> Self;
    /// Gets the count of leaves beneath this node.
    fn get_count(&self) -> u64;
    /// Gets the location of the zero branch beneath this node.
    fn get_zero(&self) -> &[u8; KEY_LEN];
    /// Gets the location of the one branch beneath this node.
    fn get_one(&self) -> &[u8; KEY_LEN];
    /// Gets the index on which to split keys when traversing this node.
    fn get_split_index(&self) -> usize;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &[u8; KEY_LEN];
    /// Sets the count of leaves below this node.
    fn set_count(&mut self, count: u64);
    /// Sets the location of the zero branch beneath this node.
    fn set_zero(&mut self, zero: [u8; KEY_LEN]);
    /// Sets the location of the one branch beneath this node..
    fn set_one(&mut self, one: [u8; KEY_LEN]);
    /// Sets the index on which to split keys when traversing this node.
    fn set_split_index(&mut self, index: usize);
    /// Sets the associated key for this node.
    fn set_key(&mut self, key: [u8; KEY_LEN]);
    /// Decomposes the `Branch` into its constituent parts.
    fn decompose(self) -> (u64, [u8; KEY_LEN], [u8; KEY_LEN], usize, [u8; KEY_LEN]);
}

/// The required interface for structs representing leaves in the tree.
pub trait Leaf {
    /// Creates a new `Leaf` node.
    fn new() -> Self;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &[u8; KEY_LEN];
    /// Gets the location of the `Data` node.
    fn get_data(&self) -> &[u8; KEY_LEN];
    /// Sets the associated key with this node.
    fn set_key(&mut self, key: [u8; KEY_LEN]);
    /// Sets the location of the `Data` node.
    fn set_data(&mut self, data: [u8; KEY_LEN]);
    /// Decomposes the `Leaf` into its constituent parts.
    fn decompose(self) -> ([u8; KEY_LEN], [u8; KEY_LEN]);
}

/// The required interface for structs representing data stored in the tree.
pub trait Data {
    /// Creates a new `Data` node.
    fn new() -> Self;
    /// Gets the value for the `Data` node.
    fn get_value(&self) -> &[u8];
    /// Sets the value for the `Data` node.
    fn set_value(&mut self, value: &[u8]);
}

/// The required interface for structs representing nodes in the tree.
pub trait Node<BranchType, LeafType, DataType>
where
    BranchType: Branch,
    LeafType: Leaf,
    DataType: Data,
{
    /// Creates a new `Node`.
    fn new(node_variant: NodeVariant<BranchType, LeafType, DataType>) -> Self;
    /// Gets the number of references to this node.
    fn get_references(&self) -> u64;
    /// Decomposes the struct into its inner type.
    fn get_variant(self) -> NodeVariant<BranchType, LeafType, DataType>;
    /// Sets the number of references to this node.
    fn set_references(&mut self, references: u64);
    /// Sets the node to contain a `Branch` node.  Mutually exclusive with `set_data` and `set_leaf`.
    fn set_branch(&mut self, branch: BranchType);
    /// Sets the node to contain a `Leaf` node.  Mututally exclusive with `set_data` and `set_branch`.
    fn set_leaf(&mut self, leaf: LeafType);
    /// Sets the node to contain a `Data` node.  Mutually exclusive with `set_leaf` and `set_branch`.
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
    /// Variant containing a `Branch` node.
    Branch(BranchType),
    /// Variant containing a `Leaf` node.
    Leaf(LeafType),
    /// Variant containing a `Data` node.
    Data(DataType),
}

/// This trait defines the required interface for connecting a storage mechanism to the `MerkleBIT`.
pub trait Database {
    /// The type of node to insert into the database.
    type NodeType;
    /// The type of entry for insertion.  Primarily for convenience and tracking what goes into the database.
    type EntryType;
    /// Opens an existing `Database`.
    fn open(path: &PathBuf) -> Result<Self, Exception>
    where
        Self: Sized;
    /// Gets a value from the database based on the given key.
    fn get_node(&self, key: &[u8; KEY_LEN]) -> Result<Option<Self::NodeType>, Exception>;
    /// Queues a key and its associated value for insertion to the database.
    fn insert(&mut self, key: [u8; KEY_LEN], node: Self::NodeType) -> Result<(), Exception>;
    /// Removes a key and its associated value from the database.
    fn remove(&mut self, key: &[u8; KEY_LEN]) -> Result<(), Exception>;
    /// Confirms previous inserts and writes the changes to the database.
    fn batch_write(&mut self) -> Result<(), Exception>;
}

/// This trait must be implemented to allow a struct to be serialized.
pub trait Encode {
    /// Encodes a struct into bytes.
    fn encode(&self) -> Result<Vec<u8>, Exception>;
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode(&self) -> Result<Self, Exception> {
        Ok(self.clone())
    }
}

/// This trait must be implemented to allow an arbitrary sized buffer to be deserialized.
pub trait Decode {
    /// Decodes bytes into a `Sized` struct.
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

/// A generic error that implements `Error`.
/// Mostly intended to be used to standardize errors across the crate.
#[derive(Debug)]
pub struct Exception {
    /// The details of an exception
    details: String,
}

impl Exception {
    /// Creates a new `Exception`.
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
