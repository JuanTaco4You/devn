//! Address encoding utilities: Base58, Bech32, Hex, WIF

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Invalid checksum")]
    InvalidChecksum,
    #[error("Invalid character in input")]
    InvalidCharacter,
    #[error("Invalid length")]
    InvalidLength,
    #[error("Bech32 encoding failed: {0}")]
    Bech32Error(String),
}

/// Base58Check encode (Bitcoin-style with 4-byte checksum)
pub fn base58check_encode(version: u8, payload: &[u8]) -> String {
    use crate::hash::double_sha256;
    
    let mut data = Vec::with_capacity(1 + payload.len() + 4);
    data.push(version);
    data.extend_from_slice(payload);
    
    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[..4]);
    
    bs58::encode(data).into_string()
}

/// Base58Check decode, returns (version, payload)
pub fn base58check_decode(input: &str) -> Result<(u8, Vec<u8>), EncodingError> {
    use crate::hash::double_sha256;
    
    let data = bs58::decode(input)
        .into_vec()
        .map_err(|_| EncodingError::InvalidCharacter)?;
    
    if data.len() < 5 {
        return Err(EncodingError::InvalidLength);
    }
    
    let (payload_with_version, checksum) = data.split_at(data.len() - 4);
    let computed_checksum = &double_sha256(payload_with_version)[..4];
    
    if checksum != computed_checksum {
        return Err(EncodingError::InvalidChecksum);
    }
    
    let version = payload_with_version[0];
    let payload = payload_with_version[1..].to_vec();
    
    Ok((version, payload))
}

/// Encode WIF (Wallet Import Format) for private key
pub fn wif_encode(private_key: &[u8; 32], compressed: bool, mainnet: bool) -> String {
    let version = if mainnet { 0x80 } else { 0xEF };
    
    if compressed {
        let mut payload = Vec::with_capacity(33);
        payload.extend_from_slice(private_key);
        payload.push(0x01);
        base58check_encode(version, &payload)
    } else {
        base58check_encode(version, private_key)
    }
}

/// Base58 encode (Solana style, no checksum)
pub fn base58_encode(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Base58 decode
pub fn base58_decode(input: &str) -> Result<Vec<u8>, EncodingError> {
    bs58::decode(input)
        .into_vec()
        .map_err(|_| EncodingError::InvalidCharacter)
}

/// Bech32 encode for SegWit addresses
pub fn bech32_encode(hrp: &str, witness_version: u8, program: &[u8]) -> Result<String, EncodingError> {
    use bech32::{Bech32m, Hrp};
    
    let hrp = Hrp::parse(hrp).map_err(|e| EncodingError::Bech32Error(e.to_string()))?;
    
    // Prepend witness version
    let mut data = Vec::with_capacity(1 + program.len());
    data.push(witness_version);
    data.extend_from_slice(program);
    
    bech32::encode::<Bech32m>(hrp, &data)
        .map_err(|e| EncodingError::Bech32Error(e.to_string()))
}

/// Bech32 encode for SegWit v0 (bech32 original encoding)
pub fn bech32_encode_v0(hrp: &str, program: &[u8]) -> Result<String, EncodingError> {
    use bech32::{Bech32, Hrp};
    
    let hrp = Hrp::parse(hrp).map_err(|e| EncodingError::Bech32Error(e.to_string()))?;
    
    // Witness version 0
    let mut data = Vec::with_capacity(1 + program.len());
    data.push(0u8);
    data.extend_from_slice(program);
    
    bech32::encode::<Bech32>(hrp, &data)
        .map_err(|e| EncodingError::Bech32Error(e.to_string()))
}

/// EIP-55 checksum encoding for Ethereum addresses
pub fn eip55_checksum(address: &[u8; 20]) -> String {
    use crate::hash::keccak256;
    
    let hex_addr = hex::encode(address);
    let hash = keccak256(hex_addr.as_bytes());
    
    let mut result = String::with_capacity(42);
    result.push_str("0x");
    
    for (i, c) in hex_addr.chars().enumerate() {
        let hash_nibble = if i % 2 == 0 {
            (hash[i / 2] >> 4) & 0x0F
        } else {
            hash[i / 2] & 0x0F
        };
        
        if hash_nibble >= 8 {
            result.push(c.to_ascii_uppercase());
        } else {
            result.push(c);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58check_roundtrip() {
        let payload = [1u8; 20];
        let encoded = base58check_encode(0x00, &payload);
        let (version, decoded) = base58check_decode(&encoded).unwrap();
        assert_eq!(version, 0x00);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn test_wif_encode() {
        // Known test vector
        let privkey = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&privkey);
        
        let wif = wif_encode(&pk, true, true);
        assert_eq!(wif, "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn");
    }

    #[test]
    fn test_eip55_checksum() {
        let addr = hex::decode("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        let mut addr_arr = [0u8; 20];
        addr_arr.copy_from_slice(&addr);
        
        let checksummed = eip55_checksum(&addr_arr);
        assert_eq!(checksummed, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    }
}
