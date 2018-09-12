use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use common::address::Address;

pub trait Hasher {
    type HashResult;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> Self::HashResult;
}

pub trait Leaf {
    fn get_key(&self) -> &[u8];
    fn get_data(&self) -> Option<&[u8]>;
    fn get_value(&self) -> Option<&[u8]>;
}

impl Hasher for Blake2b {
    type HashResult = Blake2bResult;

    fn update(&mut self, data: &[u8]) {
        Blake2b::update(self, data)
    }

    fn finalize(self) -> Self::HashResult {
        Blake2b::finalize(self)
    }
}

enum Branch {
    Zero,
    One
}

fn choose_branch(key: &[u8], bit: usize) -> Branch {
    let index = bit / 8;
    let shift = bit % 8;
    let extracted_bit = (key[index] >> (7 - shift)) & 1;
    if extracted_bit == 0 {
        return Branch::Zero
    } else {
        return Branch::One
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_recognizes_a_hasher() {
        let mut blake = Blake2b::new(32);
        let data = [0u8; 32];
        blake.update(&data);
        let hash = blake.finalize();
        let expected_hash = [
            137, 235,  13, 106, 138, 105, 29, 174,
             44, 209,  94, 208,  54, 153, 49, 206,
             10, 148, 158, 202, 250,  92, 63, 147,
            248,  18,  24,  51, 100, 110, 21, 195];
        assert_eq!(hash.as_bytes(), expected_hash);
    }

}