#![no_std]
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler,
    vec_deque_extract_if,
    vec_try_remove,
    ptr_alignment_type
)]

use core::arch::asm;

use internal_utils::{logln, try_serial_read};

extern crate alloc;

pub mod addressing;
mod ikd;
pub mod interrupts;
pub mod memory;
pub mod processes;
pub mod syscalls;

#[inline(always)]
/// Endless IKD loop calling halt continuously.
pub fn hlt_loop() -> ! {
    loop {
        hlt_loop_exitable();
    }
}

#[inline(always)]
/// Endless IKD loop calling halt continuously.
/// This version is exitable using an IKD command.
pub fn hlt_loop_exitable() {
    logln!("Beginning halt loop");
    loop {
        if ikd_check() {
            break;
        }
        x86_64::instructions::hlt();
    }
}

#[inline(always)]
/// One pass of the IKD checker, which tries to read from serial interface and parse a command, and otherwise returns
pub fn ikd_check() -> bool {
    let mut exit = false;
    try_serial_read!(|command| {
        exit = ikd::parse_command(command);
    });
    exit
}

#[inline(always)]
/// Endless IKD loop.
/// This version is unexitable and spin-waits for serial read.
pub fn hlt_loop_hard() -> ! {
    logln!("Beginning halt loop");
    loop {
        try_serial_read!(|command| {
            ikd::parse_command(command);
        });
        pause();
    }
}

fn pause() {
    unsafe {
        asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
