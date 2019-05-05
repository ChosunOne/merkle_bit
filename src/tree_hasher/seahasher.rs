use seahash::SeaHasher;
use std::hash::Hasher;

use crate::constants::KEY_LEN;

impl crate::traits::Hasher for SeaHasher {
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        for i in (0..data.len()).step_by(16) {
            let mut values = [0_u8; 16];
            if i + 16 < data.len() {
                values.copy_from_slice(&data[i..i + 16]);
            } else if data.len() < 16 {
                values[..data.len()].copy_from_slice(&data[i..]);
            } else {
                values[..data.len() - i].copy_from_slice(&data[i..]);
            }

            let num = u128::from_le_bytes(values);
            Self::write_u128(self, num);
        }
    }

    #[inline]
    fn finalize(self) -> [u8; KEY_LEN] {
        let value = Self::finish(&self).to_le_bytes();
        let mut v = [0; KEY_LEN];
        v[..8].copy_from_slice(&value);
        v
    }
}
