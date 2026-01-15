//! Hash functions for address derivation

use sha2::{Sha256, Digest as Sha2Digest};
use sha3::Keccak256;
use ripemd::Ripemd160;

/// SHA-256 hash
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Double SHA-256 (used in Bitcoin)
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

/// RIPEMD-160 hash
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash160: SHA256 then RIPEMD160 (used in Bitcoin addresses)
pub fn hash160(data: &[u8]) -> [u8; 20] {
    ripemd160(&sha256(data))
}

/// Keccak-256 (used in Ethereum, NOT SHA3-256)
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let result = sha256(b"hello");
        assert_eq!(
            hex::encode(result),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_keccak256() {
        // Empty input
        let result = keccak256(b"");
        assert_eq!(
            hex::encode(result),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn test_hash160() {
        // Test with a known public key
        let pubkey = hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap();
        let h160 = hash160(&pubkey);
        assert_eq!(
            hex::encode(h160),
            "751e76e8199196d454941c45d1b3a323f1433bd6"
        );
    }
}
