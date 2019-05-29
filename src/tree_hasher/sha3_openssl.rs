use tiny_keccak::Keccak;

use crate::constants::KEY_LEN;

pub struct Sha3Hasher(Keccak);

impl crate::traits::Hasher for Sha3Hasher {
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
    fn finalize(self) -> [u8; KEY_LEN] {
        let mut res = [0; KEY_LEN];
        self.0.finalize(&mut res);
        res
    }
}
