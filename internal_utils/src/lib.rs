#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(generic_const_exprs, core_intrinsics, unsized_const_params)]

extern crate alloc;
pub mod block_device;
pub mod clocks;
pub mod display;
pub mod gpu_device;
pub mod kernel_information;
pub mod logger;
pub mod port_extensions;
pub mod serial;
pub mod structures;

#[macro_export]
/// Macro for pushing all registers onto the stack.
macro_rules! push_all {
    () => {
        "push rax;push rbx;push rcx;push rdx;push rbp;push rsi;push rdi;push r8;push r9;push r10;push r11;push r12;push r13;push r14;push r15"
    };
}

#[macro_export]
/// Macro for popping all registers from the stack.
macro_rules! pop_all {
    () => {
        "pop r15;pop r14;pop r13;pop r12;pop r11;pop r10;pop r9;pop r8;pop rdi;pop rsi;pop rbp;pop rdx;pop rcx;pop rbx;pop rax"
    };
}

#[macro_export]
/// Macro for mov'ing all registers from a RegistersState struct stored in r9.
macro_rules! mov_all {
    () => {
        "mov r15, [r9]; mov r14, [r9 + 8]; mov r13, [r9 + 16]; mov r12, [r9 + 24]; mov r11, [r9 + 32]; mov r10, [r9 + 40]; mov r8, [r9 + 56]; mov rdi, [r9 + 64]; mov rsi, [r9 + 72]; mov rbp, [r9 + 80]; mov rdx, [r9 + 88]; mov rcx, [r9 + 96]; mov rbx, [r9 + 104]; mov rax, [r9 + 112]; mov r9, [r9 + 48]"
    };
}

#[inline(always)]
/// Fast division by 255 using additions and shifts.
pub fn div_255_fast(x: u16) -> u8 {
    (((x) + (((x) + 257) >> 8)) >> 8) as u8
}

#[inline(always)]
/// Endless loop calling halt continuously.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
