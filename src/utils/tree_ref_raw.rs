use crate::utils::tree_ref::TreeRef;
use std::ops::Deref;

/// This is a wrapper around a raw pointer of a `TreeRef`.
/// Used primarily to mark it as Send and Sync.
pub struct TreeRefRaw(pub *mut TreeRef);

impl Deref for TreeRefRaw {
    type Target = *mut TreeRef;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for TreeRefRaw {}
unsafe impl Sync for TreeRefRaw {}
