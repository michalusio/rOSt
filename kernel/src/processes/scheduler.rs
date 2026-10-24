use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use internal_utils::{clocks::get_current_tick, ilogln, structures::OnceMutex};
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::page::AddressNotAligned;

use super::{RegistersState, process::Process, thread::Thread};
use crate::processes::{
    dispatcher::dispatch_thread,
    memory_mapper::clear_user_mode_mapping,
    scheduler_table::{ProcessInfo, SchedulerTable, ThreadInfo},
    thread::ThreadState,
};

pub static SCHEDULER: OnceMutex<Box<dyn Scheduler>> = OnceMutex::new();

pub trait Scheduler: Send {
    /// Returns the thread that should be ran next, and sets that thread as currently running.
    fn schedule(&mut self) -> Arc<Mutex<Thread>>;

    /// Adds a process to the scheduling queue so it will be ran.
    fn add_process(&mut self, process: Process) -> Arc<Mutex<Process>>;

    /// Removes the process from the scheduling queue.
    fn remove_process(&mut self, process: &Arc<Mutex<Process>>);

    /// Keeps accounting of the thread ran in a tick.
    fn on_tick(&self, registers_state: RegistersState, tick: u64);

    fn get_running_thread(&self) -> Option<Arc<Mutex<Thread>>>;
    fn clear_running_thread(&mut self);

    fn get_processes_and_threads(&self) -> SchedulerTable;
}

/// Runs the scheduler, giving it control of the CPU.
///
/// Will return only if there are no threads at all to run.
pub fn run_processes() -> ! {
    let thread = { SCHEDULER.lock().unwrap().schedule() };
    dispatch_thread(thread);
}

pub fn add_process(process: Process) -> Arc<Mutex<Process>> {
    SCHEDULER.lock().unwrap().add_process(process)
}

pub const BASE_BUDGET: u64 = 100_000;

#[derive(Default)]
pub struct RoundRobinScheduler {
    /// The currently running process.
    running_thread: Option<Arc<Mutex<Thread>>>,
    /// The list of processes that are registered.
    processes: VecDeque<Arc<Mutex<Process>>>,
}

impl Scheduler for RoundRobinScheduler {
    fn get_running_thread(&self) -> Option<Arc<Mutex<Thread>>> {
        self.running_thread.clone()
    }

    fn clear_running_thread(&mut self) {
        self.running_thread = None;
    }

    fn get_processes_and_threads(&self) -> SchedulerTable {
        let current_tick = get_current_tick();
        SchedulerTable(
            self.processes
                .iter()
                .map(|p| p.lock())
                .map(|p| ProcessInfo {
                    id: p.id,
                    kernel_process: p.kernel_process,
                    load: p.tick_density(current_tick),
                    threads: p
                        .ready_threads
                        .iter()
                        .chain(p.not_started_threads.iter())
                        .chain(p.sleeping_threads.iter())
                        .map(|t| t.lock())
                        .map(|t| ThreadInfo {
                            id: t.id,
                            state: match t.state {
                                ThreadState::NotStarted => "not started",
                                ThreadState::Ready => "ready",
                                ThreadState::Sleeping(_) => "sleeping",
                                ThreadState::Terminated => "terminated",
                            },
                            load: t.tick_density(current_tick),
                        })
                        .collect(),
                })
                .collect(),
        )
    }

    fn add_process(&mut self, process: Process) -> Arc<Mutex<Process>> {
        let rc = Arc::new(Mutex::new(process));
        self.processes.push_back(rc.clone());
        rc
    }

    fn remove_process(&mut self, process: &Arc<Mutex<Process>>) {
        self.processes.retain(|p| !Arc::ptr_eq(p, process));
    }

    /// Manages scheduler operations on a timer tick
    fn on_tick(&self, registers_state: RegistersState, tick: u64) {
        if let Some(thread) = self.running_thread.clone() {
            let mut thread_mut = thread.lock();

            thread_mut.registers_state = registers_state.clone();
            let ticks_taken = tick - thread_mut.last_tick;
            thread_mut.total_ticks += ticks_taken;
            thread_mut.left_tick_budget = thread_mut.left_tick_budget.saturating_sub(ticks_taken);
            thread_mut.last_tick = tick;
            let mut process: MutexGuard<'_, Process> = thread_mut.process.lock();
            process.total_ticks += tick - process.last_tick;
            process.last_tick = tick;
        }
    }

    fn schedule(&mut self) -> Arc<Mutex<Thread>> {
        if let Some(thread) = self.running_thread.clone() {
            let still_has_budget = {
                let mut thread_mut = thread.lock();
                if thread_mut.left_tick_budget > 0
                    && matches!(thread_mut.state, ThreadState::Ready)
                {
                    true
                } else {
                    if thread_mut.left_tick_budget == 0 {
                        thread_mut.left_tick_budget = BASE_BUDGET;
                    }
                    false
                }
            };
            if still_has_budget {
                return thread;
            }
        }
        // We're taking the first process in the queue that returns a runnable thread
        let processes = &mut self.processes;
        let process = processes
            .extract_if(.., |process| {
                Process::update_sleeping_threads(process);
                let lock = process.lock();
                !lock.ready_threads.is_empty() || !lock.not_started_threads.is_empty()
            })
            .next()
            .expect("There has to be at least one process in the scheduler");
        let thread = self.get_thread_to_run(process.clone());
        // Putting the process at the back of the queue
        self.processes.push_back(process);

        self.running_thread = Some(thread.clone());
        thread
    }
}

impl RoundRobinScheduler {
    /// Returns the thread from the process that should be ran next.
    fn get_thread_to_run(&self, process: Arc<Mutex<Process>>) -> Arc<Mutex<Thread>> {
        let mut process_borrowed = process.lock();

        // Taking the first thread in the chosen process
        let thread = process_borrowed.ready_threads.try_remove(0);

        let thread = if let Some(thread) = thread {
            thread
        } else {
            let thread = process_borrowed
                .not_started_threads
                .try_remove(0)
                .expect("A process must have a ready or not started thread to run");
            thread.lock().state = ThreadState::Ready;
            thread
        };

        // Putting the thread at the back of the thread-queue
        process_borrowed.ready_threads.push(thread.clone());

        thread
    }
}

/// Removes the thread from its process. If this thread is the last one, the process is cleaned up.
pub fn exit_thread(thread: Arc<Mutex<Thread>>) -> Result<(), AddressNotAligned> {
    ilogln!("Exiting thread");
    {
        let mut scheduler = SCHEDULER.lock().unwrap();
        if scheduler
            .get_running_thread()
            .as_ref()
            .is_some_and(|running| Arc::ptr_eq(running, &thread))
        {
            scheduler.clear_running_thread();
        }
    }
    let mut borrowed_thread = thread.lock();
    let process = borrowed_thread.process.clone();
    let mut borrowed_process = process.lock();

    remove_thread_from_process_queues(&mut borrowed_process, &thread, borrowed_thread.state);

    ilogln!("Removed thread from process");

    check_should_remove_process(&mut borrowed_process, &mut borrowed_thread)?;
    Ok(())
}

/// Removes the thread from the respective process queue, depending on the thread state.
pub fn remove_thread_from_process_queues(
    borrowed_process: &mut Process,
    changed_thread: &Arc<Mutex<Thread>>,
    state: ThreadState,
) {
    match state {
        ThreadState::NotStarted => {
            borrowed_process
                .not_started_threads
                .extract_if(.., |t| Arc::ptr_eq(t, changed_thread))
                .next();
        }
        ThreadState::Ready => {
            borrowed_process
                .ready_threads
                .extract_if(.., |t| Arc::ptr_eq(t, changed_thread))
                .next();
        }
        ThreadState::Sleeping(_) => {
            borrowed_process
                .sleeping_threads
                .extract_if(.., |t| Arc::ptr_eq(t, changed_thread))
                .next();
        }
        _ => {}
    }
}

/// Checks if the process has no threads and can be safely removed.
fn check_should_remove_process(
    borrowed_process: &mut Process,
    borrowed_thread: &mut Thread,
) -> Result<(), AddressNotAligned> {
    let thread_vectors = [
        &borrowed_process.not_started_threads,
        &borrowed_process.ready_threads,
        &borrowed_process.sleeping_threads,
    ];
    if thread_vectors.into_iter().all(|v| v.is_empty()) {
        //Clean up the process
        SCHEDULER
            .lock()
            .unwrap()
            .remove_process(&borrowed_thread.process);
        ilogln!("Removed process from scheduler");
        unsafe {
            clear_user_mode_mapping(borrowed_process.cr3.0)?;
        }
    }
    Ok(())
}

pub fn add_thread_to_process_queues(
    borrowed_process: &mut MutexGuard<Process>,
    thread: &Arc<Mutex<Thread>>,
    state: ThreadState,
) {
    match state {
        ThreadState::NotStarted => borrowed_process.not_started_threads.push(thread.clone()),
        ThreadState::Ready => borrowed_process.ready_threads.push(thread.clone()),
        ThreadState::Sleeping(_) => borrowed_process.sleeping_threads.push(thread.clone()),
        ThreadState::Terminated => {}
    }
}
