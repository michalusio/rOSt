use core::fmt::Write;

use uart_16550::SerialPort;

use crate::{logger::LOGGER, logln};

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

struct SerialLogger(pub SerialPort);

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
