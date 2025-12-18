//! 6-letter "good" domain generator (pronounceable, brandable).
//!
//! We avoid the full 26^6 search space and instead generate a constrained
//! pronounceable set to keep scanning practical while still high-quality.

/// Vowels used in pronounceable patterns.
///
/// We intentionally keep this to the most common vowels to reduce the search space
/// and avoid "weird" sounding combinations.
const VOWELS: &[char] = &['a', 'e', 'i', 'o'];

/// A reduced consonant set tuned for pronounceability.
///
/// Keep this relatively small to avoid exploding the search space.
const CORE_CONSONANTS: &[char] = &[
    'b', 'c', 'd', 'f', 'g', 'h', 'l', 'm', 'n', 'p', 'r', 's', 't', 'w',
];

#[derive(Debug, Clone, Copy)]
enum Pattern6 {
    Cvcvcv,
    Vcvcvc,
}

/// Generator for pronounceable 6-letter domains.
///
/// Patterns:
/// - CVCVCV
/// - VCVCVC
pub struct SixLetterGenerator {
    patterns: [Pattern6; 2],
    pattern_sizes: [u64; 2],
    current_pattern_idx: usize,
    current_index: u64,
    total: u64,
}

impl SixLetterGenerator {
    pub fn new() -> Self {
        let size = Self::pattern_size();
        let pattern_sizes = [size, size];
        let total = pattern_sizes[0] + pattern_sizes[1];
        Self {
            patterns: [Pattern6::Cvcvcv, Pattern6::Vcvcvc],
            pattern_sizes,
            current_pattern_idx: 0,
            current_index: 0,
            total,
        }
    }

    fn pattern_size() -> u64 {
        let c = CORE_CONSONANTS.len() as u64;
        let v = VOWELS.len() as u64;
        // 3 consonants × 3 vowels
        c.pow(3) * v.pow(3)
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
        if global_index < self.pattern_sizes[0] {
            self.current_pattern_idx = 0;
            self.current_index = global_index;
            return;
        }
        let remaining = global_index - self.pattern_sizes[0];
        if remaining < self.pattern_sizes[1] {
            self.current_pattern_idx = 1;
            self.current_index = remaining;
            return;
        }
        self.current_pattern_idx = self.patterns.len();
        self.current_index = 0;
    }

    pub fn is_exhausted(&self) -> bool {
        self.current_pattern_idx >= self.patterns.len()
    }

    pub fn next_batch(&mut self, count: usize) -> Vec<String> {
        let mut batch = Vec::with_capacity(count);
        while batch.len() < count && !self.is_exhausted() {
            let pattern = self.patterns[self.current_pattern_idx];
            let size = self.pattern_sizes[self.current_pattern_idx];
            if self.current_index >= size {
                self.current_pattern_idx += 1;
                self.current_index = 0;
                continue;
            }

            if let Some(s) = self.generate_for_pattern(pattern, self.current_index) {
                batch.push(s);
            }
            self.current_index += 1;
        }
        batch
    }

    fn generate_for_pattern(&self, pattern: Pattern6, index: u64) -> Option<String> {
        let c = CORE_CONSONANTS.len() as u64;
        let v = VOWELS.len() as u64;

        match pattern {
            Pattern6::Cvcvcv => {
                // c1 v1 c2 v2 c3 v3
                // Bases: c, v, c, v, c, v
                let (i0, rem) = (index / (v * c * v * c * v), index % (v * c * v * c * v));
                let (i1, rem) = (rem / (c * v * c * v), rem % (c * v * c * v));
                let (i2, rem) = (rem / (v * c * v), rem % (v * c * v));
                let (i3, rem) = (rem / (c * v), rem % (c * v));
                let (i4, i5) = (rem / v, rem % v);

                if i0 >= c || i2 >= c || i4 >= c || i1 >= v || i3 >= v || i5 >= v {
                    return None;
                }

                Some(
                    [
                        CORE_CONSONANTS[i0 as usize],
                        VOWELS[i1 as usize],
                        CORE_CONSONANTS[i2 as usize],
                        VOWELS[i3 as usize],
                        CORE_CONSONANTS[i4 as usize],
                        VOWELS[i5 as usize],
                    ]
                    .iter()
                    .collect(),
                )
            }
            Pattern6::Vcvcvc => {
                // v1 c1 v2 c2 v3 c3
                let (i0, rem) = (index / (c * v * c * v * c), index % (c * v * c * v * c));
                let (i1, rem) = (rem / (v * c * v * c), rem % (v * c * v * c));
                let (i2, rem) = (rem / (c * v * c), rem % (c * v * c));
                let (i3, rem) = (rem / (v * c), rem % (v * c));
                let (i4, i5) = (rem / c, rem % c);

                if i1 >= c || i3 >= c || i5 >= c || i0 >= v || i2 >= v || i4 >= v {
                    return None;
                }

                Some(
                    [
                        VOWELS[i0 as usize],
                        CORE_CONSONANTS[i1 as usize],
                        VOWELS[i2 as usize],
                        CORE_CONSONANTS[i3 as usize],
                        VOWELS[i4 as usize],
                        CORE_CONSONANTS[i5 as usize],
                    ]
                    .iter()
                    .collect(),
                )
            }
        }
    }
}

impl Default for SixLetterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_reasonable() {
        let gen = SixLetterGenerator::new();
        // 2 patterns × 14^3 × 4^3 = 351,232
        assert!(gen.total() > 100_000);
        assert!(gen.total() < 500_000);
    }

    #[test]
    fn test_first_batch() {
        let mut gen = SixLetterGenerator::new();
        let batch = gen.next_batch(5);
        assert_eq!(batch.len(), 5);
        for s in batch {
            assert_eq!(s.len(), 6);
            assert!(s.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn test_resume() {
        let mut gen = SixLetterGenerator::new();
        gen.set_index(1234);
        assert_eq!(gen.current_index(), 1234);
        let b = gen.next_batch(1);
        assert_eq!(b.len(), 1);
    }
}

