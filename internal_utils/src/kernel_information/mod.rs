use alloc::sync::Arc;
use bootloader_api::{
    BootInfo,
    info::{MemoryRegions, Optional},
};
use spin::Mutex;
use x86_64::PhysAddr;

use crate::{
    display::HexNumber,
    kernel_information::{
        frame_allocator::FullFrameAllocator, kernel_frame_buffer::KernelFrameBuffer,
    },
    logln,
    structures::OnceClone,
};

pub mod allocator;
pub mod frame_allocator;
pub mod kernel_frame_buffer;

#[derive(Clone)]
#[repr(C)]
pub struct KernelInformation {
    pub bootloader_version: (u16, u16, u16),
    pub physical_memory_offset: u64,
    pub framebuffer: Optional<KernelFrameBuffer>,
    pub memory_regions: &'static MemoryRegions,
    pub allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>,
    pub kernel_start: PhysAddr,
    pub rsdp: Option<PhysAddr>,
}

impl KernelInformation {
    pub fn new(
        boot_info: &'static BootInfo,
        allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>,
    ) -> KernelInformation {
        let framebuffer = match boot_info.framebuffer.as_ref() {
            Some(framebuffer) => Optional::Some(KernelFrameBuffer::new(framebuffer)),
            None => Optional::None,
        };
        let v = boot_info.api_version;
        let kernel_info = KernelInformation {
            bootloader_version: (v.version_major(), v.version_minor(), v.version_patch()),
            physical_memory_offset: *boot_info
                .physical_memory_offset
                .as_ref()
                .expect("No physical memory mapping"),
            framebuffer,
            memory_regions: &boot_info.memory_regions,
            allocator,
            rsdp: boot_info.rsdp_addr.as_ref().copied().map(PhysAddr::new),
            kernel_start: PhysAddr::new(boot_info.kernel_addr),
        };
        KERNEL_INFORMATION.call_once(|| kernel_info.clone());
        kernel_info
    }

    pub fn print(&self) {
        logln!("[   ---{:^15}---   ]", "KERNEL INFO");
        let v = self.bootloader_version;
        logln!(
            "{:<20} {:>26}.{:02}.{:02}",
            "Bootloader version:",
            v.0,
            v.1,
            v.2
        );
        logln!(
            "{:<20} {:>32}",
            "Kernel start:",
            self.kernel_start.to_separated_hex()
        );
        if let Some(rsdp) = self.rsdp.as_ref() {
            logln!("{:<20} {:>32}", "RSDP:", rsdp.to_separated_hex());
        } else {
            logln!("{:<20} {:>32}", "RSDP:", "No RSDP found")
        }
        logln!(
            "{:<20} {:>32}",
            "Physical memory map:",
            self.physical_memory_offset.to_separated_hex()
        );
        if let Some(b) = self.framebuffer.as_ref() {
            logln!(
                "{:<20} {:>32}",
                "Frame buffer:",
                b.buffer.to_separated_hex()
            );
        } else {
            logln!("{:<20} {:>32}", "Frame buffer:", "No buffer");
        }
    }
}

pub static KERNEL_INFORMATION: OnceClone<KernelInformation> = OnceClone::new();
