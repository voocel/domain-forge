//! LLM provider implementations
//! 
//! Each provider is implemented in its own module for better organization and maintainability.

pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod ollama;

// Re-export providers for easy access
pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;

use crate::error::Result;
use crate::types::{DomainSuggestion, GenerationConfig};
use serde::{Deserialize, Serialize};

/// Common domain suggestion structure for parsing AI responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSuggestionRaw {
    pub name: String,
    pub reasoning: Option<String>,
    pub confidence: Option<f32>,
}

/// Parse domain suggestions from AI response - trust LLM completely
pub fn parse_domain_suggestions(content: &str, _config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
    let json_start = content.find('[').unwrap_or(0);
    let json_end = content.rfind(']').map(|i| i + 1).unwrap_or(content.len());
    let json_content = &content[json_start..json_end];

    let raw_suggestions: Vec<DomainSuggestionRaw> = serde_json::from_str(json_content)
        .map_err(|e| crate::error::DomainForgeError::parse(
            format!("Failed to parse AI response as JSON: {}", e),
            Some(json_content.to_string())
        ))?;

    let mut suggestions = Vec::new();

    for raw in raw_suggestions {
        let confidence = raw.confidence.unwrap_or(0.8);

        // LLM must return complete domain names with TLD
        if raw.name.contains('.') {
            let parts: Vec<&str> = raw.name.splitn(2, '.').collect();
            if parts.len() == 2 {
                suggestions.push(DomainSuggestion::new(
                    parts[0].to_string(),
                    parts[1].to_string(),
                    confidence,
                    raw.reasoning.clone(),
                ));
            }
        } else {
            // If LLM didn't return complete domain, it's an error
            return Err(crate::error::DomainForgeError::parse(
                format!("LLM returned incomplete domain '{}' - expected format: 'name.tld'", raw.name),
                Some(content.to_string())
            ));
        }
    }

    if suggestions.is_empty() {
        return Err(crate::error::DomainForgeError::parse(
            "No valid complete domain names found in LLM response".to_string(),
            Some(content.to_string())
        ));
    }

    Ok(suggestions)
}

/// Build domain generation prompt - trust LLM's intelligence completely
pub fn build_domain_prompt(config: &GenerationConfig) -> String {
    let avoid_guidance = if !config.avoid_names.is_empty() {
        format!("\n\nAvoid these taken names: {}", config.avoid_names.join(", "))
    } else {
        String::new()
    };

    format!(
        "Generate {} domain names for: {}

Style: {}
Available TLDs: {}{}

Return complete domain names as JSON:
[
  {{
    \"name\": \"example.com\",
    \"reasoning\": \"brief explanation\",
    \"confidence\": 0.85
  }}
]",
        config.count,
        config.description,
        config.style,
        config.tlds.join(", "),
        avoid_guidance
    )
}
