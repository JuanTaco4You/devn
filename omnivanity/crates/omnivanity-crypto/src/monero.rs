//! Monero (CryptoNote) logic
//!
//! Monero uses Ed25519 (EdDSA) on the Twisted Edwards curve, but with specific scalarmult ops.
//! Address = [Network Byte] + [Spend PubKey] + [View PubKey] + [Checksum] -> Base58

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::edwards::EdwardsPoint;
use sha3::{Keccak256, Digest}; // Monero uses Keccak for checksums
use std::ops::Mul;

// use crate::hash::keccak256;

// Convert 32-byte seed to a valid Ed25519 scalar (Monero style: reduce)
pub fn sc_reduce32(seed: &[u8; 32]) -> Scalar {
    Scalar::from_bytes_mod_order(*seed)
}

// Generate Monero keypair (spend or secret) from a scalar
pub fn generate_key_image(scalar: &Scalar) -> [u8; 32] {
    let point = ED25519_BASEPOINT_POINT * scalar;
    point.compress().to_bytes()
}

// Monero Base58 Encoding (block-based)
pub mod base58_monero {
    use super::*;

    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::new();
        let chunks = data.chunks(8);
        
        for chunk in chunks {
            let mut val = 0u64;
            let mut multiplier = 1u64;
            
            for &byte in chunk {
                val += (byte as u64) * multiplier;
                multiplier <<= 8;
            }
            
            // Block size depends on input length (full block = 11 chars)
            let block_len = match chunk.len() {
                8 => 11,
                7 => 10,
                6 => 9,
                5 => 7,
                4 => 6,
                3 => 4,
                2 => 3,
                1 => 2,
                _ => 0,
            };

            let mut block = String::with_capacity(block_len);
            let mut current = val;
            
            for _ in 0..block_len {
                let idx = (current % 58) as usize;
                block.push(ALPHABET[idx] as char);
                current /= 58;
            }
            
            // Reverse because we pushed LSB first
            out.push_str(&block.chars().rev().collect::<String>());
        }
        
        out
    }
}
