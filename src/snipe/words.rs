//! 5-letter meaningful word generator for domain sniping
//!
//! Focuses on valuable, pronounceable, memorable words

/// Vowels used in pronounceable patterns.
///
/// Keep this to the most common vowels to reduce "weird" combos and keep
/// the overall `-w` search space reasonable.
const VOWELS: &[char] = &['a', 'e', 'i', 'o'];

/// A smaller subset (more "English-like") to keep output size reasonable.
///
/// This is intentionally tighter than the 4-letter pronounceable set.
const CORE_CONSONANTS: &[char] = &[
    'b', 'c', 'd', 'f', 'g', 'h', 'l', 'm', 'n', 'p', 'r', 's', 't', 'w',
];

/// Common 5-letter English words (high value for domains)
pub const COMMON_WORDS: &[&str] = &[
    // Tech & Startup
    "cloud", "cyber", "pixel", "media", "audio", "video", "solar", "smart",
    "power", "spark", "flash", "blaze", "boost", "prime", "nexus", "alpha",
    "omega", "ultra", "micro", "macro", "quick", "swift", "rapid", "turbo",
    "hyper", "super", "stack", "scale", "scope", "space", "pulse", "surge",
    "forge", "craft", "build", "maker", "works", "logic", "brain", "think",
    "learn", "teach", "coach", "guide", "laser", "radar",

    // Business & Finance
    "money", "funds", "trade", "stock", "asset", "value", "worth", "trust",
    "brand", "sales", "deals", "price", "cheap", "store", "shops", "yield",
    "gains", "bonus", "prize", "award", "elite",

    // Nature & Life
    "green", "fresh", "bloom", "flora", "fauna", "earth", "ocean", "river",
    "storm", "sunny", "clear", "light", "shine", "flame", "water", "stone",
    "pearl", "amber", "coral", "maple", "glow",

    // Positive & Action
    "happy", "lucky", "magic", "dream", "vivid", "vital", "alive", "awake",
    "begin", "start", "first", "final", "quest", "reach", "climb", "speed",
    "agile", "focus", "sharp", "exact", "ideal", "soar",

    // Modern & Cool
    "delta", "sigma", "gamma", "theta", "metro", "urban", "civic", "royal",
    "noble", "grand", "titan", "giant", "brave", "solid", "sleek", "slick",
    "crisp", "clean", "pure", "bold",

    // Food & Drink
    "apple", "grape", "lemon", "melon", "berry", "mango", "peach", "olive",
    "honey", "sugar", "spice", "cream", "toast", "juice", "blend",

    // Animals
    "tiger", "eagle", "shark", "whale", "raven", "panda", "koala", "otter",
    "horse", "zebra", "cobra", "viper", "wolf",

    // Abstract & Creative
    "vibe", "aura", "echo", "wave", "flow", "flux", "drift", "glide",
    "orbit", "chaos", "order", "unity", "merge", "fuse", "link",
];

/// Tech-focused 5-letter words
pub const TECH_WORDS: &[&str] = &[
    "bytes", "codes", "nodes", "ports", "hosts", "links", "route", "proxy",
    "cache", "query", "index", "parse", "async", "batch", "queue", "stack",
    "graph", "trees", "loops", "array", "types", "class", "trait", "state",
    "event", "hooks", "props", "store", "redux", "react", "swift", "rusty",
    "cargo", "crate", "build", "debug", "tests", "bench", "specs", "docs",
];

/// Brandable word patterns (easy to say and remember)
pub const BRANDABLE_WORDS: &[&str] = &[
    // -ify/-ly pattern (5 letters only)
    "unify", "amply", "apply", "imply", "rally", "tally", "jolly", "folly",
    "truly", "newly", "daily", "early",
    // -er pattern
    "maker", "baker", "taker", "giver", "rider", "timer", "miner", "liner",
    // Double letters (memorable, 5 letters only)
    "zippy", "happy", "peppy", "fuzzy", "dizzy", "fizzy", "jazzy",
    "buzzy", "muddy", "buddy", "bunny", "funny", "sunny",
    // Rhyming/catchy
    "bingo", "mango", "tango", "tempo", "turbo", "jumbo", "combo", "promo",
];

/// Common prefixes for combinations (2-letter)
pub const PREFIXES: &[&str] = &[
    "go", "my", "we", "be", "do", "up", "on", "in", "to", "so",
    "ai", "io", "ex", "re", "co", "un", "de", "bi", "hi", "ok",
];

/// Single-letter prefixes (tech/brand style like iPhone, eBay)
pub const PREFIXES_1: &[&str] = &[
    "i", "e", "u", "x", "z", "o", "a", "n", "v", "k",
];

/// 4-letter roots for single-letter prefix combinations
pub const ROOTS_4: &[&str] = &[
    // Animals & Nature
    "fish", "bird", "wolf", "bear", "lion", "duck", "deer", "frog", "hawk", "crab",
    "leaf", "tree", "rain", "snow", "wind", "wave", "moon", "star", "sand", "rock",
    // Tech & Digital
    "code", "data", "byte", "link", "node", "port", "sync", "ping", "scan", "hash",
    "blog", "wiki", "mail", "chat", "call", "text", "send", "load", "save", "edit",
    "file", "disk", "chip", "wire", "tech", "soft", "apps", "game", "play", "tune",
    // Business & Commerce
    "shop", "mart", "bank", "cash", "coin", "gold", "sale", "deal", "work", "task",
    "desk", "book", "note", "docs", "form", "plan", "goal", "team", "club", "crew",
    // Modern & Lifestyle
    "life", "live", "love", "care", "mind", "soul", "body", "yoga", "chef", "food",
    "ride", "trip", "tour", "path", "road", "maps", "zone", "land", "city", "town",
    // Action & Motion
    "jump", "rush", "dash", "bolt", "zoom", "spin", "flip", "turn", "push", "pull",
    "snap", "grab", "pick", "drop", "kick", "bump", "slam", "bang", "boom", "blast",
    // Quality & State
    "cool", "warm", "fast", "slim", "safe", "pure", "easy", "flex", "next", "peak",
    "mega", "uber", "mini", "maxi", "plus", "zero", "full", "free", "true", "real",
];

/// Common suffixes for combinations
pub const SUFFIXES: &[&str] = &[
    "ly", "fy", "io", "ai", "go", "up", "it", "er", "ed", "en",
    "oo", "ee", "ia", "us", "ix", "ox", "ax", "ex", "uz", "az",
];

/// 3-letter roots for prefix combinations
pub const ROOTS_3: &[&str] = &[
    "app", "bot", "box", "buy", "car", "dev", "doc", "eye", "fit", "fly",
    "get", "hub", "job", "key", "lab", "map", "net", "pay", "pet", "pod",
    "run", "set", "sky", "spy", "tag", "tap", "top", "try", "van", "vet",
    "web", "win", "wow", "zen", "zip", "zoo", "ace", "aid", "aim", "air",
    "art", "ask", "bay", "bed", "bet", "big", "bit", "biz", "bus", "cab",
    "cam", "cap", "cut", "day", "dig", "dip", "dog", "dot", "dry", "duo",
    "eat", "eco", "ego", "end", "era", "fan", "fax", "fee", "few", "fin",
    "fix", "flo", "fun", "gap", "gas", "gem", "geo", "gig", "gym", "hat",
    "hex", "hit", "hot", "ice", "ink", "ion", "jam", "jet", "joy", "kit",
    "law", "led", "let", "lid", "lip", "log", "lot", "low", "lux", "max",
    "med", "met", "mid", "min", "mix", "mob", "mod", "nav", "neo", "new",
    "nex", "now", "nut", "oak", "odd", "oil", "old", "one", "opt", "orb",
    "ore", "owl", "own", "pad", "pan", "pax", "pen", "pie", "pin", "pit",
    "pix", "ply", "pop", "pot", "pro", "pry", "pub", "rad", "ram", "raw",
    "ray", "red", "rep", "rev", "rig", "rim", "rip", "rob", "rod", "row",
    "rub", "rug", "sap", "sat", "saw", "sea", "sim", "sip", "sit", "six",
    "sol", "spa", "sub", "sum", "sun", "syn", "tab", "tan", "tax", "tea",
    "tek", "ten", "tex", "tie", "tin", "tip", "ton", "too", "tot", "tow",
    "toy", "tri", "tub", "tux", "two", "uno", "urb", "use", "vat", "via",
    "vid", "vim", "vip", "viz", "vol", "vox", "war", "wax", "way", "wed",
    "wet", "wig", "wit", "wiz", "wok", "won", "yak", "yam", "yes", "yet",
    "yin", "you", "zap", "zig", "zit",
];

/// Generator for 5-letter meaningful words
pub struct WordGenerator {
    words: Vec<String>,
    current_index: usize,
}

impl WordGenerator {
    /// Create with built-in word lists
    pub fn new() -> Self {
        let mut words = Vec::new();

        // Add all built-in word lists
        words.extend(COMMON_WORDS.iter().map(|s| s.to_string()));
        words.extend(TECH_WORDS.iter().map(|s| s.to_string()));
        words.extend(BRANDABLE_WORDS.iter().map(|s| s.to_string()));

        // Generate prefix + root combinations (prefix 2 + root 3 = 5)
        for prefix in PREFIXES {
            for root in ROOTS_3 {
                let word = format!("{}{}", prefix, root);
                if word.len() == 5 {
                    words.push(word);
                }
            }
        }

        // Generate root + suffix combinations (root 3 + suffix 2 = 5)
        for root in ROOTS_3 {
            for suffix in SUFFIXES {
                let word = format!("{}{}", root, suffix);
                if word.len() == 5 {
                    words.push(word);
                }
            }
        }

        // Generate single-letter prefix + 4-letter root combinations (1 + 4 = 5)
        // Examples: ifish, ebook, xcode, uplay, zwave
        for prefix in PREFIXES_1 {
            for root in ROOTS_4 {
                let word = format!("{}{}", prefix, root);
                if word.len() == 5 {
                    words.push(word);
                }
            }
        }

        // Deduplicate and sort
        // Keep only strict 5-letter ASCII lowercase candidates.
        // (Curated lists may contain 4-letter items like "vibe" — filter them out.)
        words.retain(|w| w.len() == 5 && w.chars().all(|c| c.is_ascii_lowercase()));

        // Expand: add pronounceable 5-letter patterns using a restricted consonant set.
        // This increases coverage of brandable names without requiring extra CLI flags.
        words.extend(generate_pronounceable_5_letter());

        words.sort();
        words.dedup();

        Self {
            words,
            current_index: 0,
        }
    }

    /// Create with custom word list
    pub fn with_words(words: Vec<String>) -> Self {
        let mut words: Vec<String> = words.into_iter()
            .filter(|w| w.len() == 5 && w.chars().all(|c| c.is_ascii_lowercase()))
            .collect();
        words.sort();
        words.dedup();

        Self {
            words,
            current_index: 0,
        }
    }

    /// Load words from file (one word per line)
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let words: Vec<String> = content
            .lines()
            .map(|s| s.trim().to_lowercase())
            .filter(|w| w.len() == 5 && w.chars().all(|c| c.is_ascii_lowercase()))
            .collect();
        Ok(Self::with_words(words))
    }

    /// Total number of words
    pub fn total(&self) -> u64 {
        self.words.len() as u64
    }

    /// Current index
    pub fn current_index(&self) -> u64 {
        self.current_index as u64
    }

    /// Set index (for resume)
    pub fn set_index(&mut self, index: u64) {
        self.current_index = (index as usize).min(self.words.len());
    }

    /// Check if exhausted
    pub fn is_exhausted(&self) -> bool {
        self.current_index >= self.words.len()
    }

    /// Get next batch of words
    pub fn next_batch(&mut self, count: usize) -> Vec<String> {
        let end = (self.current_index + count).min(self.words.len());
        let batch: Vec<String> = self.words[self.current_index..end].to_vec();
        self.current_index = end;
        batch
    }

    /// Progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.words.is_empty() {
            100.0
        } else {
            (self.current_index as f64 / self.words.len() as f64) * 100.0
        }
    }
}

impl Default for WordGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for WordGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.words.len() {
            None
        } else {
            let word = self.words[self.current_index].clone();
            self.current_index += 1;
            Some(word)
        }
    }
}

fn generate_pronounceable_5_letter() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // Pattern: CVCVC (e.g. "borep", "lavig", "mopet")
    // Use CORE_CONSONANTS to keep the space manageable.
    for &c1 in CORE_CONSONANTS {
        for &v1 in VOWELS {
            for &c2 in CORE_CONSONANTS {
                for &v2 in VOWELS {
                    for &c3 in CORE_CONSONANTS {
                        out.push([c1, v1, c2, v2, c3].iter().collect());
                    }
                }
            }
        }
    }

    // Pattern: VCVCV (e.g. "aleta", "opico")
    // Keep the same restricted consonant set to avoid ballooning the list size.
    for &v1 in VOWELS {
        for &c1 in CORE_CONSONANTS {
            for &v2 in VOWELS {
                for &c2 in CORE_CONSONANTS {
                    for &v3 in VOWELS {
                        out.push([v1, c1, v2, c2, v3].iter().collect());
                    }
                }
            }
        }
    }

    // Filter to strict 5-letter lowercase (defensive), then return.
    out.retain(|w| w.len() == 5 && w.chars().all(|c| c.is_ascii_lowercase()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_generator() {
        let gen = WordGenerator::new();
        println!("Total 5-letter words: {}", gen.total());
        assert!(gen.total() > 1000); // Should have lots of words
    }

    #[test]
    fn test_word_iterator() {
        let gen = WordGenerator::new();
        let first_10: Vec<_> = gen.take(10).collect();

        for word in &first_10 {
            assert_eq!(word.len(), 5);
            println!("{}", word);
        }
    }

    #[test]
    fn test_next_batch() {
        let mut gen = WordGenerator::new();
        let batch = gen.next_batch(20);

        assert_eq!(batch.len(), 20);
        for word in &batch {
            assert_eq!(word.len(), 5);
        }
    }
}
