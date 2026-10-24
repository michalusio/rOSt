use core::ops::{Deref, DerefMut};

/// Pads and aligns a value to the length of a memory page.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
#[cfg_attr(any(target_arch = "x86_64"), repr(align(4096)))]
pub struct PagePadded<T> {
    value: T,
}

impl<T> PagePadded<T> {
    /// Pads and aligns a value to the length of a memory page.
    pub fn new(value: T) -> PagePadded<T> {
        PagePadded::<T> { value }
    }
}

impl<T> Deref for PagePadded<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for PagePadded<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
