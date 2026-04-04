#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    generic_const_exprs,
    core_intrinsics,
    adt_const_params,
    unsized_const_params,
    never_type,
    unsize
)]

extern crate alloc;
pub mod block_device;
pub mod capabilities;
pub mod clocks;
mod display;
pub use display::{HexNumber, ansi_colors, format_size};
pub mod gpu_device;
pub mod kernel_information;
pub mod logger;
pub mod port_extensions;
pub mod serial;
pub mod structures;
pub mod tag_store;

mod critical_section;
use critical_section as _;

#[inline(always)]
/// Fast division by 255 using additions and shifts.
pub fn div_255_fast(x: u16) -> u8 {
    (((x) + (((x) + 257) >> 8)) >> 8) as u8
}
