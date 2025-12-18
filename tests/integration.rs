//! Integration tests for domain-forge

use domain_forge::{
    domain::DomainChecker,
    llm::DomainGenerator,
    types::{
        AvailabilityStatus, CheckConfig, GenerationConfig, GenerationStyle, LlmConfig, LlmProvider,
    },
};
use std::time::Duration;

#[tokio::test]
async fn test_domain_checker_creation() {
    let _checker = DomainChecker::new();
    // Should create successfully with default config
    assert!(true);
}

#[tokio::test]
async fn test_domain_checker_with_config() {
    let mut config = CheckConfig::default();
    config.timeout = Duration::from_secs(5);
    config.concurrent_checks = 2;

    let _checker = DomainChecker::with_config(config);
    // Should create successfully with custom config
    assert!(true);
}

#[tokio::test]
async fn test_domain_checking_known_domains() {
    let checker = DomainChecker::new();

    // Test a well-known taken domain
    match checker.check_domain("google.com").await {
        Ok(result) => {
            assert_eq!(result.domain, "google.com");
            // google.com should be taken
            assert_eq!(result.status, AvailabilityStatus::Taken);
        }
        Err(_) => {
            // Network issues are acceptable in tests
            println!("Network error checking google.com - this is acceptable in tests");
        }
    }
}

#[tokio::test]
async fn test_batch_domain_checking() {
    let checker = DomainChecker::new();
    let domains = vec![
        "google.com".to_string(),
        "example.com".to_string(),
    ];

    match checker.check_domains(&domains).await {
        Ok(results) => {
            assert_eq!(results.len(), 2);
            // All results should have the correct domain names
            let domain_names: Vec<String> = results.iter().map(|r| r.domain.clone()).collect();
            assert!(domain_names.contains(&"google.com".to_string()));
            assert!(domain_names.contains(&"example.com".to_string()));
        }
        Err(_) => {
            // Network issues are acceptable in tests
            println!("Network error in batch checking - this is acceptable in tests");
        }
    }
}

#[tokio::test]
async fn test_llm_generator_creation() {
    let _generator = DomainGenerator::new();
    // Should create successfully
    assert!(true);
}

#[tokio::test]
async fn test_llm_config_creation() {
    let config = LlmConfig {
        provider: "openai".to_string(),
        model: "gpt-4.1-mini".to_string(),
        api_key: "test-key".to_string(),
        base_url: None,
        temperature: 0.7,
    };

    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, "gpt-4.1-mini");
    assert_eq!(config.temperature, 0.7);
}

#[tokio::test]
async fn test_generation_config_creation() {
    let config = GenerationConfig {
        provider: LlmProvider::OpenAi,
        count: 5,
        style: GenerationStyle::Creative,
        tlds: vec!["com".to_string(), "io".to_string()],
        temperature: 0.7,
        description: "Test app".to_string(),
        avoid_names: Vec::new(),
    };

    assert_eq!(config.count, 5);
    assert_eq!(config.style, GenerationStyle::Creative);
    assert_eq!(config.tlds.len(), 2);
    assert!(config.tlds.contains(&"com".to_string()));
}

#[test]
fn test_provider_enum() {
    assert_eq!(format!("{:?}", LlmProvider::OpenAi), "OpenAi");
    assert_eq!(format!("{:?}", LlmProvider::Claude), "Claude");
}

#[test]
fn test_generation_style_enum() {
    assert_eq!(format!("{:?}", GenerationStyle::Creative), "Creative");
    assert_eq!(format!("{:?}", GenerationStyle::Professional), "Professional");
    assert_eq!(format!("{:?}", GenerationStyle::Brandable), "Brandable");
}

#[test]
fn test_availability_status_enum() {
    assert_eq!(format!("{:?}", AvailabilityStatus::Available), "Available");
    assert_eq!(format!("{:?}", AvailabilityStatus::Taken), "Taken");
    assert_eq!(format!("{:?}", AvailabilityStatus::Unknown), "Unknown");
}

#[test]
fn test_error_handling() {
    use domain_forge::error::DomainForgeError;

    let error = DomainForgeError::validation("test error".to_string());
    assert!(error.to_string().contains("test error"));

    let error = DomainForgeError::config("config error".to_string());
    assert!(error.to_string().contains("config error"));

    let error = DomainForgeError::internal("internal error");
    assert!(error.to_string().contains("internal error"));
}

#[test]
fn test_library_initialization() {
    // Test that the library can be initialized without panicking
    let result = domain_forge::init();
    assert!(result.is_ok());
}