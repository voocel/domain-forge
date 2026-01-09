//! Readable domain name generator - produces pronounceable, brandable names
//!
//! Based on phonetic patterns that produce natural-sounding names.
//! Supports consonant clusters, weak vowels, and design characters.

use std::collections::HashSet;

/// Basic consonants (excluding hard-to-pronounce ones like q, w, j)
const CONSONANTS: &[char] = &[
    'b', 'c', 'd', 'f', 'g', 'h', 'k', 'l', 'm', 'n',
    'p', 'r', 's', 't', 'v', 'z',
];

/// Natural consonant clusters (common in English)
const CLUSTERS: &[&str] = &[
    "br", "bl", "cr", "cl", "dr", "fr", "gr",
    "pr", "pl", "tr", "st", "sl",
];

/// Standard vowels
const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u'];

/// Weak vowels (can only be placed in the middle, not at the end)
const WEAK_VOWELS: &[char] = &['y'];

/// Design characters (add modern feel, special placement rules)
const DESIGN_CHARS: &[char] = &['x', 'z'];

/// Banned sequences that are hard to pronounce
const BANNED_SEQS: &[&str] = &[
    "vv", "rr", "xx", "qq", "yy",
    "vx", "xv", "xr", "rx",
    "rq", "qr",
];

/// Good ending consonants (natural sounding, brandable)
const GOOD_ENDINGS: &[char] = &['n', 'r', 's', 'l'];

/// Validate if a name is readable and pronounceable
fn is_valid(name: &str) -> bool {
    // Only 5 letters (more focused, brandable)
    if name.len() != 5 {
        return false;
    }

    // At least 2 vowels (y counts as 0.5)
    let vowel_score: f32 = name.chars().map(|ch| {
        if VOWELS.contains(&ch) { 1.0 }
        else if WEAK_VOWELS.contains(&ch) { 0.5 }
        else { 0.0 }
    }).sum();

    if vowel_score < 2.0 {
        return false;
    }

    // Check banned sequences
    for bad in BANNED_SEQS {
        if name.contains(bad) {
            return false;
        }
    }

    // y cannot be at the end
    if name.ends_with('y') {
        return false;
    }

    // Must end with n/r/s/l (brandable endings like Karen, Coder, Focus, Panel)
    let last_char = name.chars().last().unwrap();
    if !GOOD_ENDINGS.contains(&last_char) {
        return false;
    }

    // No adjacent repeated letters (avoid babab, cacac)
    let chars: Vec<char> = name.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == chars[i + 1] {
            return false;
        }
    }

    // x/z cannot be followed by consonants (hard to pronounce)
    for i in 0..chars.len().saturating_sub(1) {
        if DESIGN_CHARS.contains(&chars[i]) && CONSONANTS.contains(&chars[i + 1]) {
            return false;
        }
    }

    true
}

/// Generator for readable 5-letter domain names (~27,200 total)
pub struct ReadableGenerator {
    names: Vec<String>,
    current_index: usize,
}

impl ReadableGenerator {
    /// Create a new readable name generator
    pub fn new() -> Self {
        let names = Self::generate_all_names();
        Self {
            names,
            current_index: 0,
        }
    }

    /// Generate all valid names using multiple patterns
    fn generate_all_names() -> Vec<String> {
        let mut results: HashSet<String> = HashSet::new();
        
        // Pattern 1: C V C V C (5 letters)
        Self::generate_cvcvc(&mut results);
        
        // Pattern 2: C V C V C V (6 letters)
        Self::generate_cvcvcv(&mut results);
        
        // Pattern 3: Cluster + V C V (5-6 letters)
        Self::generate_cluster_vcv(&mut results);
        
        // Pattern 4: C V C Y C (weak vowel in middle)
        Self::generate_with_weak_vowel(&mut results);
        
        // Pattern 5: C V X/Z V C (design char in middle)
        Self::generate_with_design_char(&mut results);
        
        let mut names: Vec<String> = results.into_iter().collect();
        names.sort();
        names
    }

    /// Pattern 1: CVCVC
    fn generate_cvcvc(results: &mut HashSet<String>) {
        for &c1 in CONSONANTS {
            for &v1 in VOWELS {
                for &c2 in CONSONANTS {
                    for &v2 in VOWELS {
                        for &c3 in CONSONANTS {
                            let name: String = [c1, v1, c2, v2, c3].iter().collect();
                            if is_valid(&name) {
                                results.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Pattern 2: CVCVCV
    fn generate_cvcvcv(results: &mut HashSet<String>) {
        for &c1 in CONSONANTS {
            for &v1 in VOWELS {
                for &c2 in CONSONANTS {
                    for &v2 in VOWELS {
                        for &c3 in CONSONANTS {
                            for &v3 in VOWELS {
                                let name: String = [c1, v1, c2, v2, c3, v3].iter().collect();
                                if is_valid(&name) {
                                    results.insert(name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Pattern 3: Cluster + VCV (e.g., "bravo" -> 5 letters only)
    fn generate_cluster_vcv(results: &mut HashSet<String>) {
        for cluster in CLUSTERS {
            for &v1 in VOWELS {
                for &c2 in CONSONANTS {
                    for &v2 in VOWELS {
                        // cluster(2) + v1 + c2 + v2 = 5 letters
                        let name = format!("{}{}{}{}", cluster, v1, c2, v2);
                        if is_valid(&name) {
                            results.insert(name);
                        }
                    }
                }
            }
        }
    }

    /// Pattern 4: CVCY C (weak vowel y in middle)
    fn generate_with_weak_vowel(results: &mut HashSet<String>) {
        for &c1 in CONSONANTS {
            for &v1 in VOWELS {
                for &c2 in CONSONANTS {
                    for &y in WEAK_VOWELS {
                        for &c3 in CONSONANTS {
                            let name: String = [c1, v1, c2, y, c3].iter().collect();
                            if is_valid(&name) {
                                results.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Pattern 5: CV X/Z VC (design char in middle for modern feel)
    fn generate_with_design_char(results: &mut HashSet<String>) {
        for &c1 in CONSONANTS {
            for &v1 in VOWELS {
                for &d in DESIGN_CHARS {
                    for &v2 in VOWELS {
                        for &c2 in CONSONANTS {
                            let name: String = [c1, v1, d, v2, c2].iter().collect();
                            if is_valid(&name) {
                                results.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get total count of generated names
    pub fn total_count(&self) -> usize {
        self.names.len()
    }

    /// Get current index
    pub fn current_index(&self) -> u64 {
        self.current_index as u64
    }

    /// Set current index (for resume)
    pub fn set_index(&mut self, index: u64) {
        self.current_index = index as usize;
    }

    /// Check if generator is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.current_index >= self.names.len()
    }

    /// Get next batch of names
    pub fn next_batch(&mut self, count: usize) -> Vec<String> {
        let mut batch = Vec::with_capacity(count);
        while batch.len() < count && !self.is_exhausted() {
            batch.push(self.names[self.current_index].clone());
            self.current_index += 1;
        }
        batch
    }
}

impl Default for ReadableGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for ReadableGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.names.len() {
            let name = self.names[self.current_index].clone();
            self.current_index += 1;
            Some(name)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid() {
        // Valid names (5 letters, ends with n/r/s/l, no repeated letters)
        assert!(is_valid("banan"));
        assert!(is_valid("koder"));
        assert!(is_valid("nexor"));
        assert!(is_valid("fokus"));
        assert!(is_valid("panel"));

        // Too short
        assert!(!is_valid("ban"));

        // Too long (now only 5 letters allowed)
        assert!(!is_valid("banana"));
        assert!(!is_valid("bananas"));

        // Ends with y
        assert!(!is_valid("banny"));

        // Wrong ending (must be n/r/s/l)
        assert!(!is_valid("bakat"));
        assert!(!is_valid("bakab"));

        // Adjacent repeated letters
        assert!(!is_valid("baaan"));
        assert!(!is_valid("bobbl"));

        // Banned sequence
        assert!(!is_valid("barrn"));
    }

    #[test]
    fn test_generator() {
        let gen = ReadableGenerator::new();
        assert!(gen.total_count() > 0);
        println!("Generated {} readable names", gen.total_count());
    }

    #[test]
    fn test_sample_names() {
        let gen = ReadableGenerator::new();
        let samples: Vec<_> = gen.take(20).collect();
        for name in &samples {
            println!("{}", name);
            assert!(is_valid(name));
        }
    }
}
