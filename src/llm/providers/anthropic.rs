//! Anthropic provider implementation
//!
//! Supports Anthropic's Claude API

use crate::error::{DomainForgeError, Result};
use crate::llm::LlmProvider;
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{build_domain_prompt, parse_domain_suggestions};

/// Anthropic provider implementation
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    temperature: f32,
}

impl AnthropicProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(DomainForgeError::config("Anthropic API key is required".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| DomainForgeError::network(e.to_string(), None, None))?;

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            temperature: config.temperature,
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate_domains(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let prompt = build_domain_prompt(config);
        
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: self.temperature,
            max_tokens: 1000,
        };

        let url = format!("{}/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainForgeError::network(
                format!("Failed to connect to Anthropic API: {}", e),
                None,
                Some(url.clone())
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            let error_msg = match status.as_u16() {
                401 => "Authentication failed (401). Please check your Anthropic API key".to_string(),
                403 => "Access forbidden (403). Your API key may not have permission".to_string(),
                429 => "Rate limit exceeded (429). Please try again later".to_string(),
                500..=599 => format!("Anthropic server error ({}). The API service is experiencing issues", status),
                _ => format!("Anthropic API request failed ({}): {}", status, error_text),
            };
            
            return Err(DomainForgeError::network(
                error_msg,
                Some(status.as_u16()),
                Some(url),
            ));
        }

        let anthropic_response: AnthropicResponse = response.json().await
            .map_err(|e| DomainForgeError::parse(e.to_string(), None))?;

        let content = anthropic_response.content.get(0)
            .ok_or_else(|| DomainForgeError::internal("No response from Anthropic API".to_string()))?
            .text.clone();

        parse_domain_suggestions(&content, config)
    }

    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// Anthropic API structures
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}
