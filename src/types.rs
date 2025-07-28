//! Core types and structures for domain-forge

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LLM provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAi,
    Claude,
    Ollama,
    Custom,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::OpenAi => write!(f, "openai"),
            LlmProvider::Claude => write!(f, "claude"),
            LlmProvider::Ollama => write!(f, "ollama"),
            LlmProvider::Custom => write!(f, "custom"),
        }
    }
}

/// Domain generation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStyle {
    Creative,
    Professional,
    Brandable,
    Descriptive,
    Short,
    Tech,
}

impl std::fmt::Display for GenerationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationStyle::Creative => write!(f, "creative"),
            GenerationStyle::Professional => write!(f, "professional"),
            GenerationStyle::Brandable => write!(f, "brandable"),
            GenerationStyle::Descriptive => write!(f, "descriptive"),
            GenerationStyle::Short => write!(f, "short"),
            GenerationStyle::Tech => write!(f, "tech"),
        }
    }
}

/// Domain availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AvailabilityStatus {
    Available,
    Taken,
    Unknown,
    Error,
}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailabilityStatus::Available => write!(f, "available"),
            AvailabilityStatus::Taken => write!(f, "taken"),
            AvailabilityStatus::Unknown => write!(f, "unknown"),
            AvailabilityStatus::Error => write!(f, "error"),
        }
    }
}

/// Domain checking method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckMethod {
    Rdap,
    Whois,
    Unknown,
}

impl std::fmt::Display for CheckMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckMethod::Rdap => write!(f, "rdap"),
            CheckMethod::Whois => write!(f, "whois"),
            CheckMethod::Unknown => write!(f, "unknown"),
        }
    }
}

/// Generated domain suggestion with optimized memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSuggestion {
    /// Domain name without TLD - use String for compatibility
    pub name: String,
    /// AI reasoning for this suggestion
    pub reasoning: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Top-level domain - use String for compatibility
    pub tld: String,
    /// Full domain name (lazy computed when needed)
    #[serde(skip)]
    pub full_domain: Option<String>,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

impl DomainSuggestion {
    /// Create new domain suggestion
    pub fn new(name: impl Into<String>, tld: impl Into<String>, confidence: f32, reasoning: Option<impl Into<String>>) -> Self {
        let name = name.into();
        let tld = tld.into();
        
        Self {
            name,
            reasoning: reasoning.map(Into::into),
            confidence,
            tld,
            full_domain: None,
            generated_at: Utc::now(),
        }
    }
    
    /// Get full domain name (computed lazily)
    pub fn full_domain(&mut self) -> &str {
        if self.full_domain.is_none() {
            let full = format!("{}.{}", self.name, self.tld);
            self.full_domain = Some(full);
        }
        self.full_domain.as_ref().unwrap()
    }
    
    /// Get full domain name without mutable reference (creates new string)
    pub fn get_full_domain(&self) -> String {
        format!("{}.{}", self.name, self.tld)
    }
}

/// Domain availability check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResult {
    pub domain: String,
    pub status: AvailabilityStatus,
    pub method: CheckMethod,
    pub checked_at: DateTime<Utc>,
    pub check_duration: Option<Duration>,
    pub registrar: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub nameservers: Vec<String>,
    pub error_message: Option<String>,
}

/// Combined domain generation and check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainForgeResult {
    pub suggestion: DomainSuggestion,
    pub availability: Option<DomainResult>,
}

/// Configuration for domain generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub provider: LlmProvider,
    pub count: usize,
    pub style: GenerationStyle,
    pub tlds: Vec<String>,
    pub temperature: f32,
    pub description: String,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAi,
            count: 5,
            style: GenerationStyle::Creative,
            tlds: vec!["com".to_string(), "org".to_string(), "io".to_string()],
            temperature: 0.7,
            description: "".to_string(),
        }
    }
}

/// Configuration for domain checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    pub concurrent_checks: usize,
    pub timeout: Duration,
    pub enable_rdap: bool,
    pub enable_whois: bool,
    pub detailed_info: bool,
    pub retry_attempts: usize,
    pub rate_limit: u32,
    /// Connection pool size for HTTP clients
    pub connection_pool_size: usize,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            concurrent_checks: 10,
            timeout: Duration::from_secs(30),
            enable_rdap: true,
            enable_whois: true,
            detailed_info: false,
            retry_attempts: 3,
            rate_limit: 60,
            connection_pool_size: 10,
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4.1-mini".to_string(),
            api_key: String::new(),
            base_url: None,
            temperature: 0.7,
        }
    }
}

/// Simple performance metrics (non-intrusive)
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub domains_generated: std::sync::atomic::AtomicU64,
    pub domains_checked: std::sync::atomic::AtomicU64,
    pub api_calls_made: std::sync::atomic::AtomicU64,
    pub errors_encountered: std::sync::atomic::AtomicU64,
    pub total_check_time_ms: std::sync::atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment_domains_generated(&self) {
        self.domains_generated.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn increment_domains_checked(&self) {
        self.domains_checked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn increment_api_calls(&self) {
        self.api_calls_made.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn increment_errors(&self) {
        self.errors_encountered.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn add_check_time(&self, milliseconds: u64) {
        self.total_check_time_ms.fetch_add(milliseconds, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            domains_generated: self.domains_generated.load(std::sync::atomic::Ordering::Relaxed),
            domains_checked: self.domains_checked.load(std::sync::atomic::Ordering::Relaxed),
            api_calls_made: self.api_calls_made.load(std::sync::atomic::Ordering::Relaxed),
            errors_encountered: self.errors_encountered.load(std::sync::atomic::Ordering::Relaxed),
            total_check_time_ms: self.total_check_time_ms.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub domains_generated: u64,
    pub domains_checked: u64,
    pub api_calls_made: u64,
    pub errors_encountered: u64,
    pub total_check_time_ms: u64,
}

impl MetricsSnapshot {
    pub fn avg_check_time_ms(&self) -> f64 {
        if self.domains_checked == 0 {
            0.0
        } else {
            self.total_check_time_ms as f64 / self.domains_checked as f64
        }
    }
}