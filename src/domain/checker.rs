//! Domain availability checker

use crate::domain::DomainValidator;
use crate::error::{DomainForgeError, Result};
use crate::types::{AvailabilityStatus, CheckConfig, CheckMethod, DomainResult, PerformanceMetrics};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// Domain availability checker with performance monitoring
pub struct DomainChecker {
    config: CheckConfig,
    semaphore: Semaphore,
    rdap_client: Option<RdapClient>,
    whois_client: Option<WhoisClient>,
    validator: DomainValidator,
    metrics: Arc<PerformanceMetrics>,
}

impl DomainChecker {
    /// Create a new domain checker with default configuration
    pub fn new() -> Self {
        let config = CheckConfig::default();
        Self::with_config(config)
    }

    /// Create a new domain checker with custom configuration
    pub fn with_config(config: CheckConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("domain-forge/0.1.0")
            .pool_max_idle_per_host(config.connection_pool_size)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create optimized HTTP client: {}. Using default.", e);
                Client::new()
            });

        let semaphore = Semaphore::new(config.concurrent_checks);
        
        let rdap_client = if config.enable_rdap {
            Some(RdapClient::new(client.clone()))
        } else {
            None
        };

        let whois_client = if config.enable_whois {
            Some(WhoisClient::new())
        } else {
            None
        };

        let validator = DomainValidator::new();
        let metrics = Arc::new(PerformanceMetrics::new());

        Self {
            config,
            semaphore,
            rdap_client,
            whois_client,
            validator,
            metrics,
        }
    }

    /// Check a single domain with performance monitoring
    pub async fn check_domain(&self, domain: &str) -> Result<DomainResult> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            DomainForgeError::internal(format!("Failed to acquire semaphore: {}", e))
        })?;

        let start_time = Instant::now();

        // Validate domain format
        let validated = self.validator.validate(domain)?;
        
        // Try RDAP first
        if let Some(rdap_client) = &self.rdap_client {
            match rdap_client.check_domain(&validated.get_full_domain()).await {
                Ok(result) => {
                    let duration = start_time.elapsed();
                    self.metrics.increment_domains_checked();
                    self.metrics.add_check_time(duration.as_millis() as u64);
                    
                    tracing::debug!(
                        domain = %domain,
                        method = "rdap",
                        status = ?result.status,
                        duration_ms = %duration.as_millis(),
                        "Domain check completed"
                    );
                    
                    return Ok(DomainResult {
                        domain: validated.get_full_domain(),
                        status: result.status,
                        method: CheckMethod::Rdap,
                        checked_at: Utc::now(),
                        check_duration: Some(duration),
                        registrar: result.registrar,
                        creation_date: result.creation_date,
                        expiration_date: result.expiration_date,
                        nameservers: result.nameservers,
                        error_message: None,
                    });
                }
                Err(e) => {
                    tracing::debug!(domain = %domain, method = "rdap", error = %e, "RDAP check failed");
                    
                    // If RDAP suggests domain is available, return that
                    if e.suggests_available() {
                        let duration = start_time.elapsed();
                        self.metrics.increment_domains_checked();
                        self.metrics.add_check_time(duration.as_millis() as u64);
                        
                        return Ok(DomainResult {
                            domain: validated.get_full_domain(),
                            status: AvailabilityStatus::Available,
                            method: CheckMethod::Rdap,
                            checked_at: Utc::now(),
                            check_duration: Some(duration),
                            registrar: None,
                            creation_date: None,
                            expiration_date: None,
                            nameservers: Vec::new(),
                            error_message: None,
                        });
                    }
                }
            }
        }

        // Fall back to WHOIS
        if let Some(whois_client) = &self.whois_client {
            match whois_client.check_domain(&validated.get_full_domain()).await {
                Ok(result) => {
                    let duration = start_time.elapsed();
                    self.metrics.increment_domains_checked();
                    self.metrics.add_check_time(duration.as_millis() as u64);
                    
                    tracing::debug!(
                        domain = %domain,
                        method = "whois",
                        status = ?result.status,
                        duration_ms = %duration.as_millis(),
                        "Domain check completed"
                    );
                    
                    return Ok(DomainResult {
                        domain: validated.get_full_domain(),
                        status: result.status,
                        method: CheckMethod::Whois,
                        checked_at: Utc::now(),
                        check_duration: Some(duration),
                        registrar: result.registrar,
                        creation_date: result.creation_date,
                        expiration_date: result.expiration_date,
                        nameservers: result.nameservers,
                        error_message: None,
                    });
                }
                Err(e) => {
                    tracing::debug!(domain = %domain, method = "whois", error = %e, "WHOIS check failed");
                    
                    // If WHOIS suggests domain is available, return that
                    if e.suggests_available() {
                        let duration = start_time.elapsed();
                        self.metrics.increment_domains_checked();
                        self.metrics.add_check_time(duration.as_millis() as u64);
                        
                        return Ok(DomainResult {
                            domain: validated.get_full_domain(),
                            status: AvailabilityStatus::Available,
                            method: CheckMethod::Whois,
                            checked_at: Utc::now(),
                            check_duration: Some(duration),
                            registrar: None,
                            creation_date: None,
                            expiration_date: None,
                            nameservers: Vec::new(),
                            error_message: None,
                        });
                    }
                }
            }
        }

        // Both methods failed
        let duration = start_time.elapsed();
        self.metrics.increment_errors();
        
        tracing::warn!(
            domain = %domain,
            duration_ms = %duration.as_millis(),
            "All domain checking methods failed"
        );
        
        Ok(DomainResult {
            domain: validated.get_full_domain(),
            status: AvailabilityStatus::Unknown,
            method: CheckMethod::Unknown,
            checked_at: Utc::now(),
            check_duration: Some(duration),
            registrar: None,
            creation_date: None,
            expiration_date: None,
            nameservers: Vec::new(),
            error_message: Some("All checking methods failed".to_string()),
        })
    }

    /// Check multiple domains concurrently with batch performance monitoring
    pub async fn check_domains(&self, domains: &[String]) -> Result<Vec<DomainResult>> {
        let batch_start = Instant::now();
        let futures = domains.iter().map(|domain| self.check_domain(domain));
        let results = join_all(futures).await;

        let mut success_results = Vec::new();
        let mut error_count = 0u32;

        for (domain, result) in domains.iter().zip(results.iter()) {
            match result {
                Ok(domain_result) => success_results.push(domain_result.clone()),
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(domain = %domain, error = %e, "Failed to check domain");
                }
            }
        }

        let batch_duration = batch_start.elapsed();
        tracing::info!(
            domains_requested = %domains.len(),
            domains_processed = %success_results.len(),
            errors = %error_count,
            batch_duration_ms = %batch_duration.as_millis(),
            avg_duration_ms = %(batch_duration.as_millis() / domains.len().max(1) as u128),
            "Batch domain check completed"
        );

        Ok(success_results)
    }

    /// Get checker configuration
    pub fn config(&self) -> &CheckConfig {
        &self.config
    }

    /// Check if checker is configured properly
    pub fn is_configured(&self) -> bool {
        self.rdap_client.is_some() || self.whois_client.is_some()
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

impl Default for DomainChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// RDAP client for domain checking
struct RdapClient {
    client: Client,
    rdap_servers: HashMap<String, String>,
}

impl RdapClient {
    fn new(client: Client) -> Self {
        let mut rdap_servers = HashMap::new();
        
        // Popular RDAP servers
        rdap_servers.insert("com".to_string(), "https://rdap.verisign.com/com/v1/".to_string());
        rdap_servers.insert("org".to_string(), "https://rdap.org.org/".to_string());
        rdap_servers.insert("net".to_string(), "https://rdap.verisign.com/net/v1/".to_string());
        rdap_servers.insert("io".to_string(), "https://rdap.nic.io/".to_string());
        rdap_servers.insert("ai".to_string(), "https://rdap.nic.ai/".to_string());
        rdap_servers.insert("tech".to_string(), "https://rdap.nic.tech/".to_string());
        rdap_servers.insert("app".to_string(), "https://rdap.nic.google/".to_string());
        rdap_servers.insert("dev".to_string(), "https://rdap.nic.google/".to_string());
        rdap_servers.insert("xyz".to_string(), "https://rdap.nic.xyz/".to_string());
        
        Self {
            client,
            rdap_servers,
        }
    }

    async fn check_domain(&self, domain: &str) -> Result<DomainCheckResult> {
        // Safe TLD extraction
        let tld = domain.split('.').last()
            .ok_or_else(|| DomainForgeError::validation("Invalid domain format - no TLD found".to_string()))?;
            
        let rdap_url = self.rdap_servers.get(tld)
            .ok_or_else(|| DomainForgeError::domain_check(
                domain.to_string(),
                format!("No RDAP server found for TLD: {}", tld),
                Some("rdap".to_string()),
            ))?;

        let url = format!("{}domain/{}", rdap_url, domain);
        
        let response = timeout(Duration::from_secs(10), self.client.get(&url).send()).await
            .map_err(|_| DomainForgeError::timeout("RDAP request", 10))?
            .map_err(|e| DomainForgeError::network(e.to_string(), None, Some(url.clone())))?;

        let status = response.status();
        
        if status.as_u16() == 404 {
            return Ok(DomainCheckResult {
                status: AvailabilityStatus::Available,
                registrar: None,
                creation_date: None,
                expiration_date: None,
                nameservers: Vec::new(),
            });
        }

        if !status.is_success() {
            return Err(DomainForgeError::network(
                format!("RDAP request failed with status {}", status),
                Some(status.as_u16()),
                Some(url),
            ));
        }

        let text = response.text().await.map_err(|e| {
            DomainForgeError::network(e.to_string(), None, Some(url.clone()))
        })?;

        let rdap_response: RdapResponse = serde_json::from_str(&text)
            .map_err(|e| DomainForgeError::parse(e.to_string(), Some(text)))?;

        Ok(self.parse_rdap_response(rdap_response))
    }

    fn parse_rdap_response(&self, response: RdapResponse) -> DomainCheckResult {
        // If we got a successful RDAP response with domain data, the domain is taken
        // Available domains typically return 404 or have no registration data
        let status = if !response.status.is_empty() ||
                        !response.entities.is_empty() ||
                        !response.events.is_empty() ||
                        !response.nameservers.is_empty() {
            AvailabilityStatus::Taken
        } else {
            AvailabilityStatus::Available
        };

        let registrar = response.entities
            .iter()
            .find(|e| e.roles.contains(&"registrar".to_string()))
            .and_then(|e| e.vcard_array.as_ref())
            .and_then(|vcard| {
                vcard.get(1)
                    .and_then(|props| props.as_array())
                    .and_then(|props| props.get(0))
                    .and_then(|prop| prop.as_array())
                    .and_then(|prop| prop.get(3))
                    .and_then(|name| name.as_str())
                    .map(|s| s.to_string())
            });

        let creation_date = response.events
            .iter()
            .find(|e| e.event_action == "registration")
            .and_then(|e| e.event_date.parse::<DateTime<Utc>>().ok());

        let expiration_date = response.events
            .iter()
            .find(|e| e.event_action == "expiration")
            .and_then(|e| e.event_date.parse::<DateTime<Utc>>().ok());

        let nameservers = response.nameservers
            .iter()
            .map(|ns| ns.ldh_name.clone())
            .collect();

        DomainCheckResult {
            status,
            registrar,
            creation_date,
            expiration_date,
            nameservers,
        }
    }
}

/// WHOIS client for domain checking
struct WhoisClient;

impl WhoisClient {
    fn new() -> Self {
        Self
    }

    async fn check_domain(&self, domain: &str) -> Result<DomainCheckResult> {
        let output = timeout(
            Duration::from_secs(15),
            tokio::task::spawn_blocking({
                let domain = domain.to_string();
                move || {
                    Command::new("whois")
                        .arg(&domain)
                        .output()
                }
            })
        ).await
        .map_err(|_| DomainForgeError::timeout("WHOIS request", 15))?
        .map_err(|e| DomainForgeError::internal(format!("Failed to spawn WHOIS command: {}", e)))?
        .map_err(|e| DomainForgeError::domain_check(
            domain.to_string(),
            format!("WHOIS command failed: {}", e),
            Some("whois".to_string()),
        ))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(DomainForgeError::domain_check(
                domain.to_string(),
                format!("WHOIS command failed: {}", stderr),
                Some("whois".to_string()),
            ));
        }

        self.parse_whois_response(&stdout, domain)
    }

    fn parse_whois_response(&self, output: &str, _domain: &str) -> Result<DomainCheckResult> {
        let output_lower = output.to_lowercase();

        // Check for availability indicators
        let available_patterns = [
            "no match",
            "not found",
            "no entries found",
            "domain not found",
            "domain available",
            "not registered",
            "available for registration",
        ];

        let taken_patterns = [
            "registrar:",
            "creation date:",
            "created:",
            "registered:",
            "name server:",
            "nameserver:",
            "domain status:",
            "status:",
        ];

        let is_available = available_patterns.iter().any(|pattern| output_lower.contains(pattern));
        let is_taken = taken_patterns.iter().any(|pattern| output_lower.contains(pattern));

        let status = if is_available && !is_taken {
            AvailabilityStatus::Available
        } else if is_taken {
            AvailabilityStatus::Taken
        } else {
            AvailabilityStatus::Unknown
        };

        let registrar = self.extract_field(output, &["registrar:", "registrar name:"]);
        let creation_date = self.extract_field(output, &["creation date:", "created:", "registered:"])
            .and_then(|date_str| self.parse_date(&date_str));
        let expiration_date = self.extract_field(output, &["expiration date:", "expires:", "expiry date:"])
            .and_then(|date_str| self.parse_date(&date_str));

        let nameservers = self.extract_nameservers(output);

        Ok(DomainCheckResult {
            status,
            registrar,
            creation_date,
            expiration_date,
            nameservers,
        })
    }

    fn extract_field(&self, output: &str, patterns: &[&str]) -> Option<String> {
        for pattern in patterns {
            if let Some(line) = output.lines().find(|line| line.to_lowercase().contains(pattern)) {
                if let Some(value) = line.split(':').nth(1) {
                    return Some(value.trim().to_string());
                }
            }
        }
        None
    }

    fn extract_nameservers(&self, output: &str) -> Vec<String> {
        let mut nameservers = Vec::new();
        let ns_patterns = ["name server:", "nameserver:", "nserver:"];
        
        for line in output.lines() {
            let line_lower = line.to_lowercase();
            for pattern in &ns_patterns {
                if line_lower.contains(pattern) {
                    if let Some(ns) = line.split(':').nth(1) {
                        nameservers.push(ns.trim().to_string());
                    }
                }
            }
        }
        
        nameservers
    }

    fn parse_date(&self, date_str: &str) -> Option<DateTime<Utc>> {
        // Try various date formats
        let formats = [
            "%Y-%m-%d",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d %H:%M:%S UTC",
            "%d-%b-%Y",
            "%d.%m.%Y",
        ];

        for format in &formats {
            if let Ok(dt) = DateTime::parse_from_str(date_str, format) {
                return Some(dt.with_timezone(&Utc));
            }
        }

        None
    }
}

/// Domain check result
#[derive(Debug, Clone)]
struct DomainCheckResult {
    status: AvailabilityStatus,
    registrar: Option<String>,
    creation_date: Option<DateTime<Utc>>,
    expiration_date: Option<DateTime<Utc>>,
    nameservers: Vec<String>,
}

/// RDAP response structures
#[derive(Debug, Deserialize)]
struct RdapResponse {
    #[serde(default)]
    status: Vec<String>,
    #[serde(default)]
    entities: Vec<RdapEntity>,
    #[serde(default)]
    events: Vec<RdapEvent>,
    #[serde(default)]
    nameservers: Vec<RdapNameserver>,
}

#[derive(Debug, Deserialize)]
struct RdapEntity {
    #[serde(default)]
    roles: Vec<String>,
    #[serde(rename = "vcardArray")]
    vcard_array: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RdapEvent {
    #[serde(rename = "eventAction")]
    event_action: String,
    #[serde(rename = "eventDate")]
    event_date: String,
}

#[derive(Debug, Deserialize)]
struct RdapNameserver {
    #[serde(rename = "ldhName")]
    ldh_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_domain_checker_creation() {
        let checker = DomainChecker::new();
        assert!(checker.is_configured());
    }

    #[tokio::test]
    async fn test_domain_checker_metrics() {
        let checker = DomainChecker::new();
        let metrics = checker.get_metrics_snapshot();
        
        // Initially should be zero
        assert_eq!(metrics.domains_checked, 0);
        assert_eq!(metrics.errors_encountered, 0);
    }

    #[test]
    fn test_rdap_client_creation() {
        let client = Client::new();
        let rdap_client = RdapClient::new(client);
        assert!(rdap_client.rdap_servers.contains_key("com"));
    }

    #[test]
    fn test_whois_client_creation() {
        let whois_client = WhoisClient::new();
        // Just test that it creates successfully
        assert!(true);
    }
}