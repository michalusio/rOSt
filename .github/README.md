# rOSt, a 64-Bit Rust operating system

For more information about the project, please visit the [wiki](https://github.com/michalusio/rOSt/wiki), this readme is meant to give a quick overview of the project for developers and anyone interested.

If you are interested in contributing to the project, please visit the [Contributing file](https://github.com/michalusio/rOSt/blob/main/.github/CONTRIBUTING.md).

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

### Troubleshooting

- If the build fails because of usage of unstable features, make sure that you have enabled the nightly channel using `rustup default nightly` or `rustup upgrade`

<a href="https://iconscout.com/icons/processor-chip" target="_blank">Processor Chip Icon</a> by <a href="https://iconscout.com/contributors/kolo-design" target="_blank">Kalash</a>
