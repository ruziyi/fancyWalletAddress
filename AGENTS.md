# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` drives the CLI workflow defined in `src/cli.rs`, spawns CPU workers from `src/worker.rs`, and calls GPU helpers in `src/gpu_worker.rs` plus the `shader.wgsl` compute kernel. Address encoding lives in `src/address.rs` (with inline tests), while platform glue stays in `build.rs`. Release binaries land under `target/`, and all contributor-facing docs remain in `README.md`.

## Build, Test, and Development Commands
- `make build` (or `cargo build --release`) produces the optimized binary at `target/release/fancy_wallet_address` for the host OS.
- `make linux|windows|macos-aarch64|macos-x86_64` cross-compiles via Zig or platform toolchains; call `zig` through `cargo zigbuild` when targeting non-host platforms.
- `cargo run --release -- --suffixes Tron` runs the CLI end-to-end; adjust the suffix list, `--count`, and `--threads` to mirror your scenario.
- `cargo clean` resets the workspace before benchmarking or cross-compiling.

## Coding Style & Naming Conventions
Rust sources follow `rustfmt` defaults: four-space indentation, trailing commas for multi-line literals, and imports grouped by crate. Keep modules and files in `snake_case`, structs/enums in `UpperCamelCase`, and CLI flags in kebab-case (e.g., `--gpu`). Prefer descriptive names over abbreviations, return `Result<T, E>` with `anyhow`-style error context where possible, and gate GPU-only code with `#[cfg(feature = "gpu")]`.

## Testing Guidelines
Unit tests live beside the code (see `src/address.rs`). Add new tests in the same file or in `tests/` if you need integration coverage. Run `cargo test --all-targets` before sending a PR, and include `cargo test --all-targets --features gpu` whenever touching `gpu_worker.rs` or shader logic. Name tests with `test_*` prefixes that describe the behavior (e.g., `test_base58_checksum_matches`).

## Commit & Pull Request Guidelines
The history adopts Conventional Commits (`feat(worker): parallelize suffix search`). Keep subject lines under 72 characters and scope them to the primary module. Each PR should include: a summary of changes, output from `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test --all-targets[ --features gpu]`, plus any relevant CLI sample output or screenshots showing the found address format. Reference an issue or discussion when applicable and call out platform-specific considerations (e.g., macOS linker notes from `build.rs`).

## Security & Configuration Tips
Never commit private keys or suffix lists that contain production secrets; prefer `.gitignore`d scratch files. The `--gpu` flag is optional—guard unfinished features behind `#[cfg(feature = "gpu")]` and ensure fallbacks remain safe on CPU-only hosts. When cross-compiling, confirm Zig is up-to-date and check that macOS targets link the `CoreServices` and `AppKit` frameworks as configured in `build.rs`.
