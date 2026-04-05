use alloc::{collections::btree_set::BTreeSet, string::String};
use crosstrait::register;
use internal_utils::tag_store::BooleanTag;
use internal_utils::tag_store::Identity;
use internal_utils::tag_store::Tag;
use spin::RwLock;

use crate::multi_value_index::MultiValueIndex;
use crate::tags::RandomStore;

pub struct BooleanTagImpl {
    id: Identity,
    name: String,
    index: RwLock<MultiValueIndex<bool, Identity>>,
    random_store: RandomStore,
}

impl BooleanTagImpl {
    pub fn new(name: String, store: RandomStore) -> Self {
        Self {
            id: todo!(),
            name,
            index: RwLock::new(MultiValueIndex::default()),
            random_store: store,
        }
    }

    pub unsafe fn new_unsafe(id: Identity, name: String, store: RandomStore) -> Self {
        Self {
            id,
            name,
            index: RwLock::new(MultiValueIndex::default()),
            random_store: store,
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
    fn add(&self, id: Identity) {
        let mut lock = self.index.write();
        lock.remove_value(id);
        lock.insert_pair(true, id);
    }

    fn get_identities(&self, value: bool) -> BTreeSet<Identity> {
        let lock = self.index.read();
        if value {
            lock.get_values_from_key(true).cloned().unwrap_or_default()
        } else {
            let in_tag = lock.get_values_from_key(true);
            if let Some(in_tag) = in_tag {
                BTreeSet::from_iter(
                    self.random_store
                        .read()
                        .keys()
                        .cloned()
                        .filter(|id| !in_tag.contains(id)),
                )
            } else {
                BTreeSet::from_iter(self.random_store.read().keys().cloned())
            }
        }
    }
}
