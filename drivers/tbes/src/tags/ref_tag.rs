use alloc::{collections::btree_set::BTreeSet, string::String};
use crosstrait::register;
use internal_utils::tag_store::Identity;
use internal_utils::tag_store::RefTag;
use internal_utils::tag_store::Tag;
use spin::RwLock;

use crate::multi_value_index::MultiValueIndex;
use crate::tags::RandomStore;

pub struct RefTagImpl {
    id: Identity,
    name: String,
    multi_assignable: bool,
    index: RwLock<MultiValueIndex<Identity, Identity>>,
    random_store: RandomStore,
}

impl RefTagImpl {
    pub fn new(name: String, multi_assignable: bool, store: RandomStore) -> Self {
        Self {
            id: todo!(),
            name,
            multi_assignable,
            index: RwLock::new(MultiValueIndex::default()),
            random_store: store,
        }
    }

    pub unsafe fn new_unsafe(
        id: Identity,
        name: String,
        multi_assignable: bool,
        store: RandomStore,
    ) -> Self {
        Self {
            id,
            name,
            multi_assignable,
            index: RwLock::new(MultiValueIndex::default()),
            random_store: store,
        }
    }
}

impl Tag for RefTagImpl {
    fn id(&self) -> Identity {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn multi_assignable(&self) -> bool {
        self.multi_assignable
    }
}

register! { RefTagImpl => dyn RefTag }
register! { RefTagImpl => dyn Tag }
impl RefTag for RefTagImpl {
    fn add(&self, id: Identity, value: Identity) {
        let mut lock = self.index.write();
        if !self.multi_assignable {
            lock.remove_value(id);
        }
        lock.insert_pair(value, id);
    }

    fn get_identities(&self, value: Identity, negate: bool) -> BTreeSet<Identity> {
        let lock = self.index.read();
        if negate {
            BTreeSet::from_iter(
                self.random_store
                    .read()
                    .keys()
                    .cloned()
                    .filter(|id| !lock.contains_value(*id)),
            )
        } else {
            lock.get_values_from_key(value).cloned().unwrap_or_default()
        }
    }
}
