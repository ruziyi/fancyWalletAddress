//! src/worker.rs

use crate::address::public_key_to_tron_address;
use secp256k1::{rand, Secp256k1};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc::Sender, Arc};

/// A struct to hold the result of a successful search.
pub struct FoundWallet {
    pub address: String,
    pub private_key_hex: String,
}

struct SuffixGroup<'a> {
    len: usize,
    values: Vec<&'a [u8]>,
}

/// Pre-process suffixes into sorted buckets of bytes to minimize hot-loop overhead.
fn group_suffixes_by_length(suffixes: &[String]) -> Vec<SuffixGroup<'_>> {
    let mut groups = Vec::<SuffixGroup>::new();

    for suffix in suffixes {
        let len = suffix.len();
        match groups.iter_mut().find(|g| g.len == len) {
            Some(group) => group.values.push(suffix.as_bytes()),
            None => groups.push(SuffixGroup {
                len,
                values: vec![suffix.as_bytes()],
            }),
        }
    }

    groups.sort_by_key(|g| g.len);
    groups
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
    let suffix_groups = Arc::new(group_suffixes_by_length(&suffixes));

    rayon::scope(|s| {
        for _ in 0..num_threads {
            // Clone Arcs for each thread
            let sender_clone = sender.clone();
            let attempts_clone = attempts.clone();
            let suffix_groups_clone = Arc::clone(&suffix_groups);

            s.spawn(move |_| {
                // --- Per-thread Initialization (Major Optimization) ---
                let secp = Secp256k1::new();
                let mut rng = rand::thread_rng();
                // ----------------------------------------------------
                let mut local_attempts: u64 = 0;

                // Inner hot loop
                loop {
                    if should_stop.load(Ordering::Relaxed) {
                        break;
                    }

                    local_attempts += 1;
                    if local_attempts == 1024 {
                        attempts_clone.fetch_add(local_attempts, Ordering::Relaxed);
                        local_attempts = 0;
                    }

                    // Generate keys and the full address string
                    let (private_key, public_key) = secp.generate_keypair(&mut rng);
                    let address = public_key_to_tron_address(&public_key);
                    let address_bytes = address.as_bytes();
                    let addr_len = address_bytes.len();

                    // Check for suffix matches on the generated string.
                    for group in suffix_groups_clone.iter() {
                        if addr_len < group.len {
                            continue;
                        }

                        let suffix_slice = &address_bytes[addr_len - group.len..];
                        for &candidate in &group.values {
                            if suffix_slice == candidate {
                                if local_attempts > 0 {
                                    attempts_clone.fetch_add(local_attempts, Ordering::Relaxed);
                                }
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

                if local_attempts > 0 {
                    attempts_clone.fetch_add(local_attempts, Ordering::Relaxed);
                }
            });
        }
    });
}
