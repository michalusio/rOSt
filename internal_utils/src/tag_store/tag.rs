use crate::tag_store::{Identity, U64QueryExpressionType};
use alloc::collections::btree_set::BTreeSet;

pub trait Tag: Send + Sync {
    fn id(&self) -> Identity;
    fn name(&self) -> &str;
    fn multi_assignable(&self) -> bool;
}

pub trait BooleanTag: Tag {
    fn add(&self, id: Identity) -> bool;
    fn remove(&self, id: Identity) -> bool;
    fn has(&self, id: Identity) -> bool;
    fn get_identities(&self, value: bool) -> BTreeSet<Identity>;
}

pub trait IntegerTag: Tag {
    fn add(&self, id: Identity, value: u64) -> bool;
    fn remove(&self, id: Identity, value: u64) -> bool;
    fn has(&self, id: Identity, value: u64) -> bool;
    fn get_identities(&self, value: u64, filter: U64QueryExpressionType) -> BTreeSet<Identity>;
}

pub trait RefTag: Tag {
    fn add(&self, id: Identity, value: Identity) -> bool;
    fn remove(&self, id: Identity, value: Identity) -> bool;
    fn has(&self, id: Identity, value: Identity) -> bool;
    fn get_identities(&self, value: Identity, negate: bool) -> BTreeSet<Identity>;
}
