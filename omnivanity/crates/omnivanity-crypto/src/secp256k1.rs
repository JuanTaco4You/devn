//! secp256k1 elliptic curve operations for BTC, ETH, LTC, DOGE, ZEC

use k256::{
    ecdsa::SigningKey,
    elliptic_curve::rand_core::OsRng,
    PublicKey, SecretKey,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Secp256k1Error {
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Key generation failed")]
    KeyGenFailed,
}

/// A secp256k1 keypair for ECDSA operations
#[derive(Clone)]
pub struct Secp256k1Keypair {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Secp256k1Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let secret_key = SecretKey::random(&mut OsRng);
        let public_key = secret_key.public_key();
        Self { secret_key, public_key }
    }

    /// Create from raw 32-byte private key
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Secp256k1Error> {
        let secret_key = SecretKey::from_bytes(bytes.into())
            .map_err(|_| Secp256k1Error::InvalidPrivateKey)?;
        let public_key = secret_key.public_key();
        Ok(Self { secret_key, public_key })
    }

    /// Get the private key as bytes
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.secret_key.to_bytes().into()
    }

    /// Get the uncompressed public key (65 bytes: 0x04 || x || y)
    pub fn public_key_uncompressed(&self) -> [u8; 65] {
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        let point = self.public_key.to_encoded_point(false);
        let mut result = [0u8; 65];
        result.copy_from_slice(point.as_bytes());
        result
    }

    /// Get the compressed public key (33 bytes: 0x02/0x03 || x)
    pub fn public_key_compressed(&self) -> [u8; 33] {
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        let point = self.public_key.to_encoded_point(true);
        let mut result = [0u8; 33];
        result.copy_from_slice(point.as_bytes());
        result
    }

    /// Get just the X and Y coordinates (64 bytes, no prefix)
    pub fn public_key_xy(&self) -> [u8; 64] {
        let uncompressed = self.public_key_uncompressed();
        let mut result = [0u8; 64];
        result.copy_from_slice(&uncompressed[1..65]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = Secp256k1Keypair::generate();
        assert_eq!(kp.private_key_bytes().len(), 32);
        assert_eq!(kp.public_key_uncompressed().len(), 65);
        assert_eq!(kp.public_key_compressed().len(), 33);
        assert_eq!(kp.public_key_uncompressed()[0], 0x04);
    }

    #[test]
    fn test_known_vector() {
        // Known test vector
        let privkey_hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let mut privkey = [0u8; 32];
        hex::decode_to_slice(privkey_hex, &mut privkey).unwrap();
        
        let kp = Secp256k1Keypair::from_bytes(&privkey).unwrap();
        let pubkey = kp.public_key_uncompressed();
        
        // Generator point G
        assert_eq!(
            hex::encode(&pubkey[1..33]),
            "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        );
    }
}
