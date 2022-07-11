use openssl::sha::Sha256;

use crate::Array;

pub struct Sha256Hasher(Sha256);

impl<const N: usize> crate::traits::Hasher<N> for Sha256Hasher {
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
    fn finalize(self) -> Array<N> {
        let value = self.0.finish();
        #[cfg(feature = "serde")]
        let mut v = Array::default();
        #[cfg(not(any(feature = "serde")))]
        let mut v = [0; N];
        if N > 32 {
            v[..32].copy_from_slice(&value)
        } else {
            v[..N].copy_from_slice(&value[..N]);
        }

        v
    }
}
