use core::fmt::Display;

use bitflags::bitflags;
use spin::Once;
use x86_64::instructions::{
    nop,
    port::{PortReadOnly, PortWriteOnly},
};

use crate::logln;

static mut RTC_CONTROLLER: Once<RtcController> = Once::new();

pub fn init_rtc() {
    unsafe {
        #[allow(static_mut_refs)]
        RTC_CONTROLLER.call_once(|| {
            let mut controller = RtcController {
                register_port: PortWriteOnly::<u8>::new(0x70),
                value_port: PortReadOnly::<u8>::new(0x71),
                format: RtcFormatFlags::empty(),
            };

            let format = RtcFormatFlags::from_bits_truncate(controller.read_register(0x0B));
            controller.format = format;

            controller
        })
    };
    logln!("Initialized RTC, current time: {}", get_current_time());
}

pub fn get_current_time() -> RtcTime {
    #[allow(static_mut_refs)]
    if let Some(controller) = unsafe { RTC_CONTROLLER.get_mut() } {
        controller.get_current_time()
    } else {
        RtcTime {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

pub struct RtcController {
    register_port: PortWriteOnly<u8>,
    value_port: PortReadOnly<u8>,
    format: RtcFormatFlags,
}

pub struct RtcTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RtcFormatFlags: u8 {
        /// Binary mode
        const BINARY = 0b00000100;
        /// 24-Hour format
        const FULL_HOUR = 0b00000010;
    }
}

impl RtcController {
    pub fn get_current_time(&mut self) -> RtcTime {
        let mut seconds = self.read_register(0x00);
        let mut minutes = self.read_register(0x02);
        let mut hours = self.read_register(0x04);
        let mut days = self.read_register(0x07);
        let mut months = self.read_register(0x08);
        let mut years = self.read_register(0x09);

        loop {
            let new_seconds = self.read_register(0x00);
            let new_minutes = self.read_register(0x02);
            let new_hours = self.read_register(0x04);
            let new_days = self.read_register(0x07);
            let new_months = self.read_register(0x08);
            let new_years = self.read_register(0x09);

            if new_seconds == seconds
                && new_minutes == minutes
                && new_hours == hours
                && new_days == days
                && new_months == months
                && new_years == years
            {
                // No change, so the RTC was not busy
                break;
            }

            seconds = new_seconds;
            minutes = new_minutes;
            hours = new_hours;
            days = new_days;
            months = new_months;
            years = new_years;
        }

        let hour_is_pm = hours >= 0x80;
        hours &= 0x7F;

        if !self.format.contains(RtcFormatFlags::BINARY) {
            seconds = bcd_to_binary(seconds);
            minutes = bcd_to_binary(minutes);
            hours = bcd_to_binary(hours);
            days = bcd_to_binary(days);
            months = bcd_to_binary(months);
            years = bcd_to_binary(years);
        }

        if !self.format.contains(RtcFormatFlags::FULL_HOUR) && hour_is_pm {
            hours += 12;
            if hours == 12 {
                hours = 0;
            }
        }

        RtcTime {
            year: 2000 + (years as u16),
            month: months,
            day: days,
            hour: hours,
            minute: minutes,
            second: seconds,
        }
    }

    fn read_register(&mut self, id: u8) -> u8 {
        unsafe {
            // Selecting register
            self.register_port.write(id);
            // We need a tiny delay when switching the register
            nop();
            nop();
            // Reading the status
            self.value_port.read()
        }
    }
}

impl Display for RtcTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 1) + ((bcd & 0xF0) >> 3) + (bcd & 0xf)
}
