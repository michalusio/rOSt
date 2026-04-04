#![no_std]
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler,
    vec_deque_extract_if,
    vec_try_remove,
    ptr_alignment_type
)]
extern crate alloc;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;
use internal_utils::clocks::{self};
use internal_utils::kernel_information::KernelInformation;
use internal_utils::{logln, serial};
use kernel::addressing::BOOTLOADER_CONFIG;
use kernel::interrupts::{self};
use kernel::{hlt_loop_hard, processes};
use kernel::{memory, syscalls};

use core::alloc::Layout;

entry_point!(kernel, config = &BOOTLOADER_CONFIG);
pub fn kernel(boot_info: &'static mut BootInfo) -> ! {
    serial::init_logger();
    clocks::init_rtc();
    let allocator = memory::init_kernel_memory(boot_info);
    let kernel_info = KernelInformation::new(boot_info, allocator);
    interrupts::setup();
    syscalls::setup_syscalls();
    tbes::init_tag_store();
    ata::init_disks();
    vga::init_vga(kernel_info);

    processes::init_scheduler();
    processes::run_processes();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Kernel allocation error: {:?}", layout)
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    internal_utils::serial::unlock_logger();
    logln!("{}\n{}", internal_utils::ansi_colors::Red("[PANIC]"), info);
    if let Some(location) = info.location() {
        logln!("panic occurred in {}", location);
    }
    hlt_loop_hard();
}
