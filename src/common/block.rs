use std::error::Error;

use common::meta::Meta;
use common::signed_tx::SignedTx;
use common::header::{BlockHeader, Header};
use common::{Encode, Exception, Proto};

use serialization::block::{Block as ProtoBlock, BlockDB as ProtoBlockDB};
use serialization::tx::SignedTx as ProtoTx;

use protobuf::{Message as ProtoMessage, RepeatedField};

pub struct Block<HeaderType, TxType> {
    pub header: HeaderType,
    pub txs: Option<Vec<TxType>>,
    pub meta: Option<Meta>,
}

impl<HeaderType, TxType> Block<HeaderType, TxType> 
    where HeaderType: Clone + Encode + BlockHeader {
    pub fn new(header: HeaderType, txs: Option<Vec<TxType>>, meta: Option<Meta>) -> Block<HeaderType, TxType> {
        Block {
            header,
            txs,
            meta
        }
    }

    pub fn from_header(header: HeaderType)-> Block<HeaderType, TxType> {
        Block {
            header,
            txs: None,
            meta: None
        }
    }

    pub fn save(&self) -> Result<ProtoBlockDB, Box<Error>> {
        let mut proto_meta: ProtoBlockDB;
        match self.meta.clone() {
            Some(meta) => proto_meta = meta.to_proto()?,
            None => return Err(Box::new(Exception::new("Block is missing meta data to save")))
        }
        let header_bytes = self.header.encode()?;
        proto_meta.merge_from_bytes(&header_bytes)?;
        Ok(proto_meta)
    }
}

impl Encode for Block<Header, SignedTx> {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_block = self.to_proto()?;
        Ok(proto_block.write_to_bytes()?)
    }
}

impl Proto for Block<Header, SignedTx> {
    type ProtoType = ProtoBlock;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_block = Self::ProtoType::new();
        proto_block.set_header(self.header.to_proto()?);
        match &self.txs {
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
            _ => {}
        }
        Ok(proto_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::{Address, ValidAddress};
    use common::signed_tx::SignedTx;
    use secp256k1::{RecoverableSignature, RecoveryId, Secp256k1};
    use rust_base58::FromBase58;


    #[test]
    fn it_makes_a_block_from_header() {
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
        let block: Block<Header, SignedTx> = Block::from_header(header.clone());
        match block.txs {
            Some(_) => panic!("Only a header was provided, but the block has transactions!"),
            None => {}
        }
        match block.meta {
            Some(_) => panic!("Only a header was provided, but the block has meta data!"),
            None => {}
        }
        assert_eq!(&block.header, &header);
    }

    #[test]
    fn it_makes_a_block_with_txs() {
        // Set up header
        let previous_hash = vec![vec![74,248,206,224,124,114,100,237,205,62,60,165,
            198,225,77,241,138,87,77,236,55,60,183,46,88,192,18,199,125,23,169,171]];
        let merkle_root = vec![213,169,1,8,101,229,19,22,130,84,151,145,203,
            76,212,233,112,233,14,158,72,47,144,205,35,39,124,171,111,208,24,178];
        let state_root = vec![55,0,90,28,144,19,210,55,242,210,228,153,10,
            149,25,138,245,207,148,195,66,155,204,100,46,118,70,150,151,113,71,7];
        let difficulty = 5.570758908578445e-7;
        let block_nonce = 430;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let time_stamp = 1533891416560;
        let header = Header::new(merkle_root, time_stamp, difficulty, state_root, previous_hash, block_nonce, miner);

        // Set up transaction
        let from = Address::from_string(&"H2aorYbNUbmwvsbupWKLZW7ZUD6VTAc65".to_string()).unwrap();
        let to = Address::from_string(&"H2eztpq215SA3k7bLsnUdriT5MXDMBYEg".to_string()).unwrap();
        let amount = 792673;
        let fee = 97;
        let nonce = 147;
        let signature_bytes = vec![232,181,248,214,104,238,209,39,141,146,180,89,155,42,
            167,166,4,172,51,166,189,138,7,35,100,76,86,242,143,165,171,178,73,219,
            2,255,123,68,168,35,104,15,200,149,92,37,38,242,0,132,2,201,195,19,85,
            25,93,229,34,4,173,6,48,46];
        let recovery = RecoveryId::from_i32(0 as i32).unwrap();
        let secp = Secp256k1::without_caps();
        let signature = RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);
        let txs = vec![signed_tx];

        // Set up block
        let block = Block::new(header.clone(), Some(txs.clone()), None);

        match block.meta {
            Some(_) => panic!("Only a header and txs were provided, but block has meta info!"),
            None => {}
        }

        assert_eq!(&block.header, &header);
        assert_eq!(&block.txs.unwrap(), &txs);
    }

    #[test]
    fn it_encodes_a_block_with_no_txs() {
        let previous_hash = vec![vec![167,196,139,41,65,52,154,132,218,236,238,
            209,119,24,195,185,74,193,125,161,51,205,18,11,115,28,81,195,181,95,204,235]];
        let merkle_root = vec![84,167,41,3,47,127,205,91,122,244,148,190,
            74,247,105,123,204,221,41,133,35,203,168,232,140,88,114,64,153,220,39,74];
        let state_root = vec![7,142,128,226,128,255,32,133,214,42,208,84,
            169,79,154,127,170,171,168,133,176,147,154,165,147,9,247,207,204,221,146,61];
        let time_stamp = 1533897037948;
        let difficulty = 0.000001277063050333107;
        let nonce = 655;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let header = Header::new(merkle_root.clone(), time_stamp, difficulty, state_root.clone(), previous_hash.clone(), nonce, miner);
        let block = Block::from_header(header.clone());
        match block.txs {
            Some(_) => panic!("Only a header was provided, but the block has transactions!"),
            None => {}
        }
        match block.meta {
            Some(_) => panic!("Only a header was provided, but the block has meta data!"),
            None => {}
        }
        let encoding = block.encode().unwrap();
        let expected_encoding = vec![10,143,1,10,32,167,196,139,41,65,52,154,132,218,
            236,238,209,119,24,195,185,74,193,125,161,51,205,18,11,115,28,81,195,181,
            95,204,235,18,32,84,167,41,3,47,127,205,91,122,244,148,190,74,247,105,123,
            204,221,41,133,35,203,168,232,140,88,114,64,153,220,39,74,26,32,7,142,128,
            226,128,255,32,133,214,42,208,84,169,79,154,127,170,171,168,133,176,147,154,
            165,147,9,247,207,204,221,146,61,33,175,189,89,172,241,108,181,62,40,252,
            176,141,155,210,44,48,143,5,58,20,213,49,13,190,194,137,35,119,16,249,57,
            125,207,78,117,246,36,136,151,210];
        assert_eq!(&block.header, &header);
        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_encodes_a_block_with_txs() {
        // Set up header
        let previous_hash = vec![vec![74,248,206,224,124,114,100,237,205,62,60,165,
            198,225,77,241,138,87,77,236,55,60,183,46,88,192,18,199,125,23,169,171]];
        let merkle_root = vec![213,169,1,8,101,229,19,22,130,84,151,145,203,
            76,212,233,112,233,14,158,72,47,144,205,35,39,124,171,111,208,24,178];
        let state_root = vec![55,0,90,28,144,19,210,55,242,210,228,153,10,
            149,25,138,245,207,148,195,66,155,204,100,46,118,70,150,151,113,71,7];
        let difficulty = 5.570758908578445e-7;
        let block_nonce = 430;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let time_stamp = 1533891416560;
        let header = Header::new(merkle_root, time_stamp, difficulty, state_root, previous_hash, block_nonce, miner);

        // Set up transaction
        let from = Address::from_string(&"H2aorYbNUbmwvsbupWKLZW7ZUD6VTAc65".to_string()).unwrap();
        let to = Address::from_string(&"H2eztpq215SA3k7bLsnUdriT5MXDMBYEg".to_string()).unwrap();
        let amount = 792673;
        let fee = 97;
        let nonce = 147;
        let signature_bytes = vec![232,181,248,214,104,238,209,39,141,146,180,89,155,42,
            167,166,4,172,51,166,189,138,7,35,100,76,86,242,143,165,171,178,73,219,
            2,255,123,68,168,35,104,15,200,149,92,37,38,242,0,132,2,201,195,19,85,
            25,93,229,34,4,173,6,48,46];
        let recovery = RecoveryId::from_i32(0 as i32).unwrap();
        let secp = Secp256k1::without_caps();
        let signature = RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);
        let txs = vec![signed_tx];

        // Set up block
        let block = Block::new(header.clone(), Some(txs.clone()), None);
        let encoding = block.encode().unwrap();
        let expected_encoding = vec![10,143,1,10,32,74,248,206,224,124,114,100,237,205,
            62,60,165,198,225,77,241,138,87,77,236,55,60,183,46,88,192,18,199,125,23,
            169,171,18,32,213,169,1,8,101,229,19,22,130,84,151,145,203,76,212,233,112,
            233,14,158,72,47,144,205,35,39,124,171,111,208,24,178,26,32,55,0,90,28,144,
            19,210,55,242,210,228,153,10,149,25,138,245,207,148,195,66,155,204,100,46,
            118,70,150,151,113,71,7,33,211,128,207,214,62,177,162,62,40,240,163,182,152,
            210,44,48,174,3,58,20,213,49,13,190,194,137,35,119,16,249,57,125,207,78,117,
            246,36,136,151,210,18,121,10,20,113,158,71,197,83,100,207,140,177,10,169,167,
            65,166,7,77,173,138,90,182,18,20,118,205,217,87,194,165,97,21,105,47,106,64,
            100,210,68,68,107,251,151,100,24,225,176,48,32,97,40,147,1,50,64,232,181,248,
            214,104,238,209,39,141,146,180,89,155,42,167,166,4,172,51,166,189,138,7,35,100,
            76,86,242,143,165,171,178,73,219,2,255,123,68,168,35,104,15,200,149,92,37,38,
            242,0,132,2,201,195,19,85,25,93,229,34,4,173,6,48,46,56,0];
        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_encodes_a_block_with_txs_and_meta() {
        // Set up header

        let previous_hash = vec![vec![74,248,206,224,124,114,100,237,205,62,60,165,
            198,225,77,241,138,87,77,236,55,60,183,46,88,192,18,199,125,23,169,171]];
        let merkle_root = vec![213,169,1,8,101,229,19,22,130,84,151,145,203,
            76,212,233,112,233,14,158,72,47,144,205,35,39,124,171,111,208,24,178];
        let state_root = vec![55,0,90,28,144,19,210,55,242,210,228,153,10,
            149,25,138,245,207,148,195,66,155,204,100,46,118,70,150,151,113,71,7];
        let difficulty = 5.570758908578445e-7;
        let block_nonce = 430;
        let miner = Address::from_string(&"H3yGUaF38TxQxoFrqCqPdB2pN9jyBHnaj".to_string()).unwrap();
        let time_stamp = 1533891416560;
        let header = Header::new(merkle_root, time_stamp, difficulty, state_root, previous_hash, block_nonce, miner);

        // Set up transaction
        let from = Address::from_string(&"H2aorYbNUbmwvsbupWKLZW7ZUD6VTAc65".to_string()).unwrap();
        let to = Address::from_string(&"H2eztpq215SA3k7bLsnUdriT5MXDMBYEg".to_string()).unwrap();
        let amount = 792673;
        let fee = 97;
        let nonce = 147;
        let signature_bytes = vec![232,181,248,214,104,238,209,39,141,146,180,89,155,42,
            167,166,4,172,51,166,189,138,7,35,100,76,86,242,143,165,171,178,73,219,
            2,255,123,68,168,35,104,15,200,149,92,37,38,242,0,132,2,201,195,19,85,
            25,93,229,34,4,173,6,48,46];
        let recovery = RecoveryId::from_i32(0 as i32).unwrap();
        let secp = Secp256k1::without_caps();
        let signature = RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);
        let txs = vec![signed_tx];

        // Set up Meta
        let meta = Meta::new(1, 2 as f64, 3 as f64, 4 as f64, 5 as f64, Some(6), Some(7), Some(8));
        // Set up block
        let block = Block::new(header.clone(), Some(txs.clone()), Some(meta.clone()));
        let encoding = block.encode().unwrap();
        let expected_encoding = vec![10,143,1,10,32,74,248,206,224,124,114,100,237,205,
            62,60,165,198,225,77,241,138,87,77,236,55,60,183,46,88,192,18,199,125,23,
            169,171,18,32,213,169,1,8,101,229,19,22,130,84,151,145,203,76,212,233,112,
            233,14,158,72,47,144,205,35,39,124,171,111,208,24,178,26,32,55,0,90,28,144,
            19,210,55,242,210,228,153,10,149,25,138,245,207,148,195,66,155,204,100,46,
            118,70,150,151,113,71,7,33,211,128,207,214,62,177,162,62,40,240,163,182,152,
            210,44,48,174,3,58,20,213,49,13,190,194,137,35,119,16,249,57,125,207,78,117,
            246,36,136,151,210,18,121,10,20,113,158,71,197,83,100,207,140,177,10,169,167,
            65,166,7,77,173,138,90,182,18,20,118,205,217,87,194,165,97,21,105,47,106,64,
            100,210,68,68,107,251,151,100,24,225,176,48,32,97,40,147,1,50,64,232,181,248,
            214,104,238,209,39,141,146,180,89,155,42,167,166,4,172,51,166,189,138,7,35,100,
            76,86,242,143,165,171,178,73,219,2,255,123,68,168,35,104,15,200,149,92,37,38,
            242,0,132,2,201,195,19,85,25,93,229,34,4,173,6,48,46,56,0];
        assert_eq!(encoding, expected_encoding);
    }
}