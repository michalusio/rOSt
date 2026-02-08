use tinytga::RawTga;

use crate::gpu_device::{GPUDevice, Point2D, VGAColor};

pub trait ClearableGPUDevice: GPUDevice {
    fn clear(&mut self, color: VGAColor<u8>);
}

pub trait PlaneGPUDevice: GPUDevice {
    fn draw_point(&mut self, x: u16, y: u16, color: VGAColor<u8>);
    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: VGAColor<u8>);
    fn draw_bezier(
        &mut self,
        p1: Point2D<u16>,
        p2: Point2D<u16>,
        p3: Point2D<u16>,
        p4: Point2D<u16>,
        color: VGAColor<u8>,
    );
}

pub trait ShapeGPUDevice: GPUDevice {
    fn draw_rectangle(&mut self, x: u16, y: u16, width: u16, height: u16, color: VGAColor<u8>);
    fn fill_rectangle(&mut self, x: u16, y: u16, width: u16, height: u16, color: VGAColor<u8>);
}

pub trait TextGPUDevice: GPUDevice {
    fn draw_string(
        &mut self,
        x: u16,
        y: u16,
        color: VGAColor<u8>,
        text: &str,
        reset_x: u16,
    ) -> (u16, u16);
    fn measure_string(&self, x: u16, y: u16, text: &str, reset_x: u16) -> (u16, u16);
}

pub trait ImageGPUDevice: GPUDevice {
    fn draw_image(&mut self, x: u16, y: u16, image: &RawTga);
}
