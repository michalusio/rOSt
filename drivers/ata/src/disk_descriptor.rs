#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiskDescriptor {
    pub fixed_device: bool,
    pub removable_media: bool,
    pub is_ata_device: bool,

    pub cylinders: u16,
    pub heads: u16,
    pub sectors_per_track: u16,
    pub vendor_unique: [u16; 3],
    serial_number_bytes: [u8; 20],
    pub firmware_revision: [u8; 8],
    model_number_bytes: [u8; 40],
    pub udma_available_modes: [bool; 8],
    pub udma_current_mode: u8,
    pub lba_28_addressable_sectors: u64,
    pub lba_48_addressable_sectors: Option<u64>,
}

impl DiskDescriptor {
    pub fn serial_number(&self) -> &str {
        core::str::from_utf8(&self.serial_number_bytes).unwrap()
    }

    pub fn model_number(&self) -> &str {
        core::str::from_utf8(&self.model_number_bytes).unwrap()
    }

    pub(crate) fn from_bytes(buffer: [u16; 256]) -> Self {
        let fixed_device = buffer[0] & 0x0040 != 0;
        let removable_media = buffer[0] & 0x0080 != 0;
        let is_ata_device = buffer[0] & 0x8000 != 0;

        let cylinders = buffer[1];
        let heads = buffer[3];
        let sectors_per_track = buffer[6];
        let vendor_unique = [buffer[7], buffer[8], buffer[9]];
        let mut serial_number = [0; 20];
        for (index, word) in buffer[10..20].iter().enumerate() {
            serial_number[index * 2] = (*word >> 8) as u8;
            serial_number[index * 2 + 1] = *word as u8;
        }

        let mut firmware_revision = [0; 8];
        for (index, word) in buffer[23..26].iter().enumerate() {
            firmware_revision[index * 2] = (*word >> 8) as u8;
            firmware_revision[index * 2 + 1] = *word as u8;
        }

        let mut model_number = [0u8; 40];
        for (index, word) in buffer[27..47].iter().enumerate() {
            model_number[index * 2] = (*word >> 8) as u8;
            model_number[index * 2 + 1] = *word as u8;
        }

        let udma = buffer[88];
        let udma_current_mode = (udma >> 8) as u8;
        let udma_available_modes = {
            let udma_available_modes = udma as u8;
            let mut udma_buffer = [false; 8];
            for (i, item) in udma_buffer.iter_mut().enumerate() {
                if udma_available_modes & (1 << i) != 0 {
                    *item = true;
                }
            }
            udma_buffer
        };

        let supports_lba_48 = buffer[83] & 0x0400 != 0;
        let lba_28_addressable_sectors = (buffer[61] as u64) << 16 | (buffer[60] as u64);
        let lba_48_addressable_sectors = (buffer[103] as u64) << 48
            | (buffer[102] as u64) << 32
            | (buffer[101] as u64) << 16
            | (buffer[100] as u64);

        Self {
            fixed_device,
            removable_media,
            is_ata_device,
            cylinders,
            heads,
            sectors_per_track,
            vendor_unique,
            serial_number_bytes: serial_number,
            firmware_revision,
            model_number_bytes: model_number,
            udma_available_modes,
            udma_current_mode,
            lba_28_addressable_sectors,
            lba_48_addressable_sectors: if supports_lba_48 {
                Some(lba_48_addressable_sectors)
            } else {
                None
            },
        }
    }
}
