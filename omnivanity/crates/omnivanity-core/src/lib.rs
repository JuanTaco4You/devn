//! OmniVanity Core Engine
//!
//! The main vanity search engine with multi-threaded CPU support.

mod search;
mod stats;

pub use search::{VanitySearch, SearchConfig, SearchResult};
pub use stats::SearchStats;

// Re-exports for convenience
pub use omnivanity_chains::{Chain, ChainFamily, AddressType, GeneratedAddress, all_chains, get_chain};
pub use omnivanity_pattern::{Pattern, PatternType, PatternMatcher, calculate_difficulty};
