use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;

    fn new(_size: usize) -> Self {
        Self::new()
    }
    fn update(&mut self, data: &[u8]) {
        Self::write(self, data)
    }
    fn finalize(self) -> Vec<u8> {
        let value = Self::finish(&self).to_le_bytes().to_vec();
        let mut v = vec![0; 32];
        for i in 0..32 {
            v[i] = value[i % 8];
        }
        v
    }
}
