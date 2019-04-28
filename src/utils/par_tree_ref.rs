use std::cmp::Ordering;
use std::sync::Arc;

use crate::constants::KEY_LEN;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct TreeRef {
    pub key: [u8; KEY_LEN],
    pub location: [u8; KEY_LEN],
    pub count: u64,
}

impl TreeRef {
    pub fn new(key: [u8; KEY_LEN], location: [u8; KEY_LEN], count: u64) -> TreeRef {
        TreeRef {
            key,
            location,
            count,
        }
    }
}

impl Ord for TreeRef {
    fn cmp(&self, other_ref: &TreeRef) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}
