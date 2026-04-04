use core::any::Any;

use crate::{
    Identity,
    query::{QueryWriter, Runnable},
    tags::boolean_tag::BooleanTagImpl,
};
use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use crosstrait::Cast;
use internal_utils::{
    logln,
    tag_store::{BooleanTag, Query, TAG_STORE, Tag, TagStore},
};
use spin::RwLock;

mod boolean_tag;

pub struct TBESTagStore {
    //A tag storing information about which things are tags
    tag_tag: Arc<dyn BooleanTag>,
    // The actual storage of entities
    // TODO: We should replace this with a mechanism of access to devices which can store entities with identity
    random_store: RwLock<BTreeMap<Identity, Arc<dyn Any + Sync + Send>>>,
}

impl TBESTagStore {
    pub fn new() -> Self {
        unsafe {
            let tt = BooleanTagImpl::new_unsafe(
                Identity::from_value((1u64 << 32) | 1),
                "Tag".to_string(),
            );
            let id = tt.id();
            tt.add(id, true);
            let tt: Arc<dyn Any + Sync + Send> = Arc::new(tt);
            let mut store: BTreeMap<Identity, Arc<dyn Any + Sync + Send>> = BTreeMap::new();
            store.insert(id, tt.clone());
            Self {
                tag_tag: tt.cast().unwrap(),
                random_store: RwLock::new(store),
            }
        }
    }
}

impl TagStore for TBESTagStore {
    fn query(&self, query: Query) -> (BTreeSet<Identity>, String) {
        let mut writer = QueryWriter::default();
        let normalized_query = query.normalize();

        let set = normalized_query.run(&mut writer);

        let plan = writer.to_string();
        (set, plan)
    }

    fn get_tag_tag(&self) -> Arc<dyn BooleanTag> {
        self.tag_tag.clone()
    }

    fn get_entity(&self, id: Identity) -> Option<Arc<dyn Any + Sync + Send>> {
        self.random_store.read().get(&id).cloned()
    }

    fn get_all_tags(&self) -> Vec<Arc<dyn Any + Sync + Send>> {
        let store = self.random_store.read();
        self.get_tag_tag()
            .get_identities(true)
            .iter()
            .filter_map(|id| store.get(id))
            .cloned()
            .collect()
    }
}

pub fn init_tag_store() {
    logln!("Initializing tag store");
    let store = TBESTagStore::new();
    let store = Box::new(store);
    TAG_STORE.call_once(|| store);
    logln!("Tag store initialized");
}
