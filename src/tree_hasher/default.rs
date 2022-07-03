use crate::traits::Hasher;
use crate::Array;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher as DefaultHasherTrait;

impl<const N: usize> Hasher<N> for DefaultHasher {
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        Self::write(self, data);
    }

    #[inline]
    fn finalize(self) -> Array<N> {
        let value = Self::finish(&self).to_le_bytes();
        let mut v = Array([0; N]);
        if N >= 8 {
            v.as_mut()[..8].copy_from_slice(&value);
        } else {
            v.as_mut()[..N].copy_from_slice(&value[..N]);
        }

        v
    }
}
