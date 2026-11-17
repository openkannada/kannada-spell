// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Synthetic misspellings: 1 realistic edit to a correct word.

use kannada_morph::script::is_well_formed;

/// Pairs of letters and signs that writers confuse: aspiration, retroflex and
/// dental, the 3 sibilants, vowel length, and nasals written as anusvara.
const CONFUSIONS: &[(&str, &str)] = &[
    ("ಕ", "ಖ"),
    ("ಗ", "ಘ"),
    ("ಚ", "ಛ"),
    ("ಜ", "ಝ"),
    ("ಟ", "ಠ"),
    ("ಡ", "ಢ"),
    ("ತ", "ಥ"),
    ("ದ", "ಧ"),
    ("ಪ", "ಫ"),
    ("ಬ", "ಭ"),
    ("ತ", "ಟ"),
    ("ದ", "ಡ"),
    ("ನ", "ಣ"),
    ("ಲ", "ಳ"),
    ("ಶ", "ಷ"),
    ("ಸ", "ಶ"),
    ("ಸ", "ಷ"),
    ("ಿ", "ೀ"),
    ("ು", "ೂ"),
    ("ೆ", "ೇ"),
    ("ೊ", "ೋ"),
    ("ಾ", ""),
    ("ಂ", "ನ್"),
    ("ಂ", "ಮ್"),
    ("ೃ", "ರು"),
    ("್ಯ", ""),
    ("್ರ", ""),
];

/// A small deterministic random number generator, so runs are repeatable.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// Every spelling of `word` that differs by 1 confusion, in either direction.
pub fn confusions(word: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (a, b) in CONFUSIONS {
        for (from, to) in [(a, b), (b, a)] {
            if from.is_empty() {
                continue;
            }
            for (i, _) in word.match_indices(from) {
                let edited = format!("{}{to}{}", &word[..i], &word[i + from.len()..]);
                if edited != word && !edited.is_empty() && !out.contains(&edited) {
                    out.push(edited);
                }
            }
        }
    }
    out
}

/// 1 random confusion of `word`, or `None` when no confusion applies.
pub fn misspell(word: &str, rng: &mut Rng) -> Option<String> {
    let options = confusions(word);
    (!options.is_empty()).then(|| options[rng.below(options.len())].clone())
}

const VOWEL_SIGNS: &[char] = &['ಾ', 'ಿ', 'ೀ', 'ು', 'ೂ', 'ೆ', 'ೇ', 'ೈ', 'ೊ', 'ೋ', 'ೌ'];

/// 1 random typing slip: a character deleted, doubled or swapped with the next, or a vowel
/// sign replaced. Only slips that leave well-formed Kannada script count; a stray vowel
/// sign is trivially rejected and would flatter the dictionary.
pub fn typo(word: &str, rng: &mut Rng) -> Option<String> {
    (0..8).find_map(|_| slip(word, rng).filter(|t| is_well_formed(t)))
}

fn slip(word: &str, rng: &mut Rng) -> Option<String> {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return None;
    }
    let i = 1 + rng.below(chars.len() - 1);
    match rng.below(4) {
        0 => {
            chars.remove(i);
        }
        1 => chars.insert(i, chars[i]),
        2 if i + 1 < chars.len() => chars.swap(i, i + 1),
        2 => return None,
        _ => {
            let signs: Vec<usize> = (1..chars.len())
                .filter(|&j| VOWEL_SIGNS.contains(&chars[j]))
                .collect();
            let &j = signs.get(rng.below(signs.len().max(1)))?;
            chars[j] = VOWEL_SIGNS[rng.below(VOWEL_SIGNS.len())];
        }
    }
    let edited: String = chars.into_iter().collect();
    (edited != word).then_some(edited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confuses_aspiration_length_and_nasals() {
        let found = confusions("ಸಂದರ್ಭ");
        for expected in ["ಸನ್ದರ್ಭ", "ಸಂಧರ್ಭ", "ಸಂದರ್ಬ", "ಶಂದರ್ಭ"]
        {
            assert!(
                found.contains(&expected.to_string()),
                "{expected} in {found:?}"
            );
        }
        assert!(!found.contains(&"ಸಂದರ್ಭ".to_string()));
    }

    #[test]
    fn typos_change_the_word() {
        let mut rng = Rng::new(3);
        for _ in 0..50 {
            if let Some(t) = typo("ಮನೆಯಲ್ಲಿ", &mut rng) {
                assert_ne!(t, "ಮನೆಯಲ್ಲಿ");
                assert!(is_well_formed(&t), "{t}");
            }
        }
        assert_eq!(typo("ಮರ", &mut rng), None);
    }

    #[test]
    fn rng_is_repeatable() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert_eq!(
            (0..5).map(|_| a.below(100)).collect::<Vec<_>>(),
            (0..5).map(|_| b.below(100)).collect::<Vec<_>>()
        );
    }
}
