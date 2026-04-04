use crate::tag_store::Identity;
use alloc::collections::btree_set::BTreeSet;

pub trait Tag: Send + Sync {
    fn id(&self) -> Identity;
    fn name(&self) -> &str;
    fn multi_assignable(&self) -> bool;
}

pub trait BooleanTag: Tag {
    fn add(&self, id: Identity, value: bool);
    fn get_identities(&self, value: bool) -> BTreeSet<Identity>;
}

pub trait IntegerTag: Tag {
    fn add(&self, id: Identity, value: u64);
    fn get_identities(&self, value: u64) -> BTreeSet<Identity>;
}

pub trait RefTag: Tag {
    fn add(&self, id: Identity, value: Identity);
    fn get_identities(&self, value: Identity) -> BTreeSet<Identity>;
}
