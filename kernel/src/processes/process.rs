use crate::processes::memory_mapper::get_user_mode_mapping;
use alloc::sync::Arc;
use internal_utils::{kernel_information::KERNEL_INFORMATION, logln};
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

use alloc::vec::Vec;

use super::thread::{Thread, ThreadState};

#[derive(Debug)]
pub struct Process {
    /// The process's ID.
    pub id: u64,
    /// The page table the process is using.
    pub cr3: PhysAddr,
    /// Total ticks the process has been running for.
    pub total_ticks: u64,
    /// The tick the process has been created on.
    pub start_tick: u64,
    /// The tick the process has been last ran on.
    pub last_tick: u64,
    /// Is the process a kernel process (should it run in ring 0 or 3?).
    pub kernel_process: bool,
    /// The threads of the process that have not started yet.
    pub not_started_threads: Vec<Arc<Mutex<Thread>>>,
    /// The threads of the process that are eligible to run.
    pub ready_threads: Vec<Arc<Mutex<Thread>>>,
    /// The threads of the process that are sleeping.
    pub sleeping_threads: Vec<Arc<Mutex<Thread>>>,
}

impl Process {
    /// Returns the percentage of ticks the process spent running, calculated from the creation time of the process
    pub fn tick_density(&self, current_tick: u64) -> u64 {
        let ticks_maximum = current_tick - self.start_tick;
        self.total_ticks * 100 / ticks_maximum
    }

    /// Creates a new process from a function pointer.
    ///
    /// # Safety
    /// This function is unsafe as it copies the first 1024 bytes from the function pointer.
    // TODO: Loading the process from e.g. an ELF file
    // We have to look up the structure of an ELF file and prepare the user memory mapping according to it.
    // Then we can load the program and it's data to proper places and create a process out of it.
    pub unsafe fn from_extern(function: extern "C" fn(), id: u64) -> Self {
        let function_pointer = function as *const () as *const u8;
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        unsafe {
            let (user_page_map, user_physical_address) =
                get_user_mode_mapping().expect("Error while creating user mode mapping");

            let user_mode_code_address = 0x1000u64;

            let virtual_address = VirtAddr::new(
                user_physical_address.as_u64()
                    + user_mode_code_address
                    + kernel_info.physical_memory_offset,
            )
            .as_mut_ptr::<u8>();
            logln!("Loading program");

            virtual_address.copy_from_nonoverlapping(function_pointer, 1024);

            Process {
                id,
                cr3: user_page_map.start_address(),
                total_ticks: 0,
                start_tick: 0, //get_current_tick(),
                last_tick: 0,
                kernel_process: false,
                not_started_threads: Vec::new(),
                ready_threads: Vec::new(),
                sleeping_threads: Vec::new(),
            }
        }
    }

    /// Updates the sleeping threads, waking them up if they are sleeping for too long.
    pub fn update_sleeping_threads(this: Arc<Mutex<Process>>) {
        let mut process = this.lock();
        if process.sleeping_threads.is_empty() {
            return;
        }
        let mut drained = Vec::new();
        process.sleeping_threads.retain(|thread| {
            let mut borrowed_thread = thread.lock();
            match borrowed_thread.state {
                ThreadState::Sleeping(ref mut sleep_ticks) => {
                    if *sleep_ticks > 0 {
                        *sleep_ticks -= 1;
                        true
                    } else {
                        borrowed_thread.state = ThreadState::Ready;
                        drained.push(thread.clone());
                        false
                    }
                }
                _ => unreachable!(),
            }
        });
        process.ready_threads.extend(drained);
    }
}
