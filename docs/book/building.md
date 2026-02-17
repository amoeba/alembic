# Building

## Prerequisites

This project is written in Rust.

### Windows

Install the 32-bit MSVC target:

```sh
rustup target add i686-pc-windows-msvc
```

### Linux

Install MinGW toolchain for cross-compiling Windows binaries:

```sh
# Debian/Ubuntu
sudo apt-get install gcc-mingw-w64-i686

# Add the Rust target
rustup target add i686-pc-windows-gnu
```

### All Platforms

Install cargo-make for the build system:

```sh
cargo install cargo-make
```

## Build Instructions

### Quick Start

Use cargo-make to build everything:

```sh
# Build all components (32-bit and native)
cargo make build

# Run the desktop app
cargo run --bin desktop
```

The build task automatically:
1. Builds cork and dll for 32-bit Windows (i686-pc-windows-msvc)
2. Builds desktop and cli for your native architecture

### Manual Build

If you prefer to build manually:

```sh
# Build 32-bit components first
cargo build --target i686-pc-windows-msvc -p cork -p dll

# Then build native components
cargo build -p desktop -p cli
```

## Build Artifacts

Build artifacts are placed in the `target/` directory. The layout is:

- `target/i686-pc-windows-msvc/` - 32-bit Windows binaries
- `target/debug/` or `target/release/` - Native platform binaries
