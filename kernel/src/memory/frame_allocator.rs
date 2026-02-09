use bootloader_api::{
    BootInfo,
    info::{MemoryRegionKind, MemoryRegions},
};
use internal_utils::{
    display::HexNumber, kernel_information::frame_allocator::FullFrameAllocator, log, logln,
};
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size2MiB, Size4KiB,
    },
};

use lazy_static::lazy_static;

lazy_static! {
    /// The maximum size of usable memory is 64GiB with this bitflag.
    /// Have to pre-allocate one 4K frame and one 2M frame.
    static ref TWO_MEGABYTES_FRAMES_BITFLAG: Mutex<Option<&'static mut [u64; 512]>> =
        Mutex::new(None);
    static ref FOUR_KILOBYTES_FRAMES_BITFLAG: Mutex<Option<&'static mut [u64; 262_144]>> =
        Mutex::new(None);
}

/// A Frame Allocator that allocates according to the usage bitmap of the memory.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BitmapFrameAllocator {
    memory_map: &'static MemoryRegions,
    total_region_area: u64,
}

impl BitmapFrameAllocator {
    /// Creates a FrameAllocator from the passed memory map.
    pub fn init(boot_info: &'static BootInfo) -> Self {
        let memory_map = &boot_info.memory_regions;
        let total_region_area = memory_map
            .iter()
            .map(|region| region.end - region.start)
            .sum::<u64>();

        if TWO_MEGABYTES_FRAMES_BITFLAG.lock().is_some() {
            // Frames already allocated
            BitmapFrameAllocator {
                memory_map,
                total_region_area,
            }
        } else {
            let pmo = boot_info.physical_memory_offset.as_ref().unwrap();

            // We first need to take a 2M frame and a 4K frame from the memory map for the bitflags.
            // We should probably spread it so we allocate the frames anywhere where they fit
            // , but for now let's just find a region that will fit them all.
            let usable_memory_region = memory_map
                .iter()
                .filter(|region| region.end - region.start >= Size2MiB::SIZE + Size4KiB::SIZE)
                .find(|region| region.kind == MemoryRegionKind::Usable)
                .unwrap();

            log!("Using region (");
            usable_memory_region.start.log_to_separated_hex();
            log!(") - (");
            usable_memory_region.end.log_to_separated_hex();
            logln!(") for the frame allocator");

            let (four_kilobytes_frames_bitflag, two_megabyte_frames_bitflag) =
                get_bitflag_frames(PhysAddr::new(usable_memory_region.start));

            let four_kilo_frame = unsafe {
                VirtAddr::new(four_kilobytes_frames_bitflag.start_address().as_u64() + pmo)
                    .as_mut_ptr::<[u64; 262144]>()
                    .as_mut()
                    .expect("Cannot allocate the 2M frame")
            };
            let two_mega_frame = unsafe {
                VirtAddr::new(two_megabyte_frames_bitflag.start_address().as_u64() + pmo)
                    .as_mut_ptr::<[u64; 512]>()
                    .as_mut()
                    .expect("Cannot allocate the 4K frame")
            };

            logln!("Clearing the allocation bitmaps");
            // We set everything as used because BIOS may return holes in the memory map.
            four_kilo_frame.fill(u64::MAX);
            two_mega_frame.fill(u64::MAX);

            *FOUR_KILOBYTES_FRAMES_BITFLAG.lock() = Some(four_kilo_frame);
            *TWO_MEGABYTES_FRAMES_BITFLAG.lock() = Some(two_mega_frame);

            let mut allocator = BitmapFrameAllocator {
                memory_map,
                total_region_area,
            };

            // Now we need to set the usable memory regions as unused so they're not allocated.
            for region in memory_map
                .iter()
                .filter(|region| region.kind == MemoryRegionKind::Usable)
            {
                let start = PhysFrame::containing_address(
                    PhysAddr::new(region.start).align_up(Size4KiB::SIZE),
                );
                let end = PhysFrame::containing_address(
                    PhysAddr::new(region.end - 1).align_down(Size4KiB::SIZE),
                );
                let frame_range = PhysFrame::<Size4KiB>::range_inclusive(start, end);

                frame_range.for_each(|f| {
                    allocator
                        .set_unused(f.start_address().as_u64(), Size4KiB::SIZE)
                        .expect("Failed setting memory regions as unused");
                });
            }

            // We set the regions where we placed the frame allocator as taken
            allocator.set_used(
                four_kilobytes_frames_bitflag.start_address().as_u64(),
                Size2MiB::SIZE,
            );
            allocator.set_used(
                two_megabyte_frames_bitflag.start_address().as_u64(),
                Size4KiB::SIZE,
            );

            allocator
        }
    }

    /// Unconditionally sets the frame at the start_address as used in the bitflags
    fn set_used(&mut self, start_address: u64, size: u64) -> Option<()> {
        match size {
            Size2MiB::SIZE => {
                // Align to 2M frame.
                let start_address = start_address >> 21;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;

                // We set the flag for the frame to 1
                let mut mbl = TWO_MEGABYTES_FRAMES_BITFLAG.lock();
                let value = mbl.as_mut()?[index];
                if value == value | (1 << (start_address & 63)) {
                    panic!(
                        "2M Frame at {} bit {} already set as used",
                        index,
                        start_address & 63
                    );
                }
                mbl.as_mut()?[index] |= 1 << (start_address & 63);

                // Now we need to set all the 4K frames in this 2M frame as used.
                let mut fbl = FOUR_KILOBYTES_FRAMES_BITFLAG.lock();
                let four_kilobytes_frames_bitflag_lock = fbl.as_mut()?;
                for i in (start_address << 3)..((start_address << 3) + 8) {
                    four_kilobytes_frames_bitflag_lock[i as usize] = u64::MAX;
                }
            }
            Size4KiB::SIZE => {
                // Align to 4K frame.
                let start_address = start_address >> 12;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;

                // We set the flag for the frame to 1
                let mut fbl = FOUR_KILOBYTES_FRAMES_BITFLAG.lock();
                let four_kilobytes_frames_bitflag_lock = fbl.as_mut()?;
                let value = four_kilobytes_frames_bitflag_lock[index];
                if value == value | (1 << (start_address & 63)) {
                    panic!(
                        "4K Frame at {} bit {} already set as used",
                        index,
                        start_address & 63
                    );
                }
                four_kilobytes_frames_bitflag_lock[index] |= 1 << (start_address & 63);

                // Now we need to set the 2M frame as used
                let start_address = start_address >> 9;
                let index = (start_address >> 6) as usize;
                TWO_MEGABYTES_FRAMES_BITFLAG.lock().as_mut()?[index] |= 1 << (start_address & 63);
            }
            _ => todo!("Implement 1G frame bitflags"),
        }
        Some(())
    }

    /// Unconditionally sets the frame at the start_address as unused in the bitflags
    fn set_unused(&mut self, start_address: u64, size: u64) -> Option<()> {
        match size {
            Size2MiB::SIZE => {
                // Align to 2M frame.
                let start_address = start_address >> 21;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;

                // We set the flag for the frame to 1
                let mut mbl = TWO_MEGABYTES_FRAMES_BITFLAG.lock();
                let value = mbl.as_mut()?[index];
                if value == value & !(1 << (start_address & 63)) {
                    panic!(
                        "2M Frame at {} bit {} already set as unused",
                        index,
                        start_address & 63
                    );
                }
                mbl.as_mut()?[index] &= !(1 << (start_address & 63));

                // Now we need to set all the 4K frames in this 2M frame as unused.
                let mut fbl = FOUR_KILOBYTES_FRAMES_BITFLAG.lock();
                let four_kilobytes_frames_bitflag_lock = fbl.as_mut()?;
                for i in (start_address << 3)..((start_address << 3) + 8) {
                    four_kilobytes_frames_bitflag_lock[i as usize] = 0u64;
                }
            }
            Size4KiB::SIZE => {
                // Align to 4K frame.
                let start_address = start_address >> 12;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;

                // We set the flag for the frame to 1
                let mut fbl = FOUR_KILOBYTES_FRAMES_BITFLAG.lock();
                let four_kilobytes_frames_bitflag_lock = fbl.as_mut()?;
                let value = four_kilobytes_frames_bitflag_lock[index];
                if value == value & !(1 << (start_address & 63)) {
                    panic!(
                        "4K Frame at {} bit {} already set as unused",
                        index,
                        start_address & 63
                    );
                }
                four_kilobytes_frames_bitflag_lock[index] &= !(1 << (start_address & 63));

                // If all the 4K frames in the 2M frame are unused, we need to set the 2M frame itself as unused
                let start_address = start_address >> 9;
                let index = (start_address >> 6) as usize;
                if four_kilobytes_frames_bitflag_lock
                    [(start_address << 3) as usize..((start_address << 3) + 8) as usize]
                    .iter()
                    .all(|flags| *flags == 0u64)
                {
                    TWO_MEGABYTES_FRAMES_BITFLAG.lock().as_mut()?[index] &=
                        !(1 << (start_address & 63));
                }
            }
            _ => todo!("Implement 1G frame bitflags"),
        }
        Some(())
    }
}

impl<S> FrameDeallocator<S> for BitmapFrameAllocator
where
    S: PageSize,
{
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>) {
        self.set_unused(frame.start_address().as_u64(), frame.size());
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    /// Returns the next usable frame
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame_address: PhysAddr;
        {
            let mut fbl = FOUR_KILOBYTES_FRAMES_BITFLAG.lock();
            let four_kilobytes_frames_bitflag_lock = fbl.as_mut()?;

            // We go through all the 4K frames and find the first free one
            let free_4k_frame_flag = four_kilobytes_frames_bitflag_lock
                .iter()
                .enumerate()
                .find(|(_, flag)| **flag != u64::MAX)?;

            // We get the position of the free 4K frame
            let free_4k_frame =
                free_4k_frame_flag.1.trailing_ones() as usize + (free_4k_frame_flag.0 << 6);

            // We set the 4K frame as used
            frame_address = PhysAddr::new((free_4k_frame as u64) << 12);
        }
        self.set_used(frame_address.as_u64(), Size4KiB::SIZE)?;
        PhysFrame::from_start_address(frame_address).ok()
    }
}

unsafe impl FrameAllocator<Size2MiB> for BitmapFrameAllocator {
    /// Returns the next usable frame
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        let frame_address: PhysAddr;
        {
            let mut mbl = TWO_MEGABYTES_FRAMES_BITFLAG.lock();
            let two_megabytes_frames_bitflag_lock = mbl.as_mut()?;

            // First we iterate through the negated 2M flags to find a 2M frame with free slots inside
            let free_2m_frame = two_megabytes_frames_bitflag_lock
                .iter()
                .enumerate()
                .find(|(_, flag)| **flag != u64::MAX)?;

            // We get the position of the free 2M frame
            let free_2m_frame = free_2m_frame.1.trailing_ones() as usize + (free_2m_frame.0 << 6);

            // We set the 2M frame as used
            frame_address = PhysAddr::new((free_2m_frame as u64) << 21);
        }
        self.set_used(frame_address.as_u64(), Size2MiB::SIZE)?;
        PhysFrame::from_start_address(frame_address).ok()
    }
}

/// Allocates the frames required for the frame allocator.
///
/// Chicken and egg?
fn get_bitflag_frames(start_address: PhysAddr) -> (PhysFrame<Size2MiB>, PhysFrame<Size4KiB>) {
    let four_kilobytes_frames_bitflag: PhysFrame<Size2MiB>;
    let two_megabyte_frames_bitflag: PhysFrame<Size4KiB>;
    // We need to allocate one 2M frame and 2x4K frames, but the region addresses do not have to be 2M aligned!
    // So first we need to check the alignment, and we have 3 options here:
    // 1. The start address is 2M aligned - we allocate the 2M frame then the 4K frame, easy.
    // 2. The start address + 4K is 2M aligned - we allocate the 4K frame first, then 2M after it.
    // 3. The start address is not 2M aligned at all - we allocate the 4K frame, then we allocate the 2M frame aligned wherever it is.
    if start_address.is_aligned(Size2MiB::SIZE) {
        four_kilobytes_frames_bitflag = PhysFrame::<Size2MiB>::from_start_address(start_address)
            .expect("2M frame address not aligned");
        two_megabyte_frames_bitflag =
            PhysFrame::<Size4KiB>::from_start_address(start_address + Size2MiB::SIZE)
                .expect("4K frame address not aligned");
    } else if (start_address + Size4KiB::SIZE).is_aligned(Size2MiB::SIZE) {
        four_kilobytes_frames_bitflag =
            PhysFrame::<Size2MiB>::from_start_address(start_address + Size4KiB::SIZE)
                .expect("2M frame address not aligned");
        two_megabyte_frames_bitflag = PhysFrame::<Size4KiB>::from_start_address(start_address)
            .expect("4K frame address not aligned");
    } else {
        four_kilobytes_frames_bitflag =
            PhysFrame::<Size2MiB>::from_start_address(start_address.align_up(Size2MiB::SIZE))
                .expect("2M frame address not aligned");
        two_megabyte_frames_bitflag = PhysFrame::<Size4KiB>::from_start_address(start_address)
            .expect("4K frame address not aligned");
    }
    (four_kilobytes_frames_bitflag, two_megabyte_frames_bitflag)
}

impl FullFrameAllocator for BitmapFrameAllocator {
    fn get_total_memory_size(&self) -> u64 {
        self.total_region_area
    }

    fn get_free_memory_size(&self) -> u64 {
        self.get_free_4k_frames() << 12
    }

    fn get_free_4k_frames(&self) -> u64 {
        FOUR_KILOBYTES_FRAMES_BITFLAG
            .lock()
            .as_ref()
            .unwrap()
            .iter()
            .map(|flag| flag.count_zeros() as u64)
            .sum::<u64>()
    }

    fn get_free_2m_frames(&self) -> u64 {
        TWO_MEGABYTES_FRAMES_BITFLAG
            .lock()
            .as_ref()
            .unwrap()
            .iter()
            .map(|flag| flag.count_zeros() as u64)
            .sum::<u64>()
    }
}

#[inline(always)]
pub fn print_frame_memory(allocator: &impl FullFrameAllocator) {
    #[cfg(debug_assertions)]
    {
        use internal_utils::logln;

        logln!("[   ---{:^15}---   ]", "FRAME ALLOCATOR");
        {
            let mut size = allocator.get_total_memory_size();
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
            logln!("Total memory: {:>4}{:>4}", size, size_format);
        }
        {
            let mut size = allocator.get_free_memory_size();
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
            logln!("Free memory:  {:>4}{:>4}", size, size_format);
        }
        {
            let frames = allocator.get_free_2m_frames();
            logln!("Free 2M frames:  {:>5}", frames);
        }
        {
            let frames = allocator.get_free_4k_frames();
            logln!("Free 4K frames:  {:>5}", frames);
        }
    }
}
