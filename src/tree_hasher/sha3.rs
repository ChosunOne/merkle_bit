#[cfg(feature = "use_sha3")]
use tiny_keccak::Keccak;

#[cfg(feature = "use_sha3")]
pub struct Sha3Hasher(Keccak);

#[cfg(feature = "use_sha3")]
impl crate::traits::Hasher for Sha3Hasher {
    type HashType = Self;
    type HashResultType = Vec<u8>;

    fn new(_size: usize) -> Self {
        let hasher = Keccak::new_sha3_256();
        Self(hasher)
    }
    fn update(&mut self, data: &[u8]) { self.0.update(data); }
    fn finalize(self) -> Self::HashResultType {
        let mut res = vec![0; 32];
        self.0.finalize(&mut res);
        res
    }
}