use bootloader_api::{BootloaderConfig, config::Mapping};

const ADDRESSES: &[u64] = &[
    0xFFFF_8000_0000_0000,
    0xFFFF_8010_0000_0000,
    0xFFFF_8020_0000_0000,
    0xFFFF_8030_0000_0000,
    0xFFFF_8040_0000_0000,
];

// We need to not allocate lower memory to keep it for DMA devices
pub const LOW_MEMORY_LIMIT: u64 = 0x0100_0000; // 16MiB

const KERNEL_STACK_SIZE: u64 = 16 * 1024 * 1024; // 16MiB
pub const HEAP_START: u64 = 0x0000_7FA0_0000_0000;
pub const HEAP_SIZE: u64 = 16 * 1024 * 1024; // 16MiB

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = KERNEL_STACK_SIZE;
    config.mappings.boot_info = Mapping::FixedAddress(ADDRESSES[0]);
    config.mappings.kernel_stack = Mapping::FixedAddress(ADDRESSES[1]);
    config.mappings.framebuffer = Mapping::FixedAddress(ADDRESSES[2]);
    config.mappings.kernel_base = Mapping::FixedAddress(ADDRESSES[3]);
    config.mappings.physical_memory = Some(Mapping::FixedAddress(ADDRESSES[4]));
    config
};
