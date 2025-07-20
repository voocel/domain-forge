//! Domain availability checking module

pub mod checker;
pub mod validator;

// Re-export main functionality
pub use checker::DomainChecker;
pub use validator::DomainValidator;

use crate::error::Result;
use crate::types::{CheckMethod, DomainResult};
use async_trait::async_trait;

/// Trait for domain checking methods
#[async_trait]
pub trait DomainCheckMethod: Send + Sync {
    /// Check if a domain is available
    async fn check_domain(&self, domain: &str) -> Result<DomainResult>;
    
    /// Get the method name
    fn method_name(&self) -> CheckMethod;
    
    /// Check if this method supports the given TLD
    fn supports_tld(&self, tld: &str) -> bool;
}

/// Common TLD lists
pub const POPULAR_TLDS: &[&str] = &[
    "com", "org", "net", "io", "ai", "co", "me", "app", "dev", "tech", "xyz"
];

pub const STARTUP_TLDS: &[&str] = &[
    "com", "org", "io", "ai", "tech", "app", "dev", "xyz"
];

pub const ENTERPRISE_TLDS: &[&str] = &[
    "com", "org", "net", "biz", "info", "us"
];

pub const COUNTRY_TLDS: &[&str] = &[
    "us", "uk", "de", "fr", "ca", "au", "jp", "br", "in"
];

/// Get TLD list by name
pub fn get_tld_list(name: &str) -> Option<Vec<String>> {
    match name.to_lowercase().as_str() {
        "popular" => Some(POPULAR_TLDS.iter().map(|s| s.to_string()).collect()),
        "startup" => Some(STARTUP_TLDS.iter().map(|s| s.to_string()).collect()),
        "enterprise" => Some(ENTERPRISE_TLDS.iter().map(|s| s.to_string()).collect()),
        "country" => Some(COUNTRY_TLDS.iter().map(|s| s.to_string()).collect()),
        _ => None,
    }
}

/// Get all available TLD list names
pub fn get_tld_list_names() -> Vec<&'static str> {
    vec!["popular", "startup", "enterprise", "country"]
}