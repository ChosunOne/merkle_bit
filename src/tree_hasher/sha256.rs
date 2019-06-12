use openssl::sha::Sha256;

use crate::traits::Array;

pub struct Sha256Hasher(Sha256);

impl<ArrayType> crate::traits::Hasher<ArrayType> for Sha256Hasher
where
    ArrayType: Array,
{
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        let hasher = Sha256::new();
        Self(hasher)
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        self.0.update(data)
    }

    #[inline]
    fn finalize(self) -> ArrayType {
        let mut v = ArrayType::default();
        let value = self.0.finish();
        let length = v.as_ref().len();
        v.as_mut()[..length].copy_from_slice(&value[..length]);
        v
    }
}
