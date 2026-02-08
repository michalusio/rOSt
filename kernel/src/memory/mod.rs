mod allocator;
mod debug;
pub mod frame_allocator;
mod heap;
pub use debug::print_memory_map;
mod memory_init;
mod page_table;
pub use memory_init::init;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    PhysAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::PhysFrame,
};

lazy_static! {
    static ref KERNEL_CR3: Mutex<PhysAddr> = Mutex::new(PhysAddr::new(0));
}

/// Saves the current paging table used as the kernel's paging table.
pub fn save_kernel_memory() {
    *KERNEL_CR3.lock() = x86_64::registers::control::Cr3::read().0.start_address();
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
