use blake2_rfc;

#[derive(Clone)]
pub struct Blake2bHasher(blake2_rfc::blake2b::Blake2b);

impl crate::traits::Hasher for Blake2bHasher {
    type HashType = Self;

    fn new(size: usize) -> Self {
        let hasher = blake2_rfc::blake2b::Blake2b::new(size);
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> Vec<u8> { self.0.finalize().as_ref().to_vec() }
}