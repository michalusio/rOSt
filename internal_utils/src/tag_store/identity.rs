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
    pub unsafe fn from_value(id: u64) -> Self {
        Self(unsafe { NonZero::new_unchecked(id) })
    }

    /// # Safety
    /// Using this method the caller can create identities which may not exist in the system.
    pub unsafe fn from_ids(device_id: NonZeroU32, internal_id: NonZeroU32) -> Self {
        let combined = ((device_id.get() as u64) << 32) | internal_id.get() as u64;
        unsafe { Self::from_value(combined) }
    }

    pub fn device_id(self) -> NonZeroU32 {
        // # Safety
        // This is safe because identities need to have a non-zero device id
        unsafe { NonZeroU32::new_unchecked((self.0.get() >> 32) as u32) }
    }

    pub fn internal_id(self) -> NonZeroU32 {
        // # Safety
        // This is safe because identities need to have a non-zero internal id
        unsafe { NonZeroU32::new_unchecked(self.0.get() as u32) }
    }

    pub fn as_u64(self) -> NonZeroU64 {
        self.0
    }
}

impl Display for Identity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Identity(Device = {}; Internal = {})",
            self.device_id(),
            self.internal_id()
        )
    }
}
