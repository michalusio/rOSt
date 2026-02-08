use x86_64::{
    instructions::port::{PortGeneric, PortReadAccess, PortWriteAccess},
    structures::port::{PortRead, PortWrite},
};

pub trait PortExtRead<T: PortRead> {
    /// Reads a given number of values from the port and into a buffer.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn read_to_buffer(&mut self, buffer: &mut [T]);
}

pub trait PortExtWrite<T: PortWrite> {
    /// Reads a given number of values from the port and into a buffer.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn write_from_buffer(&mut self, buffer: &[T]);
}

impl<T: PortRead, A: PortReadAccess> PortExtRead<T> for PortGeneric<T, A> {
    #[inline]
    unsafe fn read_to_buffer(self: &mut PortGeneric<T, A>, buffer: &mut [T]) {
        for data in buffer {
            *data = unsafe { self.read() };
        }
    }
}

impl<A: PortReadAccess> PortExtRead<u8> for PortGeneric<u16, A> {
    #[inline]
    unsafe fn read_to_buffer(self: &mut PortGeneric<u16, A>, buffer: &mut [u8]) {
        let mut index = 0;
        while index < buffer.len() {
            let value = unsafe { self.read() };
            buffer[index] = value as u8;
            index += 1;
            buffer[index] = (value >> 8) as u8;
            index += 1;
        }
    }
}

impl<A: PortWriteAccess> PortExtWrite<u8> for PortGeneric<u16, A> {
    /// Writes a buffer to the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    unsafe fn write_from_buffer(self: &mut PortGeneric<u16, A>, buffer: &[u8]) {
        let mut index = 0;
        while index < buffer.len() {
            let mut value = buffer[index] as u16;
            index += 1;
            value |= (buffer[index] as u16) << 8;
            unsafe { self.write(value) };
            index += 1;
        }
    }
}

impl<T: PortWrite + Copy, A: PortWriteAccess> PortExtWrite<T> for PortGeneric<T, A> {
    /// Writes a buffer to the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    unsafe fn write_from_buffer(self: &mut PortGeneric<T, A>, buffer: &[T]) {
        for data in buffer {
            unsafe { self.write(*data) };
            x86_64::instructions::nop(); // We need a tiny delay when batch-writing to IO ports
            x86_64::instructions::nop();
        }
    }
}
