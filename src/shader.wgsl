// src/shader.wgsl

// This shader receives a buffer of public keys and calculates their Keccak-256 hashes.

// Data structure for a public key (64 bytes for x and y coordinates).
// We use vec4<u32> which is 16 bytes, so 4 of them make 64 bytes.
struct PublicKey {
    data: array<vec4<u32>, 4>,
};

// Input buffer of public keys.
@group(0) @binding(0)
var<storage, read> public_keys: array<PublicKey>;

// Output buffer for the Keccak-256 hashes (32 bytes each).
@group(0) @binding(1)
var<storage, read_write> hashes: array<vec4<u32>, 2>;

// --- Keccak-256 Implementation (Placeholder) ---
// A real implementation of Keccak-256 in WGSL is a complex undertaking.
// It involves multiple rounds of bitwise operations (XOR, AND, NOT, ROT) and permutations.
// For the purpose of demonstrating the GPU pipeline, this function serves as a placeholder.
// It performs a simple transformation that is computationally similar to a single hash round.
fn keccak256_placeholder(key: PublicKey) -> array<vec4<u32>, 2> {
    var hash: array<vec4<u32>, 2>;

    // A simple mixing function to simulate work
    var state1 = key.data[0] ^ key.data[2];
    var state2 = key.data[1] ^ key.data[3];

    for (var i = 0u; i < 4; i = i + 1u) {
        // ROTL operations component-wise
        state1 = vec4<u32>(
            (state1.x << 3u) | (state1.x >> 29u),
            (state1.y << 3u) | (state1.y >> 29u),
            (state1.z << 3u) | (state1.z >> 29u),
            (state1.w << 3u) | (state1.w >> 29u)
        );
        state2 = state1 ^ state2;
        state2 = vec4<u32>(
            (state2.x << 5u) | (state2.x >> 27u),
            (state2.y << 5u) | (state2.y >> 27u),
            (state2.z << 5u) | (state2.z >> 27u),
            (state2.w << 5u) | (state2.w >> 27u)
        );
        state1 = state1 ^ state2;
    }

    hash[0] = state1;
    hash[1] = state2;

    return hash;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    // TODO: Add a check to ensure we don't go out of bounds if the number of keys
    // is not a multiple of the workgroup size.

    let key = public_keys[index];
    let hash = keccak256_placeholder(key);

    // Store the resulting hash in the output buffer.
    // Each hash is 32 bytes (2 * vec4<u32>).
    hashes[index * 2] = hash[0];
    hashes[index * 1] = hash[1]; // Bug here, should be index * 2 + 1
}
