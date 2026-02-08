use alloc::{collections::VecDeque, sync::Arc};
use spin::{Mutex, MutexGuard};

use super::{RegistersState, process::Process, thread::Thread};
use crate::processes::dispatcher::switch_to_thread;

static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

pub fn get_scheduler() -> MutexGuard<'static, Option<Scheduler>> {
    let mut l = SCHEDULER.lock();
    l.get_or_insert_default();
    l
}

/// Runs the scheduler, giving it control of the CPU.
///
/// Will return only if there are no threads at all to run.
pub fn run_processes() -> Option<()> {
    switch_to_thread(get_scheduler().as_mut().unwrap().schedule()?);
}

pub fn add_process(process: Process) -> Arc<Mutex<Process>> {
    get_scheduler().as_mut().unwrap().add_process(process)
}

pub fn run_next_thread() -> Option<()> {
    let next_thread = get_scheduler().as_mut().unwrap().schedule();
    if let Some(thread) = next_thread {
        crate::processes::dispatcher::switch_to_thread(thread);
    } else {
        Some(())
    }
}

#[derive(Default)]
pub struct Scheduler {
    /// The currently running process.
    pub running_thread: Option<Arc<Mutex<Thread>>>,
    /// The list of processes that are registered.
    processes: VecDeque<Arc<Mutex<Process>>>,
}

impl Scheduler {
    /// Adds a process to the scheduling queue so it will be ran.
    pub fn add_process(&mut self, process: Process) -> Arc<Mutex<Process>> {
        let rc = Arc::new(Mutex::new(process));
        self.processes.push_back(rc.clone());
        rc
    }

    /// Removes the process from the queue.
    pub fn remove_process(&mut self, process: Arc<Mutex<Process>>) {
        self.processes.retain(|p| !Arc::ptr_eq(p, &process));
    }

    /// Manages scheduler operations on a timer tick
    pub fn timer_tick(&self, registers_state: RegistersState, tick: u64) {
        if let Some(thread) = self.running_thread.clone() {
            let mut thread_mut = thread.lock();

            thread_mut.registers_state = registers_state;
            thread_mut.total_ticks += tick - thread_mut.last_tick;
            thread_mut.last_tick = tick;
            let mut process: MutexGuard<'_, Process> = thread_mut.process.lock();
            process.total_ticks += tick - process.last_tick;
            process.last_tick = tick;
        }
    }

    /// Returns the thread that should be ran next.
    pub fn schedule(&mut self) -> Option<Arc<Mutex<Thread>>> {
        if self.processes.is_empty() {
            return None;
        }
        // We're taking the first process in the queue that returns a runnable thread
        let processes = &self.processes;
        let process_index = processes.iter().position(|process| {
            Process::update_sleeping_threads(process.clone());
            !process.lock().ready_threads.is_empty()
        })?;
        let process = self.processes.remove(process_index)?;
        let thread = Scheduler::get_thread_to_run(process.clone())?;
        // Putting the process at the back of the queue
        self.processes.push_back(process);

        Some(thread)
    }

    /// Returns the thread from the process that should be ran next.
    fn get_thread_to_run(process: Arc<Mutex<Process>>) -> Option<Arc<Mutex<Thread>>> {
        let mut process_borrowed = process.lock();
        // Taking the first thread in the chosen process
        if process_borrowed.ready_threads.is_empty() {
            return None;
        }
        let thread = process_borrowed.ready_threads.remove(0);
        // Putting the thread at the back of the thread-queue
        process_borrowed.ready_threads.push(thread.clone());
        Some(thread)
    }
}
