#![allow(unknown_lints)]
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
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::module_name_repetitions)]

pub mod constants;
pub mod hash_tree;
pub mod merkle_bit;
pub mod traits;
pub mod tree;
/// Contains a collection of structs for implementing tree databases.
pub mod tree_db;
/// Contains a collection of structs for implementing hashing functions in the tree.
pub mod tree_hasher;
/// Contains a collection of useful structs and functions for tree operations.
pub mod utils;

#[cfg(feature = "use_rocksdb")]
pub mod rocks_tree;
