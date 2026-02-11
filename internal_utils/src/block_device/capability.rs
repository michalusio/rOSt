use crate::block_device::{BootableBlockDevice, PartitionableBlockDevice};

macro_rules! block_device_capabilities {
    ( $( $cap:ident => $tr:ident ),+ $(,)? ) => {
        #[non_exhaustive]
        #[repr(u8)]
        pub enum BlockDeviceCapabilityRequest {
            $( $cap ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum BlockDeviceCapabilityRef<'a> {
            $( $cap(&'a dyn $tr) ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum BlockDeviceCapabilityMut<'a> {
            $( $cap(&'a mut dyn $tr) ),+
        }
    };
}

#[macro_export]
macro_rules! has_block_device_capability {
    (
        $( $rq:ident),+ $(,)?
    ) => {
        fn get_capability(
            &'_ self,
            request: internal_utils::block_device::BlockDeviceCapabilityRequest,
        ) -> Option<internal_utils::block_device::BlockDeviceCapabilityRef<'_>> {
            match request {
                $(
                    internal_utils::block_device::BlockDeviceCapabilityRequest::$rq => {
                        Some(
                            internal_utils::block_device::BlockDeviceCapabilityRef::$rq(self)
                        )
                    }
                )+
                _ => None,
            }
        }

        fn get_capability_mut(
            &'_ mut self,
            request: internal_utils::block_device::BlockDeviceCapabilityRequest,
        ) -> Option<internal_utils::block_device::BlockDeviceCapabilityMut<'_>> {
            match request {
                $(
                    internal_utils::block_device::BlockDeviceCapabilityRequest::$rq => {
                        Some(
                            internal_utils::block_device::BlockDeviceCapabilityMut::$rq(self)
                        )
                    }
                )+
                _ => None,
            }
        }
    };
}

block_device_capabilities!(
    Bootable => BootableBlockDevice,
    Partitionable => PartitionableBlockDevice
);
