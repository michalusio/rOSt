#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(abi_x86_interrupt, generic_const_exprs, core_intrinsics)]

extern crate alloc;

pub mod addressing;
pub mod interrupts;
pub mod memory;
pub mod processes;
pub mod syscalls;
