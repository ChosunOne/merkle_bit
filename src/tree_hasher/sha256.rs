use openssl::sha::Sha256;

pub struct Sha256Hasher(Sha256);

impl crate::traits::Hasher for Sha256Hasher {
    type HashType = Self;

    fn new(_size: usize) -> Self {
        let hasher = Sha256::new();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data) }
    fn finalize(self) -> [u8; 32] {
        self.0.finish()
    }
}