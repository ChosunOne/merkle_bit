use blake2_rfc;

use crate::traits::Array;

#[derive(Clone)]
pub struct Blake2bHasher(blake2_rfc::blake2b::Blake2b);

impl<ArrayType> crate::traits::Hasher<ArrayType> for Blake2bHasher
    where ArrayType: Array{
    type HashType = Self;

    #[inline]
    fn new(size: usize) -> Self {
        let hasher = blake2_rfc::blake2b::Blake2b::new(size);
        Self(hasher)
    }

    #[inline]
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    #[inline]
    fn finalize(self) -> ArrayType {
        let result = self.0.finalize();
        let mut finalized = ArrayType::default();
        finalized.as_mut().copy_from_slice(result.as_ref());
        finalized
    }
}
