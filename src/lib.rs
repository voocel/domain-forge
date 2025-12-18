//! Domain Forge - AI-powered domain name generation and availability checking
//!
//! A simple and elegant CLI tool for generating domain names using AI and checking their availability.

pub mod domain;
pub mod error;
pub mod llm;
pub mod rdap;
pub mod snipe;
pub mod types;

// Re-export commonly used types
pub use error::{DomainForgeError, Result};
pub use types::{
    AvailabilityStatus, CheckConfig, DomainForgeResult, DomainResult,
    DomainSuggestion, GenerationConfig, GenerationStyle, LlmProvider, LlmConfig,
    PerformanceMetrics, MetricsSnapshot, DomainSession,
};

// Re-export main functionality
pub use domain::DomainChecker;
pub use llm::DomainGenerator;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library
pub fn init() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    Ok(())
}