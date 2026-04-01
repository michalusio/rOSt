mod timer;
pub use timer::timer_interrupt_handler;
mod keyboard;
pub use keyboard::{enable_keyboard_irq, keyboard_interrupt_handler};
mod ata;
pub use ata::{ata_primary_interrupt_handler, ata_secondary_interrupt_handler};
mod addresses;
