# rOSt, a 64-Bit Rust operating system

[![Build the project](https://github.com/michalusio/rOSt/actions/workflows/rust-pr.yml/badge.svg?branch=main)](https://github.com/michalusio/rOSt/actions/workflows/rust-pr.yml)
![Rust nightly](https://img.shields.io/badge/status-nightly-important)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

rOSt is an experimental 64-bit x86_64 operating system written in Rust.
It targets BIOS and UEFI, features a higher-half kernel and custom memory management, and serves as a research and learning project.

Documentation: [Wiki](https://github.com/michalusio/rOSt/wiki).

Contributing: [CONTRIBUTING.md](https://github.com/michalusio/rOSt/blob/main/.github/CONTRIBUTING.md).

### Structure

The project is divided into multiple folders:

1. [src](src/) contains the QEMU setup.
2. [kernel](kernel/) contains the actual OS kernel, binding everything together.
3. [internal_utils](internal_utils/) contains utility functions, constants and structures that are used throughout the kernel and drivers.
4. [drivers](drivers/) contains drivers that add extended functionality that is not in the scope of the kernel core, for example VGA and ATA support.

### Requirements

- [Rust](https://www.rust-lang.org/) using the nightly channel
- [llvm-tools-preview](https://docs.rs/llvm-tools/latest/llvm_tools/) (installed via `rustup component add llvm-tools-preview`)
- [QEMU](https://www.qemu.org/)

Rust should automatically switch to the nightly channel and install the llvm tools when it detects the `rust-toolchain.toml`.

## How to run

```bash
cargo run bios
```

Or if you want to run an UEFI image:

```bash
cargo run uefi
```

The command will build the kernel and start up a qemu instance, booting the kernel in debug mode.

### Architecture

- We want to achieve a Microkernel in the end
- for now x86_64 only

### Feature map

Legend:
✔️ - Done
🔨 - In Progress
⭕ - Not Done Yet
❌ - Probably won't be supported

- IO
  - ✔️ Framebuffer output
  - ✔️ Serial output and input (COM1)
  - ✔️ Basic logging macros
  - ⭕ Keyboard input (polling or IRQ-based)
  - ⭕ PS/2 mouse support
  - ⭕ Simple shell
  - ⭕ Pipes and redirection
  - 🔨 Block device abstraction
  - ⭕ VFS layer
  - 🔨 Filesystems
    - 🔨 Tag-Based Entity System ([TBES](/docs/Tag-Based%20Entity%20System.md))
    - ⭕ FAT32
    - ⭕ ext2
    - ❌ ext4

- Memory
  - ✔️ Physical frame allocator (2-level bitmap allocator)
  - 🔨 Paging
    - ✔️ Higher-half kernel
    - ⭕ Demand paging
    - ✔️ Identity mapping during boot
  - ✔️ Kernel heap allocator
  - ⭕ Per-process address spaces
  - ⭕ Copy-on-write
  - ⭕ Memory-mapped files
  - ✔️ Guard pages
  - ⭕ Slab allocator
  - ⭕ NUMA awareness
  - ⭕ Huge page support

- Processes
  - 🔨 Preemptive scheduler (timer IRQ driven)
    - ⭕ Round-robin scheduling
    - ⭕ Priority scheduler
  - ⭕ User mode (ring 3)
  - ✔️ Context switching
  - ⭕ ELF loader
  - ⭕ Process isolation
  - ✔️ Threads (kernel + user)
  - ⭕ IPC primitives (message passing, shared memory)
  - ⭕ Signals

- Syscalls
  - ⭕ Syscall entry via `syscall`/`sysret`
  - ⭕ Basic POSIX-like API
    - ❌ Full POSIX compliance
  - ⭕ Capability-based syscall model
  - ⭕ Async syscall support
  - ⭕ Stable syscall ABI versioning

- Drivers
  - ✔️ VGA
  - ✔️ Serial (16550 UART)
  - 🔨 PIT/APIC timer
  - 🔨 Keyboard controller
  - ✔️ ATA PIO
  - ⭕ APIC / IOAPIC full support
  - ⭕ HPET timer
  - ⭕ AHCI / NVMe
  - ⭕ PCI bus enumeration
  - ⭕ Network card
  - ⭕ USB (UHCI/EHCI/XHCI)
  - ⭕ ACPI parsing

- Interrupts & CPU
  - ✔️ IDT setup
  - ✔️ TSS with proper privilege stacks
  - 🔨 Exception handlers
  - 🔨 Timer interrupt
  - 🔨 PIC remapping
  - ⭕ FPU/SIMD context switching
  - ⭕ SMP support
  - ⭕ Per-core structures
  - ⭕ Fast syscall path

### Troubleshooting

- If the build fails due to unstable features, make sure that you have enabled the nightly channel using `rustup default nightly` or `rustup upgrade`

<sub><a href="https://iconscout.com/icons/processor-chip" target="_blank">Processor Chip Icon</a> by <a href="https://iconscout.com/contributors/kolo-design" target="_blank">Kalash</a></sub>
