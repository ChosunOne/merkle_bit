use seahash::SeaHasher;
use std::hash::Hasher;

use crate::traits::Array;

impl<ArrayType> crate::traits::Hasher<ArrayType> for SeaHasher
where
    ArrayType: Array,
{
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        Hasher::write(self, data)
    }

    #[inline]
    fn finalize(self) -> ArrayType {
        let value = Self::finish(&self).to_le_bytes();
        let mut v = ArrayType::default();
        let length = v.as_ref().len();
        if length >= 8 {
            v.as_mut()[..8].copy_from_slice(&value);
        } else {
            v.as_mut()[..length].copy_from_slice(&value[..length]);
        }
        v
    }
}
