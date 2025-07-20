//! LLM (Large Language Model) integration module
//!
//! Simple and elegant interface for generating domain names using AI.

pub mod generator;
pub mod providers;

// Re-export main functionality
pub use generator::DomainGenerator;

use crate::error::Result;
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use async_trait::async_trait;

/// Core trait for all LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate domain suggestions
    async fn generate_domains(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>>;
    
    /// Get provider name
    fn name(&self) -> &'static str;
    
    /// Get model name being used
    fn model(&self) -> &str;
    
    /// Check if provider is configured and ready
    fn is_ready(&self) -> bool;
}



/// Get available LLM providers
pub fn available_providers() -> Vec<&'static str> {
    vec!["openai", "anthropic", "gemini", "ollama"]
}

/// Create an LLM provider from configuration
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(providers::OpenAiProvider::new(config)?)),
        "anthropic" => Ok(Box::new(providers::AnthropicProvider::new(config)?)),
        "gemini" => Ok(Box::new(providers::GeminiProvider::new(config)?)),
        "ollama" => Ok(Box::new(providers::OllamaProvider::new(config)?)),
        _ => Err(crate::error::DomainForgeError::config(
            format!("Unsupported LLM provider: {}. Supported providers: {}",
                config.provider,
                available_providers().join(", ")
            )
        )),
    }
}