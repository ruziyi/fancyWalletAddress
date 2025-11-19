// build.rs

fn main() {
    // This script is only relevant when compiling on macOS.
    if cfg!(target_os = "macos") {
        // The `notify-rust` crate's dependency `mac-notification-sys` requires
        // linking against the `CoreServices` and `AppKit` frameworks on macOS.
        // This script adds the necessary linker flags.
        println!("cargo:rustc-link-lib=framework=CoreServices");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
