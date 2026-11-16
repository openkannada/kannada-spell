//! Measures the dictionary on held-out text: coverage, and acceptance of misspellings.

use kannada_morph::Analyzer;

use crate::counts::Counts;
use crate::misspell::Rng;

/// Share of held-out tokens and word types the analyzer accepts.
pub struct Coverage {
    pub tokens: u64,
    pub accepted_tokens: u64,
    pub types: usize,
    pub accepted_types: usize,
    /// Types seen at least [`FREQUENT`] times, and how many of them are accepted.
    pub frequent_types: usize,
    pub accepted_frequent: usize,
    /// The most frequent rejected words, for inspection.
    pub top_rejected: Vec<(String, u64)>,
}

pub const FREQUENT: u64 = 5;

pub fn coverage(accepts: &dyn Fn(&str) -> bool, test: &Counts, show: usize) -> Coverage {
    let mut c = Coverage {
        tokens: 0,
        accepted_tokens: 0,
        types: 0,
        accepted_types: 0,
        frequent_types: 0,
        accepted_frequent: 0,
        top_rejected: Vec::new(),
    };
    for (word, n) in test.sorted() {
        let ok = accepts(word);
        c.tokens += n;
        c.types += 1;
        if n >= FREQUENT {
            c.frequent_types += 1;
            c.accepted_frequent += usize::from(ok);
        }
        if ok {
            c.accepted_tokens += n;
            c.accepted_types += 1;
        } else if c.top_rejected.len() < show {
            c.top_rejected.push((word.to_string(), n));
        }
    }
    c
}

/// Misspellings of accepted held-out words that occur nowhere in the corpus, and how many the analyzer accepts.
pub struct FalseAcceptance {
    pub tried: usize,
    pub accepted: usize,
    pub samples: Vec<(String, String)>,
}

pub fn false_acceptance(
    analyzer: &Analyzer,
    test: &Counts,
    seen: &dyn Fn(&str) -> bool,
    corrupt: fn(&str, &mut Rng) -> Option<String>,
    limit: usize,
    seed: u64,
) -> FalseAcceptance {
    let mut rng = Rng::new(seed);
    let mut result = FalseAcceptance {
        tried: 0,
        accepted: 0,
        samples: Vec::new(),
    };
    for (word, n) in test.sorted() {
        if result.tried >= limit {
            break;
        }
        if n < 2 || !analyzer.accepts(word) {
            continue;
        }
        let Some(wrong) = corrupt(word, &mut rng) else {
            continue;
        };
        if seen(&wrong) {
            continue;
        }
        result.tried += 1;
        if analyzer.accepts(&wrong) {
            result.accepted += 1;
        }
        result.samples.push((word.to_string(), wrong));
    }
    result
}

pub fn percent(part: impl Into<f64>, whole: impl Into<f64>) -> f64 {
    let whole = whole.into();
    if whole == 0.0 {
        0.0
    } else {
        100.0 * part.into() / whole
    }
}
