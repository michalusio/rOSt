# rOSt, a 64-Bit Rust operating system

For more information about the project, please visit the [wiki](https://github.com/0xffset/rOSt/wiki), this readme is meant to give a quick overview of the project for developers and anyone interested.

If you are interested in contributing to the project, please visit the [Contributing file](https://github.com/0xffset/rOSt/blob/main/.github/CONTRIBUTING.md).

### Structure

The project is divided into multiple folders:

1. [src](src/) contains the main entry point of the kernel.
2. [boot](boot/) contains the settings for building the image with the bootloader, and QEMU settings.
3. [utils](utils/) contains utility functions, constants and structures that could be used throughout the kernel.
4. [drivers](drivers/) contains drivers that add extended functionality that is not in the scope of the kernel core.
5. [kernel](kernel/) contains the core library and functionality.

### Requirements

- [Rust](https://www.rust-lang.org/) using the nightly channel
- [llvm-tools-preview](https://docs.rs/llvm-tools/latest/llvm_tools/) (installed via `rustup component add llvm-tools-preview`)
- [QEMU](https://www.qemu.org/)

Rust should automatically switch to the nightly channel and install the llvm tools when it detects the `rust-toolchain.toml`.

## How to run

```bash
cargo krun
```

will build the kernel and start up a qemu instance booting the kernel in debug mode.

## Testing

Tests are ran after the kernel initializes the necessities like kernel heap, general memory management and interrupts.

To run the tests do:

```bash
cargo ktest
```

### Troubleshooting

- If the build fails because of usage of unstable features, make sure that you have enabled the nightly channel using `rustup default nightly` or `rustup upgrade`

<a href="https://iconscout.com/icons/processor-chip" target="_blank">Processor Chip Icon</a> by <a href="https://iconscout.com/contributors/kolo-design" target="_blank">Kalash</a>
