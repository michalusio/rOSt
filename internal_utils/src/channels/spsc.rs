use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::slice_from_raw_parts,
    sync::atomic::{AtomicUsize, Ordering},
};

use bytemuck::Pod;
use x86_64::{
    PhysAddr,
    structures::paging::{PhysFrame, Size4KiB},
};

use crate::{PagePadded, kernel_information::KERNEL_INFORMATION, prev_power_of_two};

/// One slot in the SPSC channel
#[repr(transparent)]
struct Slot<T: Pod>(UnsafeCell<MaybeUninit<T>>);

// The SPSC channel is composed of two sides - the read side and the write side
// The read side is modifiable by the consumer and readable by the producer.
// The write side, in reverse, is modifiable by the producer and readable by the consumer.
// The buffer of the channel is on the write side, so the producer can write to it and the consumer read from it.

const fn buffer<T: Pod>(buffer: &PagePadded<()>) -> &[Slot<T>] {
    unsafe {
        let buffer_ptr: *const Slot<T> = (buffer as *const PagePadded<()>).cast::<Slot<T>>();
        &*slice_from_raw_parts(buffer_ptr, capacity::<T>().unwrap())
    }
}

const fn capacity<T: Pod>() -> Option<usize> {
    let size = size_of::<T>() as u64;
    let budget_bytes = 4096;
    let capacity = prev_power_of_two(if let Some(result) = u64::checked_div(budget_bytes, size) {
        result
    } else {
        budget_bytes
    });

    if capacity > 0 {
        Some(capacity as usize)
    } else {
        None
    }
}

/// Physical frames backing an SPSC channel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelFrames {
    pub read: u64,
    pub write: u64,
    pub buffer: u64,
}

/// Creates a new SPSC channel, returning its frames, sender, and receiver.
pub fn create<T: Pod>() -> Option<(ChannelFrames, Sender<'static, T>, Receiver<'static, T>)> {
    capacity::<T>()?;
    let kernel_info = KERNEL_INFORMATION.get()?;
    let allocator = kernel_info.allocator;
    let mut allocator_lock = allocator.lock();

    let read_page: PhysFrame<Size4KiB> = allocator_lock.allocate_frame()?;
    let write_page: PhysFrame<Size4KiB> = if let Some(frame) = allocator_lock.allocate_frame() {
        Some(frame)
    } else {
        unsafe {
            allocator_lock.deallocate_frame(read_page);
        }
        None
    }?;
    let buffer_page: PhysFrame<Size4KiB> = if let Some(frame) = allocator_lock.allocate_frame() {
        Some(frame)
    } else {
        unsafe {
            allocator_lock.deallocate_frame(read_page);
            allocator_lock.deallocate_frame(write_page);
        }
        None
    }?;

    let frames = ChannelFrames {
        read: read_page.start_address().as_u64(),
        write: write_page.start_address().as_u64(),
        buffer: buffer_page.start_address().as_u64(),
    };

    let read_side =
        (kernel_info.physical_memory_offset + frames.read) as *mut PagePadded<AtomicUsize>;
    let write_side =
        (kernel_info.physical_memory_offset + frames.write) as *mut PagePadded<AtomicUsize>;
    let buffer_side = (kernel_info.physical_memory_offset + frames.buffer) as *mut PagePadded<()>;

    // Zero out the pages
    // TODO: Move zeroing of the pages to the physical memory allocator
    unsafe {
        read_side.write_bytes(0, 1);
        write_side.write_bytes(0, 1);
        buffer_side.write_bytes(0, 1);
    }

    let (sender, receiver) = unsafe { attach::<T>(frames)? };

    // TODO: Perform page mapping between processes so:
    // The producer has RW access to the write page and R to the read page
    // The consumer has R access to the write page and RW to the read page
    Some((frames, sender, receiver))
}

/// Attaches local endpoints to existing channel frames.
///
/// # Safety
/// The caller must provide valid channel frames and create at most one sender
/// and one receiver endpoint for them.
unsafe fn mapped_frames(
    frames: ChannelFrames,
) -> Option<(
    &'static PagePadded<AtomicUsize>,
    &'static PagePadded<AtomicUsize>,
    &'static PagePadded<()>,
)> {
    let kernel_info = KERNEL_INFORMATION.get()?;
    let read_side =
        (kernel_info.physical_memory_offset + frames.read) as *mut PagePadded<AtomicUsize>;
    let write_side =
        (kernel_info.physical_memory_offset + frames.write) as *mut PagePadded<AtomicUsize>;
    let buffer_side = (kernel_info.physical_memory_offset + frames.buffer) as *mut PagePadded<()>;
    let read = unsafe { &*read_side };
    let write = unsafe { &*write_side };
    let buffer = unsafe { &*buffer_side };
    Some((read, write, buffer))
}

/// Attaches local endpoints to existing channel frames.
///
/// # Safety
/// The caller must provide valid channel frames and create at most one sender
/// and one receiver endpoint for them.
pub unsafe fn attach<T: Pod>(
    frames: ChannelFrames,
) -> Option<(Sender<'static, T>, Receiver<'static, T>)> {
    let (read, write, buffer) = unsafe { mapped_frames(frames)? };
    Some((
        Sender {
            phantom: PhantomData,
            read,
            write,
            buffer,
        },
        Receiver {
            phantom: PhantomData,
            read,
            write,
            buffer,
        },
    ))
}

/// Attaches a sender to existing channel frames.
///
/// # Safety
/// The caller must provide valid channel frames and create at most one sender.
pub unsafe fn attach_sender<T: Pod>(frames: ChannelFrames) -> Option<Sender<'static, T>> {
    let (read, write, buffer) = unsafe { mapped_frames(frames)? };
    Some(Sender {
        phantom: PhantomData,
        read,
        write,
        buffer,
    })
}

/// Attaches a receiver to existing channel frames.
///
/// # Safety
/// The caller must provide valid channel frames and create at most one receiver.
pub unsafe fn attach_receiver<T: Pod>(frames: ChannelFrames) -> Option<Receiver<'static, T>> {
    let (read, write, buffer) = unsafe { mapped_frames(frames)? };
    Some(Receiver {
        phantom: PhantomData,
        read,
        write,
        buffer,
    })
}

/// Releases channel frames after both endpoints become unusable.
///
/// # Safety
/// The caller must own `frames` and guarantee that no endpoint accesses them.
pub unsafe fn destroy(frames: ChannelFrames) -> Option<()> {
    let kernel_info = KERNEL_INFORMATION.get()?;
    let mut allocator_lock = kernel_info.allocator.lock();
    for address in [frames.read, frames.write, frames.buffer] {
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(address))
            .expect("SPSC frame address not aligned");
        unsafe { allocator_lock.deallocate_frame(frame) };
    }
    Some(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendResult {
    Sent,
    Full,
    Closed,
}

pub struct Sender<'a, T: Pod> {
    read: &'a PagePadded<AtomicUsize>,
    write: &'a PagePadded<AtomicUsize>,
    buffer: &'a PagePadded<()>,
    phantom: PhantomData<T>,
}

impl<'a, T: Pod> Sender<'a, T> {
    pub fn try_send(&mut self, value: T) -> SendResult {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        let capacity = capacity::<T>().unwrap();

        // On x86_64, at 1 billion messages/second it would take 584 years to break
        if read == usize::MAX || write == usize::MAX {
            SendResult::Closed
        } else if write.wrapping_sub(read) == capacity {
            SendResult::Full
        } else {
            let index = write & (capacity - 1);
            let slot = &buffer(self.buffer)[index];
            unsafe {
                slot.0.as_mut_unchecked().write(value);
            }
            self.write.store(write.wrapping_add(1), Ordering::Release);
            SendResult::Sent
        }
    }

    pub fn is_closed(&self) -> bool {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        // On x86_64, at 1 billion messages/second it would take 584 years to break
        read == usize::MAX || write == usize::MAX
    }

    pub fn count(&self) -> usize {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        write - read
    }
}

impl<'a, T: Pod> Drop for Sender<'a, T> {
    fn drop(&mut self) {
        self.write.store(usize::MAX, Ordering::Relaxed);
    }
}

pub enum ReceiveResult<T> {
    Received(T),
    Empty,
    Closed,
}

pub struct Receiver<'a, T: Pod> {
    read: &'a PagePadded<AtomicUsize>,
    write: &'a PagePadded<AtomicUsize>,
    buffer: &'a PagePadded<()>,
    phantom: PhantomData<T>,
}

impl<'a, T: Pod> Receiver<'a, T> {
    pub fn try_receive(&mut self) -> ReceiveResult<T> {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);

        // On x86_64, at 1 billion messages/second it would take 584 years to break
        if read == usize::MAX || write == usize::MAX {
            ReceiveResult::Closed
        } else if read == write {
            ReceiveResult::Empty
        } else {
            let capacity = capacity::<T>().unwrap();
            let index = read & (capacity - 1);
            let slot = &buffer(self.buffer)[index];
            let result = unsafe { slot.0.as_ref_unchecked().assume_init_read() };
            self.read.store(read.wrapping_add(1), Ordering::Release);
            ReceiveResult::Received(result)
        }
    }

    pub fn is_closed(&self) -> bool {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        // On x86_64, at 1 billion messages/second it would take 584 years to break
        read == usize::MAX || write == usize::MAX
    }

    pub fn count(&self) -> usize {
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        write - read
    }
}

impl<'a, T: Pod> Drop for Receiver<'a, T> {
    fn drop(&mut self) {
        self.read.store(usize::MAX, Ordering::Relaxed);
    }
}
