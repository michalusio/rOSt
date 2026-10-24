use core::{
    ffi::CStr,
    fmt::{Error, Write},
};

use alloc::vec;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::{Mutex, MutexGuard};
use uart_16550::SerialPort;

use crate::{
    ilogln,
    kernel_information::allocator::ALLOCATOR,
    logger::{LOGGER, Logger},
};

static mut UART: SerialLogger = SerialLogger(Mutex::new(unsafe { SerialPort::new(0x3F8) }), None);

pub fn init_logger() {
    #[allow(static_mut_refs)]
    // Safety: We do not touch the UART but by using that reference, so it should be safe.
    unsafe {
        UART.init();
        LOGGER.call_once(|| &mut UART);
    };
    ilogln!("Initialized UART logger");
}

pub fn unlock_logger() {
    unsafe {
        #[allow(static_mut_refs)]
        UART.0.force_unlock();
    }
}

struct SerialLogger(Mutex<SerialPort>, Option<LogBuffer>);
struct LogBuffer(Vec<(String, u32)>);

impl SerialLogger {
    fn init(&mut self) {
        self.0.lock().init();
    }

    fn flush(port: &mut MutexGuard<'_, SerialPort>, buffer: &mut LogBuffer) -> core::fmt::Result {
        for (item, count) in buffer.0.drain(..) {
            match count {
                0 => {}
                1 => port.write_str(&item)?,
                _ => {
                    write!(port, "(x{}) {}", count, &item)?;
                }
            }
        }
        Ok(())
    }
}

impl Write for SerialLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(mut locked) = self.0.try_lock() {
            if let Some(buffer) = self.1.as_mut() {
                SerialLogger::flush(&mut locked, buffer)?;
            }
            locked.write_str(s)
        } else {
            // I will NOT be ignored!
            unsafe {
                self.0.force_unlock();
            }
            self.write_str(s)
        }
    }
}

impl Logger for SerialLogger {
    fn try_receive<'a>(&'_ self, buffer: &'a mut [u8]) -> Option<&'a str> {
        let maybe_locked = self.0.try_lock();
        if maybe_locked.is_some() {
            let mut index = 0;
            {
                let mut locked = maybe_locked.unwrap();
                if let Ok(ch) = locked.try_receive() {
                    if ch == b'\r' {
                        return None;
                    }
                    buffer[index] = ch;
                    locked.send(b'>');
                    locked.send(b' ');
                    locked.send(ch);
                    index += 1;
                    loop {
                        if index >= buffer.len() {
                            return None;
                        }
                        let ch = locked.receive();
                        if ch == b'\r' || ch == b'\n' {
                            locked.send(b'\r');
                            locked.send(b'\n');
                            buffer[index] = 0;
                            break;
                        } else if ch == b'\x08' || ch == b'\x7F' {
                            if index > 0 {
                                locked.send(ch);
                                index -= 1;
                                buffer[index] = 0;
                            }
                        } else {
                            locked.send(ch);
                            buffer[index] = ch;
                            index += 1;
                        }
                    }
                } else {
                    return None;
                }
            }
            CStr::from_bytes_until_nul(buffer)
                .into_iter()
                .flat_map(CStr::to_str)
                .next()
        } else {
            None
        }
    }

    fn supports_buffered_write(&mut self) -> bool {
        let has_initialized_allocator = ALLOCATOR.try_lock().is_some_and(|alloc| alloc.size() > 0);

        if !has_initialized_allocator {
            false
        } else {
            self.1.get_or_insert_with(|| LogBuffer(vec![]));
            true
        }
    }

    fn buffered_write_fmt<'a>(&'_ mut self, args: core::fmt::Arguments<'a>) -> core::fmt::Result {
        if let Some(buffer) = self.1.as_mut() {
            alloc::fmt::write(buffer, args)
        } else {
            Err(Error)
        }
    }

    fn flush(&mut self) -> core::fmt::Result {
        if let Some(buffer) = self.1.as_mut() {
            if let Some(mut locked) = self.0.try_lock() {
                SerialLogger::flush(&mut locked, buffer)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl Write for LogBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(last) = self.0.last_mut()
            && last.0 == s
        {
            last.1 += 1;
        } else {
            self.0.push((s.to_string(), 1));
        }
        Ok(())
    }
}
