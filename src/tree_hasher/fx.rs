use crate::Array;
use fxhash::FxHasher;
use std::hash::Hasher;

impl<const N: usize> crate::traits::Hasher<N> for FxHasher {
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
        if N >= 8 {
            v[..8].copy_from_slice(&value);
        } else {
            v[..N].copy_from_slice(&value[..N]);
        }
        v
    }
}
