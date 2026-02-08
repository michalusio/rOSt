// This might be reimplemented from scratch in the future.

// TODO: Implement all remaining interrupt handlers for CPU interrupts
// We need to implement all interrupt handlers and add basic handling to them so we don't double fault.
// Better handling for each of them will be added later.

mod cpu_handlers;
mod interrupt_register;
use internal_utils::logln;
pub(crate) mod gdt;
mod pic_handlers;
pub use gdt::GDT;
mod pic;
pub use pic_handlers::SHOW_CLOCK;

use crate::interrupts::{gdt::reload_gdt, interrupt_register::init_idt};

/// Initializes the PICs and enables interrupts
pub fn enable() {
    reload_gdt();
    init_idt();
    unsafe {
        // can cause undefined behaviour if the offsets were not set correctly
        pic::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
    logln!("Interrupts enabled");
}
