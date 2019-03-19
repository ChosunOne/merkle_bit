use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;

    fn new(_size: usize) -> Self { Self::new() }
    fn update(&mut self, data: &[u8]) { Self::write(self, data) }
    fn finalize(self) -> Vec<u8> { Self::finish(&self).to_le_bytes().to_vec() }
}