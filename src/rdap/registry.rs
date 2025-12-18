//! Central RDAP server registry.
//!
//! We intentionally keep this a small, static mapping (convention over configuration).

/// Get the RDAP base URL for a TLD (lowercase, without leading dot).
///
/// Returned URL is expected to end with `/` and include any version path if needed.
pub fn rdap_base_url(tld: &str) -> Option<&'static str> {
    match tld {
        "com" => Some("https://rdap.verisign.com/com/v1/"),
        "net" => Some("https://rdap.verisign.com/net/v1/"),
        "org" => Some("https://rdap.org.org/"),
        "io" => Some("https://rdap.nic.io/"),
        "ai" => Some("https://rdap.nic.ai/"),
        "tech" => Some("https://rdap.nic.tech/"),
        "app" => Some("https://rdap.nic.google/"),
        "dev" => Some("https://rdap.nic.google/"),
        "xyz" => Some("https://rdap.nic.xyz/"),
        "co" => Some("https://rdap.nic.co/"),
        "me" => Some("https://rdap.nic.me/"),
        _ => None,
    }
}

/// Build the RDAP domain query URL for a fully-qualified domain (e.g. `example.com`).
pub fn rdap_domain_url(domain: &str) -> Option<String> {
    let tld = domain.split('.').last()?;
    let base = rdap_base_url(tld)?;
    Some(format!("{base}domain/{domain}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_known() {
        assert!(rdap_base_url("com").is_some());
        assert!(rdap_base_url("io").is_some());
        assert!(rdap_base_url("unknown").is_none());
    }

    #[test]
    fn test_domain_url() {
        let url = rdap_domain_url("example.com").unwrap();
        assert!(url.contains("domain/example.com"));
    }
}


