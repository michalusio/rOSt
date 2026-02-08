use crate::interrupts::pic::{InterruptIndex, PICS};
use internal_utils::clocks::get_current_time;
use internal_utils::logln;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

lazy_static! {
    pub static ref SHOW_CLOCK: Mutex<bool> = Mutex::new(false);
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let should_show_clock: bool = { *SHOW_CLOCK.lock() };
    if should_show_clock {
        logln!("{}", get_current_time());
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
