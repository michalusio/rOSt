use internal_utils::{
    div_255_fast, gpu_device::VGAColor, kernel_information::kernel_frame_buffer::PixelFormat,
};

pub trait PixelBuffer: Send {
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>);
}

pub(crate) struct BasePixelBuffer<const P: PixelFormat> {
    pub frame_pointer: &'static mut [u8],
    pub bytes_per_pixel: usize,
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::RGB }> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let index = index * self.bytes_per_pixel;
        if index >= self.frame_pointer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        let frame_color = VGAColor {
            red: self.frame_pointer[index],
            green: self.frame_pointer[index + 1],
            blue: self.frame_pointer[index + 2],
            alpha: if self.bytes_per_pixel > 3 {
                self.frame_pointer[index + 3]
            } else {
                255
            },
        };
        let result_color = VGAColor::interpolate(frame_color, color, color.alpha);
        self.frame_pointer[index] = result_color.red;
        self.frame_pointer[index + 1] = result_color.green;
        self.frame_pointer[index + 2] = result_color.blue;
        if self.bytes_per_pixel > 3 {
            self.frame_pointer[index + 3] = result_color.alpha;
        }
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::BGR }> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let index = index * self.bytes_per_pixel;
        if index >= self.frame_pointer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        let frame_color = VGAColor {
            red: self.frame_pointer[index + 2],
            green: self.frame_pointer[index + 1],
            blue: self.frame_pointer[index],
            alpha: if self.bytes_per_pixel > 3 {
                self.frame_pointer[index + 3]
            } else {
                255
            },
        };
        let result_color = VGAColor::interpolate(frame_color, color, color.alpha);
        self.frame_pointer[index + 2] = result_color.red;
        self.frame_pointer[index + 1] = result_color.green;
        self.frame_pointer[index] = result_color.blue;
        if self.bytes_per_pixel > 3 {
            self.frame_pointer[index + 3] = result_color.alpha;
        }
    }
}

impl PixelBuffer for BasePixelBuffer<{ PixelFormat::U8 }> {
    #[inline(always)]
    fn put_pixel(&mut self, index: usize, color: VGAColor<u8>) {
        let index = index * self.bytes_per_pixel;
        if index >= self.frame_pointer.len() {
            panic!("Tried drawing outside the frame buffer!");
        }
        let gray = self.frame_pointer[index] as u16;
        let color_gray = color.to_grayscale() as u16;
        let alpha = color.alpha as u16;
        let alpha1 = 255 - alpha;
        self.frame_pointer[index] = div_255_fast(gray * alpha1 + color_gray * alpha);
    }
}
