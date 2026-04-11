use core::any::Any;

use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    string::String,
    sync::Arc,
};
use spin::Once;

mod identity;

mod query;
mod tag;

pub use identity::*;
pub use query::*;
pub use tag::*;

pub type Entity = Arc<dyn Any + Sync + Send>;

pub struct QueryOptions {
    pub show_query_plan: bool,
}

pub struct QueryResult {
    pub identities: BTreeSet<Identity>,
    pub query_plan: Option<String>,
}

/// An error meaning that the tag was not found for the specified Identity, or the entity with this Identity was not a tag.
#[derive(Debug)]
pub struct TagNotFoundOrInvalidError;

pub trait TagStore: Send + Sync {
    fn get_all_tags(&self) -> BTreeMap<String, Entity>;
    fn get_entity(&self, id: Identity) -> Option<Entity>;
    fn query(&self, query: Query, options: QueryOptions) -> QueryResult;
    fn add_entity(
        &self,
        id: Identity,
        entity: Entity,
        owner: Identity,
        timestamp: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError>;

    fn has_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn has_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn has_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;

    fn assign_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn assign_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn assign_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;

    fn unassign_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn unassign_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
    fn unassign_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError>;
}

pub static TAG_STORE: Once<Box<dyn TagStore>> = Once::new();
