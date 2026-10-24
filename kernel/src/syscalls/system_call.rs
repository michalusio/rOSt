use internal_utils::ilogln;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::VirtAddr;

use crate::interrupts::gdt::GDT;
use core::arch::asm;

pub type SysCallHandlerFunc = extern "C" fn(u64, u64) -> u64;

/// A system call handler that panics.
extern "C" fn fail_syscall(_arg1: u64, _arg2: u64) -> u64 {
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
    ilogln!("[   ---{:^15}---   ]", "SYSCALLS");
    ilogln!("Loading LSTAR, FSTAR and STAR");
    // LSTAR stores the address of the `syscall` handler.
    //model_specific::LStar::write(VirtAddr::from_ptr(_syscall as *const ()));
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
    ilogln!("Syscalls active");
}

#[allow(dead_code)]
/// Registers a system call with a handler.
pub fn register_syscall(syscall_number: u16, handler: SysCallHandlerFunc) {
    SYSCALLS.lock()[syscall_number as usize] = handler;
}

/// Performs a system call by invoking interrupt 0x80
pub fn syscall(code: u64) {
    unsafe {
        asm!("int 0x80", in("eax") code, options(nostack, nomem));
    }
}
