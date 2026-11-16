//! Measures the sound-alike keys: how often a misspelling keeps its word's key,
//! how many dictionary words share a key, and how often a romanized spelling
//! gets the same key as the Kannada word.

use std::collections::HashMap;

use kannada_phonetic::{Level, key, latin_key};

pub const LEVELS: [Level; 3] = [Level::Narrow, Level::Medium, Level::Broad];

/// Latin spellings for the test vectors: ISO 15919, Baraha and informal typing.
pub const LATIN_SAMPLES: &[&str] = &[
    "manuṣya",
    "manuShya",
    "manushya",
    "nirdishta",
    "sambandha",
    "saṁbandha",
    "bengaluru",
    "chakra",
    "cakra",
    "kṛṣṇa",
    "krishna",
    "duHkha",
    "duḥkha",
    "praj~je",
    "prajñe",
    "gūḍha",
    "pāṭha",
    "hiṁ",
    "Kannada",
    "aksharashaH",
];

/// Whole words for the test vectors, chosen to cover clusters, nasals and the visarga.
const VECTOR_WORDS: &[&str] = &[
    "ಸಂಬಂಧ",
    "ಸಮ್ಬಂದ",
    "ನಿರ್ದಿಷ್ಟ",
    "ಕೃಷ್ಣ",
    "ಕ್ಷೇತ್ರ",
    "ಪ್ರಜ್ಞೆ",
    "ದುಃಖ",
    "ಹಿಂ",
    "ಬೆಂಗಳೂರು",
    "ಬ್ಯಾಂಕ್",
    "ಅಮ್ಮ",
    "ಋಷಿ",
    "ಮರಗಳಲ್ಲಿ",
    "ಕಅ",
    "ಮನೆ123",
];

/// The test-vector words: every consonant alone, with each vowel sign, the virama, the
/// anusvara and the visarga, then [`VECTOR_WORDS`].
pub fn vector_words() -> Vec<String> {
    let consonants = ('\u{0C95}'..='\u{0CB9}').filter(|&c| kannada_morph::script::is_consonant(c));
    let signs = [
        "", "ಾ", "ಿ", "ೀ", "ು", "ೂ", "ೃ", "ೆ", "ೇ", "ೈ", "ೊ", "ೋ", "ೌ", "್", "ಂ", "ಃ",
    ];
    let mut words: Vec<String> = consonants
        .flat_map(|c| signs.iter().map(move |s| format!("{c}{s}")))
        .collect();
    words.extend(
        ('\u{0C85}'..='\u{0C94}')
            .filter(|&c| kannada_morph::script::is_independent_vowel(c))
            .map(String::from),
    );
    words.extend(VECTOR_WORDS.iter().map(|w| w.to_string()));
    words
}

/// For each level, the share of `(correct, misspelled)` pairs whose keys match.
pub fn misspelling_recall(pairs: &[(String, String)]) -> [f64; 3] {
    LEVELS.map(|level| {
        let hits = pairs
            .iter()
            .filter(|(good, bad)| key(good, level) == key(bad, level))
            .count();
        share(hits, pairs.len())
    })
}

/// For each level, how many words share a key with the average word: the cost of a match.
pub fn mean_bucket(words: &[&str]) -> [f64; 3] {
    LEVELS.map(|level| {
        let mut buckets: HashMap<String, usize> = HashMap::new();
        for w in words {
            *buckets.entry(key(w, level)).or_default() += 1;
        }
        let total: usize = words.iter().map(|w| buckets[&key(w, level)]).sum();
        total as f64 / words.len().max(1) as f64
    })
}

/// The share of `(kannada, latin)` pairs whose broad keys match.
pub fn latin_recall(pairs: &[(String, String)]) -> f64 {
    let hits = pairs
        .iter()
        .filter(|(kn, latin)| key(kn, Level::Broad) == latin_key(latin))
        .count();
    share(hits, pairs.len())
}

fn share(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn recall_rises_from_narrow_to_broad() {
        let p = pairs(&[("ದೀಪ", "ದಿಪ"), ("ಕಟ", "ಕತ"), ("ಮನೆ", "ಮಾನೆ")]);
        let [narrow, medium, broad] = misspelling_recall(&p);
        assert!(
            narrow < medium && medium <= broad,
            "{narrow} {medium} {broad}"
        );
        assert_eq!(broad, 100.0);
    }

    #[test]
    fn buckets_grow_as_keys_fold_more() {
        let [narrow, _, broad] = mean_bucket(&["ಮನೆ", "ಮಾನೆ", "ಮೇನ", "ಕರ"]);
        assert!(narrow < broad);
    }

    #[test]
    fn committed_vectors_match_the_keys() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../kannada-phonetic/vectors.tsv"
        ))
        .unwrap();
        let rows: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(
            rows.len(),
            vector_words().len() + LATIN_SAMPLES.len(),
            "regenerate with kspell phonetic-vectors"
        );
        for row in rows {
            let fields: Vec<&str> = row.split('\t').collect();
            if let Some(latin) = fields[0].strip_prefix("latin:") {
                assert_eq!(latin_key(latin), fields[1], "{latin}");
            } else {
                let expected = [fields[1], fields[2], fields[3]];
                assert_eq!(
                    kannada_phonetic::keys(fields[0]).map(|k| k.to_string()),
                    expected.map(str::to_string),
                    "{}",
                    fields[0]
                );
            }
        }
    }

    #[test]
    fn latin_pairs_match_at_broad_level() {
        assert_eq!(
            latin_recall(&pairs(&[("ಶಬ್ದ", "shabda"), ("ಮನುಷ್ಯ", "manuṣya")])),
            100.0
        );
    }
}
