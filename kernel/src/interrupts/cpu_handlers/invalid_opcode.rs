use internal_utils::ilogln;
use x86_64::structures::idt::InterruptStackFrame;

use crate::hlt_loop_hard;

pub extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    ilogln!("INVALID OPCODE\n{:#?}", _stack_frame);
    hlt_loop_hard();
}
