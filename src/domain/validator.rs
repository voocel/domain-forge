//! Domain name validation utilities

use crate::error::{DomainForgeError, Result};
use regex::Regex;
use std::collections::HashSet;

/// Domain name validator
pub struct DomainValidator {
    tld_whitelist: Option<HashSet<String>>,
    blocked_words: HashSet<String>,
}

impl DomainValidator {
    /// Create a new domain validator
    pub fn new() -> Self {
        Self {
            tld_whitelist: None,
            blocked_words: HashSet::new(),
        }
    }

    /// Create validator with TLD whitelist
    pub fn with_tld_whitelist(mut self, tlds: Vec<String>) -> Self {
        self.tld_whitelist = Some(tlds.into_iter().map(|s| s.to_lowercase()).collect());
        self
    }

    /// Create validator with blocked words
    pub fn with_blocked_words(mut self, words: Vec<String>) -> Self {
        self.blocked_words = words.into_iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Validate a domain name
    pub fn validate(&self, domain: &str) -> Result<ValidatedDomain> {
        let domain = domain.trim().to_lowercase();
        
        // Basic format validation
        self.validate_format(&domain)?;
        
        // Length validation
        self.validate_length(&domain)?;
        
        // Character validation
        self.validate_characters(&domain)?;
        
        // Parse domain parts
        let parts = self.parse_domain(&domain)?;
        
        // TLD validation
        self.validate_tld(&parts.tld)?;
        
        // Content validation
        self.validate_content(&parts.name)?;
        
        Ok(ValidatedDomain {
            original: domain.clone(),
            name: parts.name,
            tld: parts.tld,
            full_domain: domain,
            is_valid: true,
        })
    }

    /// Validate multiple domains
    pub fn validate_batch(&self, domains: &[String]) -> Vec<DomainValidationResult> {
        domains
            .iter()
            .map(|domain| DomainValidationResult {
                domain: domain.clone(),
                result: self.validate(domain),
            })
            .collect()
    }

    /// Validate domain format
    fn validate_format(&self, domain: &str) -> Result<()> {
        if domain.is_empty() {
            return Err(DomainForgeError::validation("Domain name cannot be empty"));
        }

        // Check for invalid characters at start/end
        if domain.starts_with('-') || domain.ends_with('-') {
            return Err(DomainForgeError::validation("Domain cannot start or end with hyphen"));
        }

        if domain.starts_with('.') || domain.ends_with('.') {
            return Err(DomainForgeError::validation("Domain cannot start or end with dot"));
        }

        // Check for consecutive dots
        if domain.contains("..") {
            return Err(DomainForgeError::validation("Domain cannot contain consecutive dots"));
        }

        // Check for consecutive hyphens
        if domain.contains("--") {
            return Err(DomainForgeError::validation("Domain cannot contain consecutive hyphens"));
        }

        Ok(())
    }

    /// Validate domain length
    fn validate_length(&self, domain: &str) -> Result<()> {
        if domain.len() > 253 {
            return Err(DomainForgeError::validation("Domain name too long (max 253 characters)"));
        }

        if domain.len() < 3 {
            return Err(DomainForgeError::validation("Domain name too short (min 3 characters)"));
        }

        Ok(())
    }

    /// Validate domain characters
    fn validate_characters(&self, domain: &str) -> Result<()> {
        let valid_chars = Regex::new(r"^[a-z0-9.-]+$")
            .map_err(|e| DomainForgeError::internal(e.to_string()))?;

        if !valid_chars.is_match(domain) {
            return Err(DomainForgeError::validation("Domain contains invalid characters"));
        }

        Ok(())
    }

    /// Parse domain into name and TLD
    fn parse_domain(&self, domain: &str) -> Result<DomainParts> {
        let parts: Vec<&str> = domain.split('.').collect();
        
        if parts.len() < 2 {
            return Err(DomainForgeError::validation("Domain must have at least one dot"));
        }

        let tld = parts.last().unwrap().to_string();
        let name = parts[..parts.len()-1].join(".");

        if name.is_empty() {
            return Err(DomainForgeError::validation("Domain name part cannot be empty"));
        }

        if tld.is_empty() {
            return Err(DomainForgeError::validation("TLD cannot be empty"));
        }

        Ok(DomainParts { name, tld })
    }

    /// Validate TLD
    fn validate_tld(&self, tld: &str) -> Result<()> {
        if tld.len() < 2 {
            return Err(DomainForgeError::validation("TLD too short (min 2 characters)"));
        }

        if tld.len() > 63 {
            return Err(DomainForgeError::validation("TLD too long (max 63 characters)"));
        }

        // Check against whitelist if provided
        if let Some(whitelist) = &self.tld_whitelist {
            if !whitelist.contains(tld) {
                return Err(DomainForgeError::validation(format!("TLD '{}' not in whitelist", tld)));
            }
        }

        // Basic TLD format validation
        let tld_regex = Regex::new(r"^[a-z]{2,63}$")
            .map_err(|e| DomainForgeError::internal(e.to_string()))?;

        if !tld_regex.is_match(tld) {
            return Err(DomainForgeError::validation("Invalid TLD format"));
        }

        Ok(())
    }

    /// Validate domain content
    fn validate_content(&self, name: &str) -> Result<()> {
        // Check for blocked words
        for blocked_word in &self.blocked_words {
            if name.contains(blocked_word) {
                return Err(DomainForgeError::validation(format!("Domain contains blocked word: {}", blocked_word)));
            }
        }

        // Check each label in the domain name
        for label in name.split('.') {
            if label.is_empty() {
                return Err(DomainForgeError::validation("Domain label cannot be empty"));
            }

            if label.len() > 63 {
                return Err(DomainForgeError::validation("Domain label too long (max 63 characters)"));
            }

            if label.starts_with('-') || label.ends_with('-') {
                return Err(DomainForgeError::validation("Domain label cannot start or end with hyphen"));
            }
        }

        Ok(())
    }

    /// Check if domain looks like a valid format (less strict)
    pub fn is_valid_format(&self, domain: &str) -> bool {
        self.validate(domain).is_ok()
    }

    /// Normalize domain name
    pub fn normalize(&self, domain: &str) -> String {
        domain.trim().to_lowercase()
    }

    /// Extract domain name without TLD
    pub fn extract_name(&self, domain: &str) -> Result<String> {
        let parts = self.parse_domain(&self.normalize(domain))?;
        Ok(parts.name)
    }

    /// Extract TLD from domain
    pub fn extract_tld(&self, domain: &str) -> Result<String> {
        let parts = self.parse_domain(&self.normalize(domain))?;
        Ok(parts.tld)
    }

    /// Check if domain is a subdomain
    pub fn is_subdomain(&self, domain: &str) -> bool {
        let parts: Vec<&str> = domain.split('.').collect();
        parts.len() > 2
    }

    /// Get the root domain (remove subdomains)
    pub fn get_root_domain(&self, domain: &str) -> Result<String> {
        let parts: Vec<&str> = domain.split('.').collect();
        
        if parts.len() < 2 {
            return Err(DomainForgeError::validation("Invalid domain format"));
        }

        let root = if parts.len() == 2 {
            domain.to_string()
        } else {
            format!("{}.{}", parts[parts.len()-2], parts[parts.len()-1])
        };

        Ok(root)
    }
}

impl Default for DomainValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain validation result
#[derive(Debug, Clone)]
pub struct ValidatedDomain {
    pub original: String,
    pub name: String,
    pub tld: String,
    pub full_domain: String,
    pub is_valid: bool,
}

impl ValidatedDomain {
    /// Get the full domain name
    pub fn get_full_domain(&self) -> String {
        self.full_domain.clone()
    }
}

/// Domain validation result with error
#[derive(Debug, Clone)]
pub struct DomainValidationResult {
    pub domain: String,
    pub result: Result<ValidatedDomain>,
}

/// Internal domain parts
#[derive(Debug, Clone)]
struct DomainParts {
    name: String,
    tld: String,
}

/// Utility functions for domain validation
pub mod utils {
    use super::*;

    /// Check if string looks like a domain
    pub fn looks_like_domain(input: &str) -> bool {
        input.contains('.') && input.len() >= 3 && input.len() <= 253
    }

    /// Suggest corrections for invalid domains
    pub fn suggest_corrections(domain: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let domain = domain.trim().to_lowercase();

        // Remove invalid characters
        let cleaned = domain.chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
            .collect::<String>();

        if cleaned != domain {
            suggestions.push(cleaned);
        }

        // Add common TLDs if missing
        if !domain.contains('.') {
            suggestions.push(format!("{}.com", domain));
            suggestions.push(format!("{}.org", domain));
            suggestions.push(format!("{}.net", domain));
        }

        // Fix common typos
        let typo_fixes = vec![
            ("...", "."),
            ("..", "."),
            ("--", "-"),
            (".-", "."),
            ("-.", "."),
        ];

        for (typo, fix) in typo_fixes {
            if domain.contains(typo) {
                suggestions.push(domain.replace(typo, fix));
            }
        }

        suggestions.into_iter().unique().collect()
    }

    /// Parse domain input that might be a list
    pub fn parse_domain_input(input: &str) -> Vec<String> {
        input
            .split(&[',', ' ', '\n', '\t'][..])
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Common TLD lists
    pub fn popular_tlds() -> Vec<String> {
        vec![
            "com", "org", "net", "io", "ai", "co", "me", "app", "dev", "tech", "xyz",
            "info", "biz", "name", "pro", "blog", "shop", "site", "online", "store"
        ].into_iter().map(|s| s.to_string()).collect()
    }

    pub fn country_tlds() -> Vec<String> {
        vec![
            "us", "uk", "ca", "au", "de", "fr", "jp", "cn", "in", "br", "mx", "es",
            "it", "ru", "kr", "nl", "se", "ch", "be", "at", "no", "dk", "fi", "pl"
        ].into_iter().map(|s| s.to_string()).collect()
    }

    pub fn tech_tlds() -> Vec<String> {
        vec![
            "io", "ai", "tech", "dev", "app", "cloud", "digital", "online", "software",
            "systems", "solutions", "technology", "computer", "network", "data"
        ].into_iter().map(|s| s.to_string()).collect()
    }
}

/// Extension trait for iterator uniqueness
trait IteratorExt<T>: Iterator<Item = T> + Sized {
    fn unique(self) -> std::vec::IntoIter<T>
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let mut seen = HashSet::new();
        let mut unique_items = Vec::new();
        
        for item in self {
            if seen.insert(item.clone()) {
                unique_items.push(item);
            }
        }
        
        unique_items.into_iter()
    }
}

impl<T, I> IteratorExt<T> for I where I: Iterator<Item = T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let validator = DomainValidator::new();
        
        assert!(validator.validate("example.com").is_ok());
        assert!(validator.validate("sub.example.com").is_ok());
        assert!(validator.validate("test-domain.org").is_ok());
        
        assert!(validator.validate("").is_err());
        assert!(validator.validate("invalid").is_err());
        assert!(validator.validate("-invalid.com").is_err());
        assert!(validator.validate("invalid-.com").is_err());
    }

    #[test]
    fn test_tld_whitelist() {
        let validator = DomainValidator::new()
            .with_tld_whitelist(vec!["com".to_string(), "org".to_string()]);
        
        assert!(validator.validate("example.com").is_ok());
        assert!(validator.validate("example.org").is_ok());
        assert!(validator.validate("example.net").is_err());
    }

    #[test]
    fn test_blocked_words() {
        let validator = DomainValidator::new()
            .with_blocked_words(vec!["spam".to_string(), "bad".to_string()]);
        
        assert!(validator.validate("good.com").is_ok());
        assert!(validator.validate("spam.com").is_err());
        assert!(validator.validate("bad-domain.com").is_err());
    }

    #[test]
    fn test_domain_parsing() {
        let validator = DomainValidator::new();
        
        assert_eq!(validator.extract_name("example.com").unwrap(), "example");
        assert_eq!(validator.extract_tld("example.com").unwrap(), "com");
        assert_eq!(validator.extract_name("sub.example.com").unwrap(), "sub.example");
        assert_eq!(validator.extract_tld("sub.example.com").unwrap(), "com");
    }

    #[test]
    fn test_subdomain_detection() {
        let validator = DomainValidator::new();
        
        assert!(!validator.is_subdomain("example.com"));
        assert!(validator.is_subdomain("sub.example.com"));
        assert!(validator.is_subdomain("deep.sub.example.com"));
    }

    #[test]
    fn test_root_domain_extraction() {
        let validator = DomainValidator::new();
        
        assert_eq!(validator.get_root_domain("example.com").unwrap(), "example.com");
        assert_eq!(validator.get_root_domain("sub.example.com").unwrap(), "example.com");
        assert_eq!(validator.get_root_domain("deep.sub.example.com").unwrap(), "example.com");
    }

    #[test]
    fn test_utility_functions() {
        assert!(utils::looks_like_domain("example.com"));
        assert!(!utils::looks_like_domain("invalid"));
        assert!(!utils::looks_like_domain(""));
        
        let suggestions = utils::suggest_corrections("example");
        assert!(suggestions.contains(&"example.com".to_string()));
        
        let domains = utils::parse_domain_input("example.com, test.org\n another.net");
        assert_eq!(domains.len(), 3);
    }
}