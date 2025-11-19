// build.rs

fn main() {
    // This script is only relevant when compiling for macOS.
    // Use env::var to check the target OS, not the host OS.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        // The `notify-rust` crate's dependency `mac-notification-sys` requires
        // linking against the `CoreServices` and `AppKit` frameworks on macOS.
        // This script adds the necessary linker flags.
        println!("cargo:rustc-link-lib=framework=CoreServices");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
