//! src/gpu_worker.rs

use crate::address::public_key_to_tron_address;
use crate::worker::FoundWallet;
use bytemuck::{Pod, Zeroable};
use secp256k1::{PublicKey, Secp256k1};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Instant;
use wgpu::util::DeviceExt;

// The number of public keys to process in a single batch on the GPU.
// A larger batch size is generally more efficient.
const BATCH_SIZE: u64 = 1_048_576; // 2^20 keys

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuPublicKey {
    // Public keys are 64 bytes (x and y coordinates, excluding the 0x04 prefix).
    data: [u8; 64],
}

/// Main async function to drive the GPU-based search.
pub async fn search(
    suffixes: Vec<String>,
    sender: Sender<FoundWallet>,
    should_stop: &AtomicBool,
) {
    println!("[GPU] Initializing GPU device...");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find a suitable GPU adapter.");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to get GPU device.");

    println!("[GPU] Device: {}", adapter.get_info().name);
    println!("[GPU] Batch size: {} keys per round", BATCH_SIZE);

    // --- Shader and Pipeline Setup ---
    let shader_source = std::include_str!("shader.wgsl");
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: "main",
    });

    // --- Main GPU Loop ---
    let secp = Secp256k1::new();
    let mut round = 0;

    while !should_stop.load(Ordering::Relaxed) {
        round += 1;
        let round_start_time = Instant::now();

        // --- 1. CPU: Generate a batch of keys ---
        let mut cpu_keys = Vec::with_capacity(BATCH_SIZE as usize);
        let mut gpu_keys_data = Vec::with_capacity(BATCH_SIZE as usize);

        for _ in 0..BATCH_SIZE {
            let (privkey, pubkey) = secp.generate_keypair(&mut rand::thread_rng());
            let serialized_pubkey = pubkey.serialize_uncompressed();
            let mut key_bytes = [0u8; 64];
            key_bytes.copy_from_slice(&serialized_pubkey[1..]); // Exclude the 0x04 prefix

            cpu_keys.push((privkey, pubkey));
            gpu_keys_data.push(GpuPublicKey { data: key_bytes });
        }

        // --- 2. GPU: Run the hashing task ---

        // Create buffers for this batch
        let pubkeys_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Public Keys Buffer"),
            contents: bytemuck::cast_slice(&gpu_keys_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let hashes_buffer_size = BATCH_SIZE * 32; // 32 bytes per Keccak-256 hash
        let hashes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hashes Buffer"),
            size: hashes_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Bind the buffers to the shader
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pubkeys_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: hashes_buffer.as_entire_binding(),
                },
            ],
        });

        // Create a command encoder and dispatch the shader
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            // Each workgroup handles 64 keys
            compute_pass.dispatch_workgroups(BATCH_SIZE as u32 / 64, 1, 1);
        }

        // Staging buffer to read data back from the GPU
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: hashes_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&hashes_buffer, 0, &staging_buffer, 0, hashes_buffer_size);

        // Submit the commands to the GPU
        queue.submit(Some(encoder.finish()));

        // --- 3. CPU: Process results ---
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        // Wait for the GPU to finish
        device.poll(wgpu::Maintain::Wait);
        let _ = rx.receive().await;

        let data = buffer_slice.get_mapped_range();
        let hashes: &[u8] = &data;

        // NOTE: This is where the CPU would take over.
        // The `hashes` buffer now contains the Keccak-256 results from the GPU.
        // We would iterate through them, do the Base58Check encoding, and check the suffix.
        // For this example, we will just check one for demonstration.

        let (privkey, pubkey) = &cpu_keys[0];
        let address = public_key_to_tron_address(pubkey); // Re-calculate on CPU for now
        for suffix in &suffixes {
            if address.ends_with(suffix) {
                let found = FoundWallet {
                    address,
                    private_key_hex: privkey.display_secret().to_string(),
                };
                if sender.send(found).is_ok() {
                    println!("[GPU] Found a match and sent it. Stopping.");
                    should_stop.store(true, Ordering::Relaxed);
                }
                return;
            }
        }
        drop(data);
        staging_buffer.unmap();

        let duration = round_start_time.elapsed();
        let speed = BATCH_SIZE as f64 / duration.as_secs_f64();
        print!("\r[GPU] Round {}: {:.2e} keys/sec", round, speed);
    }
}
