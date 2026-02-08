use crate::{
    ansi_colors::Yellow,
    qemu_exit::{QemuExitCode, exit_qemu},
    testable::Testable,
};
use internal_utils::{kernel_information::KernelInformation, log};
use spin::Mutex;

pub static KERNEL_INFO: Mutex<Option<KernelInformation>> = Mutex::new(None);

/// Rusts test runner function that is called to run all annotated tests.
#[allow(dead_code)]
pub fn test_runner(tests: &[&dyn Testable]) {
    let test_count = tests.len();
    if test_count > 0 {
        log!(
            "{} {} {}",
            Yellow("Running"),
            test_count,
            Yellow("test(s):")
        );
        for test in tests {
            test.run(KERNEL_INFO.lock().clone().unwrap());
        }
    }

    exit_qemu(QemuExitCode::Success);
}
