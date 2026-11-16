//! Kannada script: character classes, normalization and word tokens.

use unicode_normalization::UnicodeNormalization;

pub const VIRAMA: char = '\u{0CCD}';
pub const ANUSVARA: char = '\u{0C82}';
const ZWNJ: char = '\u{200C}';
const ZWJ: char = '\u{200D}';

/// A consonant letter, which carries an inherent "a" unless a sign follows.
pub fn is_consonant(c: char) -> bool {
    matches!(c, '\u{0C95}'..='\u{0CB9}' | '\u{0CDE}')
}

/// A dependent vowel sign such as "ಿ" or "ೊ".
pub fn is_vowel_sign(c: char) -> bool {
    matches!(c, '\u{0CBE}'..='\u{0CCC}' | '\u{0CE2}' | '\u{0CE3}') && c != VIRAMA
}

/// An independent vowel letter such as "ಅ" or "ಊ".
pub fn is_independent_vowel(c: char) -> bool {
    matches!(c, '\u{0C85}'..='\u{0C94}' | '\u{0CE0}' | '\u{0CE1}')
}

/// The sign written after a consonant for an independent vowel; `None` for "ಅ", which is inherent.
pub fn vowel_sign(vowel: char) -> Option<char> {
    Some(match vowel {
        'ಆ' => 'ಾ',
        'ಇ' => 'ಿ',
        'ಈ' => 'ೀ',
        'ಉ' => 'ು',
        'ಊ' => 'ೂ',
        'ಋ' => 'ೃ',
        'ೠ' => 'ೄ',
        'ಎ' => 'ೆ',
        'ಏ' => 'ೇ',
        'ಐ' => 'ೈ',
        'ಒ' => 'ೊ',
        'ಓ' => 'ೋ',
        'ಔ' => 'ೌ',
        _ => return None,
    })
}

/// True for vowels and vowel signs pronounced at the front of the mouth, which take the "ಯ" glide.
pub fn is_front_vowel(c: char) -> bool {
    matches!(c, 'ಇ' | 'ಈ' | 'ಎ' | 'ಏ' | 'ಐ' | 'ಿ' | 'ೀ' | 'ೆ' | 'ೇ' | 'ೈ')
}

fn is_letter(c: char) -> bool {
    is_consonant(c) || is_independent_vowel(c)
}

fn is_word_char(c: char) -> bool {
    is_letter(c)
        || is_vowel_sign(c)
        || matches!(
            c,
            VIRAMA | ANUSVARA | '\u{0C83}' | '\u{0CBC}' | '\u{0CD5}' | '\u{0CD6}' | ZWNJ | ZWJ
        )
}

/// NFC with zero-width joiners removed, so that 1 spelling has 1 byte sequence.
pub fn normalize(word: &str) -> String {
    word.nfc().filter(|&c| c != ZWNJ && c != ZWJ).collect()
}

/// The Kannada words in `text`, normalized. Digits, Latin text and punctuation separate words.
/// A run written straight after a digit or a Latin letter is a suffix of it, as in
/// "1990ರಲ್ಲಿ", and is not a word of its own.
pub fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    let mut previous = ' ';
    let mut runs = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
        match (is_word_char(c), start) {
            (true, None) => start = Some((i, previous)),
            (false, Some((s, before))) => {
                runs.push((&text[s..i], before));
                start = None;
            }
            _ => {}
        }
        previous = c;
    }
    runs.into_iter().filter_map(|(run, before)| {
        if before.is_alphanumeric() && !is_word_char(before) {
            return None;
        }
        let start = run.find(is_letter)?;
        let word = normalize(&run[start..]);
        (!word.is_empty()).then_some(word)
    })
}

/// True when `word` is written only in Kannada letters and signs, starting with a letter.
pub fn is_kannada_word(word: &str) -> bool {
    word.chars().next().is_some_and(is_letter) && word.chars().all(is_word_char)
}

/// True when every sign in `word` sits where Kannada script allows it: a vowel sign,
/// virama or nukta after a consonant (a vowel sign may follow a nukta), and an
/// anusvara or visarga after a letter or a vowel sign.
pub fn is_well_formed(word: &str) -> bool {
    let mut previous: Option<char> = None;
    for c in word.chars() {
        let ok = match c {
            '\u{0CBC}' => previous.is_some_and(is_consonant),
            VIRAMA => previous.is_some_and(|p| is_consonant(p) || p == '\u{0CBC}'),
            _ if is_vowel_sign(c) => previous.is_some_and(|p| is_consonant(p) || p == '\u{0CBC}'),
            ANUSVARA | '\u{0C83}' => previous.is_some_and(|p| is_letter(p) || is_vowel_sign(p)),
            _ => is_letter(c) || c == ZWNJ || c == ZWJ,
        };
        if !ok {
            return false;
        }
        previous = Some(c);
    }
    previous.is_some()
}

/// The number of aksharas (written syllables) in `word`.
pub fn akshara_count(word: &str) -> usize {
    let chars: Vec<char> = word.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|&(i, &c)| is_letter(c) && (i == 0 || chars[i - 1] != VIRAMA))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_words_on_punctuation_digits_and_latin() {
        let found: Vec<String> = words("ಮರ, ಮನೆ 12 ಕಾಡು (tree) ಹೂ.").collect();
        assert_eq!(found, ["ಮರ", "ಮನೆ", "ಕಾಡು", "ಹೂ"]);
    }

    #[test]
    fn skips_suffixes_written_on_numbers_and_latin_words() {
        let found: Vec<String> = words("1990ರಲ್ಲಿ BBCಯಲ್ಲಿ ೧೦ನೆಯ ಮರ").collect();
        assert_eq!(found, ["ಮರ"]);
    }

    #[test]
    fn drops_leading_signs_and_joiners() {
        let found: Vec<String> = words("\u{0CBF}ಮರ ಕ\u{0CCD}\u{200D}ಷ").collect();
        assert_eq!(found, ["ಮರ", "ಕ\u{0CCD}ಷ"]);
    }

    #[test]
    fn composes_split_vowel_signs() {
        assert_eq!(normalize("ಕ\u{0CC6}\u{0CC2}"), "ಕ\u{0CCA}");
    }

    #[test]
    fn checks_where_signs_may_stand() {
        for good in ["ಮರ", "ಕ್ಷೇತ್ರ", "ಸಂಬಂಧ", "ದುಃಖ", "ಬ್ಯಾಂಕ್"]
        {
            assert!(is_well_formed(good), "{good}");
        }
        for bad in ["ಮಾಾ", "ಾಮ", "ಕ್ಾ", "ಕ್ಂ", ""] {
            assert!(!is_well_formed(bad), "{bad}");
        }
    }

    #[test]
    fn counts_aksharas_across_conjuncts() {
        assert_eq!(akshara_count("ಮರ"), 2);
        assert_eq!(akshara_count("ಕ್ಷೇತ್ರ"), 2);
        assert_eq!(akshara_count("ಅಮ್ಮ"), 2);
    }

    #[test]
    fn classifies_characters() {
        assert!(is_consonant('ಕ') && !is_consonant('ಾ'));
        assert!(is_vowel_sign('ೊ') && !is_vowel_sign(VIRAMA));
        assert_eq!(vowel_sign('ಊ'), Some('ೂ'));
        assert_eq!(vowel_sign('ಅ'), None);
        assert!(is_kannada_word("ಮರ") && !is_kannada_word("ಮರa") && !is_kannada_word("ಾಮ"));
    }
}
