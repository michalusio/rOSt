use internal_utils::{clocks::get_current_tick};

use crate::{
    interrupts::pic::{InterruptIndex, PICS, Pics},
    processes::{RegistersState, SCHEDULER, dispatcher::dispatch_thread},
    push_registers_state,
};
use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "sysv64" fn timer_interrupt_handler() -> ! {
    naked_asm!(
        // SS      //19*8
        // RSP     //18*8
        // FLAGS   //17*8
        // CS      //16*8
        // RIP     //15*8
        // We save GPRs
        push_registers_state!(),
        "mov rax, [rsp + 18*8]",
        "push rax", // We push the RSP
        "mov rdi, rsp", // The first argument (RDI) is just pointing to the top of the stack (RSP)

        "jmp {}", // We don't need to care about cleaning up the frame because we will be switching the process anyway
        sym timer_handler
    )
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn timer_handler(registers: *const u8) {
    let state = unsafe { (*(registers as *const RegistersState)).clone() };
    let thread = {
        let mut scheduler = SCHEDULER.lock().unwrap();
        scheduler.on_tick(state, get_current_tick());
        scheduler.schedule()
    };
    unsafe {
        PICS.notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    dispatch_thread(thread);
}
