use rust_base58::{FromBase58, ToBase58};
use secp256k1::PublicKey;
use util::hash::hash;

use std::result::Result;

fn check_sum(arr: &[u8; 20]) -> String {
    let arr_hash = hash(arr, 32);
    let string = arr_hash.to_base58();
    format!("{}", &string[0..4])
}

pub type Address = [u8; 20];

pub trait ValidAddress {
    fn to_string(&self) -> String;
    fn from_string(string: &String) -> Result<Address, String>;
    fn from_pubkey(pubkey: PublicKey) -> Address;
}

impl ValidAddress for Address {
    fn to_string(&self) -> String {
        let base58_address = self.to_base58();
        "H".to_string() + &base58_address + &check_sum(&self)
    }

    fn from_string(string: &String) -> Result<Address, String> {
        let mut string_iter = string.chars();
        let first_char = string_iter.next();
        match first_char {
            Some(letter) => {
                if letter.to_string() != "H" {
                    return Err("Address must begin with an H".to_string());
                }
            }
            None => return Err("No data was supplied".to_string()),
        }

        let address_and_checksum = &string[1..string.len()];
        let address = &address_and_checksum[0..address_and_checksum.len() - 4];
        let checksum =
            &address_and_checksum[address_and_checksum.len() - 4..address_and_checksum.len()];

        let decoded = address.from_base58();
        let mut decoded_bytes: [u8; 20] = [0; 20];
        match decoded {
            Ok(s) => {
                if s.len() != 20 {
                    return Err(format!("{} is {} bytes long", address, decoded_bytes.len()));
                }
                decoded_bytes.clone_from_slice(&s[..]);
            }
            Err(e) => return Err(format!("{}", e)),
        }

        let checksum_bytes = check_sum(&decoded_bytes);
        if checksum_bytes.to_string() != checksum {
            return Err(format!(
                "{} did not match {}",
                checksum,
                checksum_bytes.to_string()
            ));
        }

        let mut address_bytes: [u8; 20] = [0; 20];
        address_bytes.clone_from_slice(&decoded_bytes[0..20]);
        Ok(address_bytes)
    }

    fn from_pubkey(pubkey: PublicKey) -> Address {
        let pub_key_hash = hash(&pubkey.serialize(), 32);
        let mut addr = [0; 20];
        for i in 12..32 {
            addr[i - 12] = pub_key_hash[i];
        }
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::Secp256k1;
    #[test]
    fn it_makes_an_address_string() {
        let address: Address = [
            11, 216, 55, 107, 140, 247, 121, 126, 90, 115, 233, 197, 108, 128, 64, 46, 135, 184,
            87, 180,
        ];
        let expected_address_string = "HAa7S1QqVRMw13VdUnrSkn5w6oNK891W";
        assert_eq!(expected_address_string, address.to_string());
    }

    #[test]
    fn it_makes_an_address_from_string() {
        let address_string = "HAa7S1QqVRMw13VdUnrSkn5w6oNK891W".to_string();
        let address: Address = Address::from_string(&address_string).unwrap();
        assert_eq!(address.to_string(), address_string);
    }

    #[test]
    fn it_makes_an_address_from_pubkey() {
        let pubkey_bytes = [
            2, 170, 74, 64, 253, 208, 212, 90, 203, 135, 144, 142, 65, 16, 248, 16, 212, 186, 252,
            206, 57, 137, 163, 162, 40, 81, 138, 12, 183, 237, 70, 48, 136,
        ];
        let secp = Secp256k1::without_caps();
        let pubkey = PublicKey::from_slice(&secp, &pubkey_bytes).unwrap();
        let address_string = "HM8NwBmP6FKyhiBaxQMWcZiAoZqQFk8a".to_string();
        let address_from_string = Address::from_string(&address_string).unwrap();
        let address_from_pubkey = Address::from_pubkey(pubkey);
        assert_eq!(address_from_string, address_from_pubkey);
    }

    #[test]
    fn it_only_allows_valid_addresses() {
        let address_string = "Not a valid address string".to_string();
        let address = Address::from_string(&address_string);
        match address {
            Ok(_addr) => panic!("{} is not a valid address string!", address_string),
            Err(_) => {}
        }
    }

    #[test]
    fn it_checks_checksum() {
        let address_string = "HAa7S1QqVRMw13VdUnrSkn5w6oNK36MM".to_string();
        let address = Address::from_string(&address_string);
        match address {
            Ok(_addr) => panic!("{} is not a valid address string!", address_string),
            Err(_) => {}
        }
    }

    #[test]
    fn it_checks_equality() {
        let address_string = "HAa7S1QqVRMw13VdUnrSkn5w6oNK891W".to_string();
        let address: Address = Address::from_string(&address_string).unwrap();

        let other_address: [u8; 20] = [
            11, 216, 55, 107, 140, 247, 121, 126, 90, 115, 233, 197, 108, 128, 64, 46, 135, 184,
            87, 180,
        ];
        assert_eq!(address, other_address);
    }
}
