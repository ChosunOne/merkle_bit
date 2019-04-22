use tiny_keccak::Keccak;

use crate::constants::KEY_LEN;

pub struct KeccakHasher(Keccak);

impl crate::traits::Hasher for KeccakHasher {
    type HashType = Self;

    fn new(_size: usize) -> Self {
        let hasher = Keccak::new_keccak256();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> [u8; KEY_LEN] {
        let mut res = [0u8; KEY_LEN];
        self.0.finalize(&mut res);
        res
    }
}