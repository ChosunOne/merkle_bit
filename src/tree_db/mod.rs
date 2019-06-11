#[cfg(feature = "use_hashbrown")]
pub mod hashbrown;
/// The module containing the implementation of a DB using a `HashMap`.
#[cfg(not(feature = "use_hashbrown"))]
pub mod hashmap;
#[cfg(feature = "use_rocksdb")]
pub mod rocksdb;

/// The type of database for the `HashTree`.
#[cfg(not(feature = "use_hashbrown"))]
pub type HashTreeDB<KeyType> = crate::tree_db::hashmap::HashDB<KeyType>;
#[cfg(feature = "use_hashbrown")]
pub type HashTreeDB<KeyType> = crate::tree_db::hashbrown::HashDB<KeyType>;
