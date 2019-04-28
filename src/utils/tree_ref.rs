use std::cmp::Ordering;

use crate::constants::KEY_LEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct TreeRef {
    pub key: [u8; KEY_LEN],
    pub location: [u8; KEY_LEN],
    pub node_count: u64,
    pub count: u32,
}

impl TreeRef {
    pub fn new(key: [u8; KEY_LEN], location: [u8; KEY_LEN], node_count: u64, count: u32) -> TreeRef {
        TreeRef {
            key,
            location,
            node_count,
            count
        }
    }
}

impl Ord for TreeRef {
    fn cmp(&self, other_ref: &TreeRef) -> Ordering {
        self.key.cmp(&other_ref.key)
    }
}
