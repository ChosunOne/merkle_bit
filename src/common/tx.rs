use std::error::Error;

use common::{Decode, Encode, Proto};
use common::address::Address;
use common::transaction::Transaction;
use serialization::tx::Tx as ProtoTx;

use secp256k1::{RecoverableSignature, RecoveryId};
use protobuf::Message as ProtoMessage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u32
}

impl Tx {
    pub fn new(from: Address, to: Address, amount: u64, fee: u64, nonce: u32) -> Tx {
        Tx {
            from,
            to,
            amount,
            fee,
            nonce
        }
    }
}

impl Transaction for Tx {
    fn get_from(&self) -> Option<Address> {Some(self.from)}
    fn get_to(&self) -> Option<Address> {Some(self.to)}
    fn get_amount(&self) -> u64 {self.amount}
    fn get_fee(&self) -> Option<u64> {Some(self.fee)}
    fn get_nonce(&self) -> Option<u32> {Some(self.nonce)}
    fn get_signature(&self) -> Option<RecoverableSignature> {None}
    fn get_recovery(&self) -> Option<RecoveryId> {None}
}

impl Proto for Tx {
    type ProtoType = ProtoTx;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_tx = ProtoTx::new();
        proto_tx.set_from(self.from.to_vec());
        proto_tx.set_to(self.to.to_vec());
        proto_tx.set_amount(self.amount);
        proto_tx.set_fee(self.fee);
        proto_tx.set_nonce(self.nonce);
        Ok(proto_tx)
    }
}

impl Encode for Tx {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_tx = self.to_proto()?;
        Ok(proto_tx.write_to_bytes()?)
    }
}

impl Decode for Tx {
    type ProtoType = ProtoTx;
    fn decode(buffer: &Vec<u8>) -> Result<Tx, Box<Error>> {
        let mut proto_tx = ProtoTx::new();
        proto_tx.merge_from_bytes(&buffer)?;
        let mut from = [0u8; 20];
        from.clone_from_slice(&proto_tx.from);
        let mut to = [0u8; 20];
        to.clone_from_slice(&proto_tx.to);

        Ok(Tx::new(from, to, proto_tx.amount, proto_tx.fee, proto_tx.nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::ValidAddress;
    use rand::{thread_rng, Rng};

    #[test]
    fn it_makes_a_transaction() {
        let from = [
            230, 104, 95, 253, 219, 134, 92, 215, 230, 126, 105, 213, 18, 95, 30, 166, 128, 229,
            233, 114,
        ];
        let to = [
            87, 217, 90, 40, 10, 141, 125, 74, 177, 128, 155, 18, 148, 149, 135, 84, 9, 224, 232,
            102,
        ];
        let amount = 123456789;
        let fee = 1;
        let nonce = 3;
        let tx = Tx::new(from, to, amount, fee, nonce);
        assert_eq!(tx.from, from);
        assert_eq!(tx.to, to);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.fee, fee);
        assert_eq!(tx.nonce, nonce);
    }

    #[test]
    fn it_encodes_like_javascript_for_non_zero() {
        let from = [
            230, 104, 95, 253, 219, 134, 92, 215, 230, 126, 105, 213, 18, 95, 30, 166, 128, 229,
            233, 114,
        ];
        let to = [
            87, 217, 90, 40, 10, 141, 125, 74, 177, 128, 155, 18, 148, 149, 135, 84, 9, 224, 232,
            102,
        ];
        let amount = 123456789;
        let fee = 1;
        let nonce = 3;
        let tx = Tx::new(from, to, amount, fee, nonce);
        let encoding = tx.encode().unwrap();
        let expected_encoding = vec![
            10, 20, 230, 104, 95, 253, 219, 134, 92, 215, 230, 126, 105, 213, 18, 95, 30, 166, 128,
            229, 233, 114, 18, 20, 87, 217, 90, 40, 10, 141, 125, 74, 177, 128, 155, 18, 148, 149,
            135, 84, 9, 224, 232, 102, 24, 149, 154, 239, 58, 32, 1, 40, 3,
        ];
        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_encodes_like_javascript_for_large_amounts() {
        let from = [41,251,67,236,239,131,69,76,102,112,26,52,242,162,24,220,242,33,163,105];
        let to = [231,178,9,132,67,165,167,239,54,145,232,222,104,147,104,123,252,196,68,82];
        let amount = 23892147312890090;
        let fee = 7787639375790336;
        let nonce = 364750872;
        let tx = Tx::new(from, to, amount, fee, nonce);
        let encoding = tx.encode().unwrap();
        let expected_encoding = vec![10,20,41,251,67,236,239,131,69,76,102,112,26,52,242,162,
                                     24,220,242,33,163,105,18,20,231,178,9,132,67,165,167,239,54,145,232,222,104,147,
                                     104,123,252,196,68,82,24,234,161,134,204,128,185,184,42,32,128,130,136,181,145,
                                     218,234,13,40,152,208,246,173,1];
        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_encodes_like_javascript_for_zero() {
        let from: Address =
            Address::from_string(&"H2rCdhQ4fhGk5qX9AwzxA61zhoUKCDVQC".to_string()).unwrap();
        let to: Address =
            Address::from_string(&"Hj3eZJpesfCjrMZfmKXpep6rVWS56Qaz".to_string()).unwrap();
        let amount = 0;
        let fee = 0;
        let nonce = 0;
        let tx = Tx::new(from, to, amount, fee, nonce);
        let encoding = tx.encode().unwrap();
        let expected_encoding = vec![
            10, 20, 132, 170, 245, 157, 55, 19, 7, 190, 193, 159, 54, 150, 44, 139, 78, 36, 165,
            149, 140, 187, 18, 20, 52, 8, 198, 113, 205, 252, 248, 236, 75, 130, 108, 209, 4, 214,
            46, 51, 111, 17, 216, 146, 24, 0, 32, 0, 40, 0,
        ];
        assert_eq!(encoding, expected_encoding);
    }

    #[test]
    fn it_decodes_an_encoded_tx() {
        let from = [
            230, 104, 95, 253, 219, 134, 92, 215, 230, 126, 105, 213, 18, 95, 30, 166, 128, 229,
            233, 114,
        ];

        let to = [
            87, 217, 90, 40, 10, 141, 125, 74, 177, 128, 155, 18, 148, 149, 135, 84, 9, 224, 232,
            102,
        ];

        let amount = 123456789;
        let fee = 1;
        let nonce = 3;
        let tx = Tx::new(from, to, amount, fee, nonce);
        let encoding = tx.encode().unwrap();
        let decoded_tx = Tx::decode(&encoding).unwrap();

        assert_eq!(tx, decoded_tx);
    }

    #[test]
    #[should_panic]
    fn it_fails_to_decode_random_bad_bytes() {
        let mut random_bytes = [0u8; 256];
        thread_rng().fill(&mut random_bytes);
        Tx::decode(&random_bytes.to_vec()).unwrap();
    }
}