use core::{arch::asm, str::SplitWhitespace};

use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crosstrait::Cast;
use internal_utils::HexNumber;

use internal_utils::tag_store::{
    BinaryBoolQueryExpression, BinaryBoolQueryExpressionType, BooleanTag, Query, TAG_STORE,
};
use internal_utils::{
    clocks::{get_current_tick, get_current_time},
    kernel_information::{KERNEL_INFORMATION, frame_allocator::print_memory},
    log, logln,
};
use x86_64::registers::read_rip;

use crate::addressing;
use crate::processes::{SCHEDULER, run_processes};

/// Parses a command. Returns whether we should exit the IKD
pub fn parse_command(command: &str) -> bool {
    let command = command.trim();
    let result = COMMANDS
        .iter()
        .find(|(prefix, _)| command.starts_with(prefix));
    if let Some((c, f)) = result {
        let mut arguments = command[c.len()..].split_whitespace();
        f(&mut arguments)
    } else {
        logln!("Invalid command. Try \"help\".");
        false
    }
}

type Arguments<'a> = &'a mut SplitWhitespace<'a>;
type StaticFunction = &'static (dyn (Fn(Arguments) -> bool) + Send + Sync);

static COMMANDS: &[(&str, StaticFunction)] = &[
    ("help", &help),
    ("memory", &memory),
    ("exit", &exit),
    ("kernel", &kernel),
    ("scheduler", &scheduler),
    ("clocks", &clocks),
    ("ip", &ip),
    ("tbes", &tbes),
    ("panic", &panic),
];

fn help(args: Arguments) -> bool {
    if args.next().is_some() {
        logln!("help does not accept arguments");
    }
    logln!("Available commands:");
    for (c, _) in COMMANDS {
        logln!("- {}", c);
    }
    false
}

fn memory(args: Arguments) -> bool {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        match subcommand {
            "info" => print_memory(kernel_info.allocator),
            "view" | "viewp" | "viewk" => {
                if let Some((from, to)) = get_from_to(args) {
                    if subcommand == "view" {
                        view_memory_slice(from, to, 0);
                    } else if subcommand == "viewp" {
                        view_memory_slice(from, to, kernel_info.physical_memory_offset);
                    } else {
                        view_memory_slice(from, to, addressing::ADDRESSES[3]);
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
        logln!(
            "- {:<20} | Shows a slice of kernel memory in a hex view",
            "viewk from:to"
        );
    }
    false
}

fn exit(args: Arguments) -> bool {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        match subcommand {
            "qemu" => exit_qemu(),
            "ikd" => return true,
            _ => logln!("Invalid subcommand"),
        }
    } else {
        logln!("exit subcommands:");
        logln!("- {:<20} | Closes QEMU (if applicable)", "qemu");
        logln!("- {:<20} | Closes IKD (if possible)", "ikd");
    }
    false
}

fn exit_qemu() -> ! {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") 0xf4u16,
            in("eax") 0x10,
            options(noreturn, nostack)
        );
    }
}

fn kernel(args: Arguments) -> bool {
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
    false
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
        for _ in 0..(16 - (index & 15)) {
            log!("   ");
        }
        logln!(
            "| {}",
            String::from_iter(buffer.iter().take((index & 15) as usize))
        );
    }
}

fn clocks(args: Arguments) -> bool {
    if args.next().is_some() {
        logln!("clocks does not accept arguments");
    }
    logln!("Ticks: {}", get_current_tick());
    logln!("RTC Time: {}", get_current_time());
    false
}

fn ip(args: Arguments) -> bool {
    if args.next().is_some() {
        logln!("ip does not accept arguments");
    }
    let ip = read_rip();
    logln!("Current instruction pointer: {}", ip.to_separated_hex());
    logln!("Though tbh it's kinda useless");
    false
}

fn panic(_: Arguments) -> bool {
    panic!("Invoked the panic handler");
}

fn scheduler(args: Arguments) -> bool {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        match subcommand {
            "processes" => SCHEDULER.lock().unwrap().get_processes_and_threads().log(),
            "run" => run_processes(),
            _ => logln!("Invalid subcommand"),
        }
    } else {
        logln!("scheduler subcommands:");
        logln!("- {:<20} | Shows processes", "processes");
        logln!("- {:<20} | Runs the scheduler", "run");
    }
    false
}

fn tbes(args: Arguments) -> bool {
    let store = TAG_STORE.get().unwrap();
    let tags: Vec<Arc<dyn BooleanTag>> = store
        .get_all_tags()
        .into_iter()
        .filter_map(|o| o.cast().clone())
        .collect();
    let mut tag_map = BTreeMap::new();
    for tag in tags.iter() {
        tag_map.insert(tag.name(), tag.clone());
    }
    logln!("Found {} tags", tag_map.len());

    let mut show_query_plan = false;
    let checks: Result<Vec<Query>, ()> = args
        .inspect(|arg| {
            if *arg == "-q" {
                show_query_plan = true;
            }
        })
        .filter(|arg| *arg != "-q")
        .map(|arg| {
            arg.split_once('=').ok_or(()).and_then(|(name, value)| {
                match (tag_map.get(name), value.to_ascii_lowercase().as_str()) {
                    (Some(tag), "true") => Ok(Query::Binary(
                        BinaryBoolQueryExpression {
                            first: (*tag).clone(),
                            second: true,
                            operation: BinaryBoolQueryExpressionType::EqualTo,
                        }
                        .into(),
                    )),
                    (Some(tag), "false") => Ok(Query::Binary(
                        BinaryBoolQueryExpression {
                            first: (*tag).clone(),
                            second: false,
                            operation: BinaryBoolQueryExpressionType::EqualTo,
                        }
                        .into(),
                    )),
                    (None, _) => {
                        logln!("Cannot find tag {}", name);
                        Err(())
                    }
                    _ => {
                        logln!("Tag condition has to be of the form TAG_NAME=TRUE/FALSE");
                        Err(())
                    }
                }
            })
        })
        .collect();

    if let Ok(checks) = checks {
        let query = Query::And(checks);
        let (data, plan) = store.query(query);
        if show_query_plan {
            logln!("Query plan: {}", plan);
        }
        for id in data {
            logln!("{}", id);
        }
    }

    false
}
