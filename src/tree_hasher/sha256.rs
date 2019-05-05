use openssl::sha::Sha256;

use crate::constants::KEY_LEN;

pub struct Sha256Hasher(Sha256);

impl crate::traits::Hasher for Sha256Hasher {
    type HashType = Self;

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
    fn finalize(self) -> [u8; KEY_LEN] {
        self.0.finish()
    }
}
