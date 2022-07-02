use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "use_digest")]
use digest::Digest;

use std::convert::Infallible;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::TryFromIntError;

/// The required interface for an object that functions like an array.
pub trait Array: AsRef<[u8]> + AsMut<[u8]> + Clone + Copy + Default + Hash + Ord + Sized {}

impl<T> Array for T where T: AsRef<[u8]> + AsMut<[u8]> + Clone + Copy + Default + Hash + Ord + Sized {}

/// The required interface for structs representing a hasher.
pub trait Hasher<ArrayType>
where
    ArrayType: Array,
{
    /// The type of hasher.
    type HashType;
    /// Creates a new `HashType`.
    fn new(size: usize) -> Self::HashType;
    /// Adds data to be hashed.
    fn update(&mut self, data: &[u8]);
    /// Outputs the hash from updated data.
    fn finalize(self) -> ArrayType;
}

#[cfg(feature = "use_digest")]
impl<T, ArrayType> Hasher<ArrayType> for T
where
    T: Digest,
    ArrayType: Array,
{
    type HashType = T;

    fn new(_size: usize) -> Self::HashType {
        Self::HashType::new()
    }

    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize(self) -> ArrayType {
        let mut finalized = ArrayType::default();
        let result = self.finalize();
        let mut size = finalized.as_ref().len();
        if size > result.len() {
            size = result.len();
        }
        finalized.as_mut()[..size].copy_from_slice(&result[..size]);
        finalized
    }
}

/// The required interface for structs representing branches in the tree.
pub trait Branch<ArrayType>
where
    ArrayType: Array,
{
    /// Creates a new `Branch`.
    fn new() -> Self;
    /// Gets the count of leaves beneath this node.
    fn get_count(&self) -> u64;
    /// Gets the location of the zero branch beneath this node.
    fn get_zero(&self) -> &ArrayType;
    /// Gets the location of the one branch beneath this node.
    fn get_one(&self) -> &ArrayType;
    /// Gets the index on which to split keys when traversing this node.
    fn get_split_index(&self) -> usize;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &ArrayType;
    /// Sets the count of leaves below this node.
    fn set_count(&mut self, count: u64);
    /// Sets the location of the zero branch beneath this node.
    fn set_zero(&mut self, zero: ArrayType);
    /// Sets the location of the one branch beneath this node..
    fn set_one(&mut self, one: ArrayType);
    /// Sets the index on which to split keys when traversing this node.
    fn set_split_index(&mut self, index: usize);
    /// Sets the associated key for this node.
    fn set_key(&mut self, key: ArrayType);
    /// Decomposes the `Branch` into its constituent parts.
    fn decompose(self) -> (u64, ArrayType, ArrayType, usize, ArrayType);
}

/// The required interface for structs representing leaves in the tree.
pub trait Leaf<ArrayType>
where
    ArrayType: Array,
{
    /// Creates a new `Leaf` node.
    fn new() -> Self;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &ArrayType;
    /// Gets the location of the `Data` node.
    fn get_data(&self) -> &ArrayType;
    /// Sets the associated key with this node.
    fn set_key(&mut self, key: ArrayType);
    /// Sets the location of the `Data` node.
    fn set_data(&mut self, data: ArrayType);
    /// Decomposes the `Leaf` into its constituent parts.
    fn decompose(self) -> (ArrayType, ArrayType);
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
pub trait Node<BranchType, LeafType, DataType, ArrayType>
where
    BranchType: Branch<ArrayType>,
    LeafType: Leaf<ArrayType>,
    DataType: Data,
    ArrayType: Array,
{
    /// Creates a new `Node`.
    fn new(node_variant: NodeVariant<BranchType, LeafType, DataType, ArrayType>) -> Self;
    /// Gets the number of references to this node.
    fn get_references(&self) -> u64;
    /// Decomposes the struct into its inner type.
    fn get_variant(self) -> NodeVariant<BranchType, LeafType, DataType, ArrayType>;
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
#[cfg_attr(any(feature = "serde",), derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum NodeVariant<
    BranchType: Branch<ArrayType>,
    LeafType: Leaf<ArrayType>,
    DataType: Data,
    ArrayType: Array,
> {
    /// Variant containing a `Branch` node.
    Branch(BranchType),
    /// Variant containing a `Leaf` node.
    Leaf(LeafType),
    /// Variant containing a `Data` node.
    Data(DataType),
    /// Marker for `KeyType`.
    Phantom(PhantomData<ArrayType>),
}

/// This trait defines the required interface for connecting a storage mechanism to the `MerkleBIT`.
pub trait Database<ArrayType>
where
    ArrayType: Array,
{
    /// The type of node to insert into the database.
    type NodeType;
    /// The type of entry for insertion.  Primarily for convenience and tracking what goes into the database.
    type EntryType;
    /// Opens an existing `Database`.
    /// # Errors
    /// `Exception` generated if the `open` does not succeed.
    fn open(path: &Path) -> Result<Self, Exception>
    where
        Self: Sized;
    /// Gets a value from the database based on the given key.
    /// # Errors
    /// `Exception` generated if the `get_node` does not succeed.
    fn get_node(&self, key: ArrayType) -> Result<Option<Self::NodeType>, Exception>;
    /// Queues a key and its associated value for insertion to the database.
    /// # Errors
    /// `Exception` generated if the `insert` does not succeed.
    fn insert(&mut self, key: ArrayType, node: Self::NodeType) -> Result<(), Exception>;
    /// Removes a key and its associated value from the database.
    /// # Errors
    /// `Exception` generated if the `remove` does not succeed.
    fn remove(&mut self, key: &ArrayType) -> Result<(), Exception>;
    /// Confirms previous inserts and writes the changes to the database.
    /// # Errors
    /// `Exception` generated if the `batch_write` does not succeed.
    fn batch_write(&mut self) -> Result<(), Exception>;
}

/// This trait must be implemented to allow a struct to be serialized.
pub trait Encode {
    /// Encodes a struct into bytes.
    /// # Errors
    /// `Exception` generated when the method encoding the structure fails.
    fn encode(&self) -> Result<Vec<u8>, Exception>;
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode(&self) -> Result<Self, Exception> {
        Ok(self.clone())
    }
}

/// This trait must be implemented to allow an arbitrary sized buffer to be deserialized.
/// # Errors
/// `Exception` generated when the buffer fails to be decoded to the target type.
pub trait Decode {
    /// Decodes bytes into a `Sized` struct.
    /// # Errors
    /// `Exception` generated when the buffer fails to be decoded to the target type.
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
    #[must_use]
    pub fn new(details: &str) -> Self {
        Self {
            details: details.to_owned(),
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

impl From<Infallible> for Exception {
    #[inline]
    fn from(_inf: Infallible) -> Self {
        Self::new("Infallible")
    }
}

impl From<TryFromIntError> for Exception {
    #[inline]
    fn from(err: TryFromIntError) -> Self {
        Self::new(&err.to_string())
    }
}
