use alloc::{format, string::String};
use x86_64::{PhysAddr, VirtAddr};

use crate::{ilog, structures::Permanent};

pub trait HexNumber {
    fn to_separated_hex(self) -> String;
    fn log_to_separated_hex(self);
}

impl HexNumber for PhysAddr {
    fn to_separated_hex(self) -> String {
        let hex = format!("{:016X}", self);
        format!(
            "PhysAddr: 0x{}_{}_{}_{}",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }
    fn log_to_separated_hex(self) {
        let value = self.as_u64();
        ilog!("PhysAddr: 0x");
        ilog!("{:04X}", (value >> 48) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 32) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 16) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", value & 0xFFFF);
    }
}

impl HexNumber for Permanent<*mut u8> {
    fn to_separated_hex(self) -> String {
        VirtAddr::new(self.get() as u64).to_separated_hex()
    }
    fn log_to_separated_hex(self) {
        VirtAddr::new(self.get() as u64).log_to_separated_hex()
    }
}

impl HexNumber for VirtAddr {
    fn to_separated_hex(self) -> String {
        let hex = format!("{:016X}", self);
        format!(
            "VirtAddr: 0x{}_{}_{}_{}",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }
    fn log_to_separated_hex(self) {
        let value = self.as_u64();
        ilog!("VirtAddr: 0x");
        ilog!("{:04X}", (value >> 48) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 32) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 16) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", value & 0xFFFF);
    }
}

impl HexNumber for u64 {
    fn to_separated_hex(self) -> String {
        let hex = format!("{:016X}", self);
        format!(
            "0x{}_{}_{}_{}",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }
    fn log_to_separated_hex(self) {
        let value = self;
        ilog!("0x");
        ilog!("{:04X}", (value >> 48) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 32) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", (value >> 16) & 0xFFFF);
        ilog!("_");
        ilog!("{:04X}", value & 0xFFFF);
    }
}

impl HexNumber for u16 {
    fn to_separated_hex(self) -> String {
        format!("0x{:04X}", self)
    }
    fn log_to_separated_hex(self) {
        ilog!("0x{:04X}", self);
    }
}
