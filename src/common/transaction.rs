use std::error::Error;

use common::{Encode, Proto};
use common::address::{Address, ValidAddress};
use util::hash::hash;
use serialization::tx::Tx as ProtoTx;

use protobuf::{Message as ProtoMessage};
use secp256k1::{Message, RecoverableSignature, RecoveryId, Secp256k1, Error as SecpError};

pub fn verify_tx(encoding: Vec<u8>, signer: Address, signature: RecoverableSignature) -> Result<(), Box<Error>> {
    let message = Message::from_slice(&hash(&encoding, 32))?;
    let secp = Secp256k1::verification_only();
    let pubkey = secp.recover(&message, &signature)?;
    let address = Address::from_pubkey(pubkey);
    if address != signer {
        return Err(Box::new(SecpError::IncorrectSignature));
    }
    let standard_signature = signature.to_standard(&secp);
    Ok(secp.verify(&message, &standard_signature, &pubkey)?)
}

pub trait Transaction {
    fn get_from(&self) -> Option<Address>;
    fn get_to(&self) -> Option<Address>;
    fn get_amount(&self) -> u64;
    fn get_fee(&self) -> Option<u64>;
    fn get_nonce(&self) -> Option<u32>;
    fn get_signature(&self) -> Option<RecoverableSignature>;
    fn get_recovery(&self) -> Option<RecoveryId>;
}

pub trait Valid {
    fn verify(&self) -> Result<(), Box<Error>>;
}


