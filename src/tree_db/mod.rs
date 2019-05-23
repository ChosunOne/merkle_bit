#[cfg(feature = "use_hashbrown")]
pub mod hashbrown;
/// The module containing the implementation of a DB using a `HashMap`.
#[cfg(not(any(feature = "use_hashbrown", feature = "use_evmap")))]
pub mod hashmap;
#[cfg(feature = "use_rocksdb")]
pub mod rocksdb;

/// The type of database for the `HashTree`.
#[cfg(not(any(feature = "use_hashbrown", feature = "use_evmap")))]
pub type HashTreeDB = crate::tree_db::hashmap::HashDB;
#[cfg(feature = "use_hashbrown")]
pub type HashTreeDB = crate::tree_db::hashbrown::HashDB;
