use alloc::sync::Arc;
use spin::Mutex;
use x86_64::VirtAddr;

use super::process::Process;

use super::RegistersState;
use super::dispatcher::remove_thread_from_process_queues;

#[derive(Debug, Clone)]
pub enum ThreadState {
    NotStarted,
    Ready,
    Running,
    Sleeping(u64),
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
    /// The process the thread is running for.
    pub process: Arc<Mutex<Process>>,
}

impl Thread {
    /// Returns the percentage of ticks the thread spent running, calculated from the creation time of the thread
    pub fn tick_density(&self, current_tick: u64) -> u64 {
        let ticks_maximum = current_tick - self.start_tick;
        self.total_ticks * 100 / ticks_maximum
    }

    pub fn change_state(thread: Arc<Mutex<Thread>>, state: ThreadState) {
        {
            let borrowed_thread = thread.lock();
            let mut borrowed_process = borrowed_thread.process.lock();
            remove_thread_from_process_queues(
                &borrowed_thread,
                thread.clone(),
                &mut borrowed_process,
            );
        }
        let mut borrowed_thread = thread.lock();
        borrowed_thread.state = state;
        {
            let mut borrowed_process = borrowed_thread.process.lock();
            match borrowed_thread.state {
                ThreadState::NotStarted => {
                    borrowed_process.not_started_threads.push(thread.clone())
                }
                ThreadState::Ready => borrowed_process.ready_threads.push(thread.clone()),
                ThreadState::Running => panic!(
                    "Trying to change a thread to running state - use dispatcher::switch_to_thread() instead"
                ),
                ThreadState::Sleeping(_) => borrowed_process.sleeping_threads.push(thread.clone()),
                ThreadState::Terminated => {}
            }
        }
    }

    /// Creates a new thread with the given starting address and stack pointer.
    ///
    /// # Safety
    /// This function is unsafe as it does not enforce pointing the instruction and stack pointers to valid addresses.
    pub unsafe fn new_native(
        address: u64,
        stack_pointer: u64,
        process: Arc<Mutex<Process>>,
    ) -> Arc<Mutex<Self>> {
        let thread = Thread {
            id: {
                let process = process.lock();
                process.not_started_threads.len()
                    + process.ready_threads.len()
                    + process.sleeping_threads.len()
            } as u64,
            state: ThreadState::NotStarted,
            total_ticks: 0,
            start_tick: 0, //get_current_tick(),
            last_tick: 0,
            process: process.clone(),
            registers_state: RegistersState::new(
                VirtAddr::new(address),
                0x200,
                VirtAddr::new(stack_pointer),
            ),
        };
        let rc = Arc::new(Mutex::new(thread));
        process.lock().not_started_threads.push(rc.clone());
        rc
    }
}
