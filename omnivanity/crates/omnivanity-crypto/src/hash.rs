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

/// Blake2b-256 (used in Sui, IOTA, Nano)
pub fn blake2b_256(data: &[u8]) -> [u8; 32] {
    use blake2::digest::VariableOutput;
    use blake2::Blake2bVar;
    let mut hasher = Blake2bVar::new(32).unwrap();
    blake2::digest::Update::update(&mut hasher, data);
    let mut output = [0u8; 32];
    hasher.finalize_variable(&mut output).unwrap();
    output
}

/// Blake2b-160 (used in Filecoin f1 addresses)
pub fn blake2b_160(data: &[u8]) -> [u8; 20] {
    use blake2::digest::VariableOutput;
    use blake2::Blake2bVar;
    let mut hasher = Blake2bVar::new(20).unwrap();
    blake2::digest::Update::update(&mut hasher, data);
    let mut output = [0u8; 20];
    hasher.finalize_variable(&mut output).unwrap();
    output
}

/// Blake2b-224 (used in Cardano)
pub fn blake2b_224(data: &[u8]) -> [u8; 28] {
    use blake2::digest::VariableOutput;
    use blake2::Blake2bVar;
    let mut hasher = Blake2bVar::new(28).unwrap();
    blake2::digest::Update::update(&mut hasher, data);
    let mut output = [0u8; 28];
    hasher.finalize_variable(&mut output).unwrap();
    output
}

/// SHA3-256 (used in Aptos - note: different from Keccak-256!)
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    use sha3::{Sha3_256, Digest};
    let mut hasher = Sha3_256::new();
    Digest::update(&mut hasher, data);
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

    #[test]
    fn test_blake2b_256() {
        // Blake2b-256 of empty string
        let result = blake2b_256(b"");
        assert_eq!(
            hex::encode(result),
            "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8"
        );
    }

    #[test]
    fn test_sha3_256() {
        // SHA3-256 of empty string (different from Keccak!)
        let result = sha3_256(b"");
        assert_eq!(
            hex::encode(result),
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }
}
