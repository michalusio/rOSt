use spin::{Mutex, MutexGuard, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[repr(transparent)]
#[derive(Default)]
pub struct OnceLock<T>(Once<RwLock<T>>);

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        OnceLock(Once::new())
    }

    pub fn read(&'_ self) -> Option<RwLockReadGuard<'_, T>> {
        self.0.get().map(|lock| lock.read())
    }

    pub fn write(&'_ self) -> Option<RwLockWriteGuard<'_, T>> {
        self.0.get().map(|lock| lock.write())
    }

    pub fn call_once<F: FnOnce() -> T>(&self, f: F) {
        self.0.call_once(|| RwLock::new(f()));
    }
}

#[repr(transparent)]
#[derive(Default)]
pub struct OnceMutex<T>(Once<Mutex<T>>);

impl<T> OnceMutex<T> {
    pub const fn new() -> Self {
        OnceMutex(Once::new())
    }

    pub fn lock(&'_ self) -> Option<MutexGuard<'_, T>> {
        self.0.get().map(|lock| lock.lock())
    }

    pub fn call_once<F: FnOnce() -> T>(&self, f: F) {
        self.0.call_once(|| Mutex::new(f()));
    }
}

#[repr(transparent)]
#[derive(Default)]
pub struct OnceClone<T: Clone>(Once<T>);

impl<T: Clone> OnceClone<T> {
    pub const fn new() -> Self {
        OnceClone(Once::new())
    }

    pub fn get(&self) -> Option<T> {
        self.0.get().cloned()
    }

    pub fn call_once<F: FnOnce() -> T>(&self, f: F) {
        self.0.call_once(f);
    }
}
