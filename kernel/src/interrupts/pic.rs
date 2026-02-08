use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Stores the interrupt address for a given interrupt type
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// Stores the interrupt address for a given interrupt type
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,

    AtaPrimary = PIC_2_OFFSET + 6,
    AtaSecondary,
}

impl InterruptIndex {
    /// Returns the corresponding interrupt number for this interrupt type as a u8
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// The PICs of the system.
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    // this is unsafe, because wrong offsets will cause undefined behavior
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});
