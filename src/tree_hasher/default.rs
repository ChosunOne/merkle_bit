use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::constants::KEY_LEN;

impl crate::traits::Hasher for DefaultHasher {
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }
    #[inline]
    fn update(&mut self, data: &[u8]) {
        Self::write(self, data)
    }
    #[inline]
    fn finalize(self) -> [u8; KEY_LEN] {
        let value = Self::finish(&self).to_le_bytes();
        let mut v = [0; KEY_LEN];
        v[..8].copy_from_slice(&value);
        v
    }
}
