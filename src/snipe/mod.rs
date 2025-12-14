//! Domain sniping module - scan for available short domains
//!
//! Phase 1: 4-letter domain scanning (any combination)
//! Phase 2: 5-letter meaningful word scanning

mod filter;
mod generator;
mod scanner;
mod state;
mod words;

pub use filter::PronounceableGenerator;
pub use generator::DomainGenerator;
pub use scanner::{DomainSniper, SnipeConfig, SnipeResult, SnipeStatus, ScanMode};
pub use state::ScanState;
pub use words::WordGenerator;

/// Character set for domain generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// Only lowercase letters (a-z)
    Letters,
    /// Letters and digits (a-z, 0-9)
    Alphanumeric,
}

impl Default for Charset {
    fn default() -> Self {
        Self::Letters
    }
}

impl Charset {
    pub fn chars(&self) -> &'static [char] {
        match self {
            Charset::Letters => &[
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
                'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ],
            Charset::Alphanumeric => &[
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
                'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            ],
        }
    }

    pub fn total_combinations(&self, length: usize) -> u64 {
        (self.chars().len() as u64).pow(length as u32)
    }
}
