use crate::block_device::{BootableBlockDevice, PartitionableBlockDevice};

macro_rules! block_device_capabilities {
    ( $( $cap:ident => $trait:ident ),+ $(,)? ) => {
        #[non_exhaustive]
        #[repr(u8)]
        pub enum BlockDeviceCapabilityRequest {
            $( $cap ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum BlockDeviceCapabilityRef<'a> {
            $( $cap(&'a dyn $trait) ),+
        }

        #[non_exhaustive]
        #[repr(C)]
        pub enum BlockDeviceCapabilityMut<'a> {
            $( $cap(&'a mut dyn $trait) ),+
        }
    };
}

#[macro_export]
macro_rules! has_block_device_capability {
    ( $( $cap:ident ),+ $(,)? ) => {
        fn get_capability(
            &'_ self,
            request: BlockDeviceCapabilityRequest,
        ) -> Option<BlockDeviceCapabilityRef<'_>> {
            match request {
                $(
                    BlockDeviceCapabilityRequest::$cap => {
                        Some(BlockDeviceCapabilityRef::$cap(self))
                    }
                )+
                _ => None,
            }
        }

        fn get_capability_mut(
            &'_ mut self,
            request: BlockDeviceCapabilityRequest,
        ) -> Option<BlockDeviceCapabilityMut<'_>> {
            match request {
                $(
                    BlockDeviceCapabilityRequest::$cap => {
                        Some(BlockDeviceCapabilityMut::$cap(self))
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
