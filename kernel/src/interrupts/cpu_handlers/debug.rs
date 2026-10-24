use internal_utils::ilogln;
use x86_64::structures::idt::InterruptStackFrame;

use crate::hlt_loop_hard;

pub extern "x86-interrupt" fn debug_handler(_: InterruptStackFrame) {
    ilogln!("DEBUG");
    hlt_loop_hard();
}
