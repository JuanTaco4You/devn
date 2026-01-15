//! OmniVanity Crypto Primitives
//! 
//! Low-level cryptographic operations for vanity address generation.

pub mod secp256k1;
pub mod ed25519;
pub mod hash;
pub mod encoding;

pub use self::secp256k1::Secp256k1Keypair;
pub use self::ed25519::Ed25519Keypair;

// Re-export dependencies for use by other crates
pub use bs58;
pub use hex;
