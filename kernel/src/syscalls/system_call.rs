use alloc::sync::Arc;
use internal_utils::logln;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::VirtAddr;

use crate::processes::thread::Thread;
use crate::{memory::with_kernel_memory, processes::get_scheduler};

use crate::interrupts::gdt::GDT;
use core::arch::naked_asm;

pub type SysCallHandlerFunc = extern "C" fn(u64, u64, Arc<Mutex<Thread>>) -> u64;

/// A system call handler that panics.
extern "C" fn fail_syscall(_arg1: u64, _arg2: u64, calling_thread: Arc<Mutex<Thread>>) -> u64 {
    panic!("NO SYSCALL DEFINED");
}

lazy_static! {
    static ref SYSCALLS: Mutex<[SysCallHandlerFunc; 1024]> = Mutex::new([fail_syscall; 1024]);
}

/// Sets up the LSTAR, FSTAR and STAR model-specific registers so it's possible to use `syscall`.
pub fn setup_syscalls() {
    use x86_64::registers::model_specific;
    use x86_64::registers::model_specific::{Efer, EferFlags};
    use x86_64::registers::rflags::RFlags;
    logln!("[   ---{:^15}---   ]", "SYSCALLS");
    logln!("Loading LSTAR, FSTAR and STAR");
    // LSTAR stores the address of the `syscall` handler.
    model_specific::LStar::write(VirtAddr::from_ptr(_syscall as *const ()));
    // FSTAR stores which bits of the flag register are cleared by `syscall`.
    model_specific::SFMask::write(RFlags::all());
    // STAR stores the indices of the GDT entries for the kernel and user descriptors.
    model_specific::Star::write(
        GDT.1.user_code_selector,
        GDT.1.user_data_selector,
        GDT.1.kernel_code_selector,
        GDT.1.kernel_data_selector,
    )
    .unwrap();
    let new_efer_flags = {
        let mut flags = Efer::read();
        flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);
        flags
    };
    unsafe {
        Efer::write(new_efer_flags);
    }
    logln!("Syscalls active");
}

#[allow(dead_code)]
/// Registers a system call with a handler.
pub fn register_syscall(syscall_number: u16, handler: SysCallHandlerFunc) {
    SYSCALLS.lock()[syscall_number as usize] = handler;
}

/// Handles a system call.
/// On entry to this function:
/// - the instruction pointer is stored in RCX
/// - the flags are stored in R11
/// - the stack pointer is still targeting the user mode stack
///
/// To properly handle this, we need to:
/// 1. save the user mode stack pointer
/// 2. set the syscall stack pointer
/// 3. save all the registers we need to preserve on the stack
/// 4. do our thing with the values we got from the user
/// 5. restore the registers from the stack
/// 6. restore the user mode stack pointer
/// 7. iretq
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _syscall() -> ! {
    naked_asm!(
        "" //    "cli",
           //    "mov r10, rsp",
           //    "mov rsp, 0x007F80014000", // User stack saved in R10, start of kernel stack loaded
           //    push_all!(),
           //    "mov r9, rsp",
           //    "push r9",
           //    "  call handler", // Return value is in RAX
           //    "  push rax",
           //    "    call get_code_selector",
           //    "    push rax",
           //    "      call get_data_selector",
           //    "    pop rbx",
           //    "    mov rcx, rax", // struct is in RBX+RCX now
           //    "  pop rax",        // We get the syscall value back
           //    "pop r9",           // We get the register state value back
           //    // Preparing iretq
           //    "push rcx",           // data selector
           //    "push [r9 + 40]",     // process stack pointer
           //    "mov r11, [r9 + 32]", // rflags
           //    "or r11, 0x200",
           //    "and r11, 0xffffffffffffbfff",
           //    "push r11",       // rflags
           //    "push rbx",       // code selector
           //    "push [r9 + 96]", // instruction address to return to
           //    "push rax",       // We want to keep the RAX
           //    mov_all!(),
           //    "pop rax",
           //    "iretq"
    );
}

#[unsafe(no_mangle)]
extern "C" fn handler(name: u64, arg1: u64, arg2: u64) -> u64 {
    // This block executes after saving the user state and before returning back
    with_kernel_memory(|| {
        let thread = get_scheduler()
            .as_mut()
            .unwrap()
            .running_thread
            .clone()
            .unwrap();
        SYSCALLS.lock()[name as u16 as usize](arg1, arg2, thread)
    })
}

// TODO Try to combine both of these functions to make it faster.
// We should be able to return a C-style struct with two u64s and manage it through ASM.
#[unsafe(no_mangle)]
extern "C" fn get_code_selector() -> u64 {
    with_kernel_memory(|| {
        let thread = get_scheduler()
            .as_mut()
            .unwrap()
            .running_thread
            .clone()
            .unwrap();
        let thread: spin::MutexGuard<'_, Thread> = thread.as_ref().lock();
        let process = thread.process.as_ref().lock();
        if process.kernel_process {
            (GDT.1.kernel_code_selector.index() * 8) as u64
        } else {
            ((GDT.1.user_code_selector.index() * 8) | 3) as u64
        }
    })
}

#[unsafe(no_mangle)]
extern "C" fn get_data_selector() -> u64 {
    with_kernel_memory(|| {
        let thread = get_scheduler()
            .as_mut()
            .unwrap()
            .running_thread
            .clone()
            .unwrap();
        let thread = thread.as_ref().lock();
        let process = thread.process.as_ref().lock();
        if process.kernel_process {
            (GDT.1.kernel_data_selector.index() * 8) as u64
        } else {
            ((GDT.1.user_data_selector.index() * 8) | 3) as u64
        }
    })
}
