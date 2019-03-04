use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self { Self::new() }
    fn update(&mut self, data: &[u8]) { Self::write(self, data) }
    fn finalize(self) -> Self::HashResultType { Self::finish(&self).to_le_bytes().to_vec() }
}