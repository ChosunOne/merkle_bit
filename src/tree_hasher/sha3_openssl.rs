use crate::Array;
use tiny_keccak::Hasher;
use tiny_keccak::Sha3;

pub struct Sha3Hasher(Sha3);

impl<const N: usize> crate::traits::Hasher<N> for Sha3Hasher {
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
    fn finalize(self) -> Array<N> {
        let mut res = [0; N];
        self.0.finalize(&mut res);
        res.into()
    }
}
