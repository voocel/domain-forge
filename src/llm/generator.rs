//! Domain generator using LLM

use crate::error::Result;
use crate::llm::{LlmProvider, create_provider};
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig, PerformanceMetrics};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Domain generator that uses LLM to generate domain suggestions
/// Enhanced with thread-safe shared state and performance metrics
#[derive(Clone)]
pub struct DomainGenerator {
    providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    default_provider: Arc<RwLock<String>>,
    metrics: Arc<PerformanceMetrics>,
}

impl DomainGenerator {
    /// Create a new domain generator
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            default_provider: Arc::new(RwLock::new("openai".to_string())),
            metrics: Arc::new(PerformanceMetrics::new()),
        }
    }

    /// Add an LLM provider (thread-safe)
    pub fn add_provider(&self, config: &LlmConfig) -> Result<()> {
        let provider = create_provider(config)?;
        let mut providers = self.providers.write();
        providers.insert(config.provider.clone(), Arc::from(provider));
        Ok(())
    }
    
    /// Set default provider (thread-safe)
    pub fn set_default_provider(&self, provider: &str) {
        let providers = self.providers.read();
        if providers.contains_key(provider) {
            let mut default = self.default_provider.write();
            *default = provider.to_string();
        }
    }

    /// Generate domain suggestions using default provider
    pub async fn generate(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let default_provider = self.default_provider.read().clone();
        self.generate_with_provider(config, &default_provider).await
    }

    /// Generate domain suggestions using specific provider
    pub async fn generate_with_provider(
        &self,
        config: &GenerationConfig,
        provider_name: &str,
    ) -> Result<Vec<DomainSuggestion>> {
        let start_time = Instant::now();
        
        // Record API call
        self.metrics.increment_api_calls();
        
        // Get provider (clone Arc to avoid holding lock during async operation)
        let provider = {
            let providers = self.providers.read();
            providers.get(provider_name)
                .ok_or_else(|| crate::error::DomainForgeError::config(
                    format!("Provider not configured: {}", provider_name)
                ))?
                .clone()
        };
        
        // Call the provider's generate_domains method (no lock held)
        let result = provider.generate_domains(config).await;
        
        match &result {
            Ok(domains) => {
                self.metrics.increment_domains_generated();
                let elapsed = start_time.elapsed();
                tracing::info!(
                    provider = %provider_name,
                    domains_count = %domains.len(),
                    duration_ms = %elapsed.as_millis(),
                    "Domain generation completed"
                );
            }
            Err(e) => {
                self.metrics.increment_errors();
                tracing::warn!(
                    provider = %provider_name,
                    error = %e,
                    duration_ms = %start_time.elapsed().as_millis(),
                    "Domain generation failed"
                );
            }
        }
        
        result
    }
    
    /// Generate with fallback to other providers (enhanced with metrics)
    pub async fn generate_with_fallback(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let mut last_error = None;
        let overall_start = Instant::now();

        // Try default provider first
        let default_provider = self.default_provider.read().clone();
        if self.has_provider(&default_provider) {
            match self.generate_with_provider(config, &default_provider).await {
                Ok(result) => {
                    tracing::info!(
                        provider = %default_provider,
                        fallback_used = false,
                        duration_ms = %overall_start.elapsed().as_millis(),
                        "Successfully generated domains with default provider"
                    );
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!(provider = %default_provider, error = %e, "Default provider failed");
                    last_error = Some(e);
                }
            }
        }

        // Try other providers
        let available_providers: Vec<String> = {
            let providers = self.providers.read();
            providers.keys()
                .filter(|&name| name != &default_provider)
                .cloned()
                .collect()
        };

        for provider_name in available_providers {
            match self.generate_with_provider(config, &provider_name).await {
                Ok(result) => {
                    tracing::info!(
                        provider = %provider_name,
                        fallback_used = true,
                        duration_ms = %overall_start.elapsed().as_millis(),
                        "Successfully generated domains with fallback provider"
                    );
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!(provider = %provider_name, error = %e, "Fallback provider failed");
                    last_error = Some(e);
                }
            }
        }

        self.metrics.increment_errors();
        Err(last_error.unwrap_or_else(|| {
            crate::error::DomainForgeError::config("No providers configured".to_string())
        }))
    }

    /// Get available providers (thread-safe)
    pub fn available_providers(&self) -> Vec<String> {
        let providers = self.providers.read();
        providers.keys().cloned().collect()
    }

    /// Check if provider is available (thread-safe)
    pub fn has_provider(&self, provider: &str) -> bool {
        let providers = self.providers.read();
        providers.contains_key(provider)
    }

    /// Check if any providers are configured (thread-safe)
    pub fn is_ready(&self) -> bool {
        let providers = self.providers.read();
        !providers.is_empty()
    }
    
    /// Get performance metrics
    pub fn get_metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }
    
    /// Get current metrics snapshot
    pub fn get_metrics_snapshot(&self) -> crate::types::MetricsSnapshot {
        self.metrics.get_stats()
    }
}

impl Default for DomainGenerator {
    fn default() -> Self {
        Self::new()
    }
}



