# Building and Running

## Prerequisites

Install Rust nightly and required components:

```bash
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview --toolchain nightly
cargo install bootimage
rustup component add rustfmt clippy --toolchain nightly
```

## Build the Kernel

Build the kernel for the custom target:

```bash
cargo build --target x86_64-zero.json
```

## Create a Bootable Disk Image

```bash
cargo bootimage
```

The generated image will be located at:

```text
target/x86_64-zero/debug/bootimage-zero.bin
```

## Run in QEMU

```bash
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-zero/debug/bootimage-zero.bin
```

## Development Checks

Check the project:

```bash
cargo check \
    -Zbuild-std=core,alloc,compiler_builtins \
    -Zbuild-std-features=compiler-builtins-mem
```

Run tests:

```bash
cargo test
```

