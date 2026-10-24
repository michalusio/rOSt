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
    ilogln,
};
use tinytga::RawTga;

use crate::vga_device::VGADeviceFactory;

pub fn init_vga(kernel_info: KernelInformation) {
    ilogln!("[   ---{:^15}---   ]", "VGA");
    if let Some(buffer) = kernel_info.framebuffer.as_ref() {
        ilogln!("Width: {} (Stride: {})", buffer.width, buffer.stride);
        ilogln!("Height: {}", buffer.height);
        ilogln!("Bytes per pixel: {}", buffer.bytes_per_pixel);
        ilogln!("Pixel format: {}", buffer.format);
    } else {
        ilogln!("NO FRAME BUFFER FOUND");
        return;
    }
    let mut vga_device = VGADeviceFactory::from_kernel_info(kernel_info);
    ilogln!("VGA device created");
    show_logo(&mut vga_device);

    GPU_DEVICE.call_once(|| Box::new(vga_device));
}

fn show_logo<T: GPUDevice>(device: &mut T) {
    let data = include_bytes!("./assets/rost-logo.tga");
    let logo = RawTga::from_slice(data).unwrap();
    ilogln!("Logo loaded");

    let logo_header = logo.header();

    if let Some(GPUDeviceCapabilityMut::Clearable(clearable)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Clearable)
    {
        clearable.clear(BLACK);
        ilogln!("VGA image cleared");
    } else {
        ilogln!("No clearing capability detected");
    }

    if let Some(GPUDeviceCapabilityMut::Image(imageable)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Image)
    {
        imageable.draw_image(
            (imageable.width() as u16 - logo_header.width) / 2,
            (imageable.height() as u16 - logo_header.height) / 2,
            &logo,
        );
        ilogln!("Logo drawn");
    } else {
        ilogln!("No logo drawing capability detected");
    }
    if let Some(GPUDeviceCapabilityMut::Flush(flush)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Flush)
    {
        flush.flush();
        ilogln!("VGA flushed");
    } else {
        ilogln!("No VGA flushing capability detected");
    }
}
