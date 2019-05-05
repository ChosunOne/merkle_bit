use crate::utils::tree_ref::TreeRef;

pub struct MergeCell {
    pub split_index: u8,
    pub tree_ref_pointer: *mut TreeRef,
    pub next_tree_ref_pointer: *mut TreeRef,
    pub index: isize
}

impl MergeCell {
    pub fn new(split_index: u8, tree_ref_pointer: *mut TreeRef, next_tree_ref_pointer: *mut TreeRef, index: isize) -> Self {
        Self {
            split_index,
            tree_ref_pointer,
            next_tree_ref_pointer,
            index
        }
    }

    pub fn deconstruct(self) -> (u8, *mut TreeRef, *mut TreeRef, isize) {
        (self.split_index, self.tree_ref_pointer, self.next_tree_ref_pointer, self.index)
    }
}

#[cfg(feature = "use_rayon")]
unsafe impl Send for MergeCell {}
#[cfg(feature = "use_rayon")]
unsafe impl Sync for MergeCell {}