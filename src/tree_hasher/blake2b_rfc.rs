use blake2_rfc;

use crate::constants::KEY_LEN;

#[derive(Clone)]
pub struct Blake2bHasher(blake2_rfc::blake2b::Blake2b);

impl crate::traits::Hasher for Blake2bHasher {
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
    fn finalize(self) -> [u8; KEY_LEN] {
        let result = self.0.finalize();
        let mut finalized = [0; KEY_LEN];
        finalized.copy_from_slice(result.as_ref());
        finalized
    }
}
