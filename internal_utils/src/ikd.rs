use core::str::SplitWhitespace;

use alloc::string::String;

use crate::{
    clocks::{get_current_tick, get_current_time},
    kernel_information::{KERNEL_INFORMATION, frame_allocator::print_memory},
    log, logln,
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
    ("clocks", &clocks),
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
                        view_memory_slice(from, to, 0);
                    } else {
                        view_memory_slice(from, to, kernel_info.physical_memory_offset);
                    }
                } else {
                    logln!("You need to pass a from:to range");
                }
            }
            _ => logln!("Invalid subcommand"),
        }
    } else {
        logln!("memory subcommands:");
        logln!("- {:<20} | Shows memory information", "info");
        logln!(
            "- {:<20} | Shows a slice of virtual memory in a hex view",
            "view from:to"
        );
        logln!(
            "- {:<20} | Shows a slice of physical memory in a hex view",
            "viewp from:to"
        );
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
        logln!("- {:<20} | Shows kernel information", "info");
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

fn view_memory_slice(from: usize, to: usize, offset: u64) {
    let from = from as u64;
    let to = to as u64;
    let mut index: u64 = 0;
    let mut buffer = ['.'; 16];
    while index <= to - from {
        let pointer = (index + from + offset) as *const u8;
        let value = unsafe { *pointer };
        log!("{:02X} ", value);
        let ch = char::from_u32(value as u32)
            .filter(char::is_ascii_alphanumeric)
            .unwrap_or('.');
        buffer[(index & 15) as usize] = ch;
        index += 1;
        if index & 15 == 0 {
            logln!("| {}", String::from_iter(buffer));
        }
    }
    if index & 15 != 0 {
        logln!(
            "| {}",
            String::from_iter(buffer.iter().take((index & 15) as usize))
        );
    }
}

fn clocks(args: Arguments) {
    if args.next().is_some() {
        logln!("clocks does not accept arguments");
    }
    logln!("Ticks: {}", get_current_tick());
    logln!("RTC Time: {}", get_current_time());
}
