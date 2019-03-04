use groestl::{Digest, Groestl256};

pub struct GroestlHasher(Groestl256);

impl crate::traits::Hasher for GroestlHasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Groestl256::new();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.input(data); }
    fn finalize(self) -> Self::HashResultType { self.0.result().to_vec() }
}