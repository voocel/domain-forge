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

/// Parse domain suggestions from AI response text
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
    
    for raw in raw_suggestions {
        let confidence = raw.confidence.unwrap_or(0.8);
        
        // Check if the AI already returned a domain with TLD
        if raw.name.contains('.') {
            // Domain already has TLD, use as-is
            let parts: Vec<&str> = raw.name.splitn(2, '.').collect();
            if parts.len() == 2 {
                let domain_name = parts[0].to_string();
                let existing_tld = parts[1].to_string();
                
                suggestions.push(DomainSuggestion::new(
                    domain_name,
                    existing_tld,
                    confidence,
                    raw.reasoning.clone(),
                ));
            }
        } else {
            // Domain name only, combine with each TLD
            for tld in &config.tlds {
                suggestions.push(DomainSuggestion::new(
                    raw.name.clone(),
                    tld.clone(),
                    confidence,
                    raw.reasoning.clone(),
                ));
            }
        }
    }

    Ok(suggestions)
}

/// Build domain generation prompt
pub fn build_domain_prompt(config: &GenerationConfig) -> String {
    let tld_list: Vec<&str> = config.tlds.iter().map(|s| s.as_str()).collect();
    
    // Check if user is asking for specific length domains
    let length_guidance = if config.description.contains("3个字母") || config.description.contains("3 letter") {
        "\n\nSPECIAL REQUIREMENT: Generate ONLY 3-letter domain names (like \"api\", \"dev\", \"app\", \"bot\", \"web\")."
    } else if config.description.contains("短") || config.description.contains("short") {
        "\n\nFocus on SHORT domain names (3-6 letters preferred)."
    } else {
        ""
    };

    // Check if targeting .ai domains specifically
    let ai_guidance = if tld_list.contains(&"ai") && (config.description.contains(".ai") || config.description.contains("AI")) {
        "\n\nFor .ai domains: Generate names that work well with artificial intelligence context."
    } else {
        ""
    };
    
    format!(
        "Generate {} creative domain names for: {}

Style: {}
Target TLDs: {}{}{}

IMPORTANT: Return ONLY the domain name WITHOUT the TLD extension. Do NOT include .com, .org, etc.

Return ONLY a JSON array of objects with this format:
[
  {{
    \"name\": \"domainname\",
    \"reasoning\": \"brief reason\",
    \"confidence\": 0.85
  }}
]

Examples of CORRECT format:
- \"name\": \"api\"  (3 letters)
- \"name\": \"dev\"  (3 letters)  
- \"name\": \"app\"  (3 letters)

Generate diverse, creative, and memorable domain names that match the requirements exactly.",
        config.count,
        config.description,
        config.style,
        tld_list.join(", "),
        length_guidance,
        ai_guidance
    )
}
