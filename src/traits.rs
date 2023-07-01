#![allow(clippy::std_instead_of_core)]
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "digest")]
use digest::Digest;

use crate::Array;
use std::convert::Infallible;
use std::num::TryFromIntError;

/// The required interface for structs representing a hasher.
pub trait Hasher<const N: usize> {
    /// Creates a new `HashType`.
    fn new(size: usize) -> Self;
    /// Adds data to be hashed.
    fn update(&mut self, data: &[u8]);
    /// Outputs the hash from updated data.
    fn finalize(self) -> Array<N>;
}

#[cfg(feature = "digest")]
impl<T: Digest + Default, const N: usize> Hasher<N> for T {
    fn new(_size: usize) -> Self {
        Self::default()
    }

    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize(self) -> Array<N> {
        #[cfg(feature = "serde")]
        let mut finalized = Array([0; N]);
        #[cfg(not(any(feature = "serde")))]
        let mut finalized = [0; N];
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
pub trait Branch<const N: usize> {
    /// Creates a new `Branch`.
    fn new() -> Self;
    /// Gets the count of leaves beneath this node.
    fn get_count(&self) -> u64;
    /// Gets the location of the zero branch beneath this node.
    fn get_zero(&self) -> &Array<N>;
    /// Gets the location of the one branch beneath this node.
    fn get_one(&self) -> &Array<N>;
    /// Gets the index on which to split keys when traversing this node.
    fn get_split_index(&self) -> usize;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &Array<N>;
    /// Sets the count of leaves below this node.
    fn set_count(&mut self, count: u64);
    /// Sets the location of the zero branch beneath this node.
    fn set_zero(&mut self, zero: Array<N>);
    /// Sets the location of the one branch beneath this node..
    fn set_one(&mut self, one: Array<N>);
    /// Sets the index on which to split keys when traversing this node.
    fn set_split_index(&mut self, index: usize);
    /// Sets the associated key for this node.
    fn set_key(&mut self, key: Array<N>);
    /// Decomposes the `Branch` into its constituent parts.
    fn decompose(self) -> (u64, Array<N>, Array<N>, usize, Array<N>);
}

/// The required interface for structs representing leaves in the tree.
pub trait Leaf<const N: usize> {
    /// Creates a new `Leaf` node.
    fn new() -> Self;
    /// Gets the associated key with this node.
    fn get_key(&self) -> &Array<N>;
    /// Gets the location of the `Data` node.
    fn get_data(&self) -> &Array<N>;
    /// Sets the associated key with this node.
    fn set_key(&mut self, key: Array<N>);
    /// Sets the location of the `Data` node.
    fn set_data(&mut self, data: Array<N>);
    /// Decomposes the `Leaf` into its constituent parts.
    fn decompose(self) -> (Array<N>, Array<N>);
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
pub trait Node<const N: usize> {
    /// The type of `Branch` for the `Node`
    type Branch: Branch<N>;
    /// The type of `Leaf` for the `Node`
    type Leaf: Leaf<N>;
    /// The type of `Data` for the `Node`
    type Data: Data;
    /// Creates a new `Node`.
    fn new(node_variant: NodeVariant<Self::Branch, Self::Leaf, Self::Data, N>) -> Self;
    /// Gets the number of references to this node.
    fn get_references(&self) -> u64;
    /// Decomposes the struct into its inner type.
    fn get_variant(self) -> NodeVariant<Self::Branch, Self::Leaf, Self::Data, N>;
    /// Sets the number of references to this node.
    fn set_references(&mut self, references: u64);
    /// Sets the node to contain a `Branch` node.  Mutually exclusive with `set_data` and `set_leaf`.
    fn set_branch(&mut self, branch: Self::Branch);
    /// Sets the node to contain a `Leaf` node.  Mututally exclusive with `set_data` and `set_branch`.
    fn set_leaf(&mut self, leaf: Self::Leaf);
    /// Sets the node to contain a `Data` node.  Mutually exclusive with `set_leaf` and `set_branch`.
    fn set_data(&mut self, data: Self::Data);
}

/// Contains the distinguishing data from the node
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(any(feature = "serde",), derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum NodeVariant<BranchType: Branch<N>, LeafType: Leaf<N>, DataType: Data, const N: usize> {
    /// Variant containing a `Branch` node.
    Branch(BranchType),
    /// Variant containing a `Leaf` node.
    Leaf(LeafType),
    /// Variant containing a `Data` node.
    Data(DataType),
}

/// This trait defines the required interface for connecting a storage mechanism to the `MerkleBIT`.
pub trait Database<const N: usize, M: Node<N>> {
    /// Opens an existing `Database`.
    /// # Errors
    /// `Exception` generated if the `open` does not succeed.
    fn open(path: &Path) -> Result<Self, MerkleBitError>
    where
        Self: Sized;
    /// Gets a value from the database based on the given key.
    /// # Errors
    /// `Exception` generated if the `get_node` does not succeed.
    fn get_node(&self, key: Array<N>) -> Result<Option<M>, MerkleBitError>;
    /// Queues a key and its associated value for insertion to the database.
    /// # Errors
    /// `Exception` generated if the `insert` does not succeed.
    fn insert(&mut self, key: Array<N>, node: M) -> Result<(), MerkleBitError>;
    /// Removes a key and its associated value from the database.
    /// # Errors
    /// `Exception` generated if the `remove` does not succeed.
    fn remove(&mut self, key: &Array<N>) -> Result<(), MerkleBitError>;
    /// Confirms previous inserts and writes the changes to the database.
    /// # Errors
    /// `Exception` generated if the `batch_write` does not succeed.
    fn batch_write(&mut self) -> Result<(), MerkleBitError>;
}

/// This trait must be implemented to allow a struct to be serialized.
pub trait Encode {
    /// Encodes a struct into bytes.
    /// # Errors
    /// `Exception` generated when the method encoding the structure fails.
    fn encode(&self) -> Result<Vec<u8>, MerkleBitError>;
}

impl Encode for Vec<u8> {
    #[inline]
    fn encode(&self) -> Result<Self, MerkleBitError> {
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
    fn decode(buffer: &[u8]) -> Result<Self, MerkleBitError>
    where
        Self: Sized;
}

impl Decode for Vec<u8> {
    #[inline]
    fn decode(buffer: &[u8]) -> Result<Self, MerkleBitError> {
        Ok(buffer.to_vec())
    }
}

/// An error that results from a corrupt database.  Can happen if the underlying data is modified
/// outside of this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum CorruptTreeError {
    /// A `data` node was found while traversing the tree
    DataInTree,
    /// A node other than `data` was found after traversing a leaf
    NonDataAfterLeaf,
    /// Failed to find the specified `leaf` in the db
    NoLeafFromDB,
    /// Failed to find the specified `node` in the db
    NoNodeFromDB,
    /// Found a `leaf` node when expecting a `branch`
    MisplacedLeaf,
}

/// A generic error that implements `Error`.
/// Mostly intended to be used to standardize errors across the crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum MerkleBitError {
    /// Failed a checked integer conversion
    TryFromIntError(TryFromIntError),
    /// The maximum depth of the tree has been exceeded
    DepthExceeded(usize),
    /// An error that indicates the tree is corrupt
    CorruptTree(CorruptTreeError),
    /// The length of the given keys and values are not the same
    KeyValueLengthMismatch((usize, usize)),
    /// No keys or values were given
    EmptyKeysOrValues,
    /// Failed to generate a root
    NoRoot,
    /// Tree refs were empty during tree generation
    EmptyTreeRefs,
    /// No nodes were specified
    NoNodes,
    /// Failed to find the specified key
    KeyNotPresent,
    /// The inclusion proof is too short
    ProofTooShort,
    /// The inclusion proof is not valid
    InvalidProof,
    /// Failed to generate a level in tree generation
    EmptyLevel,
    /// Failed to get the bit in the key
    InvalidKeyBit(usize),
    /// Infallible
    Infallible,
    /// Failed to find either a `min_key` or a `max_key`
    NoKeys,
    /// Attempted to insert a duplicate key
    DuplicateKey,
    #[cfg(feature = "bincode")]
    Bincode(Box<bincode::ErrorKind>),
    #[cfg(feature = "cbor")]
    CborSerialization(ciborium::ser::Error<std::io::Error>),
    #[cfg(feature = "cbor")]
    CborDeserialization(ciborium::de::Error<std::io::Error>),
    #[cfg(feature = "json")]
    Json(serde_json::Error),
    #[cfg(feature = "json")]
    FromUtf8Error(std::string::FromUtf8Error),
    #[cfg(feature = "yaml")]
    Yaml(serde_yaml::Error),
    #[cfg(feature = "pickle")]
    Pickle(serde_pickle::Error),
    #[cfg(feature = "ron")]
    Ron(ron::error::Error),
    #[cfg(feature = "ron")]
    RonSpanned(ron::error::SpannedError),
    #[cfg(feature = "rocksdb")]
    RocksDb(rocksdb::Error),
}

impl Display for MerkleBitError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl Error for MerkleBitError {}

impl From<TryFromIntError> for MerkleBitError {
    #[inline]
    fn from(err: TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

impl From<Infallible> for MerkleBitError {
    #[inline]
    fn from(_err: Infallible) -> Self {
        Self::Infallible
    }
}

impl From<CorruptTreeError> for MerkleBitError {
    #[inline]
    fn from(err: CorruptTreeError) -> Self {
        Self::CorruptTree(err)
    }
}
