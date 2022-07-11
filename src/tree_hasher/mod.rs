#[cfg(feature = "blake2-rfc")]
pub mod blake2b_rfc;
/// The default Rust hashing function expanded to 32 bytes.
#[cfg(not(any(
    feature = "blake2-rfc",
    feature = "sha2",
    feature = "sha3",
    feature = "keccak",
    feature = "seahash",
    feature = "fxhash",
    feature = "digest"
)))]
pub mod default;
#[cfg(feature = "fxhash")]
pub mod fx;
#[cfg(feature = "keccak")]
pub mod keccak;
/// Holds the implementation of `crate::traits::Hasher` for `SeaHasher`
#[cfg(feature = "seahash")]
pub mod seahasher;
#[cfg(feature = "sha2")]
pub mod sha256;
#[cfg(feature = "sha3")]
pub mod sha3_openssl;

/// The kind of hasher to use in the tree.
#[cfg(not(any(
    feature = "blake2-rfc",
    feature = "sha2",
    feature = "sha3",
    feature = "keccak",
    feature = "seahash",
    feature = "fxhash",
    feature = "digest"
)))]
pub type TreeHasher = std::collections::hash_map::DefaultHasher;

#[cfg(feature = "blake2-rfc")]
pub type TreeHasher = blake2b_rfc::Blake2bHasher;

#[cfg(feature = "groestl")]
pub type TreeHasher = groestl::Groestl256;
#[cfg(feature = "sha2")]
pub type TreeHasher = sha256::Sha256Hasher;
#[cfg(feature = "sha3")]
pub type TreeHasher = sha3_openssl::Sha3Hasher;
#[cfg(feature = "keccak")]
pub type TreeHasher = keccak::KeccakHasher;
#[cfg(feature = "blake2b")]
pub type TreeHasher = blake2::Blake2b512;
#[cfg(feature = "md2")]
pub type TreeHasher = md2::Md2;
#[cfg(feature = "md4")]
pub type TreeHasher = md4::Md4;
#[cfg(feature = "md5")]
pub type TreeHasher = md5::Md5;
#[cfg(feature = "ripemd160")]
pub type TreeHasher = ripemd::Ripemd160;
#[cfg(feature = "ripemd320")]
pub type TreeHasher = ripemd::Ripemd320;
#[cfg(feature = "sha1")]
pub type TreeHasher = sha1::Sha1;
#[cfg(feature = "rust_sha2")]
pub type TreeHasher = sha2::Sha256;
#[cfg(feature = "rust_sha3")]
pub type TreeHasher = sha3::Sha3_256;
#[cfg(feature = "rust_keccak")]
pub type TreeHasher = sha3::Keccak256;
#[cfg(feature = "whirlpool")]
pub type TreeHasher = whirlpool::Whirlpool;
/// The kind of hasher to use in the tree.
#[cfg(feature = "seahash")]
pub type TreeHasher = seahash::SeaHasher;
#[cfg(feature = "fxhash")]
pub type TreeHasher = fxhash::FxHasher;
