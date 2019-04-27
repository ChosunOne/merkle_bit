use std::cmp::Ordering;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::constants::KEY_LEN;
use crate::utils::par_tree_ref::TreeRef;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeRefWrapper {
    Raw(Arc<TreeRefLock>),
    Ref(Arc<RwLock<TreeRefWrapper>>),
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

    pub fn get_reference(wrapper: &Arc<RwLock<TreeRefWrapper>>) -> Arc<RwLock<TreeRefWrapper>> {
        match *wrapper.read().unwrap() {
            TreeRefWrapper::Raw(ref _t) => Arc::clone(wrapper),
            TreeRefWrapper::Ref(ref r) => TreeRefWrapper::get_reference(r),
        }
    }

    pub fn get_tree_ref_key(&self) -> Arc<[u8; KEY_LEN]> {
        match self {
            TreeRefWrapper::Raw(t) => Arc::clone(&t.read().unwrap().key),
            TreeRefWrapper::Ref(r) => r.read().unwrap().get_tree_ref_key(),
        }
    }

    pub fn get_tree_ref_location(&self) -> Arc<[u8; KEY_LEN]> {
        match self {
            TreeRefWrapper::Raw(t) => Arc::clone(&t.read().unwrap().location),
            TreeRefWrapper::Ref(r) => r.read().unwrap().get_tree_ref_location(),
        }
    }

    pub fn get_tree_ref_count(&self) -> u64 {
        match self {
            TreeRefWrapper::Raw(t) => t.read().unwrap().count,
            TreeRefWrapper::Ref(r) => r.read().unwrap().get_tree_ref_count(),
        }
    }

    pub fn set_tree_ref_key(&mut self, key: Arc<[u8; KEY_LEN]>) {
        match self {
            TreeRefWrapper::Raw(t) => t.write().unwrap().key = key,
            TreeRefWrapper::Ref(r) => r.write().unwrap().set_tree_ref_key(key),
        }
    }

    pub fn set_tree_ref_location(&mut self, location: Arc<[u8; KEY_LEN]>) {
        match self {
            TreeRefWrapper::Raw(t) => t.write().unwrap().location = location,
            TreeRefWrapper::Ref(r) => r.write().unwrap().set_tree_ref_location(location),
        }
    }

    pub fn set_tree_ref_count(&mut self, count: u64) {
        match self {
            TreeRefWrapper::Raw(t) => t.write().unwrap().count = count,
            TreeRefWrapper::Ref(r) => r.write().unwrap().set_tree_ref_count(count),
        }
    }
}

struct TreeRefLock(RwLock<TreeRef>);

impl Deref for TreeRefLock {
    type Target = RwLock<TreeRef>;

    fn deref(&self) -> &RwLock<TreeRef> {
        &self.0
    }
}

impl PartialEq for TreeRefLock {
    fn eq(&self, other: &TreeRefLock) -> bool {
        self.0.read().unwrap().eq(&other.0.read().unwrap())
    }
}