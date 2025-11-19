# Tron Vanity Address Generator

A high-performance, cross-platform command-line tool for generating Tron (TRX) vanity addresses, written in Rust.

## Features

-   **High-Performance**: Utilizes all available CPU cores for parallel searching.
-   **Custom Suffixes**: Find addresses ending with one or more desired strings.
-   **Cross-Platform**: Compiles and runs on macOS, Linux, and Windows.
-   **GPU Ready**: Includes a placeholder `--gpu` flag and code structure for future GPU acceleration.
-   **Desktop Notifications**: Notifies you when a matching address is found.

## Build Instructions

### Prerequisites

-   **Rust Toolchain**: Install from [rustup.rs](https://rustup.rs/).
-   **(Optional) Zig**: For easy cross-compilation, install Zig from [ziglang.org](https://ziglang.org/download/).

### Standard Build (for your current system)

To build an optimized binary for your machine:

```bash
make build
# The executable will be at target/release/fancy_wallet_address
```

### Cross-compilation (requires Zig)

Make sure `zig` is in your `PATH`.

-   **Build for Linux (x86_64):**
    ```bash
    make linux
    ```

-   **Build for Windows (x86_64):**
    ```bash
    make windows
    ```

-   **Build for macOS:**
    ```bash
    # For Apple Silicon
    make macos-aarch64

    # For Intel
    make macos-x86_64
    ```

## Usage

The tool requires a list of suffixes to search for.

### Basic Example

Find one address ending in `8888`:

```bash
./target/release/fancy_wallet_address --suffixes 8888
```

### Multiple Suffixes and Count

Find 5 addresses, ending in either `6666` or `COOL`:

```bash
./target/release/fancy_wallet_address --suffixes 6666,COOL --count 5
```

### Specify Thread Count

Use 16 threads to search for an address ending in `Tron`:

```bash
./target/release/fancy_wallet_address --suffixes Tron --threads 16
```

### GPU Acceleration (Future)

The `--gpu` flag is reserved for future implementation. Currently, it will show a warning and proceed with CPU-based generation.

```bash
./target/release/fancy_wallet_address --suffixes GPU --gpu
```

### Example Output

```
Searching for addresses ending with: ["8888"] on 10 threads...
Found a match!
----------------------------------------
Address:      T...<some_prefix>...8888
Private Key:  <hex_private_key>
----------------------------------------
```
