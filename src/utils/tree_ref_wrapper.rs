use std::cell::RefCell;
use std::rc::Rc;

use crate::constants::KEY_LEN;
use crate::utils::tree_ref::TreeRef;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeRefWrapper {
    Raw(Rc<RefCell<TreeRef>>),
    Ref(Rc<RefCell<TreeRefWrapper>>),
}

impl TreeRefWrapper {
    pub fn update_reference(&mut self) {
        let new_ref;
        match self {
            TreeRefWrapper::Raw(_) => return,
            TreeRefWrapper::Ref(r) => new_ref = TreeRefWrapper::get_reference(r),
        }
        *self = TreeRefWrapper::Ref(new_ref);
    }

    pub fn get_reference(wrapper: &Rc<RefCell<TreeRefWrapper>>) -> Rc<RefCell<TreeRefWrapper>> {
        match *wrapper.borrow() {
            TreeRefWrapper::Raw(ref _t) => Rc::clone(wrapper),
            TreeRefWrapper::Ref(ref r) => TreeRefWrapper::get_reference(r),
        }
    }

    pub fn get_tree_ref_key(&self) -> [u8; KEY_LEN] {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow().key,
            TreeRefWrapper::Ref(r) => r.borrow().get_tree_ref_key(),
        }
    }

    pub fn get_tree_ref_location(&self) -> [u8; KEY_LEN] {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow().location,
            TreeRefWrapper::Ref(r) => r.borrow().get_tree_ref_location(),
        }
    }

    pub fn get_tree_ref_count(&self) -> u64 {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow().count,
            TreeRefWrapper::Ref(r) => r.borrow().get_tree_ref_count(),
        }
    }

    pub fn set_tree_ref_key(&mut self, key: [u8; KEY_LEN]) {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow_mut().key = key,
            TreeRefWrapper::Ref(r) => r.borrow_mut().set_tree_ref_key(key),
        }
    }

    pub fn set_tree_ref_location(&mut self, location: [u8; KEY_LEN]) {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow_mut().location = location,
            TreeRefWrapper::Ref(r) => r.borrow_mut().set_tree_ref_location(location),
        }
    }

    pub fn set_tree_ref_count(&mut self, count: u64) {
        match self {
            TreeRefWrapper::Raw(t) => t.borrow_mut().count = count,
            TreeRefWrapper::Ref(r) => r.borrow_mut().set_tree_ref_count(count),
        }
    }
}
