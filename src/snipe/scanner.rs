//! Domain sniper - scan for available short domains

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures::future::join_all;
use tokio::sync::Semaphore;

use super::filter::PronounceableGenerator;
use super::generator::DomainGenerator;
use super::state::{ScanState, SnipedDomain};
use super::words::WordGenerator;
use super::Charset;
use crate::error::Result;
use crate::rdap::registry::rdap_base_url;

/// Scan mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanMode {
    /// Full 4-letter scan (all combinations)
    #[default]
    Full,
    /// Pronounceable 4-letter patterns only
    Pronounceable,
    /// 5-letter meaningful words
    Words,
}

/// Snipe scan status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnipeStatus {
    /// Domain is available for registration
    Available,
    /// Domain is expiring soon (within configured days)
    ExpiringSoon,
    /// Domain is taken
    Taken,
    /// Check failed
    Error,
}

/// Snipe scan result
#[derive(Debug, Clone)]
pub struct SnipeResult {
    pub domain: String,
    pub tld: String,
    pub full_domain: String,
    pub status: SnipeStatus,
    pub expiration_date: Option<chrono::DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub registrar: Option<String>,
}

/// Snipe configuration
#[derive(Debug, Clone)]
pub struct SnipeConfig {
    /// Scan mode
    pub mode: ScanMode,
    /// Domain name length to scan (for Full mode)
    pub length: usize,
    /// TLDs to check
    pub tlds: Vec<String>,
    /// Character set to use (for Full mode)
    pub charset: Charset,
    /// Only scan pronounceable combinations (deprecated, use mode)
    pub pronounceable: bool,
    /// Concurrent checks
    pub concurrency: usize,
    /// Batch size for progress saves
    pub batch_size: usize,
    /// Days threshold for "expiring soon"
    pub expiring_days: u32,
    /// State file path (for resume)
    pub state_file: Option<PathBuf>,
    /// Save progress every N domains
    pub save_interval: u64,
    /// Rate limit delay between batches (ms)
    pub rate_limit_ms: u64,
}

impl Default for SnipeConfig {
    fn default() -> Self {
        Self {
            mode: ScanMode::Full,
            length: 4,
            tlds: vec!["com".to_string()],
            charset: Charset::Letters,
            pronounceable: false,
            concurrency: 15,
            batch_size: 100,
            expiring_days: 7,
            state_file: None,
            save_interval: 1000,
            rate_limit_ms: 100,
        }
    }
}

/// Scan progress info
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub current: u64,
    pub total: u64,
    pub available_count: usize,
    pub expiring_count: usize,
    pub error_count: u64,
    pub domains_per_second: f64,
    pub estimated_remaining: Option<Duration>,
}

/// Unified generator wrapper
enum GeneratorKind {
    Full(DomainGenerator),
    Pronounceable(PronounceableGenerator),
    Words(WordGenerator),
}

impl GeneratorKind {
    fn next_batch(&mut self, count: usize) -> Vec<String> {
        match self {
            GeneratorKind::Full(g) => g.next_batch(count),
            GeneratorKind::Pronounceable(g) => g.next_batch(count),
            GeneratorKind::Words(g) => g.next_batch(count),
        }
    }

    fn is_exhausted(&self) -> bool {
        match self {
            GeneratorKind::Full(g) => g.is_exhausted(),
            GeneratorKind::Pronounceable(g) => g.is_exhausted(),
            GeneratorKind::Words(g) => g.is_exhausted(),
        }
    }

    fn current_index(&self) -> u64 {
        match self {
            GeneratorKind::Full(g) => g.current_index(),
            GeneratorKind::Pronounceable(g) => g.current_index(),
            GeneratorKind::Words(g) => g.current_index(),
        }
    }

    fn set_index(&mut self, index: u64) {
        match self {
            GeneratorKind::Full(g) => g.set_index(index),
            GeneratorKind::Pronounceable(g) => g.set_index(index),
            GeneratorKind::Words(g) => g.set_index(index),
        }
    }

}

/// Domain sniper for scanning short domains
pub struct DomainSniper {
    config: SnipeConfig,
    generator: GeneratorKind,
    state: ScanState,
    semaphore: Arc<Semaphore>,
    client: reqwest::Client,
}

impl DomainSniper {
    /// Create a new domain sniper
    pub fn new(config: SnipeConfig) -> Self {
        // Determine effective mode (support legacy pronounceable flag)
        let effective_mode = if config.pronounceable {
            ScanMode::Pronounceable
        } else {
            config.mode
        };

        let (generator, total, length) = match effective_mode {
            ScanMode::Full => {
                let total = config.charset.total_combinations(config.length) * config.tlds.len() as u64;
                let gen = DomainGenerator::new(config.length, config.charset);
                (GeneratorKind::Full(gen), total, config.length)
            }
            ScanMode::Pronounceable => {
                let gen = PronounceableGenerator::new();
                let total = gen.total() * config.tlds.len() as u64;
                (GeneratorKind::Pronounceable(gen), total, 4)
            }
            ScanMode::Words => {
                let gen = WordGenerator::new();
                let total = gen.total() * config.tlds.len() as u64;
                (GeneratorKind::Words(gen), total, 5)
            }
        };

        let state = ScanState::new(length, config.tlds.clone(), total);
        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(config.concurrency)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            generator,
            state,
            semaphore,
            client,
        }
    }

    /// Create sniper with existing state (for resume)
    pub fn with_state(config: SnipeConfig, state: ScanState) -> Self {
        let effective_mode = if config.pronounceable {
            ScanMode::Pronounceable
        } else {
            config.mode
        };

        let mut generator = match effective_mode {
            ScanMode::Full => {
                GeneratorKind::Full(DomainGenerator::new(config.length, config.charset))
            }
            ScanMode::Pronounceable => {
                GeneratorKind::Pronounceable(PronounceableGenerator::new())
            }
            ScanMode::Words => {
                GeneratorKind::Words(WordGenerator::new())
            }
        };
        generator.set_index(state.current_index);

        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(config.concurrency)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            generator,
            state,
            semaphore,
            client,
        }
    }

    /// Resume from state file
    pub fn resume(config: SnipeConfig) -> Result<Self> {
        // Get effective length based on mode
        let effective_length = match config.mode {
            ScanMode::Words => 5,
            _ => config.length,
        };

        let state_path = config
            .state_file
            .clone()
            .unwrap_or_else(|| ScanState::default_path(effective_length));

        let state = ScanState::load(&state_path)?;
        Ok(Self::with_state(config, state))
    }

    /// Run the scan with progress callback
    pub async fn run<F>(&mut self, on_progress: F) -> Result<&ScanState>
    where
        F: Fn(&ScanProgress) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        let mut last_save = 0u64;

        while !self.generator.is_exhausted() {
            // Generate batch of domain names
            let names = self.generator.next_batch(self.config.batch_size);
            if names.is_empty() {
                break;
            }

            // Build all check tasks for this batch (names × TLDs)
            let check_tasks: Vec<_> = names
                .iter()
                .flat_map(|name| {
                    self.config.tlds.iter().map(move |tld| {
                        (name.clone(), tld.clone())
                    })
                })
                .collect();

            // Check all domains concurrently
            let results = self.check_batch(&check_tasks).await;

            // Process results
            for result in results {
                match result.status {
                    SnipeStatus::Available => {
                        self.state.add_available(SnipedDomain {
                            domain: result.domain.clone(),
                            tld: result.tld.clone(),
                            full_domain: result.full_domain.clone(),
                            expiration_date: result.expiration_date,
                            days_until_expiry: result.days_until_expiry,
                            registrar: result.registrar.clone(),
                            found_at: Utc::now(),
                        });
                    }
                    SnipeStatus::ExpiringSoon => {
                        self.state.add_expiring(SnipedDomain {
                            domain: result.domain.clone(),
                            tld: result.tld.clone(),
                            full_domain: result.full_domain.clone(),
                            expiration_date: result.expiration_date,
                            days_until_expiry: result.days_until_expiry,
                            registrar: result.registrar.clone(),
                            found_at: Utc::now(),
                        });
                    }
                    SnipeStatus::Error => {
                        self.state.error_count += 1;
                    }
                    SnipeStatus::Taken => {}
                }
                self.state.checked_count += 1;
            }

            // Update state
            self.state
                .update_progress(self.generator.current_index(), self.state.checked_count, self.state.error_count);

            // Calculate progress
            let elapsed = start_time.elapsed();
            let rate = if elapsed.as_secs() > 0 {
                self.state.checked_count as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            let remaining = self.state.total_combinations.saturating_sub(self.state.checked_count);
            let estimated = if rate > 0.0 {
                Some(Duration::from_secs_f64(remaining as f64 / rate))
            } else {
                None
            };

            let progress = ScanProgress {
                current: self.state.checked_count,
                total: self.state.total_combinations,
                available_count: self.state.available.len(),
                expiring_count: self.state.expiring_soon.len(),
                error_count: self.state.error_count,
                domains_per_second: rate,
                estimated_remaining: estimated,
            };

            on_progress(&progress);

            // Save state periodically
            if self.state.checked_count - last_save >= self.config.save_interval {
                self.save_state()?;
                last_save = self.state.checked_count;
            }

            // Rate limiting between batches (not between each check)
            if self.config.rate_limit_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.rate_limit_ms)).await;
            }
        }

        self.state.mark_completed();
        self.save_state()?;

        Ok(&self.state)
    }

    /// Check a batch of (name, tld) pairs concurrently
    async fn check_batch(&self, tasks: &[(String, String)]) -> Vec<SnipeResult> {
        let futures: Vec<_> = tasks
            .iter()
            .map(|(name, tld)| {
                let name = name.clone();
                let tld = tld.clone();
                let full_domain = format!("{}.{}", name, tld);
                let semaphore = Arc::clone(&self.semaphore);
                let expiring_days = self.config.expiring_days;
                let client = self.client.clone(); // Reuse client (internally Arc-based)

                async move {
                    let _permit = semaphore.acquire().await.ok()?;

                    let rdap_url = rdap_base_url(&tld)?;
                    let url = format!("{}domain/{}", rdap_url, full_domain);

                    match client.get(&url).send().await {
                        Ok(response) => {
                            let status_code = response.status().as_u16();

                            if status_code == 404 {
                                // Domain is available
                                Some(SnipeResult {
                                    domain: name,
                                    tld,
                                    full_domain,
                                    status: SnipeStatus::Available,
                                    expiration_date: None,
                                    days_until_expiry: None,
                                    registrar: None,
                                })
                            } else if status_code == 200 {
                                // Domain is taken, try to get expiration
                                let expiration = response.json::<serde_json::Value>().await.ok()
                                    .and_then(|v| {
                                        v.get("events")?.as_array()?.iter()
                                            .find(|e| e.get("eventAction").and_then(|a| a.as_str()) == Some("expiration"))
                                            .and_then(|e| e.get("eventDate")?.as_str())
                                            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                                            .map(|d| d.with_timezone(&Utc))
                                    });

                                let days_until = expiration.map(|exp| (exp - Utc::now()).num_days());
                                let is_expiring = days_until.map(|d| d > 0 && d <= expiring_days as i64).unwrap_or(false);

                                Some(SnipeResult {
                                    domain: name,
                                    tld,
                                    full_domain,
                                    status: if is_expiring { SnipeStatus::ExpiringSoon } else { SnipeStatus::Taken },
                                    expiration_date: expiration,
                                    days_until_expiry: days_until,
                                    registrar: None,
                                })
                            } else {
                                Some(SnipeResult {
                                    domain: name,
                                    tld,
                                    full_domain,
                                    status: SnipeStatus::Error,
                                    expiration_date: None,
                                    days_until_expiry: None,
                                    registrar: None,
                                })
                            }
                        }
                        Err(_) => Some(SnipeResult {
                            domain: name,
                            tld,
                            full_domain,
                            status: SnipeStatus::Error,
                            expiration_date: None,
                            days_until_expiry: None,
                            registrar: None,
                        }),
                    }
                }
            })
            .collect();

        join_all(futures).await.into_iter().flatten().collect()
    }

    /// Save current state
    pub fn save_state(&self) -> Result<()> {
        let path = self
            .config
            .state_file
            .clone()
            .unwrap_or_else(|| ScanState::default_path(self.state.length));
        self.state.save(&path)
    }

    /// Get current state
    pub fn state(&self) -> &ScanState {
        &self.state
    }

    /// Get available domains found
    pub fn available_domains(&self) -> &[SnipedDomain] {
        &self.state.available
    }

    /// Get expiring domains found
    pub fn expiring_domains(&self) -> &[SnipedDomain] {
        &self.state.expiring_soon
    }

    /// Get scan progress
    pub fn progress(&self) -> f64 {
        self.state.progress_percent()
    }
}
