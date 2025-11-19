//! src/worker.rs

use crate::address::public_key_to_tron_address;
use secp256k1::{rand, Secp256k1};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc::Sender, Arc};

/// A struct to hold the result of a successful search.
pub struct FoundWallet {
    pub address: String,
    pub private_key_hex: String,
}

/// Pre-processes suffixes into a HashMap of string slices grouped by length.
fn group_suffixes_by_length(suffixes: &[String]) -> HashMap<usize, Vec<&String>> {
    let mut map: HashMap<usize, Vec<&String>> = HashMap::new();
    for suffix in suffixes {
        map.entry(suffix.len()).or_default().push(suffix);
    }
    map
}

/// The main search function, optimized to reuse contexts.
///
/// It uses `rayon::scope` to create a pool of long-running worker threads.
/// Each thread initializes its `Secp256k1` context and RNG once, which provides
/// a major performance boost.
pub fn search(
    suffixes: Vec<String>,
    sender: Sender<FoundWallet>,
    should_stop: &AtomicBool,
    attempts: &Arc<AtomicU64>,
    num_threads: usize,
) {
    // Pre-process suffixes for efficient lookup.
    let suffixes_by_len = Arc::new(group_suffixes_by_length(&suffixes));
    let suffix_lengths: Arc<Vec<usize>> = Arc::new(suffixes_by_len.keys().cloned().collect());

    rayon::scope(|s| {
        for _ in 0..num_threads {
            // Clone Arcs for each thread
            let sender_clone = sender.clone();
            let attempts_clone = attempts.clone();
            let suffixes_by_len_clone = Arc::clone(&suffixes_by_len);
            let suffix_lengths_clone = Arc::clone(&suffix_lengths);

            s.spawn(move |_| {
                // --- Per-thread Initialization (Major Optimization) ---
                let secp = Secp256k1::new();
                let mut rng = rand::thread_rng();
                // ----------------------------------------------------

                // Inner hot loop
                loop {
                    if should_stop.load(Ordering::Relaxed) {
                        break;
                    }

                    attempts_clone.fetch_add(1, Ordering::Relaxed);

                    // Generate keys and the full address string
                    let (private_key, public_key) = secp.generate_keypair(&mut rng);
                    let address = public_key_to_tron_address(&public_key);

                    // Check for suffix matches on the generated string.
                    for &len in &*suffix_lengths_clone {
                        if address.len() >= len {
                            if let Some(candidates) = suffixes_by_len_clone.get(&len) {
                                for suffix in candidates {
                                    if address.ends_with(*suffix) {
                                        // --- Match Found! ---
                                        let found = FoundWallet {
                                            address, // Move the address
                                            private_key_hex: private_key.display_secret().to_string(),
                                        };
                                        // Send the result and stop searching on this thread.
                                        let _ = sender_clone.send(found);
                                        return; // Exit thread
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });
}
