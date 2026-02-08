use crate::{array_combiner::Combiner, bus::get_disk_name};
use alloc::{sync::Arc, vec::Vec};
use internal_utils::{
    block_device::{
        BlockDevice, BlockDeviceCapabilityMut, BlockDeviceCapabilityRef,
        BlockDeviceCapabilityRequest, BlockDeviceError, BootableBlockDevice,
        PartitionableBlockDevice,
    },
    has_block_device_capability,
};
use spin::Mutex;

use crate::{ATABus, ATAPartition, DiskDescriptor, PartitionDescriptor};

#[derive(Clone)]
pub struct ATADisk {
    pub(crate) bus: Arc<Mutex<ATABus>>,
    pub descriptor: DiskDescriptor,
    pub(crate) master: bool,
}

impl ATADisk {
    pub fn get_partitions(&mut self) -> Result<Vec<ATAPartition>, BlockDeviceError> {
        let mbr = self.read_sector(0)?;
        let descriptors = mbr[446..510]
            .chunks(16)
            .filter_map(PartitionDescriptor::from_bytes);

        Ok(descriptors
            .map(|descriptor| ATAPartition {
                disk: self.clone(),
                descriptor,
            })
            .collect())
    }

    pub fn create_partition(
        &mut self,
        sectors: u32,
        partition_type: u8,
    ) -> Result<ATAPartition, BlockDeviceError> {
        let mut mbr = self.read_sector(0)?;
        let descriptors: Vec<PartitionDescriptor> = mbr[446..510]
            .chunks(16)
            .filter_map(PartitionDescriptor::from_bytes)
            .collect();
        if descriptors.len() >= 4 {
            return Err(BlockDeviceError::TooManyPartitions);
        }

        let start_sector_bytes = {
            let start_sector = (descriptors
                .iter()
                .map(|d| d.start_lba + d.sectors)
                .max()
                .unwrap_or(0)
                + 1) as u32;
            u32::to_le_bytes(start_sector)
        };

        let sectors_bytes = u32::to_le_bytes(sectors);

        let partition_bytes = Combiner::new()
            .with(&[0x00, 0xFF, 0xFF, 0xFF])
            .with(&[partition_type, 0xFF, 0xFF, 0xFF])
            .with(&start_sector_bytes)
            .with(&sectors_bytes)
            .build::<16>()
            .expect("Wrong number of bytes for a partition descriptor");

        let descriptor = PartitionDescriptor::from_bytes(&partition_bytes);
        if descriptor.is_none() {
            return Err(BlockDeviceError::Unknown(0));
        }
        let descriptor = descriptor.unwrap();
        let partition_free_index = mbr[446..510]
            .chunks(16)
            .enumerate()
            .map(|(index, val)| (PartitionDescriptor::from_bytes(val), index))
            .find(|(val, _)| val.is_none())
            .map(|(_, i)| i);
        if partition_free_index.is_none() {
            return Err(BlockDeviceError::Unknown(0));
        }
        let partition_free_index = partition_free_index.unwrap();
        mbr[446 + partition_free_index * 16..446 + partition_free_index * 16 + 16]
            .copy_from_slice(&partition_bytes);
        self.write_sector(0, &mbr)?;
        Ok(ATAPartition {
            disk: self.clone(),
            descriptor,
        })
    }
}

impl BlockDevice for ATADisk {
    fn name(&self) -> &str {
        get_disk_name(self.bus.lock().primary(), self.master)
    }

    fn read_sector(&self, lba: u64) -> Result<[u8; 512], BlockDeviceError> {
        self.bus
            .lock()
            .read_sector(self.master, &self.descriptor, lba)
            .map_err(|e| e.to_device_error())
    }

    fn write_sector(&mut self, lba: u64, buffer: &[u8; 512]) -> Result<(), BlockDeviceError> {
        self.bus
            .lock()
            .write_sector(self.master, &self.descriptor, lba, buffer)
            .map_err(|e| e.to_device_error())
    }

    has_block_device_capability!(Bootable);
}

impl BootableBlockDevice for ATADisk {
    fn has_bootloader(&mut self) -> Result<bool, BlockDeviceError> {
        let buffer = self.read_sector(0)?;
        Ok(buffer[510] == 0x55 && buffer[511] == 0xAA)
    }
}

impl PartitionableBlockDevice for ATADisk {}
