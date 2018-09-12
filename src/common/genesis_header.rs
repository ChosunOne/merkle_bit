use std::ops::Deref;
use std::error::Error;

use common::header::{BlockHeader, Header};
use common::{Encode, Proto};
use common::address::Address;
use serialization::blockHeader::GenesisHeader as ProtoGenesisHeader;
use protobuf::Message;

#[derive(Clone, Debug, PartialEq)]
pub struct GenesisHeader(pub Header);

impl Deref for GenesisHeader {
    type Target = Header;

    fn deref(&self) -> &Header {
        &self.0
    }
}

impl BlockHeader for GenesisHeader {
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
        None
    }
    fn get_nonce(&self) -> Option<u64> {
        None
    }
    fn get_miner(&self) -> Option<&Address> {
        None
    }
}

impl GenesisHeader {
    pub fn new(merkle_root: Vec<u8>, time_stamp: u64, difficulty: f64, state_root: Vec<u8>) -> GenesisHeader {
        let header = Header::new(merkle_root, time_stamp, difficulty, state_root, vec![vec![]], 0, [0u8; 20]);
        GenesisHeader(header)
    }
}

impl Proto for GenesisHeader {
    type ProtoType = ProtoGenesisHeader;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_genesis_block_header = Self::ProtoType::new();
        proto_genesis_block_header.set_merkleRoot(self.merkle_root.clone());
        proto_genesis_block_header.set_timeStamp(self.time_stamp);
        proto_genesis_block_header.set_difficulty(self.difficulty);
        proto_genesis_block_header.set_stateRoot(self.state_root.clone());
        Ok(proto_genesis_block_header)
    }
}

impl Encode for GenesisHeader {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_genesis_block_header: ProtoGenesisHeader = self.to_proto()?;
        Ok(proto_genesis_block_header.write_to_bytes()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_base58::ToBase58;
    use util::hash::hash;

    #[test]
    fn it_makes_the_genesis_header() {
        let merkle_root = vec![218,175,98,56,136,59,157,43,178,250,66,
            194,50,129,87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39];
        let state_root = vec![121,132,139,154,165,229,182,152,126,204,
            58,142,150,220,236,119,144,1,181,107,19,130,67,220,241,192,46,94,69,215,134,11];
        let time_stamp = 1515003305000;
        let difficulty: f64 = 0 as f64;

        let genesis_header = GenesisHeader::new(merkle_root.clone(), time_stamp, difficulty, state_root.clone());
        let encoding = genesis_header.encode().unwrap();
        let expected_encoding = vec![18,32,218,175,98,56,136,59,157,43,
            178,250,66,194,50,129,87,37,147,54,157,79,238,83,118,209,
            92,202,25,32,246,230,153,39,26,32,121,132,139,154,165,229,
            182,152,126,204,58,142,150,220,236,119,144,1,181,107,19,130,
            67,220,241,192,46,94,69,215,134,11,33,0,0,0,0,0,0,0,0,40,
            168,184,239,233,139,44];
        let genesis_header_hash = hash(&encoding, 32).to_base58();
        assert_eq!(genesis_header.merkle_root, merkle_root);
        assert_eq!(genesis_header.state_root, state_root);
        assert_eq!(genesis_header.time_stamp, time_stamp);
        assert_eq!(genesis_header.difficulty, difficulty);
        assert_eq!(encoding, expected_encoding);
        assert_eq!(genesis_header_hash, "G4qXusbRyXmf62c8Tsha7iZoyLsVGfka7ynkvb3Esd1d".to_string())
    }
}
