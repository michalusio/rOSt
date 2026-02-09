use bootloader_api::BootInfo;
use x86_64::VirtAddr;

use super::page_table::{self};

/// Initializes the page tables and kernel heap memory
pub fn init_page_tables(boot_info: &'static BootInfo) {
    let pmo = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("physical memory mapping not set"),
    );
    unsafe { page_table::init(pmo) };
}
