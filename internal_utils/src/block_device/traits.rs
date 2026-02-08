use crate::block_device::{BlockDevice, BlockDeviceError};

pub trait BootableBlockDevice: BlockDevice {
    fn has_bootloader(&mut self) -> Result<bool, BlockDeviceError>;
}

pub trait PartitionableBlockDevice: BlockDevice {}
