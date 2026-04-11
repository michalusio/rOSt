use core::{
    fmt::Display,
    num::{NonZero, NonZeroU32, NonZeroU64},
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identity(NonZeroU64);

impl Identity {
    /// # Safety
    /// Using this method the caller can create identities which may not exist in the system.
    pub const unsafe fn from_ids(device_id: NonZeroU32, internal_id: NonZeroU32) -> Self {
        let combined = ((device_id.get() as u64) << 32) | internal_id.get() as u64;
        unsafe { Self(NonZero::new_unchecked(combined)) }
    }

    pub const fn device_id(self) -> NonZeroU32 {
        // # Safety
        // This is safe because identities need to have a non-zero device id
        unsafe { NonZeroU32::new_unchecked((self.0.get() >> 32) as u32) }
    }

    pub const fn internal_id(self) -> NonZeroU32 {
        // # Safety
        // This is safe because identities need to have a non-zero internal id
        unsafe { NonZeroU32::new_unchecked(self.0.get() as u32) }
    }

    pub const fn as_u64(self) -> NonZeroU64 {
        self.0
    }
}

impl Display for Identity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Identity(Device = {:#x}; Internal = {:#x})",
            self.device_id(),
            self.internal_id()
        )
    }
}

/// The device Id of the TBES system itself.
pub const TBES_DEVICE_ID: NonZeroU32 = NonZeroU32::new(1).unwrap();

/// The identity of the tag which indexes entities which are tags themselves
pub const TAG_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(1).unwrap()) };

/// The identity of the tag which indexes the timestamp of each entity
pub const TIMESTAMP_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(2).unwrap()) };

/// The identity of the tag which indexes the owner of each entity
pub const OWNER_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(3).unwrap()) };

/// The identity of the tag which indexes entities which are users themselves
pub const USER_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(4).unwrap()) };

/// The identity of the tag which indexes entities which are processes themselves
pub const PROCESS_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(5).unwrap()) };

/// The identity of the tag which indexes entities which are channels themselves
pub const CHANNEL_TAG_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(6).unwrap()) };

/// The identity of the Kernel user.
pub const KERNEL_IDENTITY: Identity =
    unsafe { Identity::from_ids(TBES_DEVICE_ID, NonZeroU32::new(1024).unwrap()) };
