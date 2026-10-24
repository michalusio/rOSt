use internal_utils::ilogln;
use x86_64::structures::idt::InterruptStackFrame;

use crate::hlt_loop_hard;

pub extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    ilogln!("GP FAULT (error {:#?})\n{:#?}", _error_code, _stack_frame);
    hlt_loop_hard();
}
