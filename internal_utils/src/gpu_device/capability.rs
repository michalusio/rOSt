use crate::gpu_device::{
    ClearableGPUDevice, ImageGPUDevice, PlaneGPUDevice, ShapeGPUDevice, TextGPUDevice,
};

macro_rules! gpu_device_capabilities {
    ( $( $cap:ident => $trait:ident ),+ $(,)? ) => {
        #[non_exhaustive]
        #[repr(u8)]
        pub enum GPUDeviceCapabilityRequest {
            $( $cap ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum GPUDeviceCapabilityRef<'a> {
            $( $cap(&'a dyn $trait) ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum GPUDeviceCapabilityMut<'a> {
            $( $cap(&'a mut dyn $trait) ),+
        }
    };
}

#[macro_export]
macro_rules! has_gpu_device_capability {
    ( $( $cap:ident ),+ $(,)? ) => {
        fn get_capability(
            &'_ self,
            request: GPUDeviceCapabilityRequest,
        ) -> Option<GPUDeviceCapabilityRef<'_>> {
            match request {
                $(
                    GPUDeviceCapabilityRequest::$cap => {
                        Some(GPUDeviceCapabilityRef::$cap(self))
                    }
                )+
                _ => None,
            }
        }

        fn get_capability_mut(
            &'_ mut self,
            request: GPUDeviceCapabilityRequest,
        ) -> Option<GPUDeviceCapabilityMut<'_>> {
            match request {
                $(
                    GPUDeviceCapabilityRequest::$cap => {
                        Some(GPUDeviceCapabilityMut::$cap(self))
                    }
                )+
                _ => None,
            }
        }
    };
}

gpu_device_capabilities!(
    Clearable => ClearableGPUDevice,
    Plane => PlaneGPUDevice,
    Shape => ShapeGPUDevice,
    Text => TextGPUDevice,
    Image => ImageGPUDevice
);
