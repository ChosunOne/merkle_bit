use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

use crate::traits::Array;

pub struct KeccakHasher(Keccak);

impl<ArrayType> crate::traits::Hasher<ArrayType> for KeccakHasher
where
    ArrayType: Array,
{
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        let hasher = Keccak::v256();
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
