#[cfg(feature = "use_blake2b_rfc")]
pub mod blake2b_rfc;
/// The default Rust hashing function expanded to 32 bytes.
#[cfg(not(any(
    feature = "use_blake2b_rfc",
    feature = "use_sha2",
    feature = "use_sha3",
    feature = "use_keccak",
    feature = "use_seahash",
    feature = "use_fx",
    feature = "use_digest"
)))]
pub mod default;
#[cfg(feature = "use_keccak")]
pub mod keccak;
/// Holds the implementation of `crate::traits::Hasher` for `SeaHasher`
#[cfg(feature = "use_seahash")]
pub mod seahasher;
#[cfg(feature = "use_sha2")]
pub mod sha256;
#[cfg(feature = "use_sha3")]
pub mod sha3_openssl;
#[cfg(feature = "use_fx")]
pub mod fx;

/// The kind of hasher to use in the tree.
#[cfg(not(any(
    feature = "use_blake2b_rfc",
    feature = "use_sha2",
    feature = "use_sha3",
    feature = "use_keccak",
    feature = "use_seahash",
    feature = "use_fx",
    feature = "use_digest"
)))]
pub type TreeHasher = std::collections::hash_map::DefaultHasher;

#[cfg(feature = "use_blake2b_rfc")]
pub type TreeHasher = blake2b_rfc::Blake2bHasher;

#[cfg(feature = "use_groestl")]
pub type TreeHasher = groestl::Groestl256;
#[cfg(feature = "use_sha2")]
pub type TreeHasher = sha256::Sha256Hasher;
#[cfg(feature = "use_sha3")]
pub type TreeHasher = sha3_openssl::Sha3Hasher;
#[cfg(feature = "use_keccak")]
pub type TreeHasher = keccak::KeccakHasher;
#[cfg(feature = "use_blake2b")]
pub type TreeHasher = blake2::Blake2b;
#[cfg(feature = "use_md2")]
pub type TreeHasher = md2::Md2;
#[cfg(feature = "use_md4")]
pub type TreeHasher = md4::Md4;
#[cfg(feature = "use_md5")]
pub type TreeHasher = md5::Md5;
#[cfg(feature = "use_ripemd160")]
pub type TreeHasher = ripemd160::Ripemd160;
#[cfg(feature = "use_ripemd320")]
pub type TreeHasher = ripemd320::Ripemd320;
#[cfg(feature = "use_sha1")]
pub type TreeHasher = sha1::Sha1;
#[cfg(feature = "use_rust_sha2")]
pub type TreeHasher = sha2::Sha256;
#[cfg(feature = "use_rust_sha3")]
pub type TreeHasher = sha3::Sha3_256;
#[cfg(feature = "use_rust_keccak")]
pub type TreeHasher = sha3::Keccak256;
#[cfg(feature = "use_whirlpool")]
pub type TreeHasher = whirlpool::Whirlpool;
/// The kind of hasher to use in the tree.
#[cfg(feature = "use_seahash")]
pub type TreeHasher = seahash::SeaHasher;
#[cfg(feature = "use_fx")]
pub type TreeHasher = fxhash::FxHasher;
