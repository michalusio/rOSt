pub mod dispatcher;

mod memory_mapper;

pub mod process;

pub mod thread;

mod registers_state;
use core::ptr::Alignment;

use alloc::boxed::Box;
use internal_utils::{
    HexNumber,
    gpu_device::{GPU_DEVICE, GPUDeviceCapabilityMut, GPUDeviceCapabilityRequest, RED, WHITE},
    logln,
};
pub use registers_state::RegistersState;

mod scheduler;
mod scheduler_table;
use process::Process;
use scheduler::{FirstComeFirstServedScheduler, Scheduler};
pub use scheduler::{SCHEDULER, add_process, run_processes};
use x86_64::VirtAddr;

use crate::{ikd_check, processes::thread::Thread};
use alloc::alloc::{Layout, alloc};

mod wakers;

pub fn init_scheduler() {
    SCHEDULER.call_once(|| {
        let mut scheduler = FirstComeFirstServedScheduler::default();
        create_idle_process(&mut scheduler);
        Box::new(scheduler)
    });
}

fn create_idle_process(scheduler: &mut FirstComeFirstServedScheduler) {
    const STACK_SIZE: usize = 4 * 4096;

    let stack = unsafe {
        let alignment = Alignment::new(16).unwrap();
        let layout = Layout::new::<[u8; STACK_SIZE]>()
            .adjust_alignment_to(alignment)
            .unwrap();
        alloc(layout) as *mut [u8; STACK_SIZE]
    };

    let stack_start = VirtAddr::from_ptr(stack);

    let idle_process = Process::create_blank(0);
    let idle_process = scheduler.add_process(idle_process);
    unsafe {
        let thread = Thread::new_native(
            idle_process_entry as *const u8 as usize,
            stack_start.as_u64() as usize + STACK_SIZE,
            idle_process.clone(),
        );
        logln!("Thread stack:");
        logln!(
            " start: {}",
            ((stack_start.as_u64() as usize + STACK_SIZE - 8) as u64).to_separated_hex()
        );
        logln!(" end: {}", stack_start.to_separated_hex());
        thread
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn idle_process_entry() -> ! {
    logln!("Idle process started!");
    let mut position = 0;
    loop {
        ikd_check();
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
