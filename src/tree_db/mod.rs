#[cfg(feature = "use_hashbrown")]
pub mod hashbrown;
/// The module containing the implementation of a DB using a `HashMap`.
#[cfg(not(feature = "use_hashbrown"))]
pub mod hashmap;
#[cfg(feature = "use_rocksdb")]
pub mod rocksdb;

/// The type of database for the `HashTree`.
#[cfg(not(feature = "use_hashbrown"))]
pub type HashTreeDB<ArrayType> = crate::tree_db::hashmap::HashDB<ArrayType>;
#[cfg(feature = "use_hashbrown")]
pub type HashTreeDB<ArrayType> = crate::tree_db::hashbrown::HashDB<ArrayType>;
