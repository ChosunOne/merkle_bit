use std::cell::RefCell;
use std::rc::Rc;

use crate::constants::KEY_LEN;
use crate::utils::tree_ref::TreeRef;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeRefWrapper<'a> {
    Raw(TreeRef),
    Ref(&'a mut TreeRefWrapper<'a>),
}

impl<'a> TreeRefWrapper<'a> {
    pub fn update_reference(&'a mut self) {
        match self {
            TreeRefWrapper::Raw(_) => return,
            TreeRefWrapper::Ref(r) => *self = TreeRefWrapper::get_reference(r),
        }
    }

    pub fn get_reference(wrapper: &'a mut TreeRefWrapper) -> TreeRefWrapper<'a> {
        match wrapper {
            TreeRefWrapper::Raw(t) => TreeRefWrapper::Ref(&mut *wrapper),
            TreeRefWrapper::Ref(r) => TreeRefWrapper::get_reference(r),
        }
    }

    pub fn get_tree_ref_key(&self) -> [u8; KEY_LEN] {
        match self {
            TreeRefWrapper::Raw(t) => t.key,
            TreeRefWrapper::Ref(r) => r.get_tree_ref_key(),
        }
    }

    pub fn get_tree_ref_location(&self) -> [u8; KEY_LEN] {
        match self {
            TreeRefWrapper::Raw(t) => t.location,
            TreeRefWrapper::Ref(r) => r.get_tree_ref_location(),
        }
    }

    pub fn get_tree_ref_count(&self) -> u64 {
        match self {
            TreeRefWrapper::Raw(t) => t.count,
            TreeRefWrapper::Ref(r) => r.get_tree_ref_count(),
        }
    }

    pub fn set_tree_ref_key(&mut self, key: [u8; KEY_LEN]) {
        match self {
            TreeRefWrapper::Raw(t) => t.key = key,
            TreeRefWrapper::Ref(r) => r.set_tree_ref_key(key),
        }
    }

    pub fn set_tree_ref_location(&mut self, location: [u8; KEY_LEN]) {
        match self {
            TreeRefWrapper::Raw(t) => t.location = location,
            TreeRefWrapper::Ref(r) => r.set_tree_ref_location(location),
        }
    }

    pub fn set_tree_ref_count(&mut self, count: u64) {
        match self {
            TreeRefWrapper::Raw(t) => t.count = count,
            TreeRefWrapper::Ref(r) => r.set_tree_ref_count(count),
        }
    }
}
