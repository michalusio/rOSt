use core::{ffi::CStr, fmt::Write};

use uart_16550::SerialPort;

use crate::{
    logger::{LOGGER, Logger},
    logln,
};

static mut UART: SerialLogger = SerialLogger(unsafe { SerialPort::new(0x3F8) });

pub fn init_logger() {
    #[allow(static_mut_refs)]
    // Safety: We do not touch the UART but by using that reference, so it should be safe.
    unsafe {
        UART.init();
        LOGGER.call_once(|| &mut UART);
    };
    logln!("Initialized UART logger");
}

struct SerialLogger(SerialPort);

impl SerialLogger {
    fn init(&mut self) {
        self.0.init();
    }
}

impl Write for SerialLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s)
    }
}

impl Logger for SerialLogger {
    fn try_receive<'a>(&'_ mut self, buffer: &'a mut [u8]) -> Option<&'a str> {
        if let Ok(ch) = self.0.try_receive() {
            if ch == b'\r' {
                return None;
            }
            let mut index = 0;
            buffer[index] = ch;
            self.0.send(b'>');
            self.0.send(b' ');
            self.0.send(ch);
            index += 1;
            loop {
                if index >= buffer.len() {
                    return None;
                }
                let ch = self.0.receive();
                if ch == b'\r' || ch == b'\n' {
                    self.0.send(b'\r');
                    self.0.send(b'\n');
                    buffer[index] = 0;
                    break;
                } else if ch == b'\x08' || ch == b'\x7F' {
                    if index > 0 {
                        self.0.send(ch);
                        index -= 1;
                        buffer[index] = 0;
                    }
                } else {
                    self.0.send(ch);
                    buffer[index] = ch;
                    index += 1;
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
}
