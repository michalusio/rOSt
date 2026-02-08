#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
extern crate alloc;

mod constants;
use alloc::{boxed::Box, vec::Vec};
pub use constants::{ATAIdentifyError, PRIMARY_ATA_BUS, SECONDARY_ATA_BUS};

mod bus;
pub use bus::ATABus;

mod disk_descriptor;
pub use disk_descriptor::DiskDescriptor;

mod partition_descriptor;
use internal_utils::{
    block_device::{BLOCK_DEVICES, BootableBlockDevice},
    display::format_size,
    logln,
};
pub use partition_descriptor::PartitionDescriptor;

mod disk;
pub use disk::ATADisk;

mod partition;
pub use partition::ATAPartition;
use spin::Mutex;

mod array_combiner;

pub fn init_disks() {
    BLOCK_DEVICES.call_once(Vec::new);

    let disk_a = ATABus::get_disk(&crate::PRIMARY_ATA_BUS, true);
    let disk_b = ATABus::get_disk(&crate::PRIMARY_ATA_BUS, false);
    let disk_c = ATABus::get_disk(&crate::SECONDARY_ATA_BUS, true);
    let disk_d = ATABus::get_disk(&crate::SECONDARY_ATA_BUS, false);
    logln!("[   ---{:^15}---   ]", "DISKS");
    init_disk(disk_a);
    init_disk(disk_b);
    init_disk(disk_c);
    init_disk(disk_d);
}

fn init_disk(disk: (&'static str, Result<ATADisk, ATAIdentifyError>)) {
    match disk.1 {
        Ok(ata_disk) => {
            logln!(
                "[{:^11}] {:<20}: {} ({} partitions){}",
                disk.0,
                ata_disk.descriptor.model_number().trim(),
                format_size(
                    ata_disk
                        .descriptor
                        .lba_48_addressable_sectors
                        .unwrap_or(ata_disk.descriptor.lba_28_addressable_sectors)
                        * 512
                ),
                ata_disk
                    .clone()
                    .get_partitions()
                    .map(|p| p.len())
                    .unwrap_or(0),
                ata_disk
                    .clone()
                    .has_bootloader()
                    .map(|b| if b { " (has bootloader)" } else { "" })
                    .unwrap_or(", Error while reading start sector")
            );
            BLOCK_DEVICES
                .write()
                .unwrap()
                .push(Box::new(Mutex::new(ata_disk)));
        }
        Err(err) => logln!("[{:^11}] {}", disk.0, err),
    }
}
