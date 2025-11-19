//! src/address.rs

use secp256k1::PublicKey;
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

/// Generates a Tron address from a secp256k1 public key.
///
/// # Process:
/// 1. Take the uncompressed public key (65 bytes, starting with 0x04).
/// 2. Perform a Keccak-256 hash on the public key bytes *excluding* the leading 0x04.
/// 3. Take the last 20 bytes of the Keccak-256 hash.
/// 4. Prepend the Tron address prefix `0x41`.
/// 5. Base58Check encode the resulting 21 bytes.
pub fn public_key_to_tron_address(pk: &PublicKey) -> String {
    // 1. Get uncompressed public key (65 bytes: 0x04 + 32 bytes X + 32 bytes Y)
    let pk_uncompressed = pk.serialize_uncompressed();

    // 2. Keccak-256 hash of the public key excluding the prefix 0x04
    let mut keccak = Keccak::v256();
    keccak.update(&pk_uncompressed[1..]);
    let mut hashed_pk = [0u8; 32];
    keccak.finalize(&mut hashed_pk);

    // 3. Take the last 20 bytes
    let mut address_payload = [0u8; 21];
    address_payload[0] = 0x41; // 4. Prepend Tron address prefix
    address_payload[1..].copy_from_slice(&hashed_pk[12..]);

    // 5. Base58Check encode
    base58check_encode(&address_payload)
}

/// Performs Base58Check encoding on a payload.
/// This involves creating a checksum by double-SHA256 hashing the payload,
/// taking the first 4 bytes of the hash, and appending it to the payload before Base58 encoding.
fn base58check_encode(payload: &[u8]) -> String {
    let mut hasher1 = Sha256::new();
    hasher1.update(payload);
    let hash1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(&hash1);
    let hash2 = hasher2.finalize();

    let checksum = &hash2[0..4];

    let mut final_payload = Vec::with_capacity(payload.len() + 4);
    final_payload.extend_from_slice(payload);
    final_payload.extend_from_slice(checksum);

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

    #[test]
    fn test_base58check_encoding() {
        // Test vector from Bitcoin wiki for a different prefix, but logic is the same.
        let payload = hex::decode("00010966776006953D5567439E5E39F86A0D273BEED61967F6").unwrap();
        let expected_encoded = "16UwLL9Risc3QfPqBUvKofHmBQ7wMtjvM";
        let encoded = base58check_encode(&payload);
        assert_eq!(encoded, expected_encoded);
    }
}

