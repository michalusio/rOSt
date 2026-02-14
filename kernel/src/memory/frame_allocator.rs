use bootloader_api::{BootInfo, info::MemoryRegionKind};
use internal_utils::{
    display::HexNumber, kernel_information::frame_allocator::FullFrameAllocator, log, logln,
};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size2MiB, Size4KiB,
    },
};

use crate::addressing::LOW_MEMORY_LIMIT;

/// A Frame Allocator that allocates according to the usage bitmap of the memory.
/// The maximum size of usable memory is 64GiB with this bitflag.
/// Have to pre-allocate one 4K frame and one 2M frame.
#[repr(C)]
pub struct BitmapFrameAllocator {
    total_usable_memory: u64,
    low_memory_usable_size: u64,
    next_free_2m_frame_guess: usize,
    next_free_4k_frame_guess: usize,
    free_2m_frames: u64,
    free_4k_frames: u64,
    two_megabyte_frames_bitflag: &'static mut [u64; 512],
    four_kilobytes_frames_bitflag: &'static mut [u64; 262_144],
}

impl BitmapFrameAllocator {
    /// Creates a FrameAllocator from the passed memory map.
    pub fn init(boot_info: &'static BootInfo) -> Self {
        let pmo = boot_info.physical_memory_offset.as_ref().unwrap();
        let memory_map = &boot_info.memory_regions;

        let total_usable_memory = memory_map
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| region.end - region.start)
            .sum::<u64>();
        let low_memory_usable_size = memory_map
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| {
                // We skip regions below the lower limit
                let end = region.end.min(LOW_MEMORY_LIMIT);
                let start = region.start.min(end);
                end - start
            })
            .sum::<u64>();

        // We first need to take a 2M frame and a 4K frame from the memory map for the bitflags.
        // We should probably spread it so we allocate the frames anywhere where they fit
        // , but for now let's just find a region that will fit them all.
        let usable_memory_region = memory_map
            .iter()
            .filter(|region| {
                // We skip regions below the lower limit
                let start = region.start.max(LOW_MEMORY_LIMIT);
                let end = region.end.max(start);
                end - start >= Size2MiB::SIZE + Size4KiB::SIZE
            })
            .find(|region| region.kind == MemoryRegionKind::Usable)
            .unwrap();

        log!("Using region (");
        usable_memory_region.start.log_to_separated_hex();
        log!(") - (");
        usable_memory_region.end.log_to_separated_hex();
        logln!(") for the frame allocator");

        let allocation_start = usable_memory_region.start.max(LOW_MEMORY_LIMIT);

        let (four_kilobytes_frames_bitflag, two_megabyte_frames_bitflag) =
            get_bitflag_frames(PhysAddr::new(allocation_start));

        let four_kilo_frame = unsafe {
            VirtAddr::new(four_kilobytes_frames_bitflag.start_address().as_u64() + pmo)
                .as_mut_ptr::<[u64; 262144]>()
                .as_mut()
                .unwrap()
        };
        let two_mega_frame = unsafe {
            VirtAddr::new(two_megabyte_frames_bitflag.start_address().as_u64() + pmo)
                .as_mut_ptr::<[u64; 512]>()
                .as_mut()
                .unwrap()
        };

        logln!("Clearing the allocation bitmaps");
        // We set everything as used because BIOS may return holes in the memory map.
        four_kilo_frame.fill(u64::MAX);
        two_mega_frame.fill(u64::MAX);

        let mut allocator = BitmapFrameAllocator {
            total_usable_memory,
            low_memory_usable_size,
            next_free_2m_frame_guess: two_mega_frame.len(),
            next_free_4k_frame_guess: four_kilo_frame.len(),
            free_2m_frames: 0,
            free_4k_frames: 0,
            four_kilobytes_frames_bitflag: four_kilo_frame,
            two_megabyte_frames_bitflag: two_mega_frame,
        };

        // Now we need to set the usable memory regions as unused so they're not allocated.
        for region in memory_map
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
        {
            let start =
                PhysFrame::containing_address(PhysAddr::new(region.start).align_up(Size4KiB::SIZE));
            let end = PhysFrame::containing_address(
                PhysAddr::new(region.end - 1).align_down(Size4KiB::SIZE),
            );
            let frame_range = PhysFrame::<Size4KiB>::range_inclusive(start, end);

            frame_range
                .filter(|f| f.start_address().as_u64() >= LOW_MEMORY_LIMIT)
                .for_each(|f| {
                    allocator
                        .set_unused_lock(f.start_address().as_u64(), Size4KiB::SIZE)
                        .expect("Failed setting memory regions as unused");
                });
        }

        // We set the regions where we placed the frame allocator as taken
        allocator.set_used_lock(
            four_kilobytes_frames_bitflag.start_address().as_u64(),
            Size2MiB::SIZE,
        );
        allocator.set_used_lock(
            two_megabyte_frames_bitflag.start_address().as_u64(),
            Size4KiB::SIZE,
        );

        logln!(
            "Next frame hints: {} (2M) and {} (4K)",
            allocator.next_free_2m_frame_guess,
            allocator.next_free_4k_frame_guess
        );

        allocator
    }

    fn set_used_lock(&mut self, start_address: u64, size: u64) -> Option<()> {
        match size {
            Size2MiB::SIZE => {
                // Align to 2M frame.
                let start_address = start_address >> 21;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;
                if index >= self.two_megabyte_frames_bitflag.len() {
                    return None;
                }

                // We set the flag for the frame to 1
                let value = self.two_megabyte_frames_bitflag[index];
                let new_value = value | (1 << (start_address & 63));
                debug_assert_ne!(value, new_value, "2M Frame already set as used");

                self.two_megabyte_frames_bitflag[index] |= 1 << (start_address & 63);

                // Now we need to set all the 4K frames in this 2M frame as used.
                let four_kilo_index = (start_address << 3) as usize;
                self.four_kilobytes_frames_bitflag[four_kilo_index..][..8].fill(u64::MAX);

                debug_assert!(self.free_2m_frames >= 1);
                debug_assert!(self.free_4k_frames >= 512);
                self.free_2m_frames -= 1;
                self.free_4k_frames -= 512;

                if self.two_megabyte_frames_bitflag[index] == u64::MAX {
                    self.next_free_2m_frame_guess = index + 1;
                }
                if (four_kilo_index..(four_kilo_index + 8)).contains(&self.next_free_4k_frame_guess)
                {
                    self.next_free_4k_frame_guess = four_kilo_index + 8;
                }
            }
            Size4KiB::SIZE => {
                // Align to 4K frame.
                let start_address = start_address >> 12;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;
                if index >= self.four_kilobytes_frames_bitflag.len() {
                    return None;
                }

                // We set the flag for the frame to 1
                let value = self.four_kilobytes_frames_bitflag[index];
                let new_value = value | (1 << (start_address & 63));
                debug_assert_ne!(value, new_value, "4K Frame already set as used");

                self.four_kilobytes_frames_bitflag[index] = new_value;

                // Now we need to set the 2M frame as used
                let start_address = start_address >> 9;
                let two_mega_index = (start_address >> 6) as usize;
                let value = self.two_megabyte_frames_bitflag[two_mega_index];
                let new_value = value | (1 << (start_address & 63));
                if value != new_value {
                    debug_assert!(self.free_2m_frames >= 1);
                    self.free_2m_frames -= 1;
                }
                self.two_megabyte_frames_bitflag[two_mega_index] = new_value;

                debug_assert!(self.free_4k_frames >= 1);
                self.free_4k_frames -= 1;

                if self.four_kilobytes_frames_bitflag[index] == u64::MAX {
                    self.next_free_4k_frame_guess = index + 1;
                }
            }
            _ => todo!("Implement 1G frame bitflags"),
        }
        Some(())
    }

    fn set_unused_lock(&mut self, start_address: u64, size: u64) -> Option<()> {
        match size {
            Size2MiB::SIZE => {
                // Align to 2M frame.
                let start_address = start_address >> 21;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;
                if index >= self.two_megabyte_frames_bitflag.len() {
                    return None;
                }

                // We set the flag for the frame to 1
                let value = self.two_megabyte_frames_bitflag[index];
                let new_value = value & !(1 << (start_address & 63));
                debug_assert_ne!(value, new_value, "2M Frame already set as unused");

                self.two_megabyte_frames_bitflag[index] &= !(1 << (start_address & 63));

                // Now we need to set all the 4K frames in this 2M frame as unused.
                let four_kilo_index = (start_address << 3) as usize;
                self.four_kilobytes_frames_bitflag[four_kilo_index..][..8].fill(0u64);

                self.next_free_2m_frame_guess = self.next_free_2m_frame_guess.min(index);
                self.next_free_4k_frame_guess = self.next_free_4k_frame_guess.min(four_kilo_index);
                self.free_2m_frames += 1;
                self.free_4k_frames += 512;
            }
            Size4KiB::SIZE => {
                // Align to 4K frame.
                let start_address = start_address >> 12;

                // The index in the bitflags
                let index = (start_address >> 6) as usize;
                if index >= self.four_kilobytes_frames_bitflag.len() {
                    return None;
                }

                // We set the flag for the frame to 1
                let value = self.four_kilobytes_frames_bitflag[index];
                let new_value = value & !(1 << (start_address & 63));
                debug_assert_ne!(value, new_value, "4K Frame already set as unused");

                self.four_kilobytes_frames_bitflag[index] &= !(1 << (start_address & 63));

                // If all the 4K frames in the 2M frame are unused, we need to set the 2M frame itself as unused
                let start_address = start_address >> 9;
                let two_mega_index = (start_address >> 6) as usize;
                if self.four_kilobytes_frames_bitflag[(start_address << 3) as usize..][..8]
                    .iter()
                    .all(|flags| *flags == 0u64)
                {
                    let value = self.two_megabyte_frames_bitflag[two_mega_index];
                    let new_value = value & !(1 << (start_address & 63));
                    self.two_megabyte_frames_bitflag[two_mega_index] = new_value;
                    self.next_free_2m_frame_guess =
                        self.next_free_2m_frame_guess.min(two_mega_index);
                    if value != new_value {
                        self.free_2m_frames += 1;
                    }
                }
                self.next_free_4k_frame_guess = self.next_free_4k_frame_guess.min(index);
                self.free_4k_frames += 1;
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
        self.set_unused_lock(frame.start_address().as_u64(), frame.size());
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    /// Returns the next usable frame
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // We go through all the 4K frames and find the first free one
        let free_4k_frame_flag = self
            .four_kilobytes_frames_bitflag
            .iter()
            .enumerate()
            .skip(self.next_free_4k_frame_guess)
            .find(|(_, flag)| **flag != u64::MAX);

        if let Some(free_4k_frame_flag) = free_4k_frame_flag {
            // We get the position of the free 4K frame
            let free_4k_frame =
                free_4k_frame_flag.1.trailing_ones() as usize + (free_4k_frame_flag.0 << 6);

            // We calculate the 4K frame address
            let frame_address = PhysAddr::new((free_4k_frame as u64) << 12);

            // We set the 4K frame as used
            self.set_used_lock(frame_address.as_u64(), Size4KiB::SIZE)?;
            PhysFrame::from_start_address(frame_address).ok()
        } else {
            debug_assert!(self.free_4k_frames == 0);
            None
        }
    }
}

unsafe impl FrameAllocator<Size2MiB> for BitmapFrameAllocator {
    /// Returns the next usable frame
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        // First we iterate through the negated 2M flags to find a 2M frame with free slots inside
        let free_2m_frame = self
            .two_megabyte_frames_bitflag
            .iter()
            .enumerate()
            .skip(self.next_free_2m_frame_guess)
            .find(|(_, flag)| **flag != u64::MAX);

        if let Some(free_2m_frame) = free_2m_frame {
            // We get the position of the free 2M frame
            let free_2m_frame = free_2m_frame.1.trailing_ones() as usize + (free_2m_frame.0 << 6);

            // We calculate the 2M frame address
            let frame_address = PhysAddr::new((free_2m_frame as u64) << 21);

            // We set the 2M frame as used
            self.set_used_lock(frame_address.as_u64(), Size2MiB::SIZE)?;
            PhysFrame::from_start_address(frame_address).ok()
        } else {
            debug_assert!(self.free_2m_frames == 0);
            None
        }
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
    fn get_total_usable_memory(&self) -> u64 {
        self.total_usable_memory
    }

    fn get_free_memory_size(&self) -> u64 {
        self.get_free_4k_frames() << 12
    }

    fn get_free_dma_memory(&self) -> u64 {
        self.low_memory_usable_size
    }

    fn get_free_4k_frames(&self) -> u64 {
        #[cfg(debug_assertions)]
        {
            let calculated = self
                .four_kilobytes_frames_bitflag
                .iter()
                .map(|f| f.count_zeros() as u64)
                .sum::<u64>();

            assert_eq!(calculated, self.free_4k_frames);
        }
        self.free_4k_frames
    }

    fn get_free_2m_frames(&self) -> u64 {
        #[cfg(debug_assertions)]
        {
            let calculated = self
                .two_megabyte_frames_bitflag
                .iter()
                .map(|f| f.count_zeros() as u64)
                .sum::<u64>();

            assert_eq!(calculated, self.free_2m_frames);
        }
        self.free_2m_frames
    }
}
