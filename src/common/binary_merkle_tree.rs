use std::path::PathBuf;
use std::error::Error;

use common::{Encode, Exception, Decode};
use common::address::Address;

use serialization::state::{MerkleNode as ProtoMerkleNode, MerkleNode_oneof_node as ProtoMerkleNodeType};

use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use protobuf::Message as ProtoMessage;
use rocksdb::{DB, Options};


pub trait Hasher {
    type HashType;
    type HashResultType;
    fn new(size: usize) -> Self::HashType;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> Self::HashResultType;
}

pub trait Leaf {
    fn get_key(&self) -> &[u8];
    fn get_data(&self) -> Option<&[u8]>;
    fn get_value(&self) -> Option<&[u8]>;
}

impl Hasher for Blake2b {
    type HashType = Blake2b;
    type HashResultType = Blake2bResult;

    fn new(size: usize) -> Self::HashType {
        Blake2b::new(size)
    }
    fn update(&mut self, data: &[u8]) {
        Blake2b::update(self, data)
    }
    fn finalize(self) -> Self::HashResultType {
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

fn split_pairs<LeafType>(pairs: &[LeafType], bit: usize) -> (Vec<&LeafType>, Vec<&LeafType>)
    where LeafType: Leaf {
    let mut zeros = Vec::with_capacity(pairs.len() / 2);
    let mut ones = Vec::with_capacity(pairs.len() / 2);
    for pair in pairs {
        match choose_branch(pair.get_key(), bit) {
            Branch::Zero => zeros.push(pair),
            Branch::One => ones.push(pair)
        }
    }
    return (zeros, ones)
}

pub struct BinaryMerkleTree {
    db: DB,
    depth: usize
}

impl BinaryMerkleTree {
    pub fn new(path: PathBuf, depth: usize) -> Result<Self, Box<Error>> {
        let db = DB::open_default(path)?;
        Ok(Self {
            db,
            depth
        })
    }

    pub fn get<LeafType, HashResultType>(&self, root: &mut HashResultType, keys: &[u8]) -> Result<Vec<LeafType>, Box<Error>>
        where LeafType: Leaf,
              HashResultType: AsRef<[u8]> {
        let retrieved_node = self.db.get(root.as_ref())?;
        let encoded_node;
        match retrieved_node {
            Some(data) => encoded_node = data,
            None => return Err(Box::new(Exception::new("Failed to find root node in database")))
        }

        let mut node = ProtoMerkleNode::new();
        node.merge_from_bytes(&encoded_node)?;

        match node.node {
            Some(node_type) => {
                match node_type {
                    ProtoMerkleNodeType::branch(Branch) => {},
                    ProtoMerkleNodeType::leaf(Leaf) => {},
                    ProtoMerkleNodeType::data(Data) => {}
                }
            }
            None => return Err(Box::new(Exception::new("Loaded node is corrupted")))
        }
        Ok(vec![])
    }

    pub fn insert<LeafType, HashResultType>(&self, leaves: &mut Vec<LeafType>, root: Option<HashResultType>) -> Result<(), Box<Error>>
        where LeafType: Leaf + Ord + Eq,
              HashResultType: AsRef<[u8]> + Clone {
        leaves.sort();
        leaves.dedup_by(|a, b| a == b);

        let count_delta = 0;
        if let Some(root_hash) = root {
            if let Ok(data) = self.db.get(root_hash.as_ref()) {

            }
        }

        Ok(())
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