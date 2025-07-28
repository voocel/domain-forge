//! Error handling for domain-forge


use thiserror::Error;

/// Main error type for domain-forge
#[derive(Error, Debug, Clone)]
pub enum DomainForgeError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("LLM provider error ({provider}): {message}")]
    LlmProvider {
        provider: crate::types::LlmProvider,
        message: String,
        code: Option<String>,
    },

    #[error("Domain checking error for '{domain}': {message}")]
    DomainCheck {
        domain: String,
        message: String,
        method: Option<String>,
    },

    #[error("Network error: {message}")]
    Network {
        message: String,
        status_code: Option<u16>,
        url: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },

    #[error("Timeout error: {operation} timed out after {timeout_secs}s")]
    Timeout {
        operation: String,
        timeout_secs: u64,
    },

    #[error("Parse error: {message}")]
    Parse {
        message: String,
        content: Option<String>,
    },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("IO error: {message}")]
    Io {
        message: String,
        path: Option<String>,
    },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("CLI error: {message}")]
    Cli { message: String },
}

impl DomainForgeError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create an LLM provider error
    pub fn llm_provider(
        provider: crate::types::LlmProvider,
        message: impl Into<String>,
        code: Option<String>,
    ) -> Self {
        Self::LlmProvider {
            provider,
            message: message.into(),
            code,
        }
    }

    /// Create a domain checking error
    pub fn domain_check(
        domain: impl Into<String>,
        message: impl Into<String>,
        method: Option<String>,
    ) -> Self {
        Self::DomainCheck {
            domain: domain.into(),
            message: message.into(),
            method,
        }
    }

    /// Create a network error
    pub fn network(
        message: impl Into<String>,
        status_code: Option<u16>,
        url: Option<String>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            status_code,
            url,
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_secs: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_secs,
        }
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>, content: Option<String>) -> Self {
        Self::Parse {
            message: message.into(),
            content,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>, path: Option<String>) -> Self {
        Self::Io {
            message: message.into(),
            path,
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a CLI error
    pub fn cli(message: impl Into<String>) -> Self {
        Self::Cli {
            message: message.into(),
        }
    }

    /// Check if this error indicates a domain might be available
    pub fn suggests_available(&self) -> bool {
        match self {
            Self::DomainCheck { message, .. } => {
                let msg = message.to_lowercase();
                msg.contains("not found")
                    || msg.contains("no match")
                    || msg.contains("available")
                    || msg.contains("404")
            }
            Self::Network { status_code, .. } => matches!(status_code, Some(404)),
            _ => false,
        }
    }



    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            Self::Config { message } => {
                format!("❌ Configuration problem: {}\n💡 Check your .env file or configuration", message)
            }
            Self::LlmProvider { provider, message, .. } => {
                format!("❌ LLM provider ({}) error: {}\n💡 Check your API key and rate limits", provider, message)
            }
            Self::DomainCheck { domain, message, .. } => {
                format!("⚠️  Could not check domain '{}': {}", domain, message)
            }
            Self::Network { message, status_code, .. } => {
                let status = status_code.map_or(String::new(), |c| format!(" ({})", c));
                format!("❌ Network error{}: {}\n💡 Check your internet connection", status, message)
            }
            Self::Authentication { message } => {
                format!("❌ Authentication failed: {}\n💡 Verify your API keys are correct", message)
            }
            Self::RateLimit { message, retry_after } => {
                let retry = retry_after.map_or(String::new(), |s| format!(" Retry in {}s.", s));
                format!("⏱️  Rate limit exceeded: {}{}\n💡 Consider reducing concurrency or waiting", message, retry)
            }
            Self::Timeout { operation, timeout_secs } => {
                format!("⏱️  Operation '{}' timed out after {}s\n💡 Try increasing timeout or reducing concurrency", operation, timeout_secs)
            }
            Self::Parse { message, .. } => {
                format!("❌ Parse error: {}\n💡 This might be a temporary issue, try again", message)
            }
            Self::Validation { message } => {
                format!("❌ Validation error: {}\n💡 Check your input format", message)
            }
            Self::Io { message, path } => {
                let path_info = path.as_ref().map_or(String::new(), |p| format!(" ({})", p));
                format!("❌ File error{}: {}\n💡 Check file permissions and paths", path_info, message)
            }
            Self::Internal { message } => {
                format!("❌ Internal error: {}\n💡 This is a bug, please report it", message)
            }
            Self::Cli { message } => {
                format!("❌ Command error: {}\n💡 Use --help for usage information", message)
            }
        }
    }

}

/// Convert from common error types
impl From<reqwest::Error> for DomainForgeError {
    fn from(err: reqwest::Error) -> Self {
        let status_code = err.status().map(|s| s.as_u16());
        let url = err.url().map(|u| u.to_string());
        
        if err.is_timeout() {
            Self::timeout("HTTP request", 30)
        } else if err.is_connect() {
            Self::network("Connection failed", status_code, url)
        } else if err.is_request() {
            Self::network("Request failed", status_code, url)
        } else {
            Self::network(err.to_string(), status_code, url)
        }
    }
}

impl From<serde_json::Error> for DomainForgeError {
    fn from(err: serde_json::Error) -> Self {
        Self::parse(err.to_string(), None)
    }
}

impl From<std::io::Error> for DomainForgeError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string(), None)
    }
}



impl From<tokio::time::error::Elapsed> for DomainForgeError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::timeout("Operation", 30)
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, DomainForgeError>;



/// Helper macros for common error patterns
#[macro_export]
macro_rules! config_error {
    ($msg:expr) => {
        $crate::error::DomainForgeError::config($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::DomainForgeError::config(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::error::DomainForgeError::validation($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::DomainForgeError::validation(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! internal_error {
    ($msg:expr) => {
        $crate::error::DomainForgeError::internal($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::DomainForgeError::internal(format!($fmt, $($arg)*))
    };
}