pub mod dispatcher;

mod memory_mapper;

pub mod process;

pub mod thread;

mod registers_state;
use core::{
    ptr::Alignment,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::boxed::Box;
use internal_utils::{
    HexNumber,
    channels::spsc,
    gpu_device::{GPU_DEVICE, GPUDeviceCapabilityMut, GPUDeviceCapabilityRequest, RED, WHITE},
    ilogln, log_flush, logln,
    structures::OnceMutex,
};
pub use registers_state::RegistersState;

mod scheduler;
mod scheduler_table;
use process::Process;
use scheduler::{RoundRobinScheduler, Scheduler};
pub use scheduler::{SCHEDULER, add_process, exit_thread, run_processes};
use x86_64::VirtAddr;

use crate::{ikd_check, processes::thread::Thread, syscalls::syscall};
use alloc::alloc::{Layout, alloc};

mod wakers;

static SPSC_DEMO_SENDER: OnceMutex<spsc::Sender<'static, u64>> = OnceMutex::new();
static SPSC_DEMO_RECEIVER: OnceMutex<spsc::Receiver<'static, u64>> = OnceMutex::new();

pub fn init_scheduler() {
    SCHEDULER.call_once(|| {
        let mut scheduler = RoundRobinScheduler::default();
        create_gpu_process(&mut scheduler);
        create_flush_process(&mut scheduler);
        create_spsc_demo_processes(&mut scheduler);
        Box::new(scheduler)
    });
}

fn create_gpu_process(scheduler: &mut RoundRobinScheduler) {
    const STACK_SIZE: usize = 4 * 4096;

    let stack = unsafe {
        let alignment = Alignment::new(16).unwrap();
        let layout = Layout::new::<[u8; STACK_SIZE]>()
            .adjust_alignment_to(alignment)
            .unwrap();
        alloc(layout) as *mut [u8; STACK_SIZE]
    };

    let stack_start = VirtAddr::from_ptr(stack);

    let gpu_process = Process::create_blank(0);
    let gpu_process = scheduler.add_process(gpu_process);
    unsafe {
        let thread = Thread::new_native(
            gpu_process_entry as *const u8 as usize,
            stack_start.as_u64() as usize + STACK_SIZE,
            gpu_process.clone(),
        );
        ilogln!("Thread stack:");
        ilogln!(
            " start: {}",
            ((stack_start.as_u64() as usize + STACK_SIZE - 8) as u64).to_separated_hex()
        );
        ilogln!(" end: {}", stack_start.to_separated_hex());
        thread
    };
}

fn create_flush_process(scheduler: &mut RoundRobinScheduler) {
    const STACK_SIZE: usize = 4 * 4096;

    let stack = unsafe {
        let alignment = Alignment::new(16).unwrap();
        let layout = Layout::new::<[u8; STACK_SIZE]>()
            .adjust_alignment_to(alignment)
            .unwrap();
        alloc(layout) as *mut [u8; STACK_SIZE]
    };

    let stack_start = VirtAddr::from_ptr(stack);

    let flush_process = Process::create_blank(0);
    let flush_process = scheduler.add_process(flush_process);
    unsafe {
        let thread = Thread::new_native(
            flush_process_entry as *const u8 as usize,
            stack_start.as_u64() as usize + STACK_SIZE,
            flush_process.clone(),
        );
        ilogln!("Thread stack:");
        ilogln!(
            " start: {}",
            ((stack_start.as_u64() as usize + STACK_SIZE - 8) as u64).to_separated_hex()
        );
        ilogln!(" end: {}", stack_start.to_separated_hex());
        thread
    };
}

fn create_spsc_demo_processes(scheduler: &mut RoundRobinScheduler) {
    let (_, sender, receiver) = spsc::create::<u64>().expect("SPSC demo channel allocation failed");
    SPSC_DEMO_SENDER.call_once(|| sender);
    SPSC_DEMO_RECEIVER.call_once(|| receiver);
    create_kernel_process(scheduler, 1, spsc_demo_sender_entry);
    create_kernel_process(scheduler, 2, spsc_demo_receiver_entry);
}

fn create_kernel_process(
    scheduler: &mut RoundRobinScheduler,
    id: u64,
    entry: extern "C" fn() -> !,
) {
    const STACK_SIZE: usize = 4 * 4096;

    let stack = unsafe {
        let alignment = Alignment::new(16).unwrap();
        let layout = Layout::new::<[u8; STACK_SIZE]>()
            .adjust_alignment_to(alignment)
            .unwrap();
        alloc(layout)
    };
    assert!(!stack.is_null(), "Process stack allocation failed");

    let stack_start = VirtAddr::from_ptr(stack);
    let process = scheduler.add_process(Process::create_blank(id));
    unsafe {
        Thread::new_native(
            entry as *const u8 as usize,
            stack_start.as_u64() as usize + STACK_SIZE,
            process,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spsc_demo_sender_entry() -> ! {
    let mut value = 0;
    let mut sender = SPSC_DEMO_SENDER.lock().unwrap();
    loop {
        match sender.try_send(value) {
            spsc::SendResult::Full => syscall(0),
            spsc::SendResult::Sent => {
                value += 1;
            }
            spsc::SendResult::Closed => todo!(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spsc_demo_receiver_entry() -> ! {
    let mut receiver = SPSC_DEMO_RECEIVER.lock().unwrap();
    loop {
        match receiver.try_receive() {
            spsc::ReceiveResult::Empty => syscall(0),
            spsc::ReceiveResult::Received(x) => DATA.store(x, Ordering::Relaxed),
            spsc::ReceiveResult::Closed => todo!(),
        }
    }
}

static DATA: AtomicU64 = AtomicU64::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn gpu_process_entry() -> ! {
    logln!("GPU process started!");
    let mut position = 0;
    loop {
        {
            let mut lock = GPU_DEVICE.lock();
            let gpu = lock.as_mut().unwrap();
            let height = gpu.height();
            if let Some(GPUDeviceCapabilityMut::Clearable(clearable)) =
                gpu.get_capability_mut(GPUDeviceCapabilityRequest::Clearable)
            {
                clearable.clear(WHITE);
            }
            if let Some(GPUDeviceCapabilityMut::Shape(shape)) =
                gpu.get_capability_mut(GPUDeviceCapabilityRequest::Shape)
            {
                shape.fill_rectangle(position, (height - 100) as u16, 64, 64, RED);
            }
            if let Some(GPUDeviceCapabilityMut::Flush(flush)) =
                gpu.get_capability_mut(GPUDeviceCapabilityRequest::Flush)
            {
                flush.flush();
            }
            position += 1;
            if position as usize >= gpu.width() - 64 {
                position = 0;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn flush_process_entry() -> ! {
    logln!("Flush process started!");
    loop {
        ikd_check();
        log_flush!();
        syscall(0);
    }
}
