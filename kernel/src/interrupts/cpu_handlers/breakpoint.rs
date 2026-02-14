use internal_utils::{hlt_loop_exitable, logln};
use x86_64::structures::idt::InterruptStackFrame;

/// Handles a breakpoint interrupt (like `int3`).
pub extern "x86-interrupt" fn breakpoint_handler(_: InterruptStackFrame) {
    logln!("BREAKPOINT");
    hlt_loop_exitable();
}
