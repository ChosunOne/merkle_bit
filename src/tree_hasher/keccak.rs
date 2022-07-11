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
        #[cfg(feature = "serde")]
        let mut res = Array::default();
        #[cfg(not(any(feature = "serde")))]
        let mut res = [0; N];
        self.0.finalize(res.as_mut());
        res
    }
}
