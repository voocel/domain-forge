//! Domain sniper - scan for available short domains

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures::future::join_all;
use tokio::sync::Semaphore;

use super::filter::PronounceableGenerator;
use super::generator::DomainGenerator;
use super::six::SixLetterGenerator;
use super::state::{ScanState, SnipedDomain, FailedDomain};
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
    /// 6-letter pronounceable (high-quality subset)
    Six,
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
    pub rdap_status: Vec<String>,
    pub error_message: Option<String>,
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
            concurrency: 20,
            batch_size: 100,
            expiring_days: 7,
            state_file: None,
            save_interval: 1000,
            rate_limit_ms: 500,
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
    pub expired_count: usize,
    pub error_count: u64,
    pub domains_per_second: f64,
    pub estimated_remaining: Option<Duration>,
}

/// Unified generator wrapper
enum GeneratorKind {
    Full(DomainGenerator),
    Pronounceable(PronounceableGenerator),
    Words(WordGenerator),
    Six(SixLetterGenerator),
}

impl GeneratorKind {
    fn next_batch(&mut self, count: usize) -> Vec<String> {
        match self {
            GeneratorKind::Full(g) => g.next_batch(count),
            GeneratorKind::Pronounceable(g) => g.next_batch(count),
            GeneratorKind::Words(g) => g.next_batch(count),
            GeneratorKind::Six(g) => g.next_batch(count),
        }
    }

    fn is_exhausted(&self) -> bool {
        match self {
            GeneratorKind::Full(g) => g.is_exhausted(),
            GeneratorKind::Pronounceable(g) => g.is_exhausted(),
            GeneratorKind::Words(g) => g.is_exhausted(),
            GeneratorKind::Six(g) => g.is_exhausted(),
        }
    }

    fn current_index(&self) -> u64 {
        match self {
            GeneratorKind::Full(g) => g.current_index(),
            GeneratorKind::Pronounceable(g) => g.current_index(),
            GeneratorKind::Words(g) => g.current_index(),
            GeneratorKind::Six(g) => g.current_index(),
        }
    }

    fn set_index(&mut self, index: u64) {
        match self {
            GeneratorKind::Full(g) => g.set_index(index),
            GeneratorKind::Pronounceable(g) => g.set_index(index),
            GeneratorKind::Words(g) => g.set_index(index),
            GeneratorKind::Six(g) => g.set_index(index),
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
            ScanMode::Six => {
                let gen = SixLetterGenerator::new();
                let total = gen.total() * config.tlds.len() as u64;
                (GeneratorKind::Six(gen), total, 6)
            }
        };

        let state = ScanState::new(length, config.tlds.clone(), total);
        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
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
            ScanMode::Six => {
                GeneratorKind::Six(SixLetterGenerator::new())
            }
        };
        generator.set_index(state.current_index);

        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
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
            ScanMode::Six => 6,
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
                            rdap_status: result.rdap_status.clone(),
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
                            rdap_status: result.rdap_status.clone(),
                            found_at: Utc::now(),
                        });
                    }
                    SnipeStatus::Error => {
                        self.state.add_error(FailedDomain {
                            domain: result.domain.clone(),
                            tld: result.tld.clone(),
                            full_domain: result.full_domain.clone(),
                            error: result.error_message.clone().unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: Utc::now(),
                        });
                    }
                    SnipeStatus::Taken => {
                        // Track "expired but not yet available" separately for monitoring.
                        // Condition: RDAP returned 200, we parsed an expiration_date, and it's already in the past (<= now).
                        let is_expired = result
                            .days_until_expiry
                            .map(|d| d <= 0)
                            .unwrap_or(false)
                            && result.expiration_date.is_some();

                        if is_expired {
                            self.state.expired.push(SnipedDomain {
                                domain: result.domain.clone(),
                                tld: result.tld.clone(),
                                full_domain: result.full_domain.clone(),
                                expiration_date: result.expiration_date,
                                days_until_expiry: result.days_until_expiry,
                                registrar: result.registrar.clone(),
                                rdap_status: result.rdap_status.clone(),
                                found_at: Utc::now(),
                            });
                            self.state.updated_at = Utc::now();
                        }
                    }
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
                expired_count: self.state.expired.len(),
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
                                    rdap_status: Vec::new(),
                                    error_message: None,
                                })
                            } else if status_code == 200 {
                                // Domain is taken, try to get expiration
                                let (expiration, registrar, rdap_status) = response
                                    .json::<serde_json::Value>()
                                    .await
                                    .ok()
                                    .map(|v| {
                                        let expiration = v
                                            .get("events")
                                            .and_then(|ev| ev.as_array())
                                            .and_then(|events| {
                                                events.iter().find(|e| {
                                                    e.get("eventAction").and_then(|a| a.as_str())
                                                        == Some("expiration")
                                                })
                                            })
                                            .and_then(|e| e.get("eventDate").and_then(|d| d.as_str()))
                                            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                                            .map(|d| d.with_timezone(&Utc));

                                        let registrar = extract_rdap_registrar(&v);
                                        let status = extract_rdap_status(&v);

                                        (expiration, registrar, status)
                                    })
                                    .unwrap_or((None, None, Vec::new()));

                                let days_until = expiration.map(|exp| (exp - Utc::now()).num_days());
                                let is_expiring = days_until.map(|d| d > 0 && d <= expiring_days as i64).unwrap_or(false);

                                Some(SnipeResult {
                                    domain: name,
                                    tld,
                                    full_domain,
                                    status: if is_expiring { SnipeStatus::ExpiringSoon } else { SnipeStatus::Taken },
                                    expiration_date: expiration,
                                    days_until_expiry: days_until,
                                    registrar,
                                    rdap_status,
                                    error_message: None,
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
                                    rdap_status: Vec::new(),
                                    error_message: Some(format!("HTTP {}", status_code)),
                                })
                            }
                        }
                        Err(e) => Some(SnipeResult {
                            domain: name,
                            tld,
                            full_domain,
                            status: SnipeStatus::Error,
                            expiration_date: None,
                            days_until_expiry: None,
                            registrar: None,
                            rdap_status: Vec::new(),
                            error_message: Some(e.to_string()),
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

/// Report returned by `recheck_expiring_soon`.
#[derive(Debug, Clone, Default)]
pub struct RecheckReport {
    /// Total number of items checked across lists.
    pub total_checked: usize,

    /// How many `expiring_soon` entries were checked.
    pub checked_expiring: usize,
    /// How many `available` entries were checked.
    pub checked_available: usize,
    /// How many `expired` entries were checked.
    pub checked_expired: usize,

    /// Items that remain in `expiring_soon` after recheck.
    pub still_expiring: usize,
    /// Items moved from `expiring_soon` -> `available`.
    pub expiring_now_available: usize,
    /// Items removed from `expiring_soon` because they are no longer within threshold (but still taken).
    pub no_longer_expiring: usize,
    /// Items whose expiration is in the past.
    pub already_expired: usize,
    /// Expiring list items kept due to errors/unknown parsing.
    pub expiring_errors_kept: usize,

    /// Items that remain in `available` after recheck.
    pub still_available: usize,
    /// Items removed from `available` because they are now taken.
    pub no_longer_available: usize,
    /// Items moved from `available` -> `expiring_soon` (now taken but expiring within threshold).
    pub available_now_expiring: usize,
    /// Available list items kept due to errors/unknown parsing.
    pub available_errors_kept: usize,

    /// Items that remain in `expired` after recheck.
    pub still_expired: usize,
    /// Items moved from `expired` -> `available` (now 404).
    pub expired_now_available: usize,
    /// Items moved from `expired` -> `expiring_soon` (renewed/updated and now within threshold).
    pub expired_now_expiring: usize,
    /// Items removed from `expired` because they are no longer expired (but also not expiring soon).
    pub no_longer_expired: usize,
    /// Expired list items kept due to errors/unknown parsing.
    pub expired_errors_kept: usize,
}

enum RecheckTarget {
    Expiring,
    Available,
    Expired,
}

enum RecheckDecision {
    // expiring_soon list outcomes
    ExpiringStill(SnipedDomain),
    ExpiringNowAvailable(SnipedDomain),
    /// expiring_soon -> expired watchlist (still 200 but expiration <= now)
    ExpiringNowExpired(SnipedDomain),
    ExpiringNoLonger,
    ExpiringErrorKeep(SnipedDomain),

    // available list outcomes
    AvailableStill(SnipedDomain),
    AvailableNoLonger,
    AvailableNowExpiring(SnipedDomain),
    AvailableErrorKeep(SnipedDomain),

    // expired list outcomes
    ExpiredStill(SnipedDomain),
    ExpiredNowAvailable(SnipedDomain),
    ExpiredNowExpiring(SnipedDomain),
    ExpiredNoLonger,
    ExpiredErrorKeep(SnipedDomain),
}

/// Re-check the `expiring_soon` list of a previously saved scan state.
///
/// This mutates the provided `state`:
/// - entries that become **available** are moved into `state.available`
/// - entries that are **no longer expiring soon** are removed from `state.expiring_soon`
/// - entries with **errors / unknown expiry** are kept in `state.expiring_soon`
pub async fn recheck_expiring_soon(
    state: &mut ScanState,
    expiring_days: u32,
    concurrency: usize,
) -> Result<RecheckReport> {
    use std::future::Future;
    use std::pin::Pin;

    let original_expiring = std::mem::take(&mut state.expiring_soon);
    let original_available = std::mem::take(&mut state.available);
    let original_expired = std::mem::take(&mut state.expired);
    let total = original_expiring.len() + original_available.len() + original_expired.len();

    let semaphore = Arc::new(Semaphore::new(concurrency.max(1)));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .pool_max_idle_per_host(concurrency.max(1))
        .build()
        .expect("Failed to create HTTP client");

    let now = Utc::now();
    if total == 0 {
        return Ok(RecheckReport::default());
    }

    let mut tasks: Vec<Pin<Box<dyn Future<Output = RecheckDecision> + Send>>> = Vec::with_capacity(total);

    for entry in original_expiring {
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let now = now;
        tasks.push(Box::pin(recheck_one(
            RecheckTarget::Expiring,
            entry,
            expiring_days,
            now,
            client,
            semaphore,
        )));
    }

    for entry in original_available {
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let now = now;
        tasks.push(Box::pin(recheck_one(
            RecheckTarget::Available,
            entry,
            expiring_days,
            now,
            client,
            semaphore,
        )));
    }

    for entry in original_expired {
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let now = now;
        tasks.push(Box::pin(recheck_one(
            RecheckTarget::Expired,
            entry,
            expiring_days,
            now,
            client,
            semaphore,
        )));
    }

    let decisions = join_all(tasks).await;
    let mut report = RecheckReport {
        total_checked: total,
        ..Default::default()
    };

    for decision in decisions {
        match decision {
            RecheckDecision::ExpiringStill(d) => {
                state.expiring_soon.push(d);
                report.still_expiring += 1;
                report.checked_expiring += 1;
            }
            RecheckDecision::ExpiringNowAvailable(d) => {
                state.available.push(d);
                report.expiring_now_available += 1;
                report.checked_expiring += 1;
            }
            RecheckDecision::ExpiringNoLonger => {
                report.no_longer_expiring += 1;
                report.checked_expiring += 1;
            }
            RecheckDecision::ExpiringNowExpired(d) => {
                state.expired.push(d);
                report.already_expired += 1;
                report.checked_expiring += 1;
            }
            RecheckDecision::ExpiringErrorKeep(d) => {
                state.expiring_soon.push(d);
                report.expiring_errors_kept += 1;
                report.checked_expiring += 1;
            }
            RecheckDecision::AvailableStill(d) => {
                state.available.push(d);
                report.still_available += 1;
                report.checked_available += 1;
            }
            RecheckDecision::AvailableNoLonger => {
                report.no_longer_available += 1;
                report.checked_available += 1;
            }
            RecheckDecision::AvailableNowExpiring(d) => {
                state.expiring_soon.push(d);
                report.available_now_expiring += 1;
                report.checked_available += 1;
            }
            RecheckDecision::AvailableErrorKeep(d) => {
                state.available.push(d);
                report.available_errors_kept += 1;
                report.checked_available += 1;
            }

            RecheckDecision::ExpiredStill(d) => {
                state.expired.push(d);
                report.still_expired += 1;
                report.checked_expired += 1;
            }
            RecheckDecision::ExpiredNowAvailable(d) => {
                state.available.push(d);
                report.expired_now_available += 1;
                report.checked_expired += 1;
            }
            RecheckDecision::ExpiredNowExpiring(d) => {
                state.expiring_soon.push(d);
                report.expired_now_expiring += 1;
                report.checked_expired += 1;
            }
            RecheckDecision::ExpiredNoLonger => {
                report.no_longer_expired += 1;
                report.checked_expired += 1;
            }
            RecheckDecision::ExpiredErrorKeep(d) => {
                state.expired.push(d);
                report.expired_errors_kept += 1;
                report.checked_expired += 1;
            }
        }
    }

    // Record update timestamp (append-only history).
    state.updated_at = Utc::now();
    state.update_times.push(state.updated_at);

    // Keep expiring_soon sorted: closest expiration first; unknown expiration last.
    state.expiring_soon.sort_by(|a, b| {
        match (a.expiration_date, b.expiration_date) {
            (Some(da), Some(db)) => da
                .cmp(&db)
                .then_with(|| a.full_domain.cmp(&b.full_domain)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.full_domain.cmp(&b.full_domain),
        }
    });
    Ok(report)
}

async fn recheck_one(
    target: RecheckTarget,
    entry: SnipedDomain,
    expiring_days: u32,
    now: chrono::DateTime<Utc>,
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
) -> RecheckDecision {
    let _permit = semaphore.acquire().await.ok();

    let tld = entry.tld.to_lowercase();
    let rdap_url = match rdap_base_url(&tld) {
        Some(u) => u,
        None => {
            return match target {
                RecheckTarget::Expiring => RecheckDecision::ExpiringErrorKeep(entry),
                RecheckTarget::Available => RecheckDecision::AvailableErrorKeep(entry),
                RecheckTarget::Expired => RecheckDecision::ExpiredErrorKeep(entry),
            };
        }
    };

    let url = format!("{}domain/{}", rdap_url, entry.full_domain);
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => {
            return match target {
                RecheckTarget::Expiring => RecheckDecision::ExpiringErrorKeep(entry),
                RecheckTarget::Available => RecheckDecision::AvailableErrorKeep(entry),
                RecheckTarget::Expired => RecheckDecision::ExpiredErrorKeep(entry),
            };
        }
    };

    let status = resp.status().as_u16();
    if status == 404 {
        // Available for registration
        return match target {
            RecheckTarget::Expiring => RecheckDecision::ExpiringNowAvailable(SnipedDomain {
                domain: entry.domain,
                tld: entry.tld,
                full_domain: entry.full_domain,
                expiration_date: None,
                days_until_expiry: None,
                registrar: None,
                rdap_status: Vec::new(),
                found_at: now,
            }),
            RecheckTarget::Available => RecheckDecision::AvailableStill(SnipedDomain {
                found_at: now,
                ..entry
            }),
            RecheckTarget::Expired => RecheckDecision::ExpiredNowAvailable(SnipedDomain {
                domain: entry.domain,
                tld: entry.tld,
                full_domain: entry.full_domain,
                expiration_date: None,
                days_until_expiry: None,
                registrar: None,
                rdap_status: Vec::new(),
                found_at: now,
            }),
        };
    }

    if status != 200 {
        return match target {
            RecheckTarget::Expiring => RecheckDecision::ExpiringErrorKeep(entry),
            RecheckTarget::Available => RecheckDecision::AvailableErrorKeep(entry),
            RecheckTarget::Expired => RecheckDecision::ExpiredErrorKeep(entry),
        };
    }

    // Taken: refresh expiration_date (and registrar if present).
    let json: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return match target {
                RecheckTarget::Expiring => RecheckDecision::ExpiringErrorKeep(entry),
                RecheckTarget::Available => RecheckDecision::AvailableErrorKeep(entry),
                RecheckTarget::Expired => RecheckDecision::ExpiredErrorKeep(entry),
            };
        }
    };

    let rdap_status = extract_rdap_status(&json);

    let expiration = json
        .get("events")
        .and_then(|v| v.as_array())
        .and_then(|events| {
            events.iter().find(|e| {
                e.get("eventAction")
                    .and_then(|a| a.as_str())
                    == Some("expiration")
            })
        })
        .and_then(|e| e.get("eventDate").and_then(|d| d.as_str()))
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc));

    let registrar = json
        .as_object()
        .and_then(|_| extract_rdap_registrar(&json))
        .or(entry.registrar.clone());

    let days_until = expiration.map(|exp| (exp - now).num_days());

    // If we cannot parse expiration, keep in its original list (best-effort).
    if expiration.is_none() {
        return match target {
            RecheckTarget::Expiring => RecheckDecision::ExpiringErrorKeep(SnipedDomain { registrar, ..entry }),
            RecheckTarget::Available => RecheckDecision::AvailableErrorKeep(SnipedDomain { registrar, ..entry }),
            RecheckTarget::Expired => RecheckDecision::ExpiredErrorKeep(SnipedDomain { registrar, ..entry }),
        };
    }

    let days = days_until.unwrap_or(0);
    let refreshed = SnipedDomain {
        expiration_date: expiration,
        days_until_expiry: days_until,
        registrar,
        rdap_status,
        found_at: now,
        ..entry
    };

    match target {
        RecheckTarget::Expiring => {
            if days > 0 && days <= expiring_days as i64 {
                RecheckDecision::ExpiringStill(refreshed)
            } else if days <= 0 {
                // Move into dedicated `expired` watchlist.
                RecheckDecision::ExpiringNowExpired(refreshed)
            } else {
                RecheckDecision::ExpiringNoLonger
            }
        }
        RecheckTarget::Available => {
            if days > 0 && days <= expiring_days as i64 {
                RecheckDecision::AvailableNowExpiring(refreshed)
            } else {
                RecheckDecision::AvailableNoLonger
            }
        }
        RecheckTarget::Expired => {
            if days <= 0 {
                RecheckDecision::ExpiredStill(refreshed)
            } else if days > 0 && days <= expiring_days as i64 {
                RecheckDecision::ExpiredNowExpiring(refreshed)
            } else {
                RecheckDecision::ExpiredNoLonger
            }
        }
    }
}

fn extract_rdap_status(v: &serde_json::Value) -> Vec<String> {
    v.get("status")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn extract_rdap_registrar(v: &serde_json::Value) -> Option<String> {
    v.get("entities")
        .and_then(|x| x.as_array())
        .and_then(|entities| {
            entities.iter().find(|e| {
                e.get("roles")
                    .and_then(|r| r.as_array())
                    .is_some_and(|roles| roles.iter().any(|role| role.as_str() == Some("registrar")))
            })
        })
        .and_then(|entity| entity.get("vcardArray"))
        .and_then(|vcard| vcard.get(1))
        .and_then(|props| props.as_array())
        .and_then(|props| props.get(0))
        .and_then(|prop| prop.as_array())
        .and_then(|prop| prop.get(3))
        .and_then(|name| name.as_str())
        .map(|s| s.to_string())
}
