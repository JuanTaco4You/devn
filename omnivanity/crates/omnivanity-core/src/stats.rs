//! Live search statistics

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Thread-safe search statistics
#[derive(Debug)]
pub struct SearchStats {
    /// Total keys tested
    pub keys_tested: AtomicU64,
    /// Start time 
    start_time: Instant,
    /// Whether search is running
    pub running: AtomicBool,
    /// Whether a match was found
    pub found: AtomicBool,
}

impl SearchStats {
    /// Create new stats
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            keys_tested: AtomicU64::new(0),
            start_time: Instant::now(),
            running: AtomicBool::new(true),
            found: AtomicBool::new(false),
        })
    }

    /// Increment keys tested by amount
    pub fn add_keys(&self, count: u64) {
        self.keys_tested.fetch_add(count, Ordering::Relaxed);
    }

    /// Get total keys tested
    pub fn total_keys(&self) -> u64 {
        self.keys_tested.load(Ordering::Relaxed)
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get keys per second
    pub fn keys_per_second(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_keys() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the search
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Mark as found
    pub fn mark_found(&self) {
        self.found.store(true, Ordering::Relaxed);
        self.stop();
    }

    /// Check if found
    pub fn is_found(&self) -> bool {
        self.found.load(Ordering::Relaxed)
    }

    /// Get formatted stats string
    pub fn format(&self, difficulty: f64) -> String {
        let keys = self.total_keys();
        let kps = self.keys_per_second();
        let elapsed = self.elapsed().as_secs_f64();
        
        // Calculate probability
        let prob = if difficulty > 0.0 {
            1.0 - (-1.0 * keys as f64 / difficulty).exp()
        } else {
            0.0
        };

        // Calculate ETA for 50%
        let remaining_for_50 = if prob < 0.5 && kps > 0.0 {
            let keys_needed = (0.5_f64.ln() / (-1.0 / difficulty)) - keys as f64;
            keys_needed / kps
        } else {
            0.0
        };

        format!(
            "[{:.2} Mkey/s][Total {}][Prob {:.1}%][50% in {}]",
            kps / 1_000_000.0,
            format_keys(keys),
            prob * 100.0,
            format_duration(remaining_for_50)
        )
    }
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            keys_tested: AtomicU64::new(0),
            start_time: Instant::now(),
            running: AtomicBool::new(true),
            found: AtomicBool::new(false),
        }
    }
}

fn format_keys(keys: u64) -> String {
    if keys >= 1_000_000_000_000 {
        format!("{:.2}T", keys as f64 / 1e12)
    } else if keys >= 1_000_000_000 {
        format!("{:.2}G", keys as f64 / 1e9)
    } else if keys >= 1_000_000 {
        format!("{:.2}M", keys as f64 / 1e6)
    } else if keys >= 1000 {
        format!("{:.2}K", keys as f64 / 1e3)
    } else {
        format!("{}", keys)
    }
}

fn format_duration(seconds: f64) -> String {
    if seconds <= 0.0 {
        return "now".to_string();
    }
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.0}m", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.1}h", seconds / 3600.0)
    } else if seconds < 86400.0 * 365.0 {
        format!("{:.1}d", seconds / 86400.0)
    } else {
        format!("{:.1}y", seconds / (86400.0 * 365.0))
    }
}
