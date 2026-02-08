use core::fmt::Display;

use alloc::sync::Arc;
use internal_utils::block_device::BlockDeviceError;
use spin::Mutex;

use super::bus::ATABus;
use bitflags::bitflags;
use lazy_static::lazy_static;

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusRegisterFlags: u8 {
        /// Busy
        const BSY = 0b10000000;
        /// Device Ready
        const DRDY = 0b01000000;
        /// Device Fault
        const DF = 0b00100000;
        /// Seek Complete
        const DSC = 0b00010000;
        /// Data Transfer Required
        const DRQ = 0b00001000;
        /// Data Corrected
        const CORR = 0b00000100;
        /// Index Mark
        const IDX = 0b00000010;
        /// Error
        const ERR = 0b00000001;
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ErrorRegisterFlags: u8 {
    /// Bad Block
        const BBK = 0b10000000;
    /// Uncorrectable Data Error
        const UNC = 0b01000000;
    /// Media Changed
        const MC = 0b00100000;
    /// ID Mark Not Found
        const IDNF = 0b00010000;
    /// Media Change Requested
        const MCR = 0b00001000;
    /// Command Aborted
        const ABRT = 0b00000100;
    /// Track 0 Not Found
        const TK0NF = 0b00000010;
    /// Address Mark Not Found
        const AMNF = 0b00000001;
    }
}

impl ErrorRegisterFlags {
    pub fn to_device_error(self) -> BlockDeviceError {
        match self {
            ErrorRegisterFlags::ABRT => BlockDeviceError::Retry,
            ErrorRegisterFlags::MCR => BlockDeviceError::Retry,
            ErrorRegisterFlags::MC => BlockDeviceError::Retry,

            ErrorRegisterFlags::BBK => BlockDeviceError::BadSector,
            ErrorRegisterFlags::UNC => BlockDeviceError::BadSector,

            _ => BlockDeviceError::Unknown(self.bits()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusError {
    BlockDeviceError(BlockDeviceError),
    ATAError(ErrorRegisterFlags),
}

impl BusError {
    pub fn to_device_error(self) -> BlockDeviceError {
        match self {
            BusError::BlockDeviceError(e) => e,
            BusError::ATAError(e) => e.to_device_error(),
        }
    }
}

impl From<BlockDeviceError> for BusError {
    fn from(value: BlockDeviceError) -> Self {
        BusError::BlockDeviceError(value)
    }
}

impl From<ErrorRegisterFlags> for BusError {
    fn from(value: ErrorRegisterFlags) -> Self {
        BusError::ATAError(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ATAIdentifyError {
    BusNotConnected = 0,
    NoDevice,
    DeviceIsNotATA,
    DeviceIsATAPI,
    DeviceIsSATA,
    Unknown = 255,
}

impl Display for ATAIdentifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ATAIdentifyError::BusNotConnected => write!(f, "Bus not connected"),
            ATAIdentifyError::NoDevice => write!(f, "No device"),
            ATAIdentifyError::DeviceIsNotATA => write!(f, "Device is not ATA"),
            ATAIdentifyError::DeviceIsATAPI => write!(f, "Device is ATAPI"),
            ATAIdentifyError::DeviceIsSATA => write!(f, "Device is SATA"),
            ATAIdentifyError::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
#[non_exhaustive]
pub enum ATACommands {
    Identify = 0xEC,
    WriteSectors = 0x30,
    ReadSectors = 0x20,
    CacheFlush = 0xE7,
}

lazy_static! {
    pub static ref PRIMARY_ATA_BUS: Arc<Mutex<ATABus>> =
        Arc::new(Mutex::new(ATABus::new(0x1F0, true)));
    pub static ref SECONDARY_ATA_BUS: Arc<Mutex<ATABus>> =
        Arc::new(Mutex::new(ATABus::new(0x170, false)));
}
