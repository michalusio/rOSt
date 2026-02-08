/// Allows for marking a thing as permanent, which allows sending it between threads
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Permanent<T>(T);

impl<T> Permanent<T> {
    ///# Safety
    /// By creating a Permanent value you have to guarantee that the value is actually permanently available between threads and processors
    pub unsafe fn new(value: T) -> Self {
        Permanent(value)
    }

    pub fn get(self) -> T {
        self.0
    }
}

unsafe impl<T> Sync for Permanent<T> {}
unsafe impl<T> Send for Permanent<T> {}
