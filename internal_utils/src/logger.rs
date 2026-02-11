use core::fmt::{self, Write};
use x86_64::instructions::interrupts;

use crate::structures::OnceMutex;

pub trait Logger: Write + Send {
    fn try_receive<'a>(&'_ mut self, buffer: &'a mut [u8]) -> Option<&'a str>;
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ($crate::logger::__print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! logln {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn __print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let guard = LOGGER.lock();
        if let Some(mut logger) = guard {
            logger.write_fmt(args).unwrap();
        }
    });
}

#[macro_export]
macro_rules! try_serial_read {
    ($arg:expr) => {
        $crate::logger::__try_serial_read($arg)
    };
}

#[doc(hidden)]
pub fn __try_serial_read(callback: impl FnOnce(&str)) {
    let mut data = [0u8; 64];
    let read = interrupts::without_interrupts(|| {
        let guard = LOGGER.lock();
        if let Some(mut logger) = guard {
            logger.try_receive(&mut data)
        } else {
            None
        }
    });
    if let Some(str) = read {
        callback(str);
    }
}

pub static LOGGER: OnceMutex<&'static mut dyn Logger> = OnceMutex::new();
