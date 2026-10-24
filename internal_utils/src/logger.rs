use core::fmt::{self, Write};
use spin::Once;
use x86_64::instructions::interrupts;

pub trait Logger: Write + Send + Sync {
    fn try_receive<'a>(&'_ self, buffer: &'a mut [u8]) -> Option<&'a str>;
    fn supports_buffered_write(&mut self) -> bool;
    fn buffered_write_fmt<'a>(&'_ mut self, args: fmt::Arguments<'a>) -> fmt::Result;
    fn flush(&mut self) -> fmt::Result;
}

/// Logs text through the kernel logger immediately
///
/// Don't use inside normal process logging - slow
#[macro_export]
macro_rules! ilog {
    ($($arg:tt)*) => ($crate::logger::__iprint(format_args!($($arg)*)));
}

/// Logs a line through the kernel logger immediately
///
/// Don't use inside normal process logging - slow
#[macro_export]
macro_rules! ilogln {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::ilog!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn __iprint(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        #[allow(static_mut_refs)]
        let guard = unsafe { LOGGER.get_mut() };
        if let Some(logger) = guard {
            logger.write_fmt(args).unwrap();
        }
    });
}

/// Flushes the texts buffered in the logger
#[macro_export]
macro_rules! log_flush {
    () => {
        $crate::logger::__flush()
    };
}

/// Logs text through the kernel logger, buffered
///
/// Flushes after a few messages are queued
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ($crate::logger::__print(format_args!($($arg)*)));
}

/// Logs a line through the kernel logger, buffered
///
/// Flushes after a few messages are queued
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
            if logger.supports_buffered_write() {
                logger.buffered_write_fmt(args).unwrap();
            } else {
                logger.write_fmt(args).unwrap();
            }
        }
    });
}

#[doc(hidden)]
pub fn __flush() {
    interrupts::without_interrupts(|| {
        #[allow(static_mut_refs)]
        let guard = unsafe { LOGGER.get_mut() };
        if let Some(logger) = guard
            && logger.supports_buffered_write()
        {
            logger.flush().unwrap();
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
