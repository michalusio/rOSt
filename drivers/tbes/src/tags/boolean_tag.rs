use alloc::{collections::btree_set::BTreeSet, string::String};
use crosstrait::register;
use internal_utils::tag_store::BooleanTag;
use internal_utils::tag_store::Identity;
use internal_utils::tag_store::Tag;
use spin::RwLock;

use crate::tags::RandomStore;

pub struct BooleanTagImpl {
    id: Identity,
    name: String,
    index: RwLock<BTreeSet<Identity>>,
    random_store: RandomStore,
}

impl BooleanTagImpl {
    pub fn new(name: String, store: RandomStore) -> Self {
        Self {
            id: todo!(),
            name,
            index: RwLock::new(BTreeSet::default()),
            random_store: store,
        }
    }

    pub unsafe fn new_unsafe(id: Identity, name: String, store: RandomStore) -> Self {
        Self {
            id,
            name,
            index: RwLock::new(BTreeSet::default()),
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
register! { BooleanTagImpl => dyn Tag }
impl BooleanTag for BooleanTagImpl {
    fn add(&self, id: Identity) -> bool {
        let mut lock = self.index.write();
        lock.insert(id)
    }

    fn has(&self, id: Identity) -> bool {
        let lock = self.index.read();
        lock.contains(&id)
    }

    fn remove(&self, id: Identity) -> bool {
        let mut lock = self.index.write();
        lock.remove(&id)
    }

    fn get_identities(&self, value: bool) -> BTreeSet<Identity> {
        let lock = self.index.read();
        if value {
            lock.clone()
        } else {
            BTreeSet::from_iter(
                self.random_store
                    .read()
                    .keys()
                    .cloned()
                    .filter(|id| !lock.contains(id)),
            )
        }
    }
}
