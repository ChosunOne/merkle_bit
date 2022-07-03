#[cfg(feature = "hashbrown")]
pub mod hashbrown;
/// The module containing the implementation of a DB using a `HashMap`.
#[cfg(not(feature = "hashbrown"))]
pub mod hashmap;
#[cfg(feature = "rocksdb")]
pub mod rocksdb;

/// The type of database for the `HashTree`.
#[cfg(not(feature = "hashbrown"))]
pub type HashTreeDB<const N: usize> = crate::tree_db::hashmap::HashDB<N>;
#[cfg(feature = "hashbrown")]
pub type HashTreeDB<const N: usize> = crate::tree_db::hashbrown::HashDB<N>;
