use tiny_keccak::Keccak;

use crate::traits::Array;

pub struct Sha3Hasher(Keccak);

impl<ArrayType> crate::traits::Hasher<ArrayType> for Sha3Hasher
where
    ArrayType: Array,
{
    type HashType = Self;

    #[inline]
    fn new(_size: usize) -> Self {
        let hasher = Keccak::new_sha3_256();
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
