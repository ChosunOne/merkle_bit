#[cfg(feature = "hashbrown")]
pub mod hashbrown;
/// The module containing the implementation of a DB using a `HashMap`.
#[cfg(not(feature = "hashbrown"))]
pub mod hashmap;
#[cfg(feature = "rocksdb")]
pub mod rocksdb;

/// The type of database for the `HashTree`.
#[cfg(not(feature = "hashbrown"))]
pub type HashTreeDB<ArrayType> = crate::tree_db::hashmap::HashDB<ArrayType>;
#[cfg(feature = "hashbrown")]
pub type HashTreeDB<ArrayType> = crate::tree_db::hashbrown::HashDB<ArrayType>;
