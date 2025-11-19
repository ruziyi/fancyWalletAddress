//! src/main.rs

mod address;
mod cli;
mod worker;

use crate::cli::Cli;
use crate::worker::search;
use clap::Parser;
use notify_rust::Notification;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

const BASE58_CHARS: f64 = 58.0;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    if cli.gpu {
        println!("⚠️ Warning: GPU acceleration is not yet implemented. Falling back to CPU.");
        // When implemented, this would call `worker::search_gpu(...)`
    }

    // --- Calculate and print expected attempts ---
    let total_prob: f64 = cli
        .suffixes
        .iter()
        .map(|s| 1.0 / BASE58_CHARS.powi(s.len() as i32))
        .sum();
    let expected_attempts = (1.0 / total_prob) * cli.count as f64;
    println!(
        "[*] Estimated attempts required: {:e} (to find {})",
        expected_attempts, cli.count
    );

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
    let attempts = Arc::new(AtomicU64::new(0));

    // --- Speed monitor thread (if requested) ---
    if cli.show_speed {
        let should_stop_clone = Arc::clone(&should_stop);
        let attempts_clone = Arc::clone(&attempts);
        thread::spawn(move || {
            let mut last_check_attempts = 0;
            let check_interval = Duration::from_secs(2);
            while !should_stop_clone.load(Ordering::Relaxed) {
                thread::sleep(check_interval);
                let current_attempts = attempts_clone.load(Ordering::Relaxed);
                let speed = (current_attempts - last_check_attempts) as f64 / check_interval.as_secs_f64();
                last_check_attempts = current_attempts;
                print!("\r[*] Speed: {} checks/sec", speed as u64);
                let _ = stdout().flush();
            }
        });
    }

    // Start the search in the background
    let suffixes = cli.suffixes.clone();
    let should_stop_clone = Arc::clone(&should_stop);
    let attempts_clone = Arc::clone(&attempts);
    thread::spawn(move || {
        search(
            suffixes,
            sender,
            &should_stop_clone,
            &attempts_clone,
            num_threads,
        );
    });

    // Main thread waits for results
    for found in receiver {
        let current_count = found_count.fetch_add(1, Ordering::SeqCst);

        // Clear the speed line before printing result
        if cli.show_speed {
            print!("\r{}", " ".repeat(40));
            println!();
        }

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
            // Give a moment for other threads to see the flag
            thread::sleep(Duration::from_millis(100));
            std::process::exit(0);
        } else {
            println!(
                "\n🔍 Continuing search for the next address on {} threads...",
                num_threads
            );
        }
    }
}