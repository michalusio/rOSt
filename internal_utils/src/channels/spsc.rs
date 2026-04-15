use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
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

const fn get_capacity<T>(size_hint: ChannelSizeHint) -> usize {
    match (size_hint, size_of::<T>()) {
        (ChannelSizeHint::Small, ..=8) => 512,
        (ChannelSizeHint::Small, 9..=32) => 256,
        (ChannelSizeHint::Small, 33..=128) => 128,
        (ChannelSizeHint::Large, ..=8) => 2048,
        (ChannelSizeHint::Large, 9..=32) => 1024,
        (ChannelSizeHint::Large, 33..=128) => 512,
        _ => core::panic!("Cannot create channel for structs with size over 128 bytes"),
    }
}

/// Creates a new SPSC channel, returning a sender and a receiver of it
pub fn create<T>(size_hint: ChannelSizeHint) -> (Sender<T>, Receiver<T>) {
    let capacity = get_capacity::<T>(size_hint);
    let channel = Arc::new(SPSC {
        read: CachePadded::new(AtomicUsize::new(0)),
        write: CachePadded::new(AtomicUsize::new(0)),
        capacity,
        buffer: (0..capacity)
            .map(|_| Slot(UnsafeCell::new(MaybeUninit::uninit())))
            .collect(),
    });
    (Sender(channel.clone()), Receiver(channel))
}

/// Creates a new SPSC channel
pub(crate) fn new<T>(size_hint: ChannelSizeHint) -> Arc<SPSC<T>> {
    let capacity = get_capacity::<T>(size_hint);

    Arc::new(SPSC {
        read: CachePadded::new(AtomicUsize::new(0)),
        write: CachePadded::new(AtomicUsize::new(0)),
        capacity,
        buffer: (0..capacity)
            .map(|_| Slot(UnsafeCell::new(MaybeUninit::uninit())))
            .collect(),
    })
}

impl<T> SPSC<T> {
    pub fn try_receive(&self) -> ReceiveResult<T> {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);

        if read == write {
            ReceiveResult::Empty
        } else {
            let index = read & (self.capacity - 1);
            let slot = &self.buffer[index];
            let result = unsafe { slot.0.as_ref_unchecked().assume_init_read() };
            self.read.store(read.wrapping_add(1), Ordering::Relaxed);
            ReceiveResult::Received(result)
        }
    }

    pub fn try_send(&self, value: T) -> SendResult {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);

        if write.wrapping_sub(read) == self.capacity {
            SendResult::Full
        } else {
            let index = write & (self.capacity - 1);
            let slot = &self.buffer[index];
            unsafe {
                slot.0.as_mut_unchecked().write(value);
            }
            self.write.store(write.wrapping_add(1), Ordering::Release);
            SendResult::Sent
        }
    }
}

impl<T> Drop for SPSC<T> {
    fn drop(&mut self) {
        // Basically we retrieve all the items in the channel and drop them
        while let ReceiveResult::Received(v) = self.try_receive() {
            drop(v);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendResult {
    Sent,
    Full,
}

pub struct Sender<T>(Arc<SPSC<T>>);

impl<T> Sender<T> {
    pub fn try_send(&self, value: T) -> SendResult {
        self.0.try_send(value)
    }
}

pub enum ReceiveResult<T> {
    Received(T),
    Empty,
}

pub struct Receiver<T>(Arc<SPSC<T>>);

impl<T> Receiver<T> {
    pub fn try_receive(&self) -> ReceiveResult<T> {
        self.0.try_receive()
    }
}
