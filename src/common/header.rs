use std::error::Error;

use common::address::Address;
use common::{Encode, Exception, Proto};
use util::hash::hash;

use serialization::blockHeader::{HeaderPrehash, BlockHeader as ProtoBlockHeader};

use protobuf::{Message as ProtoMessage, RepeatedField};

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub merkle_root: Vec<u8>,
    pub time_stamp: u64,
    pub difficulty: f64,
    pub state_root: Vec<u8>,
    pub previous_hash: Vec<Vec<u8>>,
    pub nonce: u64,
    pub miner: Address,
}

pub trait BlockHeader {
    fn get_merkle_root(&self) -> &Vec<u8>;
    fn get_time_stamp(&self) -> u64;
    fn get_difficulty(&self) -> f64;
    fn get_state_root(&self) -> &Vec<u8>;
    fn get_previous_hash(&self) -> Option<&Vec<Vec<u8>>>;
    fn get_nonce(&self) -> Option<u64>;
    fn get_miner(&self) -> Option<&Address>;
}

impl BlockHeader for Header {
    fn get_merkle_root(&self) -> &Vec<u8> {
        &self.merkle_root
    }
    fn get_time_stamp(&self) -> u64 {
        self.time_stamp
    }
    fn get_difficulty(&self) -> f64 {
        self.difficulty
    }
    fn get_state_root(&self) -> &Vec<u8> {
        &self.state_root
    }
    fn get_previous_hash(&self) -> Option<&Vec<Vec<u8>>> {
        Some(&self.previous_hash)
    }
    fn get_nonce(&self) -> Option<u64> {
        Some(self.nonce)
    }
    fn get_miner(&self) -> Option<&Address> {
        Some(&self.miner)
    }
}

impl Header {
    pub fn new(merkle_root: Vec<u8>, 
               time_stamp: u64, 
               difficulty: f64, 
               state_root: Vec<u8>, 
               previous_hash: Vec<Vec<u8>>,
               nonce: u64,
               miner: Address) -> Header {
                   Header {
                       merkle_root,
                       time_stamp,
                       difficulty,
                       state_root,
                       previous_hash,
                       nonce,
                       miner
                   }
    }

    pub fn prehash(&self) -> Result<Vec<u8>, Box<Error>> {
        let mut proto_header = HeaderPrehash::new();
        proto_header.set_merkleRoot(self.merkle_root.clone());
        proto_header.set_timeStamp(self.time_stamp);
        proto_header.set_difficulty(self.difficulty);
        proto_header.set_stateRoot(self.state_root.clone());
        proto_header.set_previousHash(RepeatedField::from(self.previous_hash.clone()));
        proto_header.set_miner(self.miner.to_vec());
        let encoding = proto_header.write_to_bytes()?;
        Ok(hash(&encoding, 64))
    }
}

impl Encode for Header {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_block_header = self.to_proto()?;
        Ok(proto_block_header.write_to_bytes()?)
    }
}

impl Proto for Header {
    type ProtoType = ProtoBlockHeader;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_header = Self::ProtoType::new();
        proto_header.set_merkleRoot(self.merkle_root.clone());
        proto_header.set_timeStamp(self.time_stamp);
        proto_header.set_difficulty(self.difficulty);
        proto_header.set_stateRoot(self.state_root.clone());
        proto_header.set_previousHash(RepeatedField::from(self.previous_hash.clone()));
        proto_header.set_nonce(self.nonce);
        proto_header.set_miner(self.miner.to_vec());
        Ok(proto_header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::ValidAddress;
    use rust_base58::FromBase58;

    #[test]
    fn it_makes_a_header() {
        let merkle_root = vec![218,175,98,56,136,59,157,43,178,250,66,194,50,129,
            87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39];
        let state_root = vec![121,132,139,154,165,229,182,152,126,204,58,142,150,
            220,236,119,144,1,181,107,19,130,67,220,241,192,46,94,69,215,134,11];
        let time_stamp = 1515003305000;
        let difficulty = 0 as f64;
        let nonce = 0;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let previous_hash = vec!["G4qXusbRyXmf62c8Tsha7iZoyLsVGfka7ynkvb3Esd1d".from_base58().unwrap()];

        let header = Header::new(merkle_root.clone(), time_stamp, difficulty, state_root.clone(), previous_hash.clone(), nonce, miner);
        assert_eq!(header.merkle_root, merkle_root);
        assert_eq!(header.state_root, state_root);
        assert_eq!(header.time_stamp, time_stamp);
        assert_eq!(header.difficulty, difficulty);
        assert_eq!(header.nonce, nonce);
        assert_eq!(header.miner, miner);
        assert_eq!(header.previous_hash, previous_hash);
    }

    #[test]
    fn it_makes_a_raw_header() {
        let merkle_root = vec![218,175,98,56,136,59,157,43,178,250,66,194,50,129,
            87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39];
        let state_root = vec![121,132,139,154,165,229,182,152,126,204,58,142,150,
            220,236,119,144,1,181,107,19,130,67,220,241,192,46,94,69,215,134,11];
        let time_stamp = 1515003305000;
        let difficulty = 0 as f64;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let previous_hash = vec!["G4qXusbRyXmf62c8Tsha7iZoyLsVGfka7ynkvb3Esd1d".from_base58().unwrap()];
        let nonce = 0;
        let header = Header::new(merkle_root.clone(), time_stamp, difficulty, state_root.clone(), previous_hash.clone(), nonce, miner);
        let encoding = header.encode().unwrap();
        let expected_encoding = vec![10,32,223,218,236,54,245,118,35,75,80,237,
            79,63,61,46,46,228,77,128,114,163,92,252,73,201,159,108,48,48,86,
            233,136,20,18,32,218,175,98,56,136,59,157,43,178,250,66,194,50,129,
            87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39,26,
            32,121,132,139,154,165,229,182,152,126,204,58,142,150,220,236,119,
            144,1,181,107,19,130,67,220,241,192,46,94,69,215,134,11,33,
            0,0,0,0,0,0,0,0,40,168,184,239,233,139,44,48,0,58,20,213,49,13,190,
            194,137,35,119,16,249,57,125,207,78,117,246,36,136,151,210];
        let prehash = header.prehash().unwrap();
        let expected_prehash = vec![213,155,184,6,160,192,238,37,190,172,89,224,
            41,36,132,38,46,5,70,193,159,49,130,25,220,56,238,148,167,135,240,
            158,162,189,223,13,85,156,251,105,34,21,90,14,21,248,16,183,136,77,
            231,102,80,183,192,177,184,19,75,226,188,134,38,218];

        assert_eq!(encoding, expected_encoding);
        assert_eq!(prehash, expected_prehash);
    }
}