//! OpenAI provider implementation
//! 
//! Supports OpenAI API and OpenAI-compatible APIs (OpenRouter, OneAPI, etc.)

use crate::error::{DomainForgeError, Result};
use crate::llm::LlmProvider;
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{build_domain_prompt, parse_domain_suggestions};

/// OpenAI provider implementation
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    temperature: f32,
}

impl OpenAiProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(DomainForgeError::config("OpenAI API key is required".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| DomainForgeError::network(e.to_string(), None, None))?;

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            temperature: config.temperature,
        })
    }

    /// Intelligently constructs the full API URL
    fn build_url(&self, endpoint: &str) -> String {
        let base_url = self.base_url.trim_end_matches('/');
        if base_url.ends_with("/v1") {
            format!("{}{}", base_url, endpoint)
        } else {
            format!("{}/v1{}", base_url, endpoint)
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_domains(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let prompt = build_domain_prompt(config);
        
        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: "You are a domain name generator. Generate creative domain names and return them as a JSON array.".to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: self.temperature,
            max_tokens: 2000,
        };

        let url = self.build_url("/chat/completions");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainForgeError::network(
                format!("Failed to connect to API: {}", e),
                None,
                Some(url.clone())
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            let error_msg = match status.as_u16() {
                401 => format!("Authentication failed (401). Please check your API key for {}", self.base_url),
                403 => format!("Access forbidden (403). Your API key may not have permission for this endpoint"),
                429 => format!("Rate limit exceeded (429). Please try again later"),
                500..=599 => format!("Server error ({}). The API service is experiencing issues", status),
                _ => format!("API request failed ({}): {}", status, error_text),
            };
            
            return Err(DomainForgeError::network(
                error_msg,
                Some(status.as_u16()),
                Some(url),
            ));
        }

        let openai_response: OpenAiResponse = response.json().await
            .map_err(|e| DomainForgeError::parse(e.to_string(), None))?;
        
        let content = openai_response.choices.get(0)
            .ok_or_else(|| DomainForgeError::internal("No response from OpenAI API".to_string()))?
            .message.content.clone();

        parse_domain_suggestions(&content, config)
    }

    fn name(&self) -> &'static str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// OpenAI API structures
#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}


