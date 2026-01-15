//! Ed25519 elliptic curve operations for Solana

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Ed25519Error {
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Key generation failed")]
    KeyGenFailed,
}

/// An Ed25519 keypair for Solana
#[derive(Clone)]
pub struct Ed25519Keypair {
    signing_key: SigningKey,
}

impl Ed25519Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create from raw 32-byte seed (private key)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Ed25519Error> {
        let signing_key = SigningKey::from_bytes(bytes);
        Ok(Self { signing_key })
    }

    /// Get the private key seed as bytes (32 bytes)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Get the full keypair bytes (64 bytes: privkey || pubkey) - Solana format
    pub fn keypair_bytes(&self) -> [u8; 64] {
        let mut result = [0u8; 64];
        result[..32].copy_from_slice(&self.signing_key.to_bytes());
        result[32..].copy_from_slice(self.signing_key.verifying_key().as_bytes());
        result
    }

    /// Get the public key as bytes (32 bytes)
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = Ed25519Keypair::generate();
        assert_eq!(kp.private_key_bytes().len(), 32);
        assert_eq!(kp.public_key_bytes().len(), 32);
        assert_eq!(kp.keypair_bytes().len(), 64);
    }

    #[test]
    fn test_deterministic() {
        let seed = [1u8; 32];
        let kp1 = Ed25519Keypair::from_bytes(&seed).unwrap();
        let kp2 = Ed25519Keypair::from_bytes(&seed).unwrap();
        assert_eq!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }
}
