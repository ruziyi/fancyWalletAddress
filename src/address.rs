//! src/address.rs

use secp256k1::PublicKey;
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

/// Generates a Tron address from a secp256k1 public key.
pub fn public_key_to_tron_address(pk: &PublicKey) -> String {
    // 1. Get uncompressed public key (65 bytes: 0x04 + 32 bytes X + 32 bytes Y)
    let pk_uncompressed = pk.serialize_uncompressed();

    // 2. Keccak-256 hash of the public key excluding the prefix 0x04
    let mut keccak = Keccak::v256();
    keccak.update(&pk_uncompressed[1..]);
    let mut hashed_pk = [0u8; 32];
    keccak.finalize(&mut hashed_pk);

    // 3. Take the last 20 bytes and prepend Tron address prefix
    let mut address_payload = [0u8; 21];
    address_payload[0] = 0x41;
    address_payload[1..].copy_from_slice(&hashed_pk[12..]);

    // 4. Create checksum by double-SHA256
    let mut hasher1 = Sha256::new();
    hasher1.update(&address_payload);
    let hash1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(&hash1);
    let hash2 = hasher2.finalize();

    let checksum = &hash2[0..4];

    // 5. Append checksum to payload
    let mut final_payload = Vec::with_capacity(25);
    final_payload.extend_from_slice(&address_payload);
    final_payload.extend_from_slice(checksum);

    // 6. Base58 encode the final payload
    bs58::encode(final_payload).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_known_private_key_to_address() {
        // A known private key and its corresponding Tron address for validation.
        let private_key_hex = "d2dc029911480a74c6e08fea54223434bc86a4514a69c3c0d942433d5a37c328";
        let expected_address = "TJRyWwFs9wTFGZg3JbrVriV5incfS2Qd2s";

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_str(private_key_hex).expect("Failed to parse private key");
        let public_key = secret_key.public_key(&secp);

        let generated_address = public_key_to_tron_address(&public_key);

        assert_eq!(generated_address, expected_address);
    }
}

