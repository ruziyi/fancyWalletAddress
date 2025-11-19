# Tron Vanity Address Generator Makefile

# Use rustup-managed cargo to avoid conflicts with homebrew installation
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Default target: build a release version for the current host system.
build:
	@echo "Building release binary for the host system..."
	@cargo build --release

# --- Cross-compilation Targets ---
# These targets use Zig as a linker for simplified cross-compilation.
# Prerequisite: Install Zig (https://ziglang.org/download/) and ensure it's in your PATH.

# Build for Linux (x86_64)
linux:
	@echo "Building for Linux (x86_64-unknown-linux-gnu)..."
	@cargo zigbuild --release --target x86_64-unknown-linux-gnu
	@echo "Linux binary available at: target/x86_64-unknown-linux-gnu/release/fancy_wallet_address"

# Build for Windows (x86_64, GNU toolchain)
windows:
	@echo "Building for Windows (x86_64-pc-windows-gnu)..."
	@cargo zigbuild --release --target x86_64-pc-windows-gnu
	@echo "Windows binary available at: target/x86_64-pc-windows-gnu/release/fancy_wallet_address.exe"

# Build for macOS (Apple Silicon)
macos-aarch64:
	@echo "Building for macOS (aarch64-apple-darwin)..."
	@cargo build --release --target aarch64-apple-darwin
	@echo "macOS aarch64 binary available at: target/aarch64-apple-darwin/release/fancy_wallet_address"

# Build for macOS (Intel)
macos-x86_64:
	@echo "Building for macOS (x86_64-apple-darwin)..."
	@cargo build --release --target x86_64-apple-darwin
	@echo "macOS x86_64 binary available at: target/x86_64-apple-darwin/release/fancy_wallet_address"

# A generic 'macos' target that builds for the most common architectures.
# You can run 'make macos-aarch64' or 'make macos-x86_64' individually.
macos: macos-aarch64 macos-x86_64

# Clean up build artifacts.
clean:
	@cargo clean

.PHONY: build linux windows macos-aarch64 macos-x86_64 macos clean
