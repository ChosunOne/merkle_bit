use seahash::SeaHasher;
use std::hash::Hasher;

use crate::Array;

impl<const N: usize> crate::traits::Hasher<N> for SeaHasher {
    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        Hasher::write(self, data)
    }

    #[inline]
    fn finalize(self) -> Array<N> {
        let value = Self::finish(&self).to_le_bytes();
        #[cfg(feature = "serde")]
        let mut v = Array::default();
        #[cfg(not(any(feature = "serde")))]
        let mut v = [0; N];
        if N >= 8 {
            v[..8].copy_from_slice(&value);
        } else {
            v[..N].copy_from_slice(&value[..N]);
        }
        v
    }
}
