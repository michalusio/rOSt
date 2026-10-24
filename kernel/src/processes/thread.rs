use alloc::sync::Arc;
use internal_utils::clocks::get_current_tick;
use spin::Mutex;
use x86_64::VirtAddr;

use crate::processes::registers_state::Flags;
use crate::processes::scheduler::{
    BASE_BUDGET, add_thread_to_process_queues, remove_thread_from_process_queues,
};
use crate::processes::wakers::WakeHandle;

use super::process::Process;

use super::RegistersState;

#[derive(Debug, Clone, Copy)]
pub enum ThreadState {
    NotStarted,
    Ready,
    Sleeping(WakeHandle),
    Terminated,
}

#[derive(Debug)]
pub struct Thread {
    /// The thread's ID (in-process).
    pub id: u64,
    /// The thread's current state.
    pub state: ThreadState,
    /// The state of the registers.
    pub registers_state: RegistersState,
    /// Total ticks the thread has been running for.
    pub total_ticks: u64,
    /// The tick the thread has been created on.
    pub start_tick: u64,
    /// The tick the thread has been last ran on.
    pub last_tick: u64,
    /// The ticks left in the budget for that thread.
    pub left_tick_budget: u64,
    /// The process the thread is running for.
    pub process: Arc<Mutex<Process>>,
}

impl Thread {
    /// Returns the percentage of ticks the thread spent running, calculated from the creation time of the thread
    pub fn tick_density(&self, current_tick: u64) -> u64 {
        let ticks_maximum = current_tick.saturating_sub(self.start_tick).max(1);
        self.total_ticks * 100 / ticks_maximum
    }

    pub fn change_state(thread: Arc<Mutex<Thread>>, state: ThreadState) {
        let mut borrowed_thread = thread.lock();
        let process = borrowed_thread.process.clone();
        let mut borrowed_process = process.lock();
        remove_thread_from_process_queues(&mut borrowed_process, &thread, borrowed_thread.state);
        borrowed_thread.state = state;
        add_thread_to_process_queues(&mut borrowed_process, &thread, state);
    }

    /// Creates a new thread with the given starting address and stack pointer.
    ///
    /// # Safety
    /// This function is unsafe as it does not enforce pointing the instruction and stack pointers to valid addresses.
    pub unsafe fn new_native(
        address: usize,
        stack_pointer: usize,
        process: Arc<Mutex<Process>>,
    ) -> Arc<Mutex<Self>> {
        let thread = Thread {
            id: {
                let mut process = process.lock();
                process.total_threads_created += 1;
                process.total_threads_created
            },
            state: ThreadState::NotStarted,
            total_ticks: 0,
            start_tick: get_current_tick(),
            last_tick: 0,
            left_tick_budget: BASE_BUDGET,
            process: process.clone(),
            registers_state: RegistersState::new(
                VirtAddr::new(address as u64),
                Flags::IF.union(Flags::R1).union(Flags::RF),
                VirtAddr::new(stack_pointer as u64),
            ),
        };
        let thread_reference = Arc::new(Mutex::new(thread));
        process
            .lock()
            .not_started_threads
            .push(thread_reference.clone());
        thread_reference
    }
}
