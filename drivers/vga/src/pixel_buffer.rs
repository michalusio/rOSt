use core::alloc::Layout;

use alloc::alloc::alloc;

use alloc::slice;
use internal_utils::{
    gpu_device::VGAColor,
    kernel_information::kernel_frame_buffer::{KernelFrameBuffer, PixelFormat},
};

pub trait PixelBuffer: Send {
    /// Places a color on the following buffer index
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>);
    /// Flushes the backbuffer to the visible pixel buffer
    fn flush(&mut self);
}

pub(crate) struct BasePixelBuffer<const P: PixelFormat, const N: usize> {
    frame_pointer: &'static mut [u8],
    back_buffer: &'static mut [u8],
    change_buffer: &'static mut [u8],
}

impl<const P: PixelFormat, const N: usize> BasePixelBuffer<P, N> {
    pub fn new(buffer: &KernelFrameBuffer) -> Self {
        let pixels = buffer.stride * buffer.height;
        let len = buffer.bytes_per_pixel * pixels;
        unsafe {
            let back_buffer = slice::from_raw_parts_mut(
                {
                    let layout = Layout::from_size_align(len, 8).unwrap();
                    alloc(layout)
                },
                len,
            );
            back_buffer.fill(254);
            let change_buffer = slice::from_raw_parts_mut(
                {
                    let layout = Layout::from_size_align(len, 8).unwrap();
                    alloc(layout)
                },
                len,
            );
            Self {
                frame_pointer: slice::from_raw_parts_mut(buffer.buffer.get(), len),
                back_buffer,
                change_buffer,
            }
        }
    }

    fn inner_flush(&mut self) {
        let len = self.change_buffer.len() >> 3;
        let frame_ptr = self.frame_pointer.as_mut_ptr() as *mut u64;
        let buffer_ptr = self.back_buffer.as_mut_ptr() as *mut u64;
        let change_ptr = self.change_buffer.as_mut_ptr() as *mut u64;
        for i in 0..len {
            unsafe {
                let b = change_ptr.add(i).read();
                if b != buffer_ptr.add(i).read() {
                    frame_ptr.add(i).write(b);
                }
            }
        }
        self.back_buffer.copy_from_slice(self.change_buffer);
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::RGB }, 3> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let buffer_index = index * 3;
        if buffer_index >= self.change_buffer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        self.change_buffer[buffer_index] = color.red;
        self.change_buffer[buffer_index + 1] = color.green;
        self.change_buffer[buffer_index + 2] = color.blue;
    }

    fn flush(&mut self) {
        self.inner_flush();
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::RGB }, 4> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let buffer_index = index * 4;
        if buffer_index >= self.change_buffer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        self.change_buffer[buffer_index] = color.red;
        self.change_buffer[buffer_index + 1] = color.green;
        self.change_buffer[buffer_index + 2] = color.blue;
        self.change_buffer[buffer_index + 3] = 255;
    }

    fn flush(&mut self) {
        self.inner_flush();
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::BGR }, 3> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let buffer_index = index * 3;
        if buffer_index >= self.change_buffer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        self.change_buffer[buffer_index + 2] = color.red;
        self.change_buffer[buffer_index + 1] = color.green;
        self.change_buffer[buffer_index] = color.blue;
    }

    fn flush(&mut self) {
        self.inner_flush();
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::BGR }, 4> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let buffer_index = index * 4;
        if buffer_index >= self.change_buffer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        self.change_buffer[buffer_index + 2] = color.red;
        self.change_buffer[buffer_index + 1] = color.green;
        self.change_buffer[buffer_index] = color.blue;
        self.change_buffer[buffer_index + 3] = 255;
    }

    fn flush(&mut self) {
        self.inner_flush();
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::U8 }, 1> {
    #[inline(always)]
    fn put_pixel(&mut self, buffer_index: usize, color: VGAColor<u8>) {
        if buffer_index >= self.change_buffer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        let color_gray = color.to_grayscale() as u8;
        self.change_buffer[buffer_index] = color_gray;
    }

    fn flush(&mut self) {
        self.inner_flush();
    }
}
