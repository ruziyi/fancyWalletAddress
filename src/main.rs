//! src/main.rs

mod address;
mod cli;
mod worker;

use crate::cli::Cli;
use crate::worker::search;
use clap::Parser;
use notify_rust::Notification;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    if cli.gpu {
        println!("⚠️ Warning: GPU acceleration is not yet implemented. Falling back to CPU.");
        // When implemented, this would call `worker::search_gpu(...)`
    }

    // Determine the number of threads to use
    let num_threads = cli.threads.unwrap_or_else(num_cpus::get);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    println!(
        "🔍 Searching for addresses ending with: {:?} on {} threads...",
        cli.suffixes, num_threads
    );

    // Setup for communication between threads
    let (sender, receiver) = mpsc::channel();
    let found_count = AtomicUsize::new(0);
    let should_stop = Arc::new(AtomicBool::new(false));

    // Start the search in the background
    let suffixes = cli.suffixes.clone();
    let should_stop_clone = Arc::clone(&should_stop);
    thread::spawn(move || {
        search(suffixes, sender, &should_stop_clone);
    });

    // Main thread waits for results
    for found in receiver {
        let current_count = found_count.fetch_add(1, Ordering::SeqCst);

        // Print the found wallet details
        println!("\n🎉 Found a match! ({}/{})", current_count + 1, cli.count);
        println!("----------------------------------------");
        println!("Address:      {}", found.address);
        println!("Private Key:  {}", found.private_key_hex);
        println!("----------------------------------------");

        // Send a desktop notification
        if let Err(e) = Notification::new()
            .summary("Tron Vanity Address Found!")
            .body(&format!("Address: {}", found.address))
            .timeout(Duration::from_secs(10))
            .show()
        {
            eprintln!("Failed to send notification: {}", e);
        }

        // Check if we have found enough addresses
        if current_count + 1 >= cli.count {
            println!("\n✅ Desired count reached. Exiting.");
            should_stop.store(true, Ordering::Relaxed);
            // The program will exit once the background threads see the flag.
            // We can exit the main thread immediately.
            std::process::exit(0);
        } else {
             println!(
                "\n🔍 Continuing search for the next address on {} threads...",
                num_threads
            );
        }
    }
}