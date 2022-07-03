use blake2_rfc;

use crate::Array;

#[derive(Clone)]
pub struct Blake2bHasher(blake2_rfc::blake2b::Blake2b);

impl<const N: usize> crate::traits::Hasher<N> for Blake2bHasher {
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
    fn finalize(self) -> Array<N> {
        let result = self.0.finalize();
        let mut finalized = Array::default();
        finalized.copy_from_slice(result.as_ref());
        finalized
    }
}
