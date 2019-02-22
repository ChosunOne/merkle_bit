#[cfg(feature = "default_tree")] extern crate bincode;
#[cfg(feature = "default_tree")] extern crate blake2_rfc;
#[cfg(feature = "default_tree")] extern crate serde;

pub mod merkle_bit;
pub mod traits;
pub mod tree;