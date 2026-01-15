//! OmniVanity Pattern Matching Engine
//!
//! Pattern types: prefix, suffix, contains, regex (future)

mod matcher;
mod difficulty;

pub use matcher::{Pattern, PatternType, PatternMatcher};
pub use difficulty::calculate_difficulty;
