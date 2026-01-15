//! Pattern matching implementation

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatternError {
    #[error("Pattern is empty")]
    EmptyPattern,
    #[error("Pattern contains invalid character '{0}' (valid: {1})")]
    InvalidCharacter(char, String),
    #[error("Pattern too long (max {0} characters)")]
    PatternTooLong(usize),
}

/// Type of pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Match at start of address (after prefix like 0x)
    Prefix,
    /// Match at end of address
    Suffix,
    /// Match anywhere in address
    Contains,
}

/// A pattern to search for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// The pattern string to match
    pub value: String,
    /// Type of matching
    pub pattern_type: PatternType,
    /// Case insensitive matching
    pub case_insensitive: bool,
}

impl Pattern {
    /// Create a new prefix pattern
    pub fn prefix(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            pattern_type: PatternType::Prefix,
            case_insensitive: false,
        }
    }

    /// Create a new suffix pattern
    pub fn suffix(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            pattern_type: PatternType::Suffix,
            case_insensitive: false,
        }
    }

    /// Create a new contains pattern
    pub fn contains(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            pattern_type: PatternType::Contains,
            case_insensitive: false,
        }
    }

    /// Make pattern case insensitive
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Validate pattern against valid characters
    pub fn validate(&self, valid_chars: &str) -> Result<(), PatternError> {
        if self.value.is_empty() {
            return Err(PatternError::EmptyPattern);
        }

        for c in self.value.chars() {
            let check_char = if self.case_insensitive {
                c.to_ascii_lowercase()
            } else {
                c
            };
            
            let valid = if self.case_insensitive {
                valid_chars.to_lowercase().contains(check_char)
            } else {
                valid_chars.contains(c)
            };

            if !valid {
                return Err(PatternError::InvalidCharacter(c, valid_chars.to_string()));
            }
        }

        Ok(())
    }
}

/// Pattern matcher for checking addresses
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    patterns: Vec<Pattern>,
}

impl PatternMatcher {
    /// Create a new matcher with given patterns
    pub fn new(patterns: Vec<Pattern>) -> Self {
        Self { patterns }
    }

    /// Create a matcher with a single pattern
    pub fn single(pattern: Pattern) -> Self {
        Self { patterns: vec![pattern] }
    }

    /// Check if address matches any pattern
    /// Returns the index of the matching pattern, or None
    pub fn matches(&self, address: &str) -> Option<usize> {
        for (i, pattern) in self.patterns.iter().enumerate() {
            if self.check_pattern(address, pattern) {
                return Some(i);
            }
        }
        None
    }

    /// Check if address matches a specific pattern
    fn check_pattern(&self, address: &str, pattern: &Pattern) -> bool {
        let addr = if pattern.case_insensitive {
            address.to_lowercase()
        } else {
            address.to_string()
        };

        let pat = if pattern.case_insensitive {
            pattern.value.to_lowercase()
        } else {
            pattern.value.clone()
        };

        match pattern.pattern_type {
            PatternType::Prefix => {
                // Skip common prefixes like 0x
                let addr_to_check = if addr.starts_with("0x") {
                    &addr[2..]
                } else if addr.starts_with("bc1q") || addr.starts_with("bc1p") {
                    &addr[4..]
                } else if addr.starts_with("ltc1q") {
                    &addr[5..]
                } else if addr.starts_with("t1") {
                    &addr[2..]
                } else if addr.len() > 1 && (addr.starts_with('1') || addr.starts_with('3') || 
                          addr.starts_with('L') || addr.starts_with('M') || addr.starts_with('D')) {
                    &addr[1..]
                } else {
                    &addr
                };
                addr_to_check.starts_with(&pat)
            }
            PatternType::Suffix => addr.ends_with(&pat),
            PatternType::Contains => addr.contains(&pat),
        }
    }

    /// Get all patterns
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_match() {
        let matcher = PatternMatcher::single(Pattern::prefix("dead"));
        
        // ETH address with 0x prefix
        assert!(matcher.matches("0xdeadbeef1234567890abcdef1234567890abcdef").is_some());
        assert!(matcher.matches("0xabcd1234567890abcdef1234567890abcdef1234").is_none());
    }

    #[test]
    fn test_suffix_match() {
        let matcher = PatternMatcher::single(Pattern::suffix("dead"));
        
        assert!(matcher.matches("0x1234567890abcdef1234567890abcdef1234dead").is_some());
        assert!(matcher.matches("0x1234567890abcdef1234567890abcdef12341234").is_none());
    }

    #[test]
    fn test_contains_match() {
        let matcher = PatternMatcher::single(Pattern::contains("cafe"));
        
        assert!(matcher.matches("0x1234cafe567890abcdef1234567890abcdef1234").is_some());
        assert!(matcher.matches("0x1234567890abcdef1234567890abcdef12341234").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let matcher = PatternMatcher::single(Pattern::prefix("DEAD").case_insensitive());
        
        assert!(matcher.matches("0xdeadbeef1234567890abcdef1234567890abcdef").is_some());
        assert!(matcher.matches("0xDEADbeef1234567890abcdef1234567890abcdef").is_some());
    }

    #[test]
    fn test_btc_prefix() {
        let matcher = PatternMatcher::single(Pattern::prefix("Love"));
        
        // BTC legacy address starts with 1
        assert!(matcher.matches("1Love1234567890abcdef1234567890ab").is_some());
    }

    #[test]
    fn test_validate_pattern() {
        let pattern = Pattern::prefix("dead");
        assert!(pattern.validate("0123456789abcdef").is_ok());
        
        let bad_pattern = Pattern::prefix("ghij");
        assert!(bad_pattern.validate("0123456789abcdef").is_err());
    }
}
