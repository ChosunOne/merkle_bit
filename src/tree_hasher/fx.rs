use crate::Array;
use fxhash::FxHasher;
use std::hash::Hasher;

impl<const N: usize> crate::traits::Hasher<N> for FxHasher {
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::default()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        Hasher::write(self, data)
    }

    #[inline]
    fn finalize(self) -> Array<N> {
        let value = Self::finish(&self).to_le_bytes();
        let mut v = Array::default();
        let length = v.as_ref().len();
        if length >= 8 {
            v.as_mut()[..8].copy_from_slice(&value);
        } else {
            v.as_mut()[..length].copy_from_slice(&value[..length]);
        }
        v
    }
}
