use std::collections::hash_map::DefaultHasher;
use crate::traits::{Hasher, Array};
use std::hash::Hasher as DefaultHasherTrait;

impl<ArrayType> Hasher<ArrayType> for DefaultHasher
    where ArrayType: Array {
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        Self::write(self, data.as_ref())
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
