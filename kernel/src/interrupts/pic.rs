use internal_utils::structures::OnceMutex;
use pic8259::ChainedPics;

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

    /// Translate an IDT vector back into the PIC IRQ line that raised it.
    pub fn irq_line(self) -> u8 {
        self.as_u8()
            .checked_sub(PIC_1_OFFSET)
            .expect("PIC-backed interrupts should have an IDT vector at or above PIC_1_OFFSET")
    }
}

/// The PICs of the system.
pub static PICS: OnceMutex<ChainedPics> = OnceMutex::new();

pub trait Pics {
    fn initialize(&self);
    unsafe fn notify_end_of_interrupt(&self, interrupt_id: u8);
}
impl Pics for OnceMutex<ChainedPics> {
    fn initialize(&self) {
        self.call_once(|| unsafe {
            // this is unsafe, because wrong offsets will cause undefined behavior
            let mut pics = ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET);
            pics.initialize();
            pics
        });
    }

    unsafe fn notify_end_of_interrupt(&self, interrupt_id: u8) {
        unsafe { self.lock().unwrap().notify_end_of_interrupt(interrupt_id) };
    }
}

/// Unmask one PIC IRQ line without disturbing the rest of the mask state.
pub fn enable_irq(interrupt: InterruptIndex) {
    let irq_line = interrupt.irq_line();
    let mut pics = PICS.lock().unwrap();

    let [mut master_mask, mut slave_mask] = unsafe { pics.read_masks() };

    if irq_line < 8 {
        master_mask &= !(1 << irq_line);
    } else {
        // Unmask the IRQ on the slave PIC
        slave_mask &= !(1 << (irq_line - 8));
        // Ensure IRQ2 on the master PIC is unmasked for slave communication
        master_mask &= !(1 << 2);
    }
    unsafe {
        pics.write_masks(master_mask, slave_mask);
    }
}

/// Mask one PIC IRQ line without disturbing the rest of the mask state.
pub fn disable_irq(interrupt: InterruptIndex) {
    let irq_line = interrupt.irq_line();
    let mut pics = PICS.lock().unwrap();

    let [mut master_mask, mut slave_mask] = unsafe { pics.read_masks() };

    if irq_line < 8 {
        master_mask |= 1 << irq_line;
    } else {
        slave_mask |= 1 << (irq_line - 8);
    }

    unsafe {
        pics.write_masks(master_mask, slave_mask);
    }
}

/// Returns whether a PIC IRQ line is currently unmasked.
pub fn irq_enabled(interrupt: InterruptIndex) -> bool {
    let irq_line = interrupt.irq_line();
    let mut pics = PICS.lock().unwrap();
    let [master_mask, slave_mask] = unsafe { pics.read_masks() };

    if irq_line < 8 {
        master_mask & (1 << irq_line) == 0
    } else {
        slave_mask & (1 << (irq_line - 8)) == 0
    }
}
