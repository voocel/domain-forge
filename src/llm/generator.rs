//! Domain generator using LLM

use crate::error::Result;
use crate::llm::{LlmProvider, create_provider};
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use std::collections::HashMap;

/// Domain generator that uses LLM to generate domain suggestions
pub struct DomainGenerator {
    providers: HashMap<String, Box<dyn LlmProvider>>,
    default_provider: String,
}

impl DomainGenerator {
    /// Create a new domain generator
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: "openai".to_string(),
        }
    }

    /// Add an LLM provider
    pub fn add_provider(&mut self, config: &LlmConfig) -> Result<()> {
        let provider = create_provider(config)?;
        self.providers.insert(config.provider.clone(), provider);
        Ok(())
    }
    
    /// Set default provider
    pub fn set_default_provider(&mut self, provider: &str) {
        if self.providers.contains_key(provider) {
            self.default_provider = provider.to_string();
        }
    }

    /// Generate domain suggestions using default provider
    pub async fn generate(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        self.generate_with_provider(config, &self.default_provider).await
    }

    /// Generate domain suggestions using specific provider
    pub async fn generate_with_provider(
        &self,
        config: &GenerationConfig,
        provider: &str,
    ) -> Result<Vec<DomainSuggestion>> {
        let provider = self.providers.get(provider).ok_or_else(|| {
            crate::error::DomainForgeError::config(format!("Provider not configured: {}", provider))
        })?;

        provider.generate_domains(config).await
    }
    
    /// Generate with fallback to other providers
    pub async fn generate_with_fallback(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let mut last_error = None;

        // Try default provider first
        if let Some(provider) = self.providers.get(&self.default_provider) {
            match provider.generate_domains(config).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Default provider '{}' failed: {}", self.default_provider, e);
                    last_error = Some(e);
                }
            }
        }

        // Try other providers
        for (provider_name, provider) in &self.providers {
            if provider_name != &self.default_provider {
                match provider.generate_domains(config).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        eprintln!("Provider '{}' failed: {}", provider_name, e);
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| crate::error::DomainForgeError::config("No providers configured".to_string())))
    }

    /// Get available providers
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Check if provider is available
    pub fn has_provider(&self, provider: &str) -> bool {
        self.providers.contains_key(provider)
    }

    /// Check if any providers are configured
    pub fn is_ready(&self) -> bool {
        !self.providers.is_empty()
    }
}

impl Default for DomainGenerator {
    fn default() -> Self {
        Self::new()
    }
}



