use tiny_keccak::Hasher;
use tiny_keccak::Sha3;

use crate::traits::Array;

pub struct Sha3Hasher(Sha3);

impl<ArrayType> crate::traits::Hasher<ArrayType> for Sha3Hasher
where
    ArrayType: Array,
{
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        let hasher = Sha3::v256();
        Self(hasher)
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    #[inline]
    fn finalize(self) -> ArrayType {
        let mut res = ArrayType::default();
        self.0.finalize(res.as_mut());
        res
    }
}
