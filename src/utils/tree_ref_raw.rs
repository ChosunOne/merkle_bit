use std::ops::Deref;
use crate::utils::tree_ref::TreeRef;

pub struct TreeRefRaw(pub *mut TreeRef);

impl Deref for TreeRefRaw {
    type Target = *mut TreeRef;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for TreeRefRaw {}
unsafe impl Sync for TreeRefRaw {}