use core::any::Any;

use alloc::{boxed::Box, collections::btree_set::BTreeSet, string::String, sync::Arc, vec::Vec};
use spin::Once;

mod identity;

mod query;
mod tag;

pub use identity::*;
pub use query::*;
pub use tag::*;

pub type Entity = Arc<dyn Any + Sync + Send>;

pub trait TagStore: Send + Sync {
    fn get_tag_tag(&self) -> Arc<dyn BooleanTag>;
    fn get_all_tags(&self) -> Vec<Entity>;
    fn get_entity(&self, id: Identity) -> Option<Entity>;
    fn query(&self, query: Query) -> (BTreeSet<Identity>, String);
    fn add_entity(&self, id: Identity, entity: Entity, owner: Identity, timestamp: u64) -> bool;
}

pub static TAG_STORE: Once<Box<dyn TagStore>> = Once::new();
