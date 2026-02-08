use bootloader_api::{BootloaderConfig, config::Mapping};

const ADDRESSES: &[u64] = &[
    0x0000_7FB0_0000_0000,
    0x0000_7FC0_0000_0000,
    0x0000_7FD0_0000_0000,
    0x0000_7FE0_0000_0000,
    0x0000_7FF0_0000_0000,
];
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
