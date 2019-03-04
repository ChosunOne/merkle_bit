#[cfg(feature = "use_bincode")] extern crate bincode;
#[cfg(any(feature = "use_serde", feature = "use_bincode", feature = "use_json", feature = "use_cbor", feature = "use_yaml", feature = "use_pickle", feature = "use_ron"))] extern crate serde;
#[cfg(feature = "use_json")] extern crate serde_json;
#[cfg(feature = "use_cbor")] extern crate serde_cbor;
#[cfg(feature = "use_yaml")] extern crate serde_yaml;
#[cfg(feature = "use_pickle")] extern crate serde_pickle;
#[cfg(feature = "use_ron")] extern crate ron;

#[cfg(feature = "use_blake2b")] extern crate blake2_rfc;
#[cfg(feature = "use_groestl")] extern crate groestl;
#[cfg(feature = "use_sha2")] extern crate openssl;
#[cfg(feature = "use_sha3")] extern crate tiny_keccak;
#[cfg(feature = "use_keccak")] extern crate tiny_keccak;

pub mod merkle_bit;
pub mod traits;
pub mod hash_tree;
pub mod tree;
pub mod tree_hasher;