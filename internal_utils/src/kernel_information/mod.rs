use alloc::sync::Arc;
use bootloader_api::{
    BootInfo,
    info::{MemoryRegions, Optional},
};
use spin::Mutex;
use x86_64::PhysAddr;

use crate::{
    kernel_information::{
        frame_allocator::FullFrameAllocator, kernel_frame_buffer::KernelFrameBuffer,
    },
    structures::OnceClone,
};

pub mod frame_allocator;
pub mod kernel_frame_buffer;

#[derive(Clone)]
#[repr(C)]
pub struct KernelInformation {
    pub bootloader_version: [u16; 3],
    pub physical_memory_offset: u64,
    pub framebuffer: Optional<KernelFrameBuffer>,
    pub memory_regions: &'static MemoryRegions,
    pub allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>,
    /// The start address of the kernel space in all page maps
    pub kernel_start: PhysAddr,
}

impl KernelInformation {
    pub fn new(
        boot_info: &'static BootInfo,
        allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>,
    ) -> KernelInformation {
        let bootloader_version = [
            boot_info.api_version.version_major(),
            boot_info.api_version.version_minor(),
            boot_info.api_version.version_patch(),
        ];
        let framebuffer = match boot_info.framebuffer.as_ref() {
            Some(framebuffer) => Optional::Some(KernelFrameBuffer::new(framebuffer)),
            None => Optional::None,
        };
        let kernel_info = KernelInformation {
            bootloader_version,
            physical_memory_offset: *boot_info
                .physical_memory_offset
                .as_ref()
                .expect("No physical memory mapping"),
            framebuffer,
            memory_regions: &boot_info.memory_regions,
            allocator,
            kernel_start: PhysAddr::new(0x007F_C000_0000u64),
        };
        KERNEL_INFORMATION.call_once(|| kernel_info.clone());
        kernel_info
    }
}

pub static KERNEL_INFORMATION: OnceClone<KernelInformation> = OnceClone::new();
