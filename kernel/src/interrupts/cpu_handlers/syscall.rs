use core::arch::naked_asm;

use internal_utils::{clocks::get_current_tick, ilogln};

use crate::{
    processes::{RegistersState, SCHEDULER, dispatcher::dispatch_thread, exit_thread},
    push_registers_state,
};

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "sysv64" fn syscall_interrupt_handler() -> ! {
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
        sym syscall_handler
    )
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn syscall_handler(registers: *const u8) {
    let state = unsafe { (*(registers as *const RegistersState)).clone() };
    match state.rax {
        0 => syscall_yield(state),
        _ => invalid_syscall(state),
    };
}

fn syscall_yield(state: RegistersState) -> ! {
    let thread = {
        let mut scheduler = SCHEDULER.lock().unwrap();
        scheduler.on_tick(state, get_current_tick());
        scheduler.schedule()
    };
    dispatch_thread(thread);
}

fn invalid_syscall(state: RegistersState) -> ! {
    ilogln!("Invalid syscall: {}", state.rax);
    let running_thread = { SCHEDULER.lock().unwrap().get_running_thread() };

    if let Some(thread) = running_thread {
        exit_thread(thread).unwrap();
    }

    let thread = { SCHEDULER.lock().unwrap().schedule() };
    dispatch_thread(thread);
}
