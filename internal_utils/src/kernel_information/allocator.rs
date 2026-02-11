use linked_list_allocator::LockedHeap;

/// The global memory allocator
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
