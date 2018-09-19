use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub mod binary_merkle_tree;
pub mod traits;

pub trait Encode {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>>;
}

pub trait Decode {
    fn decode(buffer: &[u8]) -> Result<Self, Box<Error>>
        where Self: Sized;
}

pub trait Proto {
    type ProtoType;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>>;
}

#[derive(Debug)]
pub struct Exception {
    details:  String
}

impl Exception {
    pub fn new(details: &str) -> Exception {
        Exception {
            details: details.to_string()
        }
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f,"{}",self.details)
    }
}

impl Error for Exception {
    fn description(&self) -> &str {
        &self.details
    }
}