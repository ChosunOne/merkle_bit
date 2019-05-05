use crate::utils::tree_ref::TreeRef;

/// This is primarily for marking this collection as `Send` and `Sync`.
pub struct MergeCell {
    /// The split index to be used.
    pub split_index: u8,
    /// A raw pointer to the `TreeRef`.
    pub tree_ref_pointer: *mut TreeRef,
    /// A raw pointer to the adjacent `TreeRef`.
    pub next_tree_ref_pointer: *mut TreeRef,
    /// The index in the list of `tree_refs` of `tree_ref_pointer`.
    pub index: usize
}

impl MergeCell {
    /// Creates a new `MergeCell`.
    #[inline]
    pub const fn new(split_index: u8, tree_ref_pointer: *mut TreeRef, next_tree_ref_pointer: *mut TreeRef, index: usize) -> Self {
        Self {
            split_index,
            tree_ref_pointer,
            next_tree_ref_pointer,
            index
        }
    }

    /// Decomposes the structure into its constituent parts
    #[inline]
    pub const fn deconstruct(self) -> (u8, *mut TreeRef, *mut TreeRef, usize) {
        (self.split_index, self.tree_ref_pointer, self.next_tree_ref_pointer, self.index)
    }
}

#[cfg(feature = "use_rayon")]
unsafe impl Send for MergeCell {}
#[cfg(feature = "use_rayon")]
unsafe impl Sync for MergeCell {}