//! The sender owns active subscriptions. A separate, publishable receiver
//! factory registers subscriptions through an inbox without exposing that list.
//! The factory uses address-space-local Rust synchronization, just like the
//! current tag store. Isolated processes need a privileged registration proxy
//! and individual SPSC mappings, neither of which is implemented here yet.
//!
//! Messages are pointer-free POD wire values. This restriction does not protect
//! against a peer concurrently modifying shared memory or forging counters.

use core::{
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::vec::Vec;
use bytemuck::Pod;
use x86_64::{
    PhysAddr,
    structures::paging::{PhysFrame, Size4KiB},
};

use crate::{PagePadded, channels::spsc, kernel_information::KERNEL_INFORMATION};

const REGISTRATION_CAPACITY: usize = 64;
const REGISTRATION_READY: u64 = 1;

#[repr(C)]
struct RegistrationEntry {
    state: AtomicU64,
    read: AtomicU64,
    write: AtomicU64,
    buffer: AtomicU64,
}

#[repr(C, align(4096))]
struct RegistrationPage {
    owners: AtomicU64,
    closed: AtomicU64,
    lock: AtomicU64,
    entries: [RegistrationEntry; REGISTRATION_CAPACITY],
}

impl RegistrationPage {
    fn lock(&self) {
        while self
            .lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }

    fn register(&self, frames: spsc::ChannelFrames) -> Result<(), RegistrationError> {
        self.lock();
        let result = if self.closed.load(Ordering::Relaxed) != 0 {
            Err(RegistrationError::Closed)
        } else if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.state.load(Ordering::Relaxed) == 0)
        {
            entry.read.store(frames.read, Ordering::Relaxed);
            entry.write.store(frames.write, Ordering::Relaxed);
            entry.buffer.store(frames.buffer, Ordering::Relaxed);
            entry.state.store(REGISTRATION_READY, Ordering::Release);
            Ok(())
        } else {
            Err(RegistrationError::Full)
        };
        self.unlock();
        result
    }

    fn take(&self) -> Option<spsc::ChannelFrames> {
        self.lock();
        let frames = self.entries.iter().find_map(|entry| {
            if entry.state.swap(0, Ordering::Acquire) == REGISTRATION_READY {
                Some(spsc::ChannelFrames {
                    read: entry.read.load(Ordering::Relaxed),
                    write: entry.write.load(Ordering::Relaxed),
                    buffer: entry.buffer.load(Ordering::Relaxed),
                })
            } else {
                None
            }
        });
        self.unlock();
        frames
    }

    fn retain(&self) {
        self.owners.fetch_add(1, Ordering::Relaxed);
    }

    fn close_and_release(&self) -> bool {
        self.lock();
        self.closed.store(1, Ordering::Release);
        self.unlock();
        self.owners.fetch_sub(1, Ordering::AcqRel) == 1
    }

    fn release(&self) -> bool {
        self.owners.fetch_sub(1, Ordering::AcqRel) == 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistrationFrame {
    pub page: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegistrationError {
    Closed,
    Full,
}

unsafe fn destroy_registration_frame(frame: RegistrationFrame) -> Option<()> {
    let kernel_info = KERNEL_INFORMATION.get()?;
    let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(frame.page))
        .expect("SPMC registration frame address not aligned");
    unsafe { kernel_info.allocator.lock().deallocate_frame(frame) };
    Some(())
}

/// Creates a broadcast sender and a separately publishable receiver factory.
///
/// The factory can be stored in the current tag index. It grants subscription
/// access, not sending access, and does not keep the sender alive.
/// Messages must not contain references or process-local ownership.
///
/// ```no_run,standalone_crate
/// use std::sync::Arc;
/// use internal_utils::channels::spmc;
/// use internal_utils::tag_store::Entity;
/// let (_, mut sender, factory) = spmc::create::<[u8; 32]>().unwrap();
/// assert_eq!(sender.try_send([0; 32]), spmc::SendResult::NoConnections);
/// let entity: Entity = Arc::new(factory);
/// let factory = entity.downcast::<spmc::ReceiverFactory<[u8; 32]>>().unwrap();
/// let receiver = factory.connect().unwrap();
/// ```
///
/// ```compile_fail
/// use internal_utils::channels::spmc;
/// use std::sync::Arc;
/// let sender = spmc::create::<Arc<u64>>();
/// ```
///
/// ```compile_fail
/// use internal_utils::channels::spmc;
/// let sender = spmc::create::<&'static u64>();
/// ```
pub fn create<T: Pod>() -> Option<(RegistrationFrame, Sender<T>, ReceiverFactory<T>)> {
    let kernel_info = KERNEL_INFORMATION.get()?;
    let frame: PhysFrame<Size4KiB> = kernel_info.allocator.lock().allocate_frame()?;
    let page_address = frame.start_address().as_u64();
    let page =
        (kernel_info.physical_memory_offset + page_address) as *mut PagePadded<RegistrationPage>;
    unsafe { page.write_bytes(0, 1) };
    let registrations = unsafe { &*page };
    registrations.owners.store(2, Ordering::Relaxed);
    let registration_frame = RegistrationFrame { page: page_address };

    Some((
        registration_frame,
        Sender {
            fibers: Vec::new(),
            registrations,
            registration_frame,
        },
        ReceiverFactory {
            registrations,
            registration_frame,
            phantom: PhantomData,
        },
    ))
}

/// Attaches to an existing shared registration page.
///
/// # Safety
/// The caller must provide a valid registration frame shared by its producer.
pub unsafe fn attach_registration<T: Pod>(frame: RegistrationFrame) -> Option<ReceiverFactory<T>> {
    let kernel_info = KERNEL_INFORMATION.get()?;
    let page =
        (kernel_info.physical_memory_offset + frame.page) as *const PagePadded<RegistrationPage>;
    let registrations = unsafe { &*page };
    registrations.retain();
    Some(ReceiverFactory {
        registrations,
        registration_frame: frame,
        phantom: PhantomData,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendResult {
    Sent,
    PartiallyFull,
    Full,
    NoConnections,
}

/// Owns the subscriber list in the producer's address space.
pub struct Sender<T: Pod> {
    fibers: Vec<spsc::Sender<'static, T>>,
    registrations: &'static RegistrationPage,
    registration_frame: RegistrationFrame,
}

impl<T: Pod> Sender<T> {
    /// Warning: Broadcast cost is `O(subscribers × size_of(T))`.
    ///
    /// For larger payloads, send validated offsets and lengths into separately
    /// mapped storage, not pointers or `Arc` values.
    /// A `PartiallyFull` result means some subscribers already received the
    /// message; retrying the broadcast can deliver duplicates to them.
    pub fn try_send(&mut self, value: T) -> SendResult {
        let mut any_failed = false;
        let mut any_success = false;

        while let Some(frames) = self.registrations.take() {
            if let Some(fiber) = unsafe { spsc::attach_sender::<T>(frames) } {
                self.fibers.push(fiber);
            } else {
                unsafe {
                    spsc::destroy(frames).expect("SPSC allocator unavailable during cleanup");
                }
            }
        }
        self.fibers.retain(|fiber| !fiber.is_closed());

        self.fibers.retain_mut(|fiber| match fiber.try_send(value) {
            spsc::SendResult::Sent => {
                any_success = true;
                true
            }
            spsc::SendResult::Full => {
                any_failed = true;
                true
            }
            spsc::SendResult::Closed => false,
        });
        match (any_success, any_failed) {
            (false, false) => SendResult::NoConnections, // No fibers in channel
            (true, false) => SendResult::Sent,           // Best case
            (false, true) => SendResult::Full,           // Everything is backed up
            (true, true) => SendResult::PartiallyFull, // Some channels are full, some accepted the message
        }
    }
}

impl<T: Pod> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.registrations.close_and_release() {
            unsafe {
                destroy_registration_frame(self.registration_frame)
                    .expect("SPMC allocator unavailable during cleanup");
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReceiverCreationError {
    FiberAllocationError,
    RegistrationFull,
    Closed,
}

/// A cloneable subscription handle suitable for the current tag index.
///
/// Only the registration inbox is shared; active subscriptions remain private
/// to the sender. This is not a cross-address-space capability: isolated
/// processes must use a privileged proxy rather than mapping this Rust object.
///
/// ```compile_fail
/// use internal_utils::channels::spmc;
/// let (_, _, mut factory) = spmc::create::<[u8; 32]>().unwrap();
/// factory.try_send([0; 32]);
/// ```
///
/// ```no_run,standalone_crate
/// use internal_utils::channels::spmc;
/// let (_, sender, factory) = spmc::create::<[u8; 32]>().unwrap();
/// let published_factory = factory;
/// drop(sender);
/// assert!(matches!(
///     published_factory.connect(),
///     Err(spmc::ReceiverCreationError::Closed)
/// ));
/// ```
pub struct ReceiverFactory<T: Pod> {
    registrations: &'static RegistrationPage,
    registration_frame: RegistrationFrame,
    phantom: PhantomData<T>,
}

impl<T: Pod> Clone for ReceiverFactory<T> {
    fn clone(&self) -> Self {
        self.registrations.retain();
        Self {
            registrations: self.registrations,
            registration_frame: self.registration_frame,
            phantom: PhantomData,
        }
    }
}

impl<T: Pod> Drop for ReceiverFactory<T> {
    fn drop(&mut self) {
        if self.registrations.release() {
            unsafe {
                destroy_registration_frame(self.registration_frame)
                    .expect("SPMC allocator unavailable during cleanup");
            }
        }
    }
}

impl<T: Pod> ReceiverFactory<T> {
    /// Creates a receiver without needing access to the broadcast sender.
    ///
    /// A successful registration participates in sends after the producer next
    /// collects the inbox. Concurrent sends may already have collected it.
    pub fn connect(&self) -> Result<(spsc::ChannelFrames, Receiver<T>), ReceiverCreationError> {
        let (frames, sender, receiver) =
            spsc::create::<T>().ok_or(ReceiverCreationError::FiberAllocationError)?;
        match self.registrations.register(frames) {
            Ok(()) => Ok((frames, Receiver(receiver))),
            Err(error) => {
                drop(receiver);
                drop(sender);
                unsafe {
                    spsc::destroy(frames).expect("SPSC allocator unavailable during cleanup");
                }
                Err(match error {
                    RegistrationError::Closed => ReceiverCreationError::Closed,
                    RegistrationError::Full => ReceiverCreationError::RegistrationFull,
                })
            }
        }
    }
}

pub enum ReceiveResult<T> {
    Received(T),
    Empty,
    Closed,
}

/// A subscriber's SPSC endpoint, with no access to broadcast bookkeeping.
///
/// ```compile_fail
/// use internal_utils::channels::spmc;
/// fn subscribe_again(mut receiver: spmc::Receiver<[u8; 32]>) {
///     receiver.connect();
/// }
/// ```
pub struct Receiver<T: Pod>(spsc::Receiver<'static, T>);

impl<T: Pod> Receiver<T> {
    /// Attaches to an existing subscriber channel.
    ///
    /// # Safety
    /// The caller must provide valid channel frames and create at most one receiver.
    pub unsafe fn attach(frames: spsc::ChannelFrames) -> Option<Self> {
        Some(Self(unsafe { spsc::attach_receiver(frames)? }))
    }

    pub fn try_receive(&mut self) -> ReceiveResult<T> {
        match self.0.try_receive() {
            spsc::ReceiveResult::Received(result) => ReceiveResult::Received(result),
            spsc::ReceiveResult::Closed => ReceiveResult::Closed,
            spsc::ReceiveResult::Empty => ReceiveResult::Empty,
        }
    }
}
