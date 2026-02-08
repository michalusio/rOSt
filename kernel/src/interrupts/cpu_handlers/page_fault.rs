use internal_utils::hlt_loop;
use internal_utils::logln;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

/// Handles a page fault.
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    x86_64::instructions::interrupts::disable();

    logln!("EXCEPTION: PAGE FAULT");
    logln!("{:?}", error_code);
    logln!("Page: {:X?}", Cr2::read_raw());
    logln!("{:#?}", stack_frame);
    hlt_loop();
}
