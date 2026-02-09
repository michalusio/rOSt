mod allocator;
mod debug;
mod frame_allocator;
mod heap;
mod memory_init;
mod page_table;
use alloc::sync::Arc;
use bootloader_api::BootInfo;
use internal_utils::{
    display::format_size, kernel_information::frame_allocator::FullFrameAllocator, logln,
};
use memory_init::init_page_tables;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    PhysAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::PhysFrame,
};

use crate::memory::{
    allocator::ALLOCATOR,
    debug::print_memory_map,
    frame_allocator::{BitmapFrameAllocator, print_frame_memory},
    heap::init_heap,
    page_table::MEMORY_MAPPER,
};

lazy_static! {
    static ref KERNEL_CR3: Mutex<PhysAddr> = Mutex::new(PhysAddr::new(0));
}

/// Saves the current paging table used as the kernel's paging table.
pub fn init_kernel_memory(
    boot_info: &'static BootInfo,
) -> Arc<Mutex<dyn FullFrameAllocator + Send + Sync>> {
    *KERNEL_CR3.lock() = x86_64::registers::control::Cr3::read().0.start_address();
    print_memory_map(&boot_info.memory_regions);

    let mut allocator = BitmapFrameAllocator::init(boot_info);
    init_page_tables(boot_info);

    let mut mapper = MEMORY_MAPPER.lock();
    init_heap(mapper.as_mut().unwrap(), &mut allocator).expect("heap initialization failed");

    print_frame_memory(&allocator);

    let (used, size) = {
        let heap_allocator = ALLOCATOR.lock();
        (heap_allocator.used(), heap_allocator.size())
    };
    logln!(
        "Allocator memory: {:>4}/{:>4}",
        format_size(used as u64),
        format_size(size as u64)
    );

    Arc::new(Mutex::new(allocator))
}

/// Switches the paging table used to the kernel's paging table.
fn switch_to_kernel_memory() {
    let kernel_cr3 = *KERNEL_CR3.lock();
    if !kernel_cr3.is_null() {
        unsafe {
            Cr3::write(
                PhysFrame::from_start_address_unchecked(kernel_cr3),
                Cr3Flags::empty(),
            );
        }
    }
}

/// Performs an action while having kernel paging table. Then switches back.
pub fn with_kernel_memory<V>(action: impl FnOnce() -> V) -> V {
    let cr3 = Cr3::read().0.start_address();
    switch_to_kernel_memory();
    let result = action();
    unsafe {
        Cr3::write(
            PhysFrame::from_start_address_unchecked(cr3),
            Cr3Flags::empty(),
        )
    };
    result
}
