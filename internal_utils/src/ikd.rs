use core::str::SplitWhitespace;

use crate::{
    kernel_information::{KERNEL_INFORMATION, KernelInformation, frame_allocator::print_memory},
    logln,
};

pub fn parse_command(command: &str) {
    let command = command.trim();
    let result = COMMANDS
        .iter()
        .find(|(prefix, _)| command.starts_with(prefix));
    if let Some((c, f)) = result {
        let mut arguments = command[c.len()..].split_whitespace();
        f(&mut arguments);
    } else {
        logln!("Invalid command. Try \"help\".");
    }
}

type Arguments<'a> = &'a mut SplitWhitespace<'a>;
type StaticFunction = &'static (dyn Fn(Arguments) + Send + Sync);

static COMMANDS: &[(&str, StaticFunction)] = &[
    ("help", &help),
    ("memory", &memory),
    ("exit", &exit_qemu),
    ("kernel", &kernel),
];

fn help(args: Arguments) {
    if args.next().is_some() {
        logln!("help does not accept arguments");
    }
    logln!("Available commands:");
    for (c, _) in COMMANDS {
        logln!("- {}", c);
    }
}

fn memory(args: Arguments) {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        match subcommand {
            "info" => print_memory(kernel_info.allocator),
            "view" | "viewp" => {
                if let Some((from, to)) = get_from_to(args) {
                    if subcommand == "view" {
                        view_memory_slice(from, to, kernel_info);
                    } else {
                        view_physical_memory_slice(from, to, kernel_info);
                    }
                } else {
                    logln!("You need to pass a from:to range");
                }
            }
            _ => logln!("Invalid subcommand"),
        }
    } else {
        logln!("memory subcommands:");
        logln!("- info | Shows memory information");
        logln!("- view from:to | Shows a slice of virtual memory in a hex view");
        logln!("- viewp from:to | Shows a slice of physical memory in a hex view");
    }
}

fn exit_qemu(_: Arguments) {
    crate::exit_qemu();
}

fn kernel(args: Arguments) {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        match subcommand {
            "info" => kernel_info.print(),
            _ => logln!("Invalid subcommand"),
        }
    } else {
        logln!("kernel subcommands:");
        logln!("- info: Shows kernel information");
    }
}

fn get_from_to(args: Arguments) -> Option<(usize, usize)> {
    let mut args = args.next()?.split(':');
    let from = args.next()?;
    let from = usize::from_str_radix(from, 16).ok()?;
    let to = args.next()?;
    let to = usize::from_str_radix(to, 16).ok()?;
    Some((from, to))
}

fn view_memory_slice(from: usize, to: usize, kernel_info: KernelInformation) {
    for slice in (from..to).step_by(16) {

    }
}

fn view_physical_memory_slice(from: usize, to: usize, kernel_info: KernelInformation) {}
