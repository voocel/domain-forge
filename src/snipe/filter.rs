//! Pronounceable domain filter - generates only valuable domain combinations

const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u'];
const CONSONANTS: &[char] = &[
    'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm',
    'n', 'p', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z',
];

/// Common valuable prefixes for 4-letter domains
const VALUABLE_PREFIXES: &[&str] = &[
    "go", "my", "ai", "be", "we", "up", "on", "in", "to", "do",
    "no", "so", "hi", "ok", "io", "ex", "re", "co", "un", "de",
];

/// Common valuable suffixes for 4-letter domains
const VALUABLE_SUFFIXES: &[&str] = &[
    "ly", "io", "ai", "go", "up", "it", "me", "us", "fy", "oo",
    "er", "ed", "en", "ey", "ie", "ty", "by", "ry", "ny", "xy",
];

/// Pronounceable pattern types
#[derive(Debug, Clone, Copy)]
pub enum Pattern {
    /// Consonant-Vowel-Consonant-Vowel (e.g., "boca", "dune", "kite")
    CVCV,
    /// Consonant-Vowel-Consonant-Consonant (e.g., "bold", "camp", "disk")
    CVCC,
    /// Consonant-Consonant-Vowel-Consonant (e.g., "blog", "club", "drop")
    CCVC,
    /// Consonant-Vowel-Vowel-Consonant (e.g., "boat", "team", "road")
    CVVC,
    /// Vowel-Consonant-Vowel-Consonant (e.g., "apex", "icon", "uber")
    VCVC,
    /// Valuable prefix + 2 letters
    PrefixBased,
    /// 2 letters + Valuable suffix
    SuffixBased,
}

/// Generator for pronounceable 4-letter domains
pub struct PronounceableGenerator {
    patterns: Vec<Pattern>,
    current_pattern_idx: usize,
    current_index: u64,
    pattern_sizes: Vec<u64>,
    total: u64,
}

impl PronounceableGenerator {
    pub fn new() -> Self {
        let patterns = vec![
            Pattern::CVCV,
            Pattern::CVCC,
            Pattern::CCVC,
            Pattern::CVVC,
            Pattern::VCVC,
            Pattern::PrefixBased,
            Pattern::SuffixBased,
        ];

        let pattern_sizes: Vec<u64> = patterns.iter().map(|p| Self::pattern_size(*p)).collect();
        let total = pattern_sizes.iter().sum();

        Self {
            patterns,
            current_pattern_idx: 0,
            current_index: 0,
            pattern_sizes,
            total,
        }
    }

    fn pattern_size(pattern: Pattern) -> u64 {
        let c = CONSONANTS.len() as u64;
        let v = VOWELS.len() as u64;

        match pattern {
            Pattern::CVCV => c * v * c * v,           // 20 * 5 * 20 * 5 = 10,000
            Pattern::CVCC => c * v * c * c,           // 20 * 5 * 20 * 20 = 40,000
            Pattern::CCVC => c * c * v * c,           // 20 * 20 * 5 * 20 = 40,000
            Pattern::CVVC => c * v * v * c,           // 20 * 5 * 5 * 20 = 10,000
            Pattern::VCVC => v * c * v * c,           // 5 * 20 * 5 * 20 = 10,000
            Pattern::PrefixBased => VALUABLE_PREFIXES.len() as u64 * 26 * 26, // 20 * 676 = 13,520
            Pattern::SuffixBased => 26 * 26 * VALUABLE_SUFFIXES.len() as u64, // 676 * 20 = 13,520
        }
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn current_index(&self) -> u64 {
        let mut idx = self.current_index;
        for i in 0..self.current_pattern_idx {
            idx += self.pattern_sizes[i];
        }
        idx
    }

    pub fn set_index(&mut self, global_index: u64) {
        let mut remaining = global_index;
        for (i, &size) in self.pattern_sizes.iter().enumerate() {
            if remaining < size {
                self.current_pattern_idx = i;
                self.current_index = remaining;
                return;
            }
            remaining -= size;
        }
        // Exhausted
        self.current_pattern_idx = self.patterns.len();
        self.current_index = 0;
    }

    pub fn is_exhausted(&self) -> bool {
        self.current_pattern_idx >= self.patterns.len()
    }

    pub fn progress_percent(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current_index() as f64 / self.total as f64) * 100.0
        }
    }

    fn generate_for_pattern(&self, pattern: Pattern, index: u64) -> Option<String> {
        let c = CONSONANTS.len() as u64;
        let v = VOWELS.len() as u64;

        match pattern {
            Pattern::CVCV => {
                let (i0, rem) = (index / (v * c * v), index % (v * c * v));
                let (i1, rem) = (rem / (c * v), rem % (c * v));
                let (i2, i3) = (rem / v, rem % v);
                Some(format!(
                    "{}{}{}{}",
                    CONSONANTS[i0 as usize],
                    VOWELS[i1 as usize],
                    CONSONANTS[i2 as usize],
                    VOWELS[i3 as usize]
                ))
            }
            Pattern::CVCC => {
                let (i0, rem) = (index / (v * c * c), index % (v * c * c));
                let (i1, rem) = (rem / (c * c), rem % (c * c));
                let (i2, i3) = (rem / c, rem % c);
                Some(format!(
                    "{}{}{}{}",
                    CONSONANTS[i0 as usize],
                    VOWELS[i1 as usize],
                    CONSONANTS[i2 as usize],
                    CONSONANTS[i3 as usize]
                ))
            }
            Pattern::CCVC => {
                let (i0, rem) = (index / (c * v * c), index % (c * v * c));
                let (i1, rem) = (rem / (v * c), rem % (v * c));
                let (i2, i3) = (rem / c, rem % c);
                Some(format!(
                    "{}{}{}{}",
                    CONSONANTS[i0 as usize],
                    CONSONANTS[i1 as usize],
                    VOWELS[i2 as usize],
                    CONSONANTS[i3 as usize]
                ))
            }
            Pattern::CVVC => {
                let (i0, rem) = (index / (v * v * c), index % (v * v * c));
                let (i1, rem) = (rem / (v * c), rem % (v * c));
                let (i2, i3) = (rem / c, rem % c);
                Some(format!(
                    "{}{}{}{}",
                    CONSONANTS[i0 as usize],
                    VOWELS[i1 as usize],
                    VOWELS[i2 as usize],
                    CONSONANTS[i3 as usize]
                ))
            }
            Pattern::VCVC => {
                let (i0, rem) = (index / (c * v * c), index % (c * v * c));
                let (i1, rem) = (rem / (v * c), rem % (v * c));
                let (i2, i3) = (rem / c, rem % c);
                Some(format!(
                    "{}{}{}{}",
                    VOWELS[i0 as usize],
                    CONSONANTS[i1 as usize],
                    VOWELS[i2 as usize],
                    CONSONANTS[i3 as usize]
                ))
            }
            Pattern::PrefixBased => {
                let prefix_count = VALUABLE_PREFIXES.len() as u64;
                let (prefix_idx, rem) = (index / (26 * 26), index % (26 * 26));
                let (c1, c2) = (rem / 26, rem % 26);

                if prefix_idx >= prefix_count {
                    return None;
                }

                let prefix = VALUABLE_PREFIXES[prefix_idx as usize];
                let ch1 = (b'a' + c1 as u8) as char;
                let ch2 = (b'a' + c2 as u8) as char;
                Some(format!("{}{}{}", prefix, ch1, ch2))
            }
            Pattern::SuffixBased => {
                let suffix_count = VALUABLE_SUFFIXES.len() as u64;
                let (char_idx, suffix_idx) = (index / suffix_count, index % suffix_count);
                let (c1, c2) = (char_idx / 26, char_idx % 26);

                if suffix_idx >= suffix_count || c1 >= 26 {
                    return None;
                }

                let ch1 = (b'a' + c1 as u8) as char;
                let ch2 = (b'a' + c2 as u8) as char;
                let suffix = VALUABLE_SUFFIXES[suffix_idx as usize];
                Some(format!("{}{}{}", ch1, ch2, suffix))
            }
        }
    }

    pub fn next_batch(&mut self, count: usize) -> Vec<String> {
        let mut batch = Vec::with_capacity(count);
        let mut seen = std::collections::HashSet::new();

        while batch.len() < count && !self.is_exhausted() {
            let pattern = self.patterns[self.current_pattern_idx];
            let pattern_size = self.pattern_sizes[self.current_pattern_idx];

            if self.current_index >= pattern_size {
                self.current_pattern_idx += 1;
                self.current_index = 0;
                continue;
            }

            if let Some(domain) = self.generate_for_pattern(pattern, self.current_index) {
                // Deduplicate (prefix/suffix patterns may overlap)
                if seen.insert(domain.clone()) {
                    batch.push(domain);
                }
            }
            self.current_index += 1;
        }

        batch
    }
}

impl Default for PronounceableGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for PronounceableGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_exhausted() {
            return None;
        }

        loop {
            let pattern = self.patterns.get(self.current_pattern_idx)?;
            let pattern_size = self.pattern_sizes[self.current_pattern_idx];

            if self.current_index >= pattern_size {
                self.current_pattern_idx += 1;
                self.current_index = 0;
                continue;
            }

            let result = self.generate_for_pattern(*pattern, self.current_index);
            self.current_index += 1;
            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pronounceable_generator() {
        let gen = PronounceableGenerator::new();
        // CVCV: 10,000 + CVCC: 40,000 + CCVC: 40,000 + CVVC: 10,000 + VCVC: 10,000
        // + Prefix: 13,520 + Suffix: 13,520 = ~137,040
        assert!(gen.total() > 100_000);
        assert!(gen.total() < 150_000);
        println!("Total pronounceable combinations: {}", gen.total());
    }

    #[test]
    fn test_cvcv_pattern() {
        let mut gen = PronounceableGenerator::new();
        let batch = gen.next_batch(5);

        // First few should be CVCV pattern
        for domain in &batch {
            assert_eq!(domain.len(), 4);
            println!("{}", domain);
        }
    }

    #[test]
    fn test_iterator() {
        let gen = PronounceableGenerator::new();
        let first_10: Vec<_> = gen.take(10).collect();
        assert_eq!(first_10.len(), 10);

        for domain in &first_10 {
            assert_eq!(domain.len(), 4);
        }
    }

    #[test]
    fn test_prefix_suffix() {
        let gen = PronounceableGenerator::new();
        let all: Vec<_> = gen.collect();

        // Check some expected domains exist
        assert!(all.contains(&"goaa".to_string()) || all.contains(&"myaa".to_string()));
        assert!(all.contains(&"aaly".to_string()) || all.contains(&"aaio".to_string()));
    }
}
