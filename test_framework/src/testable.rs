use internal_utils::{kernel_information::KernelInformation, log, logln};

use crate::ansi_colors::Green;

/// Self documenting test runner trait
pub trait Testable {
    fn run(&self, kernel_information: KernelInformation);
}

impl<T> Testable for T
where
    T: Fn(KernelInformation),
{
    /// Runs the test and prints the result
    fn run(&self, kernel_information: KernelInformation) {
        log!("{}...\t", core::any::type_name::<T>());
        self(kernel_information);
        logln!("{}", Green("[ok]"));
    }
}
