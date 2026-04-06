use core::{arch::asm, str::SplitWhitespace};

use alloc::borrow::Cow;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crosstrait::Cast;
use internal_utils::HexNumber;

use internal_utils::tag_store::{
    BoolQueryExpression, BoolQueryExpressionType, BooleanTag, IntegerTag, Query, QueryOptions,
    QueryResult, TAG_STORE, U64QueryExpression, U64QueryExpressionType,
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
        let result = f(&mut arguments);
        match result {
            Err(s) => {
                logln!("{}", s);
                false
            }
            Ok(f) => f,
        }
    } else {
        logln!("Invalid command. Try \"help\".");
        false
    }
}

type Arguments<'a> = &'a mut SplitWhitespace<'a>;
type StaticFunction =
    &'static (dyn (Fn(Arguments) -> Result<bool, Cow<'static, str>>) + Send + Sync);

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

fn help(args: Arguments) -> Result<bool, Cow<'static, str>> {
    if args.next().is_some() {
        return Err("help does not accept arguments".into());
    }
    logln!("Available commands:");
    for (c, _) in COMMANDS {
        logln!("- {}", c);
    }
    Ok(false)
}

fn memory(args: Arguments) -> Result<bool, Cow<'static, str>> {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        match subcommand {
            "info" => {
                print_memory(kernel_info.allocator);
                Ok(false)
            }
            "view" | "viewp" | "viewk" => {
                if let Some((from, to)) = get_from_to(args) {
                    if subcommand == "view" {
                        view_memory_slice(from, to, 0);
                    } else if subcommand == "viewp" {
                        view_memory_slice(from, to, kernel_info.physical_memory_offset);
                    } else {
                        view_memory_slice(from, to, addressing::ADDRESSES[3]);
                    }
                    Ok(false)
                } else {
                    Err("You need to pass a from:to range".into())
                }
            }
            _ => Err("Invalid subcommand".into()),
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
        Ok(false)
    }
}

fn exit(args: Arguments) -> Result<bool, Cow<'static, str>> {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        match subcommand {
            "qemu" => exit_qemu(),
            "ikd" => Ok(true),
            _ => Err("Invalid subcommand".into()),
        }
    } else {
        logln!("exit subcommands:");
        logln!("- {:<20} | Closes QEMU (if applicable)", "qemu");
        logln!("- {:<20} | Closes IKD (if possible)", "ikd");
        Ok(false)
    }
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

fn kernel(args: Arguments) -> Result<bool, Cow<'static, str>> {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        let kernel_info = KERNEL_INFORMATION.get().unwrap();
        match subcommand {
            "info" => {
                kernel_info.print();
                Ok(false)
            }
            _ => Err("Invalid subcommand".into()),
        }
    } else {
        logln!("kernel subcommands:");
        logln!("- {:<20} | Shows kernel information", "info");
        Ok(false)
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
        for _ in 0..(16 - (index & 15)) {
            log!("   ");
        }
        logln!(
            "| {}",
            String::from_iter(buffer.iter().take((index & 15) as usize))
        );
    }
}

fn clocks(args: Arguments) -> Result<bool, Cow<'static, str>> {
    if args.next().is_some() {
        Err("clocks does not accept arguments".into())
    } else {
        logln!("Ticks: {}", get_current_tick());
        logln!("RTC Time: {}", get_current_time());
        Ok(false)
    }
}

fn ip(args: Arguments) -> Result<bool, Cow<'static, str>> {
    if args.next().is_some() {
        Err("ip does not accept arguments".into())
    } else {
        let ip = read_rip();
        logln!("Current instruction pointer: {}", ip.to_separated_hex());
        logln!("Though tbh it's kinda useless");
        Ok(false)
    }
}

fn panic(_: Arguments) -> Result<bool, Cow<'static, str>> {
    panic!("Invoked the panic handler");
}

fn scheduler(args: Arguments) -> Result<bool, Cow<'static, str>> {
    let subcommand = args.next();
    if let Some(subcommand) = subcommand {
        match subcommand {
            "processes" => {
                SCHEDULER.lock().unwrap().get_processes_and_threads().log();
                Ok(false)
            }
            "run" => run_processes(),
            _ => Err("Invalid subcommand".into()),
        }
    } else {
        logln!("scheduler subcommands:");
        logln!("- {:<20} | Shows processes", "processes");
        logln!("- {:<20} | Runs the scheduler", "run");
        Ok(false)
    }
}

fn tbes(args: Arguments) -> Result<bool, Cow<'static, str>> {
    let store = TAG_STORE.get().unwrap();
    let tag_map = store.get_all_tags();

    let mut show_query_plan = false;
    let checks: Vec<Query> = args
        .inspect(|arg| {
            if *arg == "-q" {
                show_query_plan = true;
            }
        })
        .filter(|arg| *arg != "-q")
        .map(|arg| {
            arg.split_once(['=', '>', '<'])
                .ok_or(Cow::Borrowed("Tag condition has to be of the form TAG_NAME=VALUE"))
                .map(|(name, value)| if let Some(stripped) = value.strip_prefix('=') {
                    (name, stripped)
                } else {
                    (name, value)
                })
                .and_then(|(name, value)| match tag_map.get(name) {
                    Some(tag) => {
                        let bt: Option<Arc<dyn BooleanTag>> = tag.clone().cast();
                        let it: Option<Arc<dyn IntegerTag>> = tag.clone().cast();
                        if let Some(boolean_tag) = bt {
                            if !arg.contains('=') || arg.contains(['<', '>']) {
                                Err("Boolean Tag condition has to be of the form TAG_NAME=TRUE/FALSE".into())
                            } else if value.eq_ignore_ascii_case("true") {
                                Ok(BoolQueryExpression {
                                    first: boolean_tag,
                                    operation: BoolQueryExpressionType::EqualTo,
                                    second: true
                                }.into())
                            } else if value.eq_ignore_ascii_case("false") {
                                Ok(BoolQueryExpression {
                                    first: boolean_tag,
                                    operation: BoolQueryExpressionType::EqualTo,
                                    second: false
                                }.into())
                            } else {
                                Err("Boolean Tag condition has to be of the form TAG_NAME=TRUE/FALSE".into())
                            }
                        } else if let Some(int_tag) = it {
                            let integer = value.parse::<u64>();
                            match integer {
                                Ok(int) => Ok(U64QueryExpression {
                                    first: int_tag,
                                    operation: match (arg.contains('='), arg.contains('>'), arg.contains('<')) {
                                        (true, true, false) => U64QueryExpressionType::GreaterThanOrEqualTo,
                                        (true, false, true) => U64QueryExpressionType::LessThanOrEqualTo,
                                        (false, true, true) => U64QueryExpressionType::NotEqualTo,
                                        (false, true, false) => U64QueryExpressionType::GreaterThan,
                                        (false, false, true) => U64QueryExpressionType::LessThan,
                                        _ => U64QueryExpressionType::EqualTo
                                    },
                                    second: int
                                }.into()),
                                Err(_) => Err("Integer Tag condition has to be of the form TAG_NAME=<integer>".into()),
                            }
                        } else {
                            Err("You cannot query ref tags using IKD (yet!)".into())
                        }
                    },
                    None => {
                        let s = format!("Cannot find tag {}", name);
                        Err(s.into())
                    }
                })
        })
        .map(|q| q.map(Query::Binary))
        .collect::<Result<_, Cow<'static, str>>>()?;

    let query = Query::And(checks);
    let QueryResult {
        identities,
        query_plan,
    } = store.query(query, QueryOptions { show_query_plan });
    if let Some(plan) = query_plan {
        logln!("Query plan: {}", plan);
    }
    for id in identities {
        logln!("{}", id);
    }
    Ok(false)
}
