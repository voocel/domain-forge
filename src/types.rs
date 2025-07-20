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

/// Generated domain suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSuggestion {
    pub name: String,
    pub reasoning: Option<String>,
    pub confidence: f32,
    pub tld: String,
    pub full_domain: String,
    pub generated_at: DateTime<Utc>,
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
            description: String::new(),
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