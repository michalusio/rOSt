use alloc::{collections::btree_set::BTreeSet, string::String};
use crosstrait::register;
use internal_utils::tag_store::BooleanTag;
use internal_utils::tag_store::Identity;
use internal_utils::tag_store::Tag;
use spin::RwLock;

use crate::multi_value_index::MultiValueIndex;

pub struct BooleanTagImpl {
    id: Identity,
    name: String,
    index: RwLock<MultiValueIndex<bool, Identity>>,
}

impl BooleanTagImpl {
    pub fn new(name: String) -> Self {
        Self {
            id: todo!(),
            name,
            index: RwLock::new(MultiValueIndex::default()),
        }
    }

    pub unsafe fn new_unsafe(id: Identity, name: String) -> Self {
        Self {
            id,
            name,
            index: RwLock::new(MultiValueIndex::default()),
        }
    }
}

impl Tag for BooleanTagImpl {
    fn id(&self) -> Identity {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn multi_assignable(&self) -> bool {
        false
    }
}

register! { BooleanTagImpl => dyn BooleanTag }
impl BooleanTag for BooleanTagImpl {
    fn add(&self, id: Identity, value: bool) {
        let mut lock = self.index.write();
        lock.remove_value(id);
        lock.insert_pair(value, id);
    }

    fn get_identities(&self, value: bool) -> BTreeSet<Identity> {
        let lock = self.index.read();
        lock.get_values_from_key(value).cloned().unwrap_or_default()
    }
}
