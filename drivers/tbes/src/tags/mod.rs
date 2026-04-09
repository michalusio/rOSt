use core::any::Any;

use crate::{
    Identity,
    query::{QueryContext, Runnable},
    tags::{boolean_tag::BooleanTagImpl, integer_tag::IntegerTagImpl, ref_tag::RefTagImpl},
};
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use crosstrait::Cast;
use internal_utils::{
    logln,
    tag_store::{
        BooleanTag, Entity, IntegerTag, KERNEL_IDENTITY, OWNER_TAG_IDENTITY, Query, QueryOptions,
        QueryResult, RefTag, TAG_STORE, TAG_TAG_IDENTITY, TIMESTAMP_TAG_IDENTITY, Tag,
        TagNotFoundOrInvalidError, TagStore, USER_TAG_IDENTITY,
    },
};
use spin::RwLock;

mod boolean_tag;
mod integer_tag;
mod ref_tag;

pub type RandomStore = Arc<RwLock<BTreeMap<Identity, Entity>>>;

pub struct TBESTagStore {
    //A tag storing information about which things are tags
    tag_tag: Arc<dyn BooleanTag>,
    // The actual storage of entities
    // TODO: We should replace this with a mechanism of access to devices which can store entities with identity
    random_store: RandomStore,
}

impl TBESTagStore {
    pub fn new() -> Self {
        let store: RandomStore = Arc::new(RwLock::new(BTreeMap::new()));
        let tag_tag = Self::add_tag_tag(&store);
        Self::add_owner_tag(&store);
        Self::add_timestamp_tag(&store);
        Self::add_kernel_user(&store);
        Self {
            tag_tag: tag_tag.cast().unwrap(),
            random_store: store,
        }
    }

    fn add_tag_tag(store: &RandomStore) -> Entity {
        let tt = unsafe {
            BooleanTagImpl::new_unsafe(TAG_TAG_IDENTITY, "Tag".to_string(), store.clone())
        };
        let id = tt.id();
        tt.add(id);
        let tt: Arc<dyn Any + Sync + Send> = Arc::new(tt);
        store.write().insert(id, tt.clone());
        tt
    }

    fn add_owner_tag(store: &RandomStore) {
        let ot = unsafe {
            RefTagImpl::new_unsafe(
                OWNER_TAG_IDENTITY,
                "Owner".to_string(),
                false,
                store.clone(),
            )
        };
        let id = ot.id();
        let mut store_lock = store.write();
        let tag_tag: Arc<dyn BooleanTag> = store_lock
            .get(&TAG_TAG_IDENTITY)
            .unwrap()
            .clone()
            .cast()
            .unwrap();
        tag_tag.add(id);

        ot.add(id, KERNEL_IDENTITY);
        store_lock.keys().for_each(|id| {
            ot.add(*id, KERNEL_IDENTITY);
        });
        store_lock.insert(id, Arc::new(ot));
    }

    fn add_timestamp_tag(store: &RandomStore) {
        let tt = unsafe {
            IntegerTagImpl::new_unsafe(
                TIMESTAMP_TAG_IDENTITY,
                "Timestamp".to_string(),
                false,
                store.clone(),
            )
        };
        let id = tt.id();
        let mut store_lock = store.write();
        let tag_tag: Arc<dyn BooleanTag> = store_lock
            .get(&TAG_TAG_IDENTITY)
            .unwrap()
            .clone()
            .cast()
            .unwrap();
        tag_tag.add(id);

        let owner_tag: Arc<dyn RefTag> = store_lock
            .get(&OWNER_TAG_IDENTITY)
            .unwrap()
            .clone()
            .cast()
            .unwrap();
        owner_tag.add(id, KERNEL_IDENTITY);

        tt.add(id, 0);
        store_lock.keys().for_each(|id| {
            tt.add(*id, 0);
        });
        store_lock.insert(id, Arc::new(tt));
    }

    fn add_kernel_user(store: &RandomStore) {
        let user = ();
        let mut store_lock = store.write();
        let timestamp_tag: Arc<dyn IntegerTag> = store_lock
            .get(&TIMESTAMP_TAG_IDENTITY)
            .unwrap()
            .clone()
            .cast()
            .unwrap();
        timestamp_tag.add(KERNEL_IDENTITY, 0);

        let owner_tag: Arc<dyn RefTag> = store_lock
            .get(&OWNER_TAG_IDENTITY)
            .unwrap()
            .clone()
            .cast()
            .unwrap();
        owner_tag.add(KERNEL_IDENTITY, KERNEL_IDENTITY);

        store_lock.insert(KERNEL_IDENTITY, Arc::new(user));
    }
}

impl TagStore for TBESTagStore {
    fn query(&self, query: Query, options: QueryOptions) -> QueryResult {
        let mut writer = QueryContext::new(options.show_query_plan);
        let normalized_query = query.normalize();

        let set = normalized_query.run(&mut writer);

        QueryResult {
            identities: set,
            query_plan: options.show_query_plan.then(|| writer.to_string()),
        }
    }

    fn get_entity(&self, id: Identity) -> Option<Entity> {
        self.random_store.read().get(&id).cloned()
    }

    fn get_all_tags(&self) -> BTreeMap<String, Entity> {
        let store = self.random_store.read();
        BTreeMap::from_iter(
            self.tag_tag
                .get_identities(true)
                .iter()
                .filter_map(|id| store.get(id))
                .cloned()
                .map(|tag| {
                    let t: Arc<dyn Tag> = tag.clone().cast().unwrap();
                    (t.name().to_owned(), tag)
                }),
        )
    }

    fn add_entity(
        &self,
        id: Identity,
        entity: Entity,
        owner: Identity,
        timestamp: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let is_owner_a_user = self.has_binary_tag(owner, USER_TAG_IDENTITY)?;
        if is_owner_a_user && self.get_entity(id).is_none() {
            self.assign_integer_tag(id, TIMESTAMP_TAG_IDENTITY, timestamp)
                .unwrap();
            self.assign_ref_tag(id, OWNER_TAG_IDENTITY, owner).unwrap();
            {
                let mut store_lock = self.random_store.write();
                store_lock.insert(id, entity);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn has_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn BooleanTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.has(id))
    }

    fn has_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn IntegerTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.has(id, value))
    }

    fn has_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn RefTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.has(id, value))
    }

    fn assign_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn BooleanTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.add(id))
    }

    fn assign_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn IntegerTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.add(id, value))
    }

    fn assign_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn RefTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.add(id, value))
    }

    fn unassign_binary_tag(
        &self,
        id: Identity,
        tag_id: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn BooleanTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.remove(id))
    }

    fn unassign_integer_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: u64,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn IntegerTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.remove(id, value))
    }

    fn unassign_ref_tag(
        &self,
        id: Identity,
        tag_id: Identity,
        value: Identity,
    ) -> Result<bool, TagNotFoundOrInvalidError> {
        let tag: Arc<dyn RefTag> = self
            .get_entity(tag_id)
            .ok_or(TagNotFoundOrInvalidError)?
            .cast()
            .ok_or(TagNotFoundOrInvalidError)?;
        Ok(tag.remove(id, value))
    }
}

pub fn init_tag_store() {
    logln!("Initializing tag store");
    let store = TBESTagStore::new();
    let store = Box::new(store);
    TAG_STORE.call_once(|| store);
    logln!("Tag store initialized");
}
