#![no_std]
#![feature(abi_x86_interrupt)]

// ########################################################
// #   This library is the core of the operating system   #
// ########################################################

use bootloader::boot_info::FrameBuffer;


pub use crate::interrupts::gtd;
pub use crate::test_framework::ansi_colors;
use crate::test_framework::qemu_exit::exit_qemu;
use crate::test_framework::qemu_exit::QemuExitCode;
pub use crate::test_framework::serial;

use core::panic::PanicInfo;

pub mod interrupts;
pub mod test_framework;
pub mod vga;
pub mod logger;

pub fn init(framebuffer: &'static mut FrameBuffer) {
    logger::init(framebuffer);
    interrupts::init_gdt();
    interrupts::init_idt();
    unsafe {
        // can cause undefined behaviour if the offsets were not set correctly
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{}", ansi_colors::Red("[failed]\n"));
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    hlt_loop();
}
