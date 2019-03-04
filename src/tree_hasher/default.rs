#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
use std::hash::Hasher;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
use std::collections::hash_map::DefaultHasher;

#[cfg(not(any(feature = "use_blake2b", feature = "use_groestl", feature = "use_sha2", feature = "use_sha3", feature = "use_keccak")))]
impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self { Self::new() }
    fn update(&mut self, data: &[u8]) { Self::write(self, data) }
    fn finalize(self) -> Self::HashResultType { Self::finish(&self).to_le_bytes().to_vec() }
}