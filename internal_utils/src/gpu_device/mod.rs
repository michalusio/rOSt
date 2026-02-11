mod vga_color;

use alloc::boxed::Box;
pub use vga_color::{
    BLACK, BLUE, BSOD_BLUE, CHARLOTTE, CLAY, GREEN, RED, TRANSPARENT, VGAColor, WHITE,
};

mod point_2d;
pub use point_2d::Point2D;

mod capability;
pub use capability::{GPUDeviceCapabilityMut, GPUDeviceCapabilityRef, GPUDeviceCapabilityRequest};

mod traits;
use crate::{capabilities::Device, structures::OnceMutex};
pub use traits::{
    ClearableGPUDevice, ImageGPUDevice, PlaneGPUDevice, ShapeGPUDevice, TextGPUDevice,
};

pub trait GPUDevice: Device {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get_capability(
        &'_ self,
        request: GPUDeviceCapabilityRequest,
    ) -> Option<GPUDeviceCapabilityRef<'_>>;
    fn get_capability_mut(
        &'_ mut self,
        request: GPUDeviceCapabilityRequest,
    ) -> Option<GPUDeviceCapabilityMut<'_>>;
}

pub static GPU_DEVICE: OnceMutex<Box<dyn GPUDevice>> = OnceMutex::new();
