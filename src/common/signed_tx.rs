use std::ops::Deref;
use std::error::Error;

use common::address::Address;
use common::transaction::{Transaction, Valid, verify_tx};
use common::tx::Tx;
use common::{Decode, Encode, Exception, Proto};

use serialization::tx::SignedTx as ProtoSignedTx;

use protobuf::Message as ProtoMessage;
use secp256k1::{Error as SecpError, RecoverableSignature, RecoveryId, Secp256k1};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTx {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u32,
    pub signature: RecoverableSignature,
    pub recovery: RecoveryId,
}

impl Transaction for SignedTx {
    fn get_from(&self) -> Option<Address> {Some(self.from)}
    fn get_to(&self) -> Option<Address> {Some(self.to)}
    fn get_amount(&self) -> u64 {self.amount}
    fn get_fee(&self) -> Option<u64> {Some(self.fee)}
    fn get_nonce(&self) -> Option<u32> {Some(self.nonce)}
    fn get_signature(&self) -> Option<RecoverableSignature> {Some(self.signature)}
    fn get_recovery(&self) -> Option<RecoveryId> {Some(self.recovery)}
}

impl SignedTx {
    pub fn new(from: Address, to: Address, amount: u64, fee: u64, nonce: u32, signature: RecoverableSignature, recovery: RecoveryId) -> SignedTx {
        SignedTx {
            from,
            to,
            amount,
            fee,
            nonce,
            signature,
            recovery,
        }
    }

    pub fn from_tx(tx: &Tx, signature: RecoverableSignature, recovery: RecoveryId) -> SignedTx {
        SignedTx {
            from: tx.from,
            to: tx.to,
            amount: tx.amount,
            fee: tx.fee,
            nonce: tx.nonce,
            signature,
            recovery
        }
    }
}

impl Proto for SignedTx {
    type ProtoType = ProtoSignedTx;
    fn to_proto(&self) -> Result<ProtoSignedTx, Box<Error>> {
        let mut proto_signed_tx = ProtoSignedTx::new();
        proto_signed_tx.set_from(self.from.to_vec());
        proto_signed_tx.set_to(self.to.to_vec());
        proto_signed_tx.set_amount(self.amount);
        proto_signed_tx.set_fee(self.fee);
        proto_signed_tx.set_nonce(self.nonce);
        proto_signed_tx.set_recovery(self.recovery.to_i32() as u32);
        let secp = Secp256k1::without_caps();
        proto_signed_tx.set_signature(self.signature.serialize_compact(&secp).1.to_vec());
        Ok(proto_signed_tx)
    }
}

impl Encode for SignedTx {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_signed_tx = self.to_proto()?;
        Ok(proto_signed_tx.write_to_bytes()?)
    }
}

impl Decode for SignedTx {
    type ProtoType = ProtoSignedTx;
    fn decode(buffer: &Vec<u8>) -> Result<SignedTx, Box<Error>> {
        let secp = Secp256k1::without_caps();
        let mut proto_signed_tx = ProtoSignedTx::new();
        proto_signed_tx.merge_from_bytes(&buffer)?;
        let mut from = [0u8; 20];
        from.clone_from_slice(&proto_signed_tx.from);
        let mut to = [0u8; 20];
        to.clone_from_slice(&proto_signed_tx.to);
        let recovery = RecoveryId::from_i32(proto_signed_tx.recovery as i32)?;
        let signature = RecoverableSignature::from_compact(&secp, &proto_signed_tx.signature, recovery)?;
        Ok(SignedTx::new(from, to, proto_signed_tx.amount, proto_signed_tx.fee, proto_signed_tx.nonce, signature, recovery))
    }
}

impl Valid for SignedTx {
    fn verify(&self) -> Result<(), Box<Error>> {
        let tx = Tx::new(self.from, self.to, self.amount, self.fee, self.nonce);
        let encoding = tx.encode()?;
        verify_tx(encoding, self.from, self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::ValidAddress;
    use rand::{thread_rng, Rng};

    #[test]
    fn it_verifies_a_signed_tx() {
        let from_addr = "H27McLosW8psFMbQ8VPQwXxnUY8QAHBHr".to_string();
        let from = Address::from_string(&from_addr).unwrap();
        let to_addr = "H4JSXdLtkXVs6G7fk2xea1dB4hTgQ3ps6".to_string();
        let to = Address::from_string(&to_addr).unwrap();
        let amount = 100;
        let fee = 1;
        let nonce = 1;
        let recovery = RecoveryId::from_i32(0).unwrap();

        let signature_bytes = [
            208, 50, 197, 4, 84, 254, 196, 173, 123, 37, 234, 93, 48, 249, 247, 56, 156, 54, 7,
            211, 17, 121, 174, 74, 111, 1, 7, 184, 82, 196, 94, 176, 73, 221, 78, 105, 137, 12,
            165, 212, 15, 47, 134, 101, 221, 69, 158, 19, 237, 120, 63, 173, 92, 215, 144, 224,
            100, 78, 84, 128, 237, 25, 234, 206,
        ];
        let secp = Secp256k1::without_caps();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();

        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);
        signed_tx.verify().unwrap();
    }

    #[test]
    #[should_panic]
    fn it_rejects_a_forged_tx() {
        let from_addr = "H27McLosW8psFMbQ8VPQwXxnUY8QAHBHr".to_string();
        let from = Address::from_string(&from_addr).unwrap();
        let to_addr = "H4JSXdLtkXVs6G7fk2xea1dB4hTgQ3ps6".to_string();
        let to = Address::from_string(&to_addr).unwrap();
        let amount = 200;
        let fee = 1;
        let nonce = 1;
        let recovery = RecoveryId::from_i32(0).unwrap();

        let signature_bytes = [
            208, 50, 197, 4, 84, 254, 196, 173, 123, 37, 234, 93, 48, 249, 247, 56, 156, 54, 7,
            211, 17, 121, 174, 74, 111, 1, 7, 184, 82, 196, 94, 176, 73, 221, 78, 105, 137, 12,
            165, 212, 15, 47, 134, 101, 221, 69, 158, 19, 237, 120, 63, 173, 92, 215, 144, 224,
            100, 78, 84, 128, 237, 25, 234, 206,
        ];
        let secp = Secp256k1::without_caps();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();

        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);

        signed_tx.verify().unwrap();
    }

    #[test]
    fn it_decodes_a_signed_tx() {
        let from_addr = "H27McLosW8psFMbQ8VPQwXxnUY8QAHBHr".to_string();
        let from = Address::from_string(&from_addr).unwrap();
        let to_addr = "H4JSXdLtkXVs6G7fk2xea1dB4hTgQ3ps6".to_string();
        let to = Address::from_string(&to_addr).unwrap();
        let amount = 100;
        let fee = 1;
        let nonce = 1;
        let recovery = RecoveryId::from_i32(0).unwrap();

        let signature_bytes = [
            208, 50, 197, 4, 84, 254, 196, 173, 123, 37, 234, 93, 48, 249, 247, 56, 156, 54, 7,
            211, 17, 121, 174, 74, 111, 1, 7, 184, 82, 196, 94, 176, 73, 221, 78, 105, 137, 12,
            165, 212, 15, 47, 134, 101, 221, 69, 158, 19, 237, 120, 63, 173, 92, 215, 144, 224,
            100, 78, 84, 128, 237, 25, 234, 206,
        ];
        let secp = Secp256k1::without_caps();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();

        let signed_tx = SignedTx::new(from, to, amount, fee, nonce, signature, recovery);
        let encoding = signed_tx.encode().unwrap();
        let decoded_signed_tx = SignedTx::decode(&encoding).unwrap();

        assert_eq!(signed_tx, decoded_signed_tx);
    }

    #[test]
    #[should_panic]
    fn it_fails_to_decode_random_bad_bytes() {
        let mut random_bytes = [0u8; 256];
        thread_rng().fill(&mut random_bytes);
        SignedTx::decode(&random_bytes.to_vec()).unwrap();
    }
}