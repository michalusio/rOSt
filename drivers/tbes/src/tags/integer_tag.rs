use alloc::{collections::btree_set::BTreeSet, string::String};
use crosstrait::register;
use internal_utils::tag_store::Identity;
use internal_utils::tag_store::IntegerTag;
use internal_utils::tag_store::Tag;
use internal_utils::tag_store::U64QueryExpressionType;
use spin::RwLock;

use crate::multi_value_index::MultiValueIndex;
use crate::tags::RandomStore;

pub struct IntegerTagImpl {
    id: Identity,
    name: String,
    multi_assignable: bool,
    index: RwLock<MultiValueIndex<u64, Identity>>,
    random_store: RandomStore,
}

impl IntegerTagImpl {
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

impl Tag for IntegerTagImpl {
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

register! { IntegerTagImpl => dyn IntegerTag }
register! { IntegerTagImpl => dyn Tag }
impl IntegerTag for IntegerTagImpl {
    fn add(&self, id: Identity, value: u64) {
        let mut lock = self.index.write();
        if !self.multi_assignable {
            lock.remove_value(id);
        }
        lock.insert_pair(value, id);
    }

    fn get_identities(&self, value: u64, filter: U64QueryExpressionType) -> BTreeSet<Identity> {
        let lock = self.index.read();
        match filter {
            U64QueryExpressionType::EqualTo => {
                lock.get_values_from_key(value).cloned().unwrap_or_default()
            }
            U64QueryExpressionType::NotEqualTo => BTreeSet::from_iter(
                self.random_store
                    .read()
                    .keys()
                    .cloned()
                    .filter(|id| !lock.contains_pair(value, *id)),
            ),
            U64QueryExpressionType::LessThan => BTreeSet::from_iter(
                lock.get_values_from_key_and_below(value)
                    .filter(|pair| *pair.0 != value)
                    .map(|pair| pair.1)
                    .cloned(),
            ),
            U64QueryExpressionType::LessThanOrEqualTo => BTreeSet::from_iter(
                lock.get_values_from_key_and_below(value)
                    .map(|pair| pair.1)
                    .cloned(),
            ),
            U64QueryExpressionType::GreaterThan => BTreeSet::from_iter(
                lock.get_values_from_key_and_above(value)
                    .filter(|pair| *pair.0 != value)
                    .map(|pair| pair.1)
                    .cloned(),
            ),
            U64QueryExpressionType::GreaterThanOrEqualTo => BTreeSet::from_iter(
                lock.get_values_from_key_and_above(value)
                    .map(|pair| pair.1)
                    .cloned(),
            ),
        }
    }
}
