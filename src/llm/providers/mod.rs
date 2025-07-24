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
use std::sync::Arc;

/// Common domain suggestion structure for parsing AI responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSuggestionRaw {
    pub name: String,
    pub reasoning: Option<String>,
    pub confidence: Option<f32>,
}

/// Parse domain suggestions from AI response text with optimized memory usage
pub fn parse_domain_suggestions(content: &str, config: &GenerationConfig) -> Result<Vec<DomainSuggestion>> {
    // Try to extract JSON from the response
    let json_start = content.find('[').unwrap_or(0);
    let json_end = content.rfind(']').map(|i| i + 1).unwrap_or(content.len());
    let json_content = &content[json_start..json_end];

    let raw_suggestions: Vec<DomainSuggestionRaw> = serde_json::from_str(json_content)
        .map_err(|e| crate::error::DomainForgeError::parse(
            format!("Failed to parse AI response as JSON: {}", e),
            Some(json_content.to_string())
        ))?;

    let mut suggestions = Vec::new();
    
    // Pre-compute shared Arc<str> for TLDs to reduce allocations
    let tld_arcs: Vec<Arc<str>> = config.tlds.iter().cloned().collect();
    
    for raw in raw_suggestions {
        // Convert to Arc<str> once
        let name_arc: Arc<str> = if raw.name.contains('.') {
            raw.name.into()
        } else {
            raw.name.into()
        };
        
        let reasoning_arc = raw.reasoning.map(|r| r.into());
        let confidence = raw.confidence.unwrap_or(0.8);
        
        for tld_arc in &tld_arcs {
            suggestions.push(DomainSuggestion::new(
                name_arc.clone(),
                tld_arc.clone(),
                confidence,
                reasoning_arc.clone(),
            ));
        }
    }

    Ok(suggestions)
}

/// Build domain generation prompt
pub fn build_domain_prompt(config: &GenerationConfig) -> String {
    let tld_list: Vec<&str> = config.tlds.iter().map(|s| s.as_ref()).collect();
    
    format!(
        "Generate {} creative domain names for: {}

Style: {}
Target TLDs: {}

Return ONLY a JSON array of objects with this format:
[
  {{
    \"name\": \"domainname\",
    \"reasoning\": \"why this name works\",
    \"confidence\": 0.85
  }}
]

Make sure each domain name is creative, memorable, and relevant to the description.",
        config.count,
        config.description,
        config.style,
        tld_list.join(", ")
    )
}
