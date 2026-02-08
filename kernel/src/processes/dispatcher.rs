use core::arch::asm;

use alloc::sync::Arc;
use internal_utils::logln;
use spin::Mutex;
use spin::MutexGuard;
use x86_64::PhysAddr;
use x86_64::structures::paging::page::AddressNotAligned;

use crate::interrupts::GDT;

use super::RegistersState;
use super::get_scheduler;
use super::memory_mapper::clear_user_mode_mapping;
use super::process::Process;
use super::thread::Thread;
use super::thread::ThreadState;

/// Runs the thread immediately.
pub fn switch_to_thread(thread: Arc<Mutex<Thread>>) -> ! {
    let code_selector_id: u64;
    let data_selector_id: u64;
    let cr3: PhysAddr;
    let state: RegistersState;
    x86_64::instructions::interrupts::disable();
    {
        let tick = 0;
        let mut thread_mut = thread.lock();
        thread_mut.last_tick = tick;
        let mut process = thread_mut.process.lock();
        process.last_tick = tick;
        code_selector_id = if process.kernel_process {
            (GDT.1.kernel_code_selector.index() * 8) as u64
        } else {
            ((GDT.1.user_code_selector.index() * 8) | 3) as u64
        };
        data_selector_id = if process.kernel_process {
            (GDT.1.kernel_data_selector.index() * 8) as u64
        } else {
            ((GDT.1.user_data_selector.index() * 8) | 3) as u64
        };
        cr3 = process.cr3;
        state = thread_mut.registers_state;
    }

    get_scheduler()
        .as_mut()
        .unwrap()
        .running_thread
        .replace(thread.clone());
    unsafe {
        // We decrement the counter forcefully because that function doesn't return by Rust.
        Arc::decrement_strong_count(Arc::into_raw(thread));
        asm!(
            "",
            //"mov cr3, r10",
            //"push r14", // data selector
            //"push r12", // process stack pointer
            //"or r11, 0x200",
            //"and r11, 0xffffffffffffbfff",
            //"push r11", // rflags
            //"push r13", // code selector
            //"push r15", // instruction address to return to
            //// Loading register state before jumping into thread
            //mov_all!(),
            //"iretq",
            //in("r9") (&state as *const RegistersState as *const u8),
            //in("r10") (cr3.as_u64()),
            //in("r11") (state.rflags),
            //in("r12") (state.rsp.as_u64()),
            //in("r13") (code_selector_id),
            //in("r14") (data_selector_id),
            //in("r15") (state.rip.as_u64()),
            options(noreturn, nostack)
        );
    }
}

/// Removes the thread from it's process. If this thread is the last one, the process is cleaned up.
pub fn exit_thread(thread: Arc<Mutex<Thread>>) -> Result<(), AddressNotAligned> {
    logln!("Exiting thread");
    let borrowed_thread = thread.lock();
    let mut borrowed_process = borrowed_thread.process.lock();

    remove_thread_from_process_queues(&borrowed_thread, thread.clone(), &mut borrowed_process);

    logln!("Removed thread from process");

    check_should_remove_process(borrowed_process, &borrowed_thread)?;
    Ok(())
}

/// Removes the thread from the respective process queue, depending on the thread state.
pub(crate) fn remove_thread_from_process_queues(
    borrowed_thread: &MutexGuard<Thread>,
    thread: Arc<Mutex<Thread>>,
    borrowed_process: &mut MutexGuard<Process>,
) {
    match borrowed_thread.state {
        ThreadState::Running => {
            let mut lock = get_scheduler();
            let scheduler = lock.as_mut().unwrap();
            if let Some(current_thread) = scheduler.running_thread.clone()
                && Arc::ptr_eq(&thread, &current_thread)
            {
                scheduler.running_thread = None;
            }
        }
        ThreadState::NotStarted => {
            let nst_pos = borrowed_process
                .not_started_threads
                .iter()
                .position(|t| Arc::ptr_eq(t, &thread))
                .unwrap();
            borrowed_process.not_started_threads.swap_remove(nst_pos);
        }
        ThreadState::Ready => {
            let nst_pos = borrowed_process
                .ready_threads
                .iter()
                .position(|t| Arc::ptr_eq(t, &thread))
                .unwrap();
            borrowed_process.ready_threads.swap_remove(nst_pos);
        }
        ThreadState::Sleeping(_) => {
            let nst_pos = borrowed_process
                .sleeping_threads
                .iter()
                .position(|t| Arc::ptr_eq(t, &thread))
                .unwrap();
            borrowed_process.sleeping_threads.swap_remove(nst_pos);
        }
        _ => {}
    }
}

/// Checks if the process has no threads and can be safely removed.
fn check_should_remove_process(
    borrowed_process: MutexGuard<Process>,
    borrowed_thread: &MutexGuard<Thread>,
) -> Result<(), AddressNotAligned> {
    let thread_vectors = [
        &borrowed_process.not_started_threads,
        &borrowed_process.ready_threads,
        &borrowed_process.sleeping_threads,
    ];
    if thread_vectors.into_iter().all(|v| v.is_empty()) {
        //Clean up the process
        get_scheduler()
            .as_mut()
            .unwrap()
            .remove_process(borrowed_thread.process.clone());
        logln!("Removed process from scheduler");
        unsafe {
            clear_user_mode_mapping(borrowed_process.cr3)?;
        }
    }
    Ok(())
}
