//! src/worker.rs

use crate::address::public_key_to_tron_address;
use rayon::prelude::*;
use secp256k1::{rand, Secp256k1};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc::Sender, Arc};

/// A struct to hold the result of a successful search.
pub struct FoundWallet {
    pub address: String,
    pub private_key_hex: String,
}

/// Pre-processes suffixes into a HashMap grouped by length for faster matching.
fn group_suffixes_by_length(suffixes: &[String]) -> HashMap<usize, Vec<&String>> {
    let mut map: HashMap<usize, Vec<&String>> = HashMap::new();
    for suffix in suffixes {
        map.entry(suffix.len()).or_default().push(suffix);
    }
    map
}

/// The main search function that runs on multiple threads.
///
/// It continuously generates keypairs, converts them to Tron addresses, and checks
/// if they match any of the desired suffixes. Once a match is found, it's sent
/// through the `sender` channel. The search can be stopped by the `should_stop` flag.
pub fn search(
    suffixes: Vec<String>,
    sender: Sender<FoundWallet>,
    should_stop: &AtomicBool,
    attempts: &Arc<AtomicU64>,
) {
    // Pre-process suffixes for efficient lookup.
    let suffixes_by_len = group_suffixes_by_length(&suffixes);
    let suffix_lengths: Vec<usize> = suffixes_by_len.keys().cloned().collect();

    // Use Rayon's parallel iterator to search across all available CPU cores.
    (0..u64::MAX).into_par_iter().for_each_with(sender, |s, _| {
        // If the main thread signals to stop, exit the loop.
        if should_stop.load(Ordering::Relaxed) {
            return;
        }

        // Increment the attempt counter for every iteration.
        attempts.fetch_add(1, Ordering::Relaxed);

        // Generate a new wallet (keypair and address).
        // This is the hot loop.
        let secp = Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let address = public_key_to_tron_address(&public_key);

        // Efficiently check for suffix matches.
        for &len in &suffix_lengths {
            if address.len() >= len {
                let address_suffix = &address[address.len() - len..];
                if let Some(candidates) = suffixes_by_len.get(&len) {
                    for &suffix in candidates {
                        if address_suffix == suffix {
                            // Match found! Send the result back to the main thread.
                            let found = FoundWallet {
                                address: address.clone(),
                                private_key_hex: private_key.display_secret().to_string(),
                            };
                            if s.send(found).is_ok() {
                                // If send is successful, this thread's job might be done.
                                // The main thread will coordinate stopping.
                            }
                            // A single wallet can't match multiple suffixes in the same check,
                            // so we can break early.
                            return;
                        }
                    }
                }
            }
        }
    });
}

// TODO: GPU Acceleration Implementation
//
// The `--gpu` flag is detected in `main.rs`. If enabled, a different search function
// like `search_gpu` would be called.
//
// A potential `wgpu` implementation would look something like this:
//
// 1.  **Shader (WGSL):**
//     -   Write a compute shader that takes a large batch of random seeds as input.
//     -   Each shader invocation would perform:
//         -   secp256k1 private key generation from the seed.
//         -   secp256k1 public key derivation (this is the hardest part, may require pre-computed tables or a pure WGSL implementation).
//         -   Keccak-256 hash.
//         -   Address formatting and Base58 encoding (or a simplified check).
//         -   Suffix matching.
//     -   Write found keypairs/addresses to an output buffer.
//
// 2.  **Rust `wgpu` Host Code:**
//     -   Initialize `wgpu` adapter and device.
//     -   Create buffers for random seeds (input) and results (output).
//     -   Create a `ComputePipeline` with the compiled WGSL shader.
//     -   In a loop:
//         -   Fill the input buffer with fresh random data.
//         -   Dispatch the compute shader.
//         -   Read the results back from the output buffer.
//         -   Process and print any found wallets.
//
// pub fn search_gpu(
//     suffixes: Vec<String>,
//     sender: Sender<FoundWallet>,
//     should_stop: &AtomicBool,
// ) {
//     // 1. Setup wgpu device, queue, and pipeline.
//     // 2. Create buffers for input (random seeds) and output (found wallets).
//     // 3. Loop:
//     //    a. Generate a large batch of random seeds on the CPU.
//     //    b. Write seeds to the GPU input buffer.
//     //    c. Dispatch the compute shader.
//     //    d. Read the output buffer back to the CPU.
//     //    e. If any wallets were found, send them via the `sender`.
//     //    f. Check `should_stop` flag.
//     unimplemented!("GPU acceleration is not yet implemented.");
// }
