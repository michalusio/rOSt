use core::{fmt::Display, marker::ConstParamTy_};

use bootloader_api::info::FrameBuffer;

use crate::structures::Permanent;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct KernelFrameBuffer {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub bytes_per_pixel: usize,
    pub stride: usize,
    pub buffer: Permanent<*mut u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PixelFormat {
    /// One byte red, then one byte green, then one byte blue.
    ///
    /// Length might be larger than 3, check [`bytes_per_pixel`][FrameBufferInfo::bytes_per_pixel]
    /// for this.
    RGB,
    /// One byte blue, then one byte green, then one byte red.
    ///
    /// Length might be larger than 3, check [`bytes_per_pixel`][FrameBufferInfo::bytes_per_pixel]
    /// for this.
    BGR,
    /// A single byte, representing the grayscale value.
    ///
    /// Length might be larger than 1, check [`bytes_per_pixel`][FrameBufferInfo::bytes_per_pixel]
    /// for this.
    U8,
}

impl Display for PixelFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PixelFormat::RGB => write!(f, "RGB"),
            PixelFormat::BGR => write!(f, "BGR"),
            PixelFormat::U8 => write!(f, "U8"),
        }
    }
}

impl ConstParamTy_ for PixelFormat {}

impl KernelFrameBuffer {
    pub(crate) fn new(buffer: &FrameBuffer) -> KernelFrameBuffer {
        let info = buffer.info();
        KernelFrameBuffer {
            width: info.width,
            height: info.height,
            format: match info.pixel_format {
                bootloader_api::info::PixelFormat::Rgb => PixelFormat::RGB,
                bootloader_api::info::PixelFormat::Bgr => PixelFormat::BGR,
                bootloader_api::info::PixelFormat::U8 => PixelFormat::U8,
                _ => panic!("Unsupported pixel format: {:?}", info.pixel_format),
            },
            bytes_per_pixel: info.bytes_per_pixel,
            stride: info.stride,
            buffer: unsafe { Permanent::new(buffer.buffer().as_ptr() as *mut u8) },
        }
    }
}
