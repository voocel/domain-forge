//! Ollama provider implementation
//! 
//! Supports local Ollama API for running LLMs locally

use crate::error::{DomainForgeError, Result};
use crate::llm::LlmProvider;
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{build_domain_prompt, parse_domain_suggestions};

/// Ollama provider implementation for local LLM inference
pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
    temperature: f32,
}

impl OllamaProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Longer timeout for local inference
            .build()
            .map_err(|e| DomainForgeError::network(e.to_string(), None, None))?;

        Ok(Self {
            client,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
            temperature: config.temperature,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_domains(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let prompt = build_domain_prompt(config);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            temperature: self.temperature,
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainForgeError::network(
                format!("Failed to connect to Ollama: {}", e),
                None,
                Some(url.clone())
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            let error_msg = match status.as_u16() {
                404 => format!("Model '{}' not found. Please pull the model first: ollama pull {}", self.model, self.model),
                500..=599 => format!("Ollama server error ({}). Make sure Ollama is running", status),
                _ => format!("Ollama API request failed ({}): {}", status, error_text),
            };
            
            return Err(DomainForgeError::network(
                error_msg,
                Some(status.as_u16()),
                Some(url),
            ));
        }

        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| DomainForgeError::parse(e.to_string(), None))?;

        parse_domain_suggestions(&ollama_response.response, config)
    }

    fn name(&self) -> &'static str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_ready(&self) -> bool {
        true // Ollama doesn't need API key
    }
}

// Ollama API structures
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}
