#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
extern crate alloc;

mod pixel_buffer;
mod static_stack;
pub mod vga_core;
pub mod vga_device;

use alloc::boxed::Box;
use internal_utils::{
    gpu_device::{
        BLACK, GPU_DEVICE, GPUDevice, GPUDeviceCapabilityMut, GPUDeviceCapabilityRequest,
    },
    kernel_information::KernelInformation,
    logln,
};
use tinytga::RawTga;

use crate::vga_device::VGADeviceFactory;

pub fn init_vga(kernel_info: KernelInformation) {
    logln!("[   ---{:^15}---   ]", "VGA");
    if let Some(buffer) = kernel_info.framebuffer.as_ref() {
        logln!("Width: {} (Stride: {})", buffer.width, buffer.stride);
        logln!("Height: {}", buffer.height);
        logln!("Bytes per pixel: {}", buffer.bytes_per_pixel);
        logln!("Pixel format: {}", buffer.format);
    } else {
        logln!("NO FRAME BUFFER FOUND");
        return;
    }
    let mut vga_device = VGADeviceFactory::from_kernel_info(kernel_info);
    logln!("VGA device created");
    show_logo(&mut vga_device);

    GPU_DEVICE.call_once(|| Box::new(vga_device));
}

fn show_logo<T: GPUDevice>(device: &mut T) {
    let data = include_bytes!("./assets/rost-logo.tga");
    let logo = RawTga::from_slice(data).unwrap();
    logln!("Logo loaded");

    let logo_header = logo.header();

    if let Some(GPUDeviceCapabilityMut::Clearable(clearable)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Clearable)
    {
        clearable.clear(BLACK);
        logln!("VGA image cleared");
    } else {
        logln!("No clearing capability detected");
    }

    if let Some(GPUDeviceCapabilityMut::Image(imageable)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Image)
    {
        imageable.draw_image(
            (imageable.width() as u16 - logo_header.width) / 2,
            (imageable.height() as u16 - logo_header.height) / 2,
            &logo,
        );
        logln!("Logo drawn");
    } else {
        logln!("No logo drawing capability detected");
    }
}
