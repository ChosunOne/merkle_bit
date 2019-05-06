#[cfg(feature = "use_blake2b")]
pub mod blake2b;
/// The default Rust hashing function expanded to 32 bytes.
#[cfg(not(any(
    feature = "use_blake2b",
    feature = "use_groestl",
    feature = "use_sha2",
    feature = "use_sha3",
    feature = "use_keccak",
    feature = "use_seahash",
)))]
pub mod default;
#[cfg(feature = "use_groestl")]
pub mod groestl;
#[cfg(feature = "use_keccak")]
pub mod keccak;
/// Holds the implementation of `crate::traits::Hasher` for `SeaHasher`
#[cfg(feature = "use_seahash")]
pub mod seahasher;
#[cfg(feature = "use_sha2")]
pub mod sha256;
#[cfg(feature = "use_sha3")]
pub mod sha3;

/// The kind of hasher to use in the tree.
#[cfg(not(any(
    feature = "use_blake2b",
    feature = "use_groestl",
    feature = "use_sha2",
    feature = "use_sha3",
    feature = "use_keccak",
    feature = "use_seahash",
)))]
pub type TreeHasher = std::collections::hash_map::DefaultHasher;

#[cfg(feature = "use_blake2b")]
pub type TreeHasher = blake2b::Blake2bHasher;

#[cfg(feature = "use_groestl")]
pub type TreeHasher = groestl::GroestlHasher;
#[cfg(feature = "use_sha2")]
pub type TreeHasher = sha256::Sha256Hasher;
#[cfg(feature = "use_sha3")]
pub type TreeHasher = sha3::Sha3Hasher;
#[cfg(feature = "use_keccak")]
pub type TreeHasher = keccak::KeccakHasher;

/// The kind of hasher to use in the tree.
#[cfg(feature = "use_seahash")]
pub type TreeHasher = seahash::SeaHasher;
