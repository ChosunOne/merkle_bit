// Clippy configurations
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::implicit_return)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::module_name_repetitions)]

/// Defines constants for the tree.
pub mod constants;
/// An implementation of the `MerkleBIT` with a `HashMap` backend database.
pub mod hash_tree;
/// Contains the actual operations of inserting, getting, and removing items from a tree.
pub mod merkle_bit;
/// Contains the traits necessary for tree operations
pub mod traits;
/// Contains a collection of structs for representing locations within the tree.
pub mod tree;
/// Contains a collection of structs for implementing tree databases.
pub mod tree_db;
/// Contains a collection of structs for implementing hashing functions in the tree.
pub mod tree_hasher;
/// Contains a collection of useful structs and functions for tree operations.
pub mod utils;

/// An implementation of the `MerkleBIT` with a `RocksDB` backend database.
#[cfg(feature = "use_rocksdb")]
pub mod rocks_tree;
