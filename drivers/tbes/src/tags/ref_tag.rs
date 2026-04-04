use alloc::string::String;
use spin::RwLock;

use crate::{Identity, indexes::MultiValueIndex, tags::tag::Tag};

pub struct RefTagImpl {
    id: Identity,
    name: String,
    multi_assignable: bool,
    index: RwLock<MultiValueIndex<u64, Identity>>,
}

impl Tag for RefTagImpl {
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
