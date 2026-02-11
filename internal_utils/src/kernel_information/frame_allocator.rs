use alloc::sync::Arc;
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, Size2MiB, Size4KiB};

use crate::{display::format_size, kernel_information::allocator::ALLOCATOR, logln};

pub trait FullFrameAllocator:
    FrameAllocator<Size4KiB>
    + FrameAllocator<Size2MiB>
    + FrameDeallocator<Size4KiB>
    + FrameDeallocator<Size2MiB>
{
    /// Returns total memory available in the system.
    fn get_total_memory_size(&self) -> u64;

    /// Returns the amount of memory free to use.
    fn get_free_memory_size(&self) -> u64;

    /// Returns the number of free 4K frames.
    fn get_free_4k_frames(&self) -> u64;

    /// Returns the number of free 2M frames.
    fn get_free_2m_frames(&self) -> u64;
}

pub fn print_memory(allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>) {
    print_frame_memory(allocator);
    print_heap_memory();
}

pub fn print_heap_memory() {
    let (used, size) = {
        let heap_allocator = ALLOCATOR.lock();
        (heap_allocator.used(), heap_allocator.size())
    };
    logln!(
        "{:<14}{:>7}/{:>7}",
        "Heap memory:",
        format_size(used as u64),
        format_size(size as u64)
    );
}

pub fn print_frame_memory(allocator: Arc<Mutex<dyn FullFrameAllocator + Send + Sync>>) {
    #[cfg(debug_assertions)]
    {
        let locked = allocator.lock();
        logln!("[   ---{:^15}---   ]", "FRAME ALLOCATOR");
        {
            let mut size = locked.get_total_memory_size();
            let mut size_format = "B";
            if size >= 2 * 1024 {
                if size < 2 * 1024 * 1024 {
                    size /= 1024;
                    size_format = "KiB";
                } else if size < 2 * 1024 * 1024 * 1024 {
                    size /= 1024 * 1024;
                    size_format = "MiB";
                } else {
                    size /= 1024 * 1024 * 1024;
                    size_format = "GiB";
                }
            }
            logln!("{:<15}{:>10}{:>4}", "Total memory:", size, size_format);
        }
        {
            let mut size = locked.get_free_memory_size();
            let mut size_format = "B";
            if size >= 2 * 1024 {
                if size < 2 * 1024 * 1024 {
                    size /= 1024;
                    size_format = "KiB";
                } else if size < 2 * 1024 * 1024 * 1024 {
                    size /= 1024 * 1024;
                    size_format = "MiB";
                } else {
                    size /= 1024 * 1024 * 1024;
                    size_format = "GiB";
                }
            }
            logln!("{:<15}{:>10}{:>4}", "Free memory:", size, size_format);
        }
        {
            let frames = locked.get_free_2m_frames();
            logln!("{:<15}{:>14}", "Free 2M frames:", frames);
        }
        {
            let frames = locked.get_free_4k_frames();
            logln!("{:<15}{:>14}", "Free 4K frames:", frames);
        }
    }
}
