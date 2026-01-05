//! Scan state persistence for resume capability

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{DomainForgeError, Result};

/// Persistent scan state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanState {
    /// Recheck/update timestamps history (append-only).
    /// This is used by `snipe recheck` to record each update time.
    #[serde(default)]
    pub update_times: Vec<DateTime<Utc>>,
    /// Scan identifier
    pub scan_id: String,
    /// Domain length being scanned
    pub length: usize,
    /// TLDs to scan
    pub tlds: Vec<String>,
    /// Current index in generation sequence
    pub current_index: u64,
    /// Total combinations to scan
    pub total_combinations: u64,
    /// Available domains found
    pub available: Vec<SnipedDomain>,
    /// Domains that appear expired (expiration_date <= now) but are not yet available (RDAP still returns 200).
    ///
    /// These are often high-value to monitor because they may transition to available later.
    #[serde(default)]
    pub expired: Vec<SnipedDomain>,
    /// Domains expiring soon
    pub expiring_soon: Vec<SnipedDomain>,
    /// Failed domain checks with error details
    #[serde(default)]
    pub errors: Vec<FailedDomain>,
    /// Number of domains checked
    pub checked_count: u64,
    /// Number of errors encountered
    pub error_count: u64,
    /// Scan start time
    pub started_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Scan completed
    pub completed: bool,
}

/// A sniped domain result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipedDomain {
    pub domain: String,
    pub tld: String,
    pub full_domain: String,
    pub expiration_date: Option<DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub registrar: Option<String>,
    /// RDAP status values (e.g. "active", "pendingDelete", "redemptionPeriod").
    /// Kept as a list because RDAP can return multiple statuses.
    #[serde(default)]
    pub rdap_status: Vec<String>,
    pub found_at: DateTime<Utc>,
}

/// A failed domain check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedDomain {
    pub domain: String,
    pub tld: String,
    pub full_domain: String,
    pub error: String,
    pub failed_at: DateTime<Utc>,
}

impl ScanState {
    /// Create a new scan state
    pub fn new(length: usize, tlds: Vec<String>, total_combinations: u64) -> Self {
        let now = Utc::now();
        Self {
            update_times: Vec::new(),
            scan_id: format!("scan_{}_{}", length, now.format("%Y%m%d_%H%M%S")),
            length,
            tlds,
            current_index: 0,
            total_combinations,
            available: Vec::new(),
            expired: Vec::new(),
            expiring_soon: Vec::new(),
            errors: Vec::new(),
            checked_count: 0,
            error_count: 0,
            started_at: now,
            updated_at: now,
            completed: false,
        }
    }

    /// Load state from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            DomainForgeError::io(e.to_string(), Some(path.to_string_lossy().to_string()))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            DomainForgeError::parse(e.to_string(), Some(content))
        })
    }

    /// Save state to file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainForgeError::io(e.to_string(), Some(parent.to_string_lossy().to_string()))
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            DomainForgeError::internal(format!("Failed to serialize state: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            DomainForgeError::io(e.to_string(), Some(path.to_string_lossy().to_string()))
        })
    }

    /// Get default state file path
    pub fn default_path(length: usize) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("output/snipe_{}letter.json", length))
    }

    /// Add an available domain
    pub fn add_available(&mut self, domain: SnipedDomain) {
        self.available.push(domain);
        self.updated_at = Utc::now();
    }

    /// Add an expiring domain
    pub fn add_expiring(&mut self, domain: SnipedDomain) {
        self.expiring_soon.push(domain);
        self.updated_at = Utc::now();
    }

    /// Add a failed domain check
    pub fn add_error(&mut self, failed: FailedDomain) {
        self.errors.push(failed);
        self.error_count += 1;
        self.updated_at = Utc::now();
    }

    /// Update progress
    pub fn update_progress(&mut self, index: u64, checked: u64, errors: u64) {
        self.current_index = index;
        self.checked_count = checked;
        self.error_count = errors;
        self.updated_at = Utc::now();
    }

    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.updated_at = Utc::now();
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.total_combinations == 0 {
            100.0
        } else {
            (self.current_index as f64 / self.total_combinations as f64) * 100.0
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

    /// Estimate remaining time based on current progress
    pub fn estimate_remaining(&self) -> Option<chrono::Duration> {
        if self.current_index == 0 {
            return None;
        }

        let elapsed = self.elapsed();
        let rate = self.current_index as f64 / elapsed.num_seconds().max(1) as f64;
        let remaining = self.total_combinations.saturating_sub(self.current_index);
        let seconds = (remaining as f64 / rate) as i64;

        Some(chrono::Duration::seconds(seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = ScanState::new(4, vec!["com".to_string()], 456976);
        assert_eq!(state.length, 4);
        assert_eq!(state.total_combinations, 456976);
        assert!(!state.completed);
    }

    #[test]
    fn test_progress() {
        let mut state = ScanState::new(4, vec!["com".to_string()], 1000);
        state.update_progress(500, 500, 0);
        assert_eq!(state.progress_percent(), 50.0);
    }
}
