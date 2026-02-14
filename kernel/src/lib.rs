#![no_std]
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler
)]

extern crate alloc;

pub mod addressing;
pub mod interrupts;
pub mod memory;
pub mod processes;
pub mod syscalls;
