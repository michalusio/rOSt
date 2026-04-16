use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, sync::Arc};

use crate::channels::{CachePadded, ChannelSizeHint};

/// One slot in the SPSC channel
#[repr(transparent)]
struct Slot<T>(UnsafeCell<MaybeUninit<T>>);

/// The internals of an SPSC channel
pub struct SPSC<T> {
    read: CachePadded<AtomicUsize>,
    write: CachePadded<AtomicUsize>,
    capacity: usize,
    buffer: Box<[Slot<T>]>,
}

const fn get_capacity<T>(size_hint: ChannelSizeHint) -> NonZeroUsize {
    let size = size_of::<T>();
    let budget_bytes = match size_hint {
        ChannelSizeHint::Small => 4096,
        ChannelSizeHint::Large => 2 * 1024 * 1024,
    };
    let capacity = if let Some(result) = usize::checked_div(budget_bytes, size) {
        result
    } else {
        budget_bytes
    };

    if capacity > 0 {
        NonZeroUsize::new(capacity).unwrap()
    } else {
        NonZeroUsize::new(1).unwrap()
    }
}

/// Creates a new SPSC channel, returning a sender and a receiver of it
pub fn create<T>(size_hint: ChannelSizeHint) -> (Sender<T>, Receiver<T>) {
    let channel = new(size_hint);
    (Sender(channel.clone()), Receiver(channel))
}

/// Creates a new SPSC channel
pub(crate) fn new<T>(size_hint: ChannelSizeHint) -> Arc<SPSC<T>> {
    let capacity = get_capacity::<T>(size_hint);

    Arc::new(SPSC {
        read: CachePadded::new(AtomicUsize::new(0)),
        write: CachePadded::new(AtomicUsize::new(0)),
        capacity: capacity.into(),
        buffer: (0..capacity.into())
            .map(|_| Slot(UnsafeCell::new(MaybeUninit::uninit())))
            .collect(),
    })
}

impl<T> SPSC<T> {
    pub(crate) fn try_receive(&self) -> ReceiveResultInner<T> {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);

        if read == write {
            ReceiveResultInner::Empty
        } else {
            let index = read & (self.capacity - 1);
            let slot = &self.buffer[index];
            let result = unsafe { slot.0.as_ref_unchecked().assume_init_read() };
            self.read.store(read.wrapping_add(1), Ordering::Relaxed);
            ReceiveResultInner::Received(result)
        }
    }

    pub(crate) fn try_send(&self, value: T) -> SendResultInner {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);

        if write.wrapping_sub(read) == self.capacity {
            SendResultInner::Full
        } else {
            let index = write & (self.capacity - 1);
            let slot = &self.buffer[index];
            unsafe {
                slot.0.as_mut_unchecked().write(value);
            }
            self.write.store(write.wrapping_add(1), Ordering::Release);
            SendResultInner::Sent
        }
    }
}

impl<T> Drop for SPSC<T> {
    fn drop(&mut self) {
        // Basically we retrieve all the items in the channel and drop them
        while let ReceiveResultInner::Received(v) = self.try_receive() {
            drop(v);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SendResultInner {
    Sent,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendResult {
    Sent,
    Full,
    Closed,
}

impl From<SendResultInner> for SendResult {
    fn from(value: SendResultInner) -> Self {
        match value {
            SendResultInner::Sent => SendResult::Sent,
            SendResultInner::Full => SendResult::Full,
        }
    }
}

pub struct Sender<T>(Arc<SPSC<T>>);

impl<T> Sender<T> {
    pub fn try_send(&self, value: T) -> SendResult {
        if Arc::strong_count(&self.0) == 1 {
            SendResult::Closed
        } else {
            self.0.try_send(value).into()
        }
    }
}

pub(crate) enum ReceiveResultInner<T> {
    Received(T),
    Empty,
}

pub enum ReceiveResult<T> {
    Received(T),
    Empty,
    Closed,
}

impl<T> From<ReceiveResultInner<T>> for ReceiveResult<T> {
    fn from(value: ReceiveResultInner<T>) -> Self {
        match value {
            ReceiveResultInner::Received(v) => ReceiveResult::Received(v),
            ReceiveResultInner::Empty => ReceiveResult::Empty,
        }
    }
}

pub struct Receiver<T>(Arc<SPSC<T>>);

impl<T> Receiver<T> {
    pub fn try_receive(&self) -> ReceiveResult<T> {
        if Arc::strong_count(&self.0) == 1 {
            ReceiveResult::Closed
        } else {
            self.0.try_receive().into()
        }
    }
}
