use core::fmt::{self, Write};
use spin::Once;
use x86_64::instructions::interrupts;

pub trait Logger: Write + Send + Sync {
    fn try_receive<'a>(&'_ self, buffer: &'a mut [u8]) -> Option<&'a str>;
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
        #[allow(static_mut_refs)]
        let guard = unsafe { LOGGER.get_mut() };
        if let Some(logger) = guard {
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
        #[allow(static_mut_refs)]
        let guard = unsafe { LOGGER.get() };
        if let Some(logger) = guard {
            logger.try_receive(&mut data)
        } else {
            None
        }
    });
    if let Some(str) = read {
        callback(str);
    }
}

pub static mut LOGGER: Once<&'static mut dyn Logger> = Once::new();
