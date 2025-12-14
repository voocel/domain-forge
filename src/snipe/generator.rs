//! Domain name generator for sniping

use super::Charset;

/// Generator for domain name combinations
pub struct DomainGenerator {
    charset: Charset,
    length: usize,
    current_index: u64,
    total: u64,
}

impl DomainGenerator {
    /// Create a new generator for domains of given length
    pub fn new(length: usize, charset: Charset) -> Self {
        let total = charset.total_combinations(length);
        Self {
            charset,
            length,
            current_index: 0,
            total,
        }
    }

    /// Get total number of combinations
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Get current progress index
    pub fn current_index(&self) -> u64 {
        self.current_index
    }

    /// Set current index (for resume)
    pub fn set_index(&mut self, index: u64) {
        self.current_index = index.min(self.total);
    }

    /// Generate domain at specific index
    pub fn domain_at(&self, index: u64) -> Option<String> {
        if index >= self.total {
            return None;
        }

        let chars = self.charset.chars();
        let base = chars.len() as u64;
        let mut result = vec![' '; self.length];
        let mut n = index;

        for i in (0..self.length).rev() {
            result[i] = chars[(n % base) as usize];
            n /= base;
        }

        Some(result.into_iter().collect())
    }

    /// Generate next batch of domains
    pub fn next_batch(&mut self, count: usize) -> Vec<String> {
        let mut batch = Vec::with_capacity(count);

        for _ in 0..count {
            if let Some(domain) = self.domain_at(self.current_index) {
                batch.push(domain);
                self.current_index += 1;
            } else {
                break;
            }
        }

        batch
    }

    /// Check if generator is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.current_index >= self.total
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current_index as f64 / self.total as f64) * 100.0
        }
    }

    /// Remaining count
    pub fn remaining(&self) -> u64 {
        self.total.saturating_sub(self.current_index)
    }
}

impl Iterator for DomainGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let domain = self.domain_at(self.current_index)?;
        self.current_index += 1;
        Some(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_total() {
        let gen = DomainGenerator::new(4, Charset::Letters);
        assert_eq!(gen.total(), 26_u64.pow(4)); // 456,976
    }

    #[test]
    fn test_domain_at() {
        let gen = DomainGenerator::new(4, Charset::Letters);
        assert_eq!(gen.domain_at(0), Some("aaaa".to_string()));
        assert_eq!(gen.domain_at(1), Some("aaab".to_string()));
        assert_eq!(gen.domain_at(25), Some("aaaz".to_string()));
        assert_eq!(gen.domain_at(26), Some("aaba".to_string()));
    }

    #[test]
    fn test_generator_iterator() {
        let mut gen = DomainGenerator::new(2, Charset::Letters);
        assert_eq!(gen.next(), Some("aa".to_string()));
        assert_eq!(gen.next(), Some("ab".to_string()));
    }

    #[test]
    fn test_next_batch() {
        let mut gen = DomainGenerator::new(4, Charset::Letters);
        let batch = gen.next_batch(3);
        assert_eq!(batch, vec!["aaaa", "aaab", "aaac"]);
        assert_eq!(gen.current_index(), 3);
    }

    #[test]
    fn test_resume() {
        let mut gen = DomainGenerator::new(4, Charset::Letters);
        gen.set_index(100);
        assert_eq!(gen.current_index(), 100);
        assert!(gen.domain_at(100).is_some());
    }
}
