use core::fmt::{self, Write};
use x86_64::instructions::interrupts;

use crate::structures::OnceMutex;

pub trait Logger: Write + Send {
    fn log(&mut self, message: &str);
    fn logln(&mut self, message: &str);
}

impl<T: Write + Send> Logger for T {
    fn log(&mut self, message: &str) {
        self.write_str(message).unwrap();
    }

    fn logln(&mut self, message: &str) {
        self.write_str(message).unwrap();
        self.write_char('\n').unwrap();
    }
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
macro_rules! log {
    ($($arg:tt)*) => ($crate::logger::__print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! logln {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!("{}\n", format_args!($($arg)*)));
}

pub static LOGGER: OnceMutex<&'static mut dyn Logger> = OnceMutex::new();
