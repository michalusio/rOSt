use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::structures::OnceLock;

mod traits;
pub use traits::{BootableBlockDevice, PartitionableBlockDevice};
mod capability;
pub use capability::{
    BlockDeviceCapabilityMut, BlockDeviceCapabilityRef, BlockDeviceCapabilityRequest,
};

pub trait BlockDevice: Send {
    fn name(&self) -> &str;
    fn read_sector(&self, lba: u64) -> Result<[u8; 512], BlockDeviceError>;
    fn write_sector(&mut self, lba: u64, buffer: &[u8; 512]) -> Result<(), BlockDeviceError>;

    fn get_capability(
        &'_ self,
        request: BlockDeviceCapabilityRequest,
    ) -> Option<BlockDeviceCapabilityRef<'_>>;
    fn get_capability_mut(
        &'_ mut self,
        request: BlockDeviceCapabilityRequest,
    ) -> Option<BlockDeviceCapabilityMut<'_>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceError {
    /// A bad sector has been written to or read from
    BadSector,
    /// A retryable error has occured
    Retry,
    /// The operation has tried to access an out-of-range item
    OutOfRange,
    /// The operation tried to add a partition when no additional partitions can be added
    TooManyPartitions,
    /// Unknown error
    Unknown(u8),
}

pub static BLOCK_DEVICES: OnceLock<Vec<Box<Mutex<dyn BlockDevice>>>> = OnceLock::new();
