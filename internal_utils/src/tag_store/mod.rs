use core::any::Any;

use alloc::{boxed::Box, collections::btree_set::BTreeSet, string::String, sync::Arc, vec::Vec};
use spin::Once;

mod identity;

mod query;
mod tag;

pub use identity::Identity;
pub use query::*;
pub use tag::*;

pub trait TagStore: Send + Sync {
    fn get_tag_tag(&self) -> Arc<dyn BooleanTag>;
    fn get_all_tags(&self) -> Vec<Arc<dyn Any + Sync + Send>>;
    fn get_entity(&self, id: Identity) -> Option<Arc<dyn Any + Sync + Send>>;
    fn query(&self, query: Query) -> (BTreeSet<Identity>, String);
}

pub static TAG_STORE: Once<Box<dyn TagStore>> = Once::new();
