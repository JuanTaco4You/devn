//! Difficulty calculation for vanity patterns

use crate::PatternType;

/// Calculate the difficulty (expected number of attempts) for a pattern
pub fn calculate_difficulty(
    pattern: &str,
    pattern_type: PatternType,
    alphabet_size: usize,
    case_insensitive: bool,
) -> f64 {
    let effective_alphabet = if case_insensitive {
        // For case insensitive, we have more matches possible
        // Rough estimate: divide by ratio of upper+lower to total
        alphabet_size
    } else {
        alphabet_size
    };

    let pattern_len = pattern.len();

    match pattern_type {
        PatternType::Prefix | PatternType::Suffix => {
            // Difficulty = alphabet_size ^ pattern_length
            let base_difficulty = (effective_alphabet as f64).powi(pattern_len as i32);
            
            if case_insensitive {
                // Reduce difficulty by factor of 2^num_letters
                let num_letters = pattern.chars().filter(|c| c.is_alphabetic()).count();
                base_difficulty / (2.0_f64).powi(num_letters as i32)
            } else {
                base_difficulty
            }
        }
        PatternType::Contains => {
            // For contains, it's more complex - approximate
            // Roughly: address_length / alphabet_size^pattern_length
            // Assuming ~40 char address
            let address_len = 40.0;
            let positions = (address_len - pattern_len as f64 + 1.0).max(1.0);
            
            let base_difficulty = (effective_alphabet as f64).powi(pattern_len as i32) / positions;
            
            if case_insensitive {
                let num_letters = pattern.chars().filter(|c| c.is_alphabetic()).count();
                base_difficulty / (2.0_f64).powi(num_letters as i32)
            } else {
                base_difficulty
            }
        }
    }
}

/// Format difficulty as human-readable string
pub fn format_difficulty(difficulty: f64) -> String {
    if difficulty >= 1e15 {
        format!("{:.2}P", difficulty / 1e15)
    } else if difficulty >= 1e12 {
        format!("{:.2}T", difficulty / 1e12)
    } else if difficulty >= 1e9 {
        format!("{:.2}G", difficulty / 1e9)
    } else if difficulty >= 1e6 {
        format!("{:.2}M", difficulty / 1e6)
    } else if difficulty >= 1e3 {
        format!("{:.2}K", difficulty / 1e3)
    } else {
        format!("{:.0}", difficulty)
    }
}

/// Estimate time to 50% probability of finding a match
pub fn estimate_time_50pct(difficulty: f64, keys_per_second: f64) -> f64 {
    // 50% probability requires ln(0.5) / ln(1 - 1/difficulty) attempts
    // For large difficulty, this approximates to difficulty * ln(2)
    let ln_half = 0.693147;
    (difficulty * ln_half) / keys_per_second
}

/// Format duration in human-readable format
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let mins = seconds / 60.0;
        format!("{:.1}m", mins)
    } else if seconds < 86400.0 {
        let hours = seconds / 3600.0;
        format!("{:.1}h", hours)
    } else if seconds < 86400.0 * 365.0 {
        let days = seconds / 86400.0;
        format!("{:.1}d", days)
    } else {
        let years = seconds / (86400.0 * 365.0);
        format!("{:.1}y", years)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_calculation() {
        // Hex alphabet (16 chars), 4 char prefix
        let diff = calculate_difficulty("dead", PatternType::Prefix, 16, false);
        assert_eq!(diff, 65536.0); // 16^4
    }

    #[test]
    fn test_case_insensitive_reduces_difficulty() {
        let case_sensitive = calculate_difficulty("dead", PatternType::Prefix, 16, false);
        let case_insensitive = calculate_difficulty("dead", PatternType::Prefix, 16, true);
        
        // Case insensitive should be easier (lower difficulty)
        assert!(case_insensitive < case_sensitive);
    }

    #[test]
    fn test_format_difficulty() {
        assert_eq!(format_difficulty(1000.0), "1.00K");
        assert_eq!(format_difficulty(1500000.0), "1.50M");
        assert_eq!(format_difficulty(1e12), "1.00T");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.5), "500ms");
        assert_eq!(format_duration(30.0), "30.0s");
        assert_eq!(format_duration(120.0), "2.0m");
        assert_eq!(format_duration(7200.0), "2.0h");
    }
}
