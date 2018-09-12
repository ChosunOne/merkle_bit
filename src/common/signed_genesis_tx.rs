use std::error::Error;

use common::address::Address;
use common::transaction::{Transaction, Valid, verify_tx};
use common::genesis_tx::GenesisTx;
use common::{Decode, Encode, Exception, Proto};
use serialization::tx::GenesisSignedTx as ProtoGenesisSignedTx;

use secp256k1::{RecoverableSignature, RecoveryId, Secp256k1};
use protobuf::Message as ProtoMessage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedGenesisTx {
    to: Address,
    amount: u64,
    signature: RecoverableSignature,
    recovery: RecoveryId,
}

impl Transaction for SignedGenesisTx {
    fn get_from(&self) -> Option<Address> {None}
    fn get_to(&self) -> Option<Address> {Some(self.to)}
    fn get_amount(&self) -> u64 {self.amount}
    fn get_fee(&self) -> Option<u64> {None}
    fn get_nonce(&self) -> Option<u32> {None}
    fn get_signature(&self) -> Option<RecoverableSignature> {Some(self.signature)}
    fn get_recovery(&self) -> Option<RecoveryId> {Some(self.recovery)}
}

impl SignedGenesisTx {
    pub fn new(to: Address, amount: u64, signature: RecoverableSignature, recovery: RecoveryId) -> SignedGenesisTx {
        SignedGenesisTx {
            to,
            amount,
            signature,
            recovery,
        }
    }
}

impl Proto for SignedGenesisTx {
    type ProtoType = ProtoGenesisSignedTx;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_signed_genesis_tx = ProtoGenesisSignedTx::new();
        proto_signed_genesis_tx.set_to(self.to.to_vec());
        proto_signed_genesis_tx.set_amount(self.amount);
        proto_signed_genesis_tx.set_recovery(self.recovery.to_i32() as u32);
        let secp = Secp256k1::without_caps();
        proto_signed_genesis_tx.set_signature(self.signature.serialize_compact(&secp).1.to_vec());
        Ok(proto_signed_genesis_tx)
    }
}

impl Encode for SignedGenesisTx {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_signed_genesis_tx = self.to_proto()?;
        Ok(proto_signed_genesis_tx.write_to_bytes()?)
    }
}

impl Decode for SignedGenesisTx {
    type ProtoType = ProtoGenesisSignedTx;
    fn decode(buffer: &Vec<u8>) -> Result<SignedGenesisTx, Box<Error>> {
        let secp = Secp256k1::without_caps();
        let mut proto_signed_genesis_tx = ProtoGenesisSignedTx::new();
        proto_signed_genesis_tx.merge_from_bytes(&buffer)?;
        let mut to = [0u8; 20];
        to.clone_from_slice(&proto_signed_genesis_tx.to);
        let recovery = RecoveryId::from_i32(proto_signed_genesis_tx.recovery as i32)?;
        let signature = RecoverableSignature::from_compact(&secp, &proto_signed_genesis_tx.signature, recovery)?;
        Ok(SignedGenesisTx::new(to, proto_signed_genesis_tx.amount, signature, recovery))
    }
}

impl Valid for SignedGenesisTx {
    fn verify(&self) -> Result<(), Box<Error>> {
        let genesis_tx = GenesisTx::new(self.to, self.amount);
        let encoding = genesis_tx.encode()?;
        verify_tx(encoding, self.to, self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::address::ValidAddress;
    use rand::{thread_rng, Rng};

    #[test]
    fn it_creates_a_signed_genesis_transaction() {
        let to: Address = Address::from_string(&"HLjHZYkjRNkjH3zPmXoU8FDEJ3ALDkuA".to_string()).unwrap();
        let amount = 100;
        let signature_bytes = [155,15,206,7,232,20,132,186,33,220,220,31,36,100,48,103,61,198,40,
        155,48,189,196,64,162,132,254,252,160,242,136,253,42,105,138,104,227,162,198,254,59,114,252,
        62,3,211,77,93,196,72,221,18,128,112,143,185,199,178,56,0,141,232,12,201];
        let secp = Secp256k1::without_caps();
        let recovery = RecoveryId::from_i32(0).unwrap();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_genesis_tx = SignedGenesisTx::new(to, amount, signature, recovery);
        assert_eq!(to, signed_genesis_tx.to);
        assert_eq!(amount, signed_genesis_tx.amount);
        assert_eq!(signature, signed_genesis_tx.signature);
        assert_eq!(recovery, signed_genesis_tx.recovery);
    }

    #[test]
    fn it_verifies_a_signed_genesis_transaction() {
        let to: Address = Address::from_string(&"HLjHZYkjRNkjH3zPmXoU8FDEJ3ALDkuA".to_string()).unwrap();
        let amount = 100;
        let signature_bytes = [155,15,206,7,232,20,132,186,33,220,220,31,36,100,48,103,61,198,40,
        155,48,189,196,64,162,132,254,252,160,242,136,253,42,105,138,104,227,162,198,254,59,114,252,
        62,3,211,77,93,196,72,221,18,128,112,143,185,199,178,56,0,141,232,12,201];
        let secp = Secp256k1::without_caps();
        let recovery = RecoveryId::from_i32(0).unwrap();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_genesis_tx = SignedGenesisTx::new(to, amount, signature, recovery);
        signed_genesis_tx.verify().unwrap();
    }
    
    #[test]
    fn it_rejects_a_forged_genesis_transaction() {
        let to: Address = Address::from_string(&"HLjHZYkjRNkjH3zPmXoU8FDEJ3ALDkuA".to_string()).unwrap();
        let amount = 200;
        let signature_bytes = [155,15,206,7,232,20,132,186,33,220,220,31,36,100,48,103,61,198,40,
        155,48,189,196,64,162,132,254,252,160,242,136,253,42,105,138,104,227,162,198,254,59,114,252,
        62,3,211,77,93,196,72,221,18,128,112,143,185,199,178,56,0,141,232,12,201];
        let secp = Secp256k1::without_caps();
        let recovery = RecoveryId::from_i32(0).unwrap();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_genesis_tx = SignedGenesisTx::new(to, amount, signature, recovery);
        match signed_genesis_tx.verify() {
            Ok(_) => panic!("Invalid signature was reported as verified"),
            Err(_) => {}
        }
    }

    #[test]
    fn it_decodes_a_signed_genesis_transaction() {
        let to: Address = Address::from_string(&"HLjHZYkjRNkjH3zPmXoU8FDEJ3ALDkuA".to_string()).unwrap();
        let amount = 100;
        let signature_bytes = [155,15,206,7,232,20,132,186,33,220,220,31,36,100,48,103,61,198,40,
            155,48,189,196,64,162,132,254,252,160,242,136,253,42,105,138,104,227,162,198,254,59,114,252,
            62,3,211,77,93,196,72,221,18,128,112,143,185,199,178,56,0,141,232,12,201];
        let secp = Secp256k1::without_caps();
        let recovery = RecoveryId::from_i32(0).unwrap();
        let signature =
            RecoverableSignature::from_compact(&secp, &signature_bytes, recovery).unwrap();
        let signed_genesis_tx = SignedGenesisTx::new(to, amount, signature, recovery);
        let encoding = signed_genesis_tx.encode().unwrap();
        let decoded_signed_genesis_tx = SignedGenesisTx::decode(&encoding).unwrap();
        assert_eq!(signed_genesis_tx, decoded_signed_genesis_tx);
    }

    #[test]
    #[should_panic]
    fn it_fails_to_decode_random_bad_bytes() {
        let mut random_bytes = [0u8; 256];
        thread_rng().fill(&mut random_bytes);
        SignedGenesisTx::decode(&random_bytes.to_vec()).unwrap();
    }
}