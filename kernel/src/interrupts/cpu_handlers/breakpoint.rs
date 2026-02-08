use internal_utils::logln;
use x86_64::structures::idt::InterruptStackFrame;

/// Handles a breakpoint interrupt (like `int3`).
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    logln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
