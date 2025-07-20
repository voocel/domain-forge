//! Google Gemini provider implementation
//! 
//! Supports Google's Gemini API

use crate::error::{DomainForgeError, Result};
use crate::llm::LlmProvider;
use crate::types::{DomainSuggestion, GenerationConfig, LlmConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{build_domain_prompt, parse_domain_suggestions};

/// Google Gemini provider implementation
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    temperature: f32,
}

impl GeminiProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(DomainForgeError::config("Gemini API key is required".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| DomainForgeError::network(e.to_string(), None, None))?;

        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            temperature: config.temperature,
        })
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn generate_domains(&self, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
        let prompt = build_domain_prompt(config);
        
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt,
                }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: self.temperature,
                max_output_tokens: 1000,
            },
        };

        let url = format!("{}/models/{}:generateContent?key={}", 
            self.base_url, self.model, self.api_key);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainForgeError::network(
                format!("Failed to connect to Gemini API: {}", e),
                None,
                Some(url.clone())
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            let error_msg = match status.as_u16() {
                401 => "Authentication failed (401). Please check your Gemini API key".to_string(),
                403 => "Access forbidden (403). Your API key may not have permission".to_string(),
                429 => "Rate limit exceeded (429). Please try again later".to_string(),
                500..=599 => format!("Gemini server error ({}). The API service is experiencing issues", status),
                _ => format!("Gemini API request failed ({}): {}", status, error_text),
            };
            
            return Err(DomainForgeError::network(
                error_msg,
                Some(status.as_u16()),
                Some(url),
            ));
        }

        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| DomainForgeError::parse(e.to_string(), None))?;
        
        let content = gemini_response.candidates.get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .ok_or_else(|| DomainForgeError::internal("No response from Gemini API".to_string()))?;

        parse_domain_suggestions(&content, config)
    }

    fn name(&self) -> &'static str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// Gemini API structures
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}
