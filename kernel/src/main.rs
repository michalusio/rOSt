#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features, internal_features)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler
)]
#![test_runner(test_framework::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;
use internal_utils::clocks::{self, get_current_tick};
use internal_utils::kernel_information::KernelInformation;
use internal_utils::{logln, serial};
use kernel::addressing::BOOTLOADER_CONFIG;
use kernel::interrupts::{self, SHOW_CLOCK};
use kernel::{memory, syscalls};

use core::alloc::Layout;

entry_point!(kernel, config = &BOOTLOADER_CONFIG);
pub fn kernel(boot_info: &'static mut BootInfo) -> ! {
    serial::init_logger();
    clocks::init_rtc();
    let allocator = memory::init_kernel_memory(boot_info);
    let kernel_info = KernelInformation::new(boot_info, allocator);
    interrupts::enable();
    syscalls::setup_syscalls();
    ata::init_disks();
    vga::init_vga(kernel_info);

    #[cfg(test)]
    kernel_test(kernel_info);
    //#[cfg(not(test))]
    //kernel_main(kernel_info);

    let clock = get_current_tick();
    loop {
        if get_current_tick() - clock > 1_000_000 {
            break;
        }
    }

    *SHOW_CLOCK.lock() = true;
    internal_utils::hlt_loop();
}

pub fn kernel_main(kernel_info: KernelInformation) {
    //use kernel::processes::{
    //    add_process,
    //    process::Process,
    //    run_processes,
    //    thread::{Thread, ThreadState},
    //};
    //let process1: Arc<Mutex<Process>>;
    //let thread1: Arc<Mutex<Thread>>;
    //unsafe {
    //    process1 = add_process(Process::from_extern(user_mode_check_1, 1));
    //    thread1 = Thread::new_native(0x1000, 2 * MIB, process1);
    //}
    //Thread::change_state(thread1, ThreadState::Ready);

    //let process2 = add_process(Process::new(user_mode_check_2, 2));
    //let _thread2 = Thread::new(0x1000, 2 * MIB, process2);

    //run_processes();
    logln!("Something went wrong");
    /*
        let test = Box::new(4);
        log_println!("New boxed value: {:#?}", test);
        log_println!("im not dying :)");
    */
    /*
        log_println!("Getting all disks...");
        let disks = ata::get_all_disks();
        log_println!("Got {} disks, taking the non-bootable one...", disks.len());
        let mut disk = disks
            .into_iter()
            .map(|mut disk| (disk.has_bootloader(), disk))
            .find(|(boot, _)| !boot.unwrap_or(true))
            .expect("No non-bootable disk found")
            .1;
        log_println!("Got a disk, looking for partitions...");
        let mut partitions = disk.get_partitions().expect("Error getting partitions");
        if partitions.len() == 0 {
            log_println!("No partitions found, creating a new one...");
            let partition_size = disk.descriptor.lba_48_addressable_sectors as u32 / 2;
            disk.create_partition(partition_size, 0xED)
                .expect("Error creating partition");
            log_println!("Partition created, double-checking...");
            partitions = disk.get_partitions().expect("Error getting partitions");
            if partitions.len() == 0 {
                log_println!("No partitions found, giving up.");
                return;
            }
        }
        log_println!("Found {} partitions:", partitions.len());
        for partition in partitions {
            log_println!(
                "{:8} - starting at {:8X}",
                format_size(partition.descriptor.sectors * 512),
                partition.descriptor.start_lba
            )
        }
    */
}

/// This is the main function for tests.
#[cfg(test)]
pub fn kernel_test(_kernel_info: KernelInformation) {
    use test_framework::test_runner::KERNEL_INFO;

    *KERNEL_INFO.lock() = Some(_kernel_info);
    test_main();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Kernel allocation error: {:?}", layout)
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    use internal_utils::logln;
    use test_framework::ansi_colors;

    logln!("{}", ansi_colors::Red("[PANIC]"));
    logln!("Error: {}\n", info);
    internal_utils::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    use test_framework::{
        ansi_colors,
        qemu_exit::{QemuExitCode, exit_qemu},
    };

    logln!("{}", ansi_colors::Red("[PANIC]"));
    logln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    internal_utils::hlt_loop();
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use internal_utils::structures::kernel_information::KernelInformation;
    use x86_64::structures::paging::{Size2MiB, Size4KiB};

    #[test_case]
    fn should_allocate_frame(kernel_information: KernelInformation) {
        use x86_64::structures::paging::PhysFrame;
        let mut allocator = kernel_information.allocator.lock();
        let size = allocator.get_free_memory_size();
        let frame: Option<PhysFrame<Size4KiB>> = allocator.allocate_frame();
        assert!(frame.is_some());
        assert_eq!(4096, size - allocator.get_free_memory_size());
    }

    #[test_case]
    fn should_allocate_big_frame(kernel_information: KernelInformation) {
        use x86_64::structures::paging::PhysFrame;
        let mut allocator = kernel_information.allocator.lock();
        let size = allocator.get_free_memory_size();
        let frame: Option<PhysFrame<Size2MiB>> = allocator.allocate_frame();
        assert!(frame.is_some());
        assert_eq!(2 * 1024 * 1024, size - allocator.get_free_memory_size());
    }

    #[test_case]
    fn should_allocate_small_box(_: KernelInformation) {
        let boxed = Box::new(4);
        assert_eq!(4, *boxed);
    }

    #[test_case]
    fn should_allocate_large_box(_: KernelInformation) {
        let boxed = Box::new([13u8; 256]);
        assert_eq!(boxed.len(), 256);
        for i in 0..256 {
            assert_eq!(boxed[i], 13);
        }
    }
}
