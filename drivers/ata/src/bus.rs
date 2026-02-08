use alloc::sync::Arc;
use internal_utils::{
    block_device::BlockDeviceError,
    port_extensions::{PortExtRead, PortExtWrite},
};
use spin::Mutex;
use x86_64::instructions::{
    interrupts::without_interrupts,
    port::{Port, PortReadOnly, PortWriteOnly},
};

use crate::{
    ATADisk,
    constants::{ATACommands, BusError, ErrorRegisterFlags, StatusRegisterFlags},
};

use super::{constants::ATAIdentifyError, disk_descriptor::DiskDescriptor};

#[allow(dead_code)]
pub struct ATABus {
    primary: bool,
    data_register_rw: Port<u16>,
    error_register_r: PortReadOnly<u8>,
    features_register_w: PortWriteOnly<u8>,
    sector_count_register_rw: Port<u8>,
    lba_low_register_rw: Port<u8>,
    lba_mid_register_rw: Port<u8>,
    lba_high_register_rw: Port<u8>,
    drive_head_register_rw: Port<u8>,
    status_register_r: PortReadOnly<u8>,
    command_register_w: PortWriteOnly<u8>,
    alternate_status_register_r: PortReadOnly<u8>,
    device_control_register_w: PortWriteOnly<u8>,
    drive_address_register_r: PortReadOnly<u8>,

    disk_1_descriptor: Option<DiskDescriptor>,
    disk_2_descriptor: Option<DiskDescriptor>,
}

impl ATABus {
    pub(crate) const fn new(base_port: u16, primary: bool) -> Self {
        ATABus {
            primary,
            data_register_rw: Port::new(base_port),
            error_register_r: PortReadOnly::new(base_port + 0x01),
            features_register_w: PortWriteOnly::new(base_port + 0x01),
            sector_count_register_rw: Port::new(base_port + 0x02),
            lba_low_register_rw: Port::new(base_port + 0x03),
            lba_mid_register_rw: Port::new(base_port + 0x04),
            lba_high_register_rw: Port::new(base_port + 0x05),
            drive_head_register_rw: Port::new(base_port + 0x06),
            status_register_r: PortReadOnly::new(base_port + 0x07),
            command_register_w: PortWriteOnly::new(base_port + 0x07),
            alternate_status_register_r: PortReadOnly::new(base_port + 0x206),
            device_control_register_w: PortWriteOnly::new(base_port + 0x206),
            drive_address_register_r: PortReadOnly::new(base_port + 0x207),
            disk_1_descriptor: None,
            disk_2_descriptor: None,
        }
    }

    pub fn primary(&self) -> bool {
        self.primary
    }

    pub fn connected(&mut self) -> bool {
        unsafe { self.status_register_r.read() != 0xFF }
    }

    pub fn wait_for(
        &mut self,
        flag: StatusRegisterFlags,
        should_be_on: bool,
    ) -> Result<(), ErrorRegisterFlags> {
        let condition = if should_be_on {
            flag
        } else {
            StatusRegisterFlags::empty()
        };
        loop {
            unsafe {
                let status = StatusRegisterFlags::from_bits_truncate(self.status_register_r.read());
                if status.intersection(flag) == condition {
                    break;
                }
                if status.contains(StatusRegisterFlags::ERR) {
                    let error = self.error_register_r.read();
                    if error != 0 {
                        return Err(ErrorRegisterFlags::from_bits_truncate(error));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn wait_400ns(&mut self) -> Result<(), ErrorRegisterFlags> {
        for _ in 0..15 {
            unsafe {
                let status = StatusRegisterFlags::from_bits_truncate(self.status_register_r.read());
                if status.contains(StatusRegisterFlags::ERR) {
                    let error = self.error_register_r.read();
                    if error != 0 {
                        return Err(ErrorRegisterFlags::from_bits_truncate(error));
                    }
                }
            }
        }
        Ok(())
    }
}

impl ATABus {
    pub(crate) fn identify(&mut self, master: bool) -> Result<DiskDescriptor, ATAIdentifyError> {
        if master && let Some(descriptor) = &self.disk_1_descriptor {
            return Ok(descriptor.clone());
        }
        if !master && let Some(descriptor) = &self.disk_2_descriptor {
            return Ok(descriptor.clone());
        }
        unsafe fn handle_identify_error(
            bus: &mut ATABus,
            error: ErrorRegisterFlags,
        ) -> ATAIdentifyError {
            if error != ErrorRegisterFlags::ABRT {
                return ATAIdentifyError::DeviceIsATAPI;
            }
            let mid = unsafe { bus.lba_mid_register_rw.read() };
            let high = unsafe { bus.lba_high_register_rw.read() };

            match (mid, high) {
                (0x14, 0xEB) => ATAIdentifyError::DeviceIsATAPI,
                (0x3C, 0xC3) => ATAIdentifyError::DeviceIsSATA,
                (0, 0) => ATAIdentifyError::Unknown,
                (_, _) => ATAIdentifyError::DeviceIsNotATA,
            }
        }

        unsafe {
            if !self.connected() {
                return Err(ATAIdentifyError::BusNotConnected);
            }
            if self.wait_for(StatusRegisterFlags::BSY, false).is_err() {
                return Err(ATAIdentifyError::Unknown);
            }
            without_interrupts(|| {
                self.drive_head_register_rw
                    .write(if master { 0xA0 } else { 0xB0 });
                self.device_control_register_w.write(0x00);
                if self.wait_400ns().is_err() {
                    return Err(ATAIdentifyError::Unknown);
                }
                self.sector_count_register_rw.write(0x00);
                self.lba_low_register_rw.write(0x00);
                self.lba_mid_register_rw.write(0x00);
                self.lba_high_register_rw.write(0x00);
                self.command_register_w.write(ATACommands::Identify as u8);
                let status = self.status_register_r.read();
                if status == 0 {
                    return Err(ATAIdentifyError::NoDevice);
                }
                if let Err(error) = self.wait_for(StatusRegisterFlags::BSY, false) {
                    return Err(handle_identify_error(self, error));
                }
                if let Err(error) = self.wait_for(StatusRegisterFlags::DRQ, true) {
                    return Err(handle_identify_error(self, error));
                }
                let mut identify_buffer: [u16; 256] = [0; 256];
                self.data_register_rw.read_to_buffer(&mut identify_buffer);
                let descriptor = DiskDescriptor::from_bytes(identify_buffer);
                if master {
                    self.disk_1_descriptor = Some(descriptor);
                    Ok(self.disk_1_descriptor.as_ref().unwrap().clone())
                } else {
                    self.disk_2_descriptor = Some(descriptor);
                    Ok(self.disk_2_descriptor.as_ref().unwrap().clone())
                }
            })
        }
    }

    pub(crate) fn read_sector(
        &mut self,
        master: bool,
        descriptor: &DiskDescriptor,
        lba: u64,
    ) -> Result<[u8; 512], BusError> {
        if lba > descriptor.lba_28_addressable_sectors {
            if descriptor
                .lba_48_addressable_sectors
                .is_none_or(|lba_48| lba >= lba_48)
            {
                return Err(BlockDeviceError::OutOfRange.into());
            }
            // TODO Add LBA48 support to the ATA driver
            // We need to check the IO calls for LBA48 support
            todo!("LBA48 not supported");
        }
        let slave = if master { 0xE0 } else { 0xF0 };
        let mut buffer = [0u8; 512];
        let head = (lba >> 24) & 0x0F;
        let lba_mid = lba >> 8;
        let lba_high = lba >> 16;
        without_interrupts(|| unsafe {
            self.drive_head_register_rw.write(slave | head as u8);
            self.features_register_w.write(0x00);
            self.sector_count_register_rw.write(0x01);
            self.lba_low_register_rw.write(lba as u8);
            self.lba_mid_register_rw.write(lba_mid as u8);
            self.lba_high_register_rw.write(lba_high as u8);
            self.command_register_w
                .write(ATACommands::ReadSectors as u8);
            self.wait_for(StatusRegisterFlags::BSY, false)?;
            self.wait_for(StatusRegisterFlags::DRQ, true)?;
            self.data_register_rw.read_to_buffer(&mut buffer);
            self.wait_400ns()?;
            Ok(buffer)
        })
    }

    pub(crate) fn write_sector(
        &mut self,
        master: bool,
        descriptor: &DiskDescriptor,
        lba: u64,
        buffer: &[u8; 512],
    ) -> Result<(), BusError> {
        if lba > descriptor.lba_28_addressable_sectors {
            if descriptor
                .lba_48_addressable_sectors
                .is_none_or(|lba_48| lba >= lba_48)
            {
                return Err(BlockDeviceError::OutOfRange.into());
            }
            // TODO Add LBA48 support to the ATA driver
            // We need to check the IO calls for LBA48 support
            todo!("LBA48 not supported");
        }
        let slave = if master { 0xE0 } else { 0xF0 };
        let head = (lba >> 24) & 0x0F;
        let lba_mid = lba >> 8;
        let lba_high = lba >> 16;
        without_interrupts(|| unsafe {
            self.drive_head_register_rw.write(slave | head as u8);
            self.features_register_w.write(0x00);
            self.sector_count_register_rw.write(0x01);
            self.lba_low_register_rw.write(lba as u8);
            self.lba_mid_register_rw.write(lba_mid as u8);
            self.lba_high_register_rw.write(lba_high as u8);
            self.command_register_w
                .write(ATACommands::WriteSectors as u8);
            self.wait_for(StatusRegisterFlags::BSY, false)?;
            self.wait_for(StatusRegisterFlags::DRQ, true)?;
            self.data_register_rw.write_from_buffer(buffer);
            self.wait_400ns()?;
            self.wait_for(StatusRegisterFlags::DRQ, false)?;
            self.wait_for(StatusRegisterFlags::BSY, false)?;
            self.command_register_w.write(ATACommands::CacheFlush as u8);
            self.wait_for(StatusRegisterFlags::BSY, false)?;
            Ok(())
        })
    }

    pub fn get_disk(
        this: &Arc<Mutex<Self>>,
        master: bool,
    ) -> (&'static str, Result<ATADisk, ATAIdentifyError>) {
        let mut locked = this.lock();
        let name = get_disk_name(locked.primary, master);
        match locked.identify(master) {
            Ok(descriptor) => (
                name,
                Ok(ATADisk {
                    bus: this.clone(),
                    descriptor,
                    master,
                }),
            ),
            Err(e) => (name, Err(e)),
        }
    }
}

pub fn get_disk_name(primary_bus: bool, master_disk: bool) -> &'static str {
    match (primary_bus, master_disk) {
        (false, false) => "ATA Secondary Slave",
        (false, true) => "ATA Secondary Master",
        (true, false) => "ATA Primary Slave",
        (true, true) => "ATA Primary Master",
    }
}
