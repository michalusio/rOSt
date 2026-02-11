use alloc::{format, string::String};

mod hex_number;
pub use hex_number::HexNumber;

/// Formats the size in bytes to a human readable string.
pub fn format_size(bytes: u64) -> String {
    match bytes {
        b if b < KIB => format!("{} B", b),
        b if b < MIB => format!("{} KiB", b / KIB),
        b if b < GIB => format!("{} MiB", b / MIB),
        b => format!("{} GiB", b / GIB),
    }
}

pub const KIB: u64 = 1 << 10;
pub const MIB: u64 = 1 << 20;
pub const GIB: u64 = 1 << 30;
