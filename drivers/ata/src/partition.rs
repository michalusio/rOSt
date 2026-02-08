use internal_utils::block_device::{BlockDevice, BlockDeviceError};

use crate::{ATADisk, DiskDescriptor, PartitionDescriptor};

#[derive(Clone)]
pub struct ATAPartition {
    pub(crate) disk: ATADisk,
    pub descriptor: PartitionDescriptor,
}

impl ATAPartition {
    pub fn disk_descriptor(&self) -> DiskDescriptor {
        self.disk.descriptor.clone()
    }

    pub fn read_sector(&mut self, lba: u64) -> Result<[u8; 512], BlockDeviceError> {
        if lba >= self.descriptor.sectors {
            Err(BlockDeviceError::OutOfRange)
        } else {
            self.disk.read_sector(lba + self.descriptor.start_lba)
        }
    }

    pub fn write_sector(&mut self, lba: u64, buffer: &[u8; 512]) -> Result<(), BlockDeviceError> {
        if lba >= self.descriptor.sectors {
            Err(BlockDeviceError::OutOfRange)
        } else {
            self.disk
                .write_sector(lba + self.descriptor.start_lba, buffer)
        }
    }
}
