use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

use crate::Array;

pub struct KeccakHasher(Keccak);

impl<const N: usize> crate::traits::Hasher<N> for KeccakHasher {
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
    fn finalize(self) -> Array<N> {
        let mut res = Array::default();
        self.0.finalize(res.as_mut());
        res
    }
}
