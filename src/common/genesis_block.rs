use std::ops::Deref;
use std::error::Error;

use common::block::Block;
use common::signed_genesis_tx::SignedGenesisTx;
use common::genesis_header::GenesisHeader;
use common::{Encode, Proto};

use serialization::tx::GenesisSignedTx as ProtoTx;
use serialization::block::GenesisBlock as ProtoBlock;

use protobuf::{Message as ProtoMessage, RepeatedField};

struct GenesisBlock(Block<GenesisHeader, SignedGenesisTx>);

impl Deref for GenesisBlock {
    type Target = Block<GenesisHeader, SignedGenesisTx>;
    fn deref(&self) -> &Block<GenesisHeader, SignedGenesisTx> {
        &self.0
    }
}

impl Proto for GenesisBlock {
    type ProtoType = ProtoBlock;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_block = Self::ProtoType::new();
        let proto_header = self.header.to_proto()?;
        proto_block.set_header(proto_header);
        match self.txs.clone() {
            Some(tx_vec) => {
                let mut proto_txs: Vec<ProtoTx> = vec![];
                for tx in tx_vec.into_iter() {
                    match tx.to_proto() {
                        Ok(proto_tx) => proto_txs.push(proto_tx),
                        Err(_) => {} 
                    }
                }
                proto_block.set_txs(RepeatedField::from(proto_txs));
            },
            None => {}
        }
        Ok(proto_block)
    }
}

impl Encode for GenesisBlock {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_block = self.to_proto()?;
        Ok(proto_block.write_to_bytes()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::{Address, ValidAddress};
    use common::header::Header;
    use common::genesis_tx::GenesisTx;

    use secp256k1::{RecoverableSignature, RecoveryId, Secp256k1};

    #[test]
    fn it_makes_a_genesis_block_with_no_txs() {
        let genesis_header = create_genesis_header();
        let block = Block::new(genesis_header.clone(), None, None);
        let genesis_block = GenesisBlock(block);

        match genesis_block.txs {
            Some(_) => panic!("No transactions were given, but the genesis block has transactions!"),
            None => {}
        }

        match genesis_block.meta {
            Some(_) => panic!("No meta information was given, but the genesis block has meta information!"),
            None => {}
        }

        assert_eq!(&genesis_block.header, &genesis_header);
    }

    #[test]
    fn it_makes_a_genesis_block_with_txs() {
        let genesis_header = create_genesis_header();
        let genesis_txs = create_genesis_txs();
        let block = Block::new(genesis_header.clone(), Some(genesis_txs.clone()), None);
        let genesis_block = GenesisBlock(block);

        match genesis_block.meta {
            Some(_) => panic!("No meta information was supplied, but genesis block has meta information!"),
            None => {}
        }

        assert_eq!(&genesis_block.header, &genesis_header);
        assert_eq!(genesis_block.txs.clone().unwrap(), genesis_txs);
    }

    #[test]
    fn it_encodes_a_genesis_block_with_no_txs() {
        let genesis_header = create_genesis_header();
        let block = Block::new(genesis_header, None, None);
        let genesis_block = GenesisBlock(block);
        let encoding = genesis_block.encode().unwrap();
        let expected_encoding = vec![10,84,18,32,218,175,98,56,136,59,157,43,178,250,66,
            194,50,129,87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39,
            26,32,121,132,139,154,165,229,182,152,126,204,58,142,150,220,236,119,144,1,
            181,107,19,130,67,220,241,192,46,94,69,215,134,11,33,0,0,0,0,0,0,0,0,40,168,
            184,239,233,139,44];

        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_encodes_a_genesis_block_with_txs() {
        let genesis_header = create_genesis_header();
        let genesis_txs = create_genesis_txs();
        let block = Block::new(genesis_header.clone(), Some(genesis_txs.clone()), None);
        let genesis_block = GenesisBlock(block);

        let encoding = genesis_block.encode().unwrap();
        let expected_encoding = create_expected_genesis_encoding();

        assert_eq!(encoding, expected_encoding);
    }

    fn create_genesis_header() -> GenesisHeader {
        // Set up header
        let merkle_root = vec![218,175,98,56,136,59,157,43,178,250,66,
            194,50,129,87,37,147,54,157,79,238,83,118,209,92,202,25,32,246,230,153,39];
        let state_root = vec![121,132,139,154,165,229,182,152,126,204,
            58,142,150,220,236,119,144,1,181,107,19,130,67,220,241,192,46,94,69,215,134,11];
        let time_stamp = 1515003305000;
        let difficulty: f64 = 0 as f64;
        GenesisHeader::new(merkle_root.clone(), time_stamp, difficulty, state_root.clone())
    }

    fn create_genesis_txs() -> Vec<SignedGenesisTx> {
        // Set up genesis txs
        let secp = Secp256k1::without_caps();
        let address_1 = Address::from_string(&"HATiUU3eT7ghypqZSW7gfK3ikQp25oPA".to_string()).unwrap();
        let amount_1 = 2000000000000000000;
        let recovery_1 = RecoveryId::from_i32(1).unwrap();
        let signature_1_bytes = vec![22,221,209,1,153,187,59,100,249,224,81,229,6,89,230,150,
            104,199,229,7,191,94,31,116,224,83,41,58,104,35,204,189,96,77,86,99,69,243,237,
            35,106,215,176,110,113,85,133,62,69,176,245,74,80,40,164,234,107,132,100,135,195,32,76,158];
        let signature_1 = RecoverableSignature::from_compact(&secp, &signature_1_bytes[..], recovery_1).unwrap();
        let genesis_signed_tx_1 = SignedGenesisTx::new(address_1, amount_1, signature_1, recovery_1);

        let address_2 = Address::from_string(&"HHhrFzwkhbZHm49WJS7Aqfy4SSnj35DH".to_string()).unwrap();
        let amount_2 = 100000000000000000;
        let recovery_2 = RecoveryId::from_i32(0).unwrap();
        let signature_2_bytes = vec![189,36,210,45,239,228,233,146,171,234,170,126,142,158,
            221,102,7,58,121,80,106,207,12,105,254,81,148,142,43,11,95,225,114,152,212,239,
            105,128,115,242,243,71,57,227,161,132,226,223,6,219,4,24,167,103,233,89,0,178,
            229,199,229,237,214,232];
        let signature_2 = RecoverableSignature::from_compact(&secp, &signature_2_bytes[..], recovery_2).unwrap();
        let genesis_signed_tx_2 = SignedGenesisTx::new(address_2, amount_2, signature_2, recovery_2);

        let address_3 = Address::from_string(&"H235yexRUBEWiSC9xG4a7A5b6vjUaBsr7".to_string()).unwrap();
        let amount_3 = 500000000000000000;
        let recovery_3 = RecoveryId::from_i32(0).unwrap();
        let signature_3_bytes = vec![8,106,101,224,229,144,115,64,238,176,220,45,192,68,110,
            22,152,116,80,142,140,194,87,181,89,105,7,178,116,88,132,64,93,221,2,101,38,212,
            119,41,233,180,120,15,141,3,22,76,121,31,156,41,67,220,0,255,255,129,128,130,38,111,188,190];
        let signature_3 = RecoverableSignature::from_compact(&secp, &signature_3_bytes[..], recovery_3).unwrap();
        let genesis_signed_tx_3 = SignedGenesisTx::new(address_3, amount_3, signature_3, recovery_3);

        let address_4 = Address::from_string(&"H27MU6pdoAmNfsrjd1QRwmaYMtTrT988U".to_string()).unwrap();
        let amount_4 = 500000000000000000;
        let recovery_4 = RecoveryId::from_i32(1).unwrap();
        let signature_4_bytes = vec![216,216,182,240,69,245,211,112,118,195,196,166,109,0,
            39,224,75,99,37,58,63,236,142,1,194,33,186,55,3,134,255,249,88,117,48,29,208,
            169,54,108,80,250,110,196,252,98,38,130,202,47,68,195,150,254,230,226,96,76,222,104,235,63,140,160];
        let signature_4 = RecoverableSignature::from_compact(&secp, &signature_4_bytes[..], recovery_4).unwrap();
        let genesis_signed_tx_4 = SignedGenesisTx::new(address_4, amount_4, signature_4, recovery_4);

        let address_5 = Address::from_string(&"H2QC1ebYRgvSV4xQyZGC5DbWWGZST6M3W".to_string()).unwrap();
        let amount_5 = 400000000000000000;
        let recovery_5 = RecoveryId::from_i32(1).unwrap();
        let signature_5_bytes = vec![100,74,23,91,232,118,63,48,190,221,108,193,208,65,69,
            9,23,186,93,187,13,243,54,51,49,144,127,148,41,36,138,174,60,114,62,139,193,
            225,72,241,31,26,186,189,155,145,185,217,135,255,158,227,168,225,62,17,34,246,227,47,80,90,195,93];
        let signature_5 = RecoverableSignature::from_compact(&secp, &signature_5_bytes[..], recovery_5).unwrap();
        let genesis_signed_tx_5 = SignedGenesisTx::new(address_5, amount_5, signature_5, recovery_5);

        let address_6 = Address::from_string(&"H3gxigHtbRWi3nqVA6FHHfmtkyu7d9suC".to_string()).unwrap();
        let amount_6 = 500000000000000000;
        let recovery_6 = RecoveryId::from_i32(0).unwrap();
        let signature_6_bytes = vec![128,85,24,160,100,145,206,75,9,170,203,75,122,52,207,
            109,100,133,252,204,84,3,236,158,38,195,33,100,12,112,129,205,104,206,252,101,
            101,196,190,231,190,129,57,29,166,137,35,236,58,27,190,228,14,123,55,148,247,92,199,22,238,89,6,152];
        let signature_6 = RecoverableSignature::from_compact(&secp, &signature_6_bytes[..], recovery_6).unwrap();
        let genesis_signed_tx_6 = SignedGenesisTx::new(address_6, amount_6, signature_6, recovery_6);

        vec![genesis_signed_tx_1, genesis_signed_tx_2, genesis_signed_tx_3, genesis_signed_tx_4, genesis_signed_tx_5, genesis_signed_tx_6]
    }

    fn create_expected_genesis_encoding() -> Vec<u8> {
        vec![10,84,18,32,218,175,98,56,136,59,157,43,178,250,66,194,50,129,87,37,147,54,157,
            79,238,83,118,209,92,202,25,32,246,230,153,39,26,32,121,132,139,154,165,229,182,
            152,126,204,58,142,150,220,236,119,144,1,181,107,19,130,67,220,241,192,46,94,69,
            215,134,11,33,0,0,0,0,0,0,0,0,40,168,184,239,233,139,44,18,100,18,20,11,181,71,
            187,17,26,124,178,99,162,96,88,212,122,136,90,16,252,254,235,24,128,128,160,246,
            244,172,219,224,27,50,64,22,221,209,1,153,187,59,100,249,224,81,229,6,89,230,150,
            104,199,229,7,191,94,31,116,224,83,41,58,104,35,204,189,96,77,86,99,69,243,237,
            35,106,215,176,110,113,85,133,62,69,176,245,74,80,40,164,234,107,132,100,135,195,
            32,76,158,56,1,18,100,18,20,20,172,52,145,135,93,229,13,193,97,80,104,183,138,132,
            87,177,224,233,120,24,128,128,168,236,133,175,209,177,1,50,64,189,36,210,45,239,
            228,233,146,171,234,170,126,142,158,221,102,7,58,121,80,106,207,12,105,254,81,148,
            142,43,11,95,225,114,152,212,239,105,128,115,242,243,71,57,227,161,132,226,223,6,
            219,4,24,167,103,233,89,0,178,229,199,229,237,214,232,56,0,18,100,18,20,74,92,52,
            70,84,37,197,115,110,79,0,154,9,198,192,6,200,12,132,127,24,128,128,200,157,157,235,
            150,248,6,50,64,8,106,101,224,229,144,115,64,238,176,220,45,192,68,110,22,152,116,
            80,142,140,194,87,181,89,105,7,178,116,88,132,64,93,221,2,101,38,212,119,41,233,180,
            120,15,141,3,22,76,121,31,156,41,67,220,0,255,255,129,128,130,38,111,188,190,56,0,
            18,100,18,20,79,164,22,41,92,131,196,210,126,157,6,109,177,113,128,7,16,121,88,180,
            24,128,128,200,157,157,235,150,248,6,50,64,216,216,182,240,69,245,211,112,118,195,
            196,166,109,0,39,224,75,99,37,58,63,236,142,1,194,33,186,55,3,134,255,249,88,117,48,
            29,208,169,54,108,80,250,110,196,252,98,38,130,202,47,68,195,150,254,230,226,96,76,
            222,104,235,63,140,160,56,1,18,100,18,20,100,122,82,165,32,27,125,117,206,33,252,177,
            254,17,2,172,179,165,145,12,24,128,128,160,177,151,188,197,198,5,50,64,100,74,23,91,
            232,118,63,48,190,221,108,193,208,65,69,9,23,186,93,187,13,243,54,51,49,144,127,
            148,41,36,138,174,60,114,62,139,193,225,72,241,31,26,186,189,155,145,185,217,135,
            255,158,227,168,225,62,17,34,246,227,47,80,90,195,93,56,1,18,100,18,20,193,2,252,
            220,116,178,175,175,186,175,179,5,219,15,64,216,8,88,8,32,24,128,128,200,157,157,
            235,150,248,6,50,64,128,85,24,160,100,145,206,75,9,170,203,75,122,52,207,109,100,
            133,252,204,84,3,236,158,38,195,33,100,12,112,129,205,104,206,252,101,101,196,190,
            231,190,129,57,29,166,137,35,236,58,27,190,228,14,123,55,148,247,92,199,22,238,89,
            6,152,56,0]
    }
}