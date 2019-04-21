use groestl::{Digest, Groestl256};

pub struct GroestlHasher(Groestl256);

impl crate::traits::Hasher for GroestlHasher {
    type HashType = Self;

    fn new(_size: usize) -> Self {
        let hasher = Groestl256::new();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.input(data); }
    fn finalize(self) -> [u8; 32] {
        let mut finalized = [0; 32];
        let result = self.0.result();
        for (i, byte) in result.into_iter().enumerate() {
            finalized[i] = byte;
        }
        finalized
    }
}