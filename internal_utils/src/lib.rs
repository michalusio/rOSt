#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    generic_const_exprs,
    core_intrinsics,
    adt_const_params,
    unsized_const_params,
    never_type,
    unsize,
    unsafe_cell_access
)]

extern crate alloc;
pub mod block_device;
pub mod capabilities;
pub mod clocks;
mod display;
pub use display::{HexNumber, ansi_colors, format_size};
pub mod channels;
pub mod gpu_device;
pub mod kernel_information;
pub mod logger;
pub mod port_extensions;
pub mod serial;
pub mod structures;
pub mod tag_store;

mod cache_padded;
mod page_padded;

pub use cache_padded::CachePadded;
pub use page_padded::PagePadded;

mod critical_section;
#[allow(unused_imports)]
// Ensuring the critical_section implementation is available to the linker
use critical_section as _;

/// Fast division by 255 using additions and shifts.
pub const fn div_255_fast(x: u16) -> u8 {
    (((x) + (((x) + 257) >> 8)) >> 8) as u8
}

/// Returns the greatest power of two less than or equal to `self`, or 0 otherwise.
pub const fn prev_power_of_two(n: u64) -> u64 {
    // n = 0 gives highest_bit_set_idx = 0.
    let highest_bit_set_idx = 63 - (n | 1).leading_zeros();
    // Binary AND of highest bit with n is a no-op, except zero gets wiped.
    (1 << highest_bit_set_idx) & n
}
