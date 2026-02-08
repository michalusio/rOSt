use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, Size2MiB, Size4KiB};

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
