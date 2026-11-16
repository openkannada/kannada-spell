//! Sound-alike keys for Kannada words, for fuzzy search.
//!
//! Writers and searchers confuse letters that sound alike: aspirated and plain
//! stops (ದ, ಧ), short and long vowels (ಿ, ೀ), retroflex and dental (ಟ, ತ),
//! the 3 sibilants (ಶ, ಷ, ಸ), and nasals written as ಂ or as a letter. A key
//! maps a word to a string of sound classes, so that spellings that sound alike
//! share a key. There are 3 keys, from strict to loose:
//!
//! - [`Level::Narrow`] folds aspiration, vowel length, doubled consonants and
//!   the anusvara. ಸಂಬಂಧ and ಸಮ್ಬಂದ match.
//! - [`Level::Medium`] also folds retroflex with dental, ಳ with ಲ, the 3
//!   sibilants, ಜ with ಝ and ಫ with ಪ. ನಿರ್ದಿಷ್ಟ and ನಿರ್ಧಿಸ್ಟ match.
//! - [`Level::Broad`] keeps only the consonants and a leading vowel. It is the
//!   key for Latin input too: "nirdishta" and ನಿರ್ದಿಷ್ಟ match.
//!
//! The keys are plain ASCII.

/// How much a key folds. See the crate documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Narrow,
    Medium,
    Broad,
}

/// 1 sound in a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sound {
    Consonant(Place),
    Vowel(char),
    /// ಂ before a consonant: a nasal at that consonant's place.
    Nasal,
}

/// A consonant class at narrow level. Upper case is retroflex where Kannada contrasts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Place {
    narrow: char,
    medium: char,
}

const fn place(narrow: char, medium: char) -> Place {
    Place { narrow, medium }
}

fn consonant(c: char) -> Option<Place> {
    Some(match c {
        'ಕ' | 'ಖ' => place('k', 'k'),
        'ಗ' | 'ಘ' => place('g', 'g'),
        'ಙ' => place('n', 'n'),
        'ಚ' | 'ಛ' => place('c', 'c'),
        'ಜ' | 'ಝ' => place('j', 'j'),
        'ಞ' => place('n', 'n'),
        'ಟ' | 'ಠ' => place('T', 't'),
        'ಡ' | 'ಢ' => place('D', 'd'),
        'ಣ' => place('N', 'n'),
        'ತ' | 'ಥ' => place('t', 't'),
        'ದ' | 'ಧ' => place('d', 'd'),
        'ನ' => place('n', 'n'),
        'ಪ' | 'ಫ' => place('p', 'p'),
        'ಬ' | 'ಭ' => place('b', 'b'),
        'ಮ' => place('m', 'm'),
        'ಯ' => place('y', 'y'),
        'ರ' | 'ಱ' => place('r', 'r'),
        'ಲ' => place('l', 'l'),
        'ಳ' | 'ೞ' => place('L', 'l'),
        'ವ' => place('v', 'v'),
        'ಶ' => place('S', 's'),
        'ಷ' => place('Z', 's'),
        'ಸ' => place('s', 's'),
        'ಹ' => place('h', 'h'),
        _ => return None,
    })
}

/// The vowel class of an independent vowel or a vowel sign: length is folded.
fn vowel(c: char) -> Option<char> {
    Some(match c {
        'ಅ' | 'ಆ' | 'ಾ' => 'a',
        'ಇ' | 'ಈ' | 'ಿ' | 'ೀ' => 'i',
        'ಉ' | 'ಊ' | 'ು' | 'ೂ' => 'u',
        'ಎ' | 'ಏ' | 'ೆ' | 'ೇ' => 'e',
        'ಐ' | 'ೈ' => 'e',
        'ಒ' | 'ಓ' | 'ೊ' | 'ೋ' | 'ಔ' | 'ೌ' => 'o',
        _ => return None,
    })
}

const VIRAMA: char = '\u{0CCD}';
const ANUSVARA: char = '\u{0C82}';
const VISARGA: char = '\u{0C83}';

/// The sounds of a word in Kannada script. Other characters are skipped.
fn sounds(word: &str) -> Vec<Sound> {
    let chars: Vec<char> = word.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(p) = consonant(c) {
            out.push(Sound::Consonant(p));
            let next = chars.get(i + 1).copied();
            match next {
                Some(VIRAMA) => i += 1,
                Some('ೃ' | 'ೄ') => {
                    out.push(Sound::Consonant(place('r', 'r')));
                    out.push(Sound::Vowel('u'));
                    i += 1;
                }
                Some(sign @ '\u{0CBE}'..='\u{0CCC}') if vowel(sign).is_some() => {
                    out.extend(vowel(sign).map(Sound::Vowel));
                    i += 1;
                }
                _ => out.push(Sound::Vowel('a')),
            }
        } else if let Some(v) = vowel(c) {
            out.push(Sound::Vowel(v));
        } else if matches!(c, 'ಋ' | 'ೠ') {
            out.push(Sound::Consonant(place('r', 'r')));
            out.push(Sound::Vowel('u'));
        } else if c == ANUSVARA {
            out.push(Sound::Nasal);
        } else if c == VISARGA {
            out.push(Sound::Consonant(place('h', 'h')));
        }
        i += 1;
    }
    out
}

/// A nasal before a consonant sounds at that consonant's place: m before p and b, n before
/// the others. An anusvara at the end of a word sounds as m (ಹಿಂ, "him").
fn nasal_before(next: Option<&Sound>) -> char {
    match next {
        Some(Sound::Consonant(p)) if matches!(p.narrow, 'p' | 'b' | 'm') => 'm',
        None => 'm',
        _ => 'n',
    }
}

fn key_of_sounds(sounds: &[Sound], level: Level) -> String {
    let mut out = String::new();
    for (i, sound) in sounds.iter().enumerate() {
        let code = match *sound {
            Sound::Consonant(p) => {
                let nasal = matches!(p.narrow, 'n' | 'N' | 'm')
                    && matches!(sounds.get(i + 1), Some(Sound::Consonant(_)));
                if nasal {
                    nasal_before(sounds.get(i + 1))
                } else if level == Level::Narrow {
                    p.narrow
                } else {
                    p.medium
                }
            }
            Sound::Nasal => nasal_before(sounds.get(i + 1)),
            Sound::Vowel(v) => {
                if level == Level::Broad && i > 0 {
                    continue;
                }
                v
            }
        };
        if out.ends_with(code) && !matches!(sound, Sound::Vowel(_)) {
            continue;
        }
        out.push(code);
    }
    out
}

/// The sound-alike key of a word written in Kannada script.
pub fn key(word: &str, level: Level) -> String {
    key_of_sounds(&sounds(word), level)
}

/// All 3 keys, from narrow to broad.
pub fn keys(word: &str) -> [String; 3] {
    let s = sounds(word);
    [Level::Narrow, Level::Medium, Level::Broad].map(|level| key_of_sounds(&s, level))
}

/// Latin letter groups and the sounds they spell; the longest group that matches wins. Matches common
/// informal romanization (sh, th, aa, ee) as well as ISO 15919 letters. "N" stands
/// for the anusvara, which sounds at the place of what follows it.
const LATIN: &[(&str, &str)] = &[
    ("ksh", "ks"),
    ("aa", "a"),
    ("ee", "i"),
    ("ii", "i"),
    ("oo", "u"),
    ("uu", "u"),
    ("ai", "e"),
    ("au", "o"),
    ("ou", "o"),
    ("kh", "k"),
    ("gh", "g"),
    ("ch", "c"),
    ("jh", "j"),
    ("th", "t"),
    ("dh", "d"),
    ("ph", "p"),
    ("bh", "b"),
    ("sh", "s"),
    ("~j", "n"),
    ("~g", "n"),
    ("ḥ", "h"),
    ("ā", "a"),
    ("ī", "i"),
    ("ū", "u"),
    ("ē", "e"),
    ("ō", "o"),
    ("ṭ", "t"),
    ("ḍ", "d"),
    ("ṇ", "n"),
    ("ḷ", "l"),
    ("ṭh", "t"),
    ("ḍh", "d"),
    ("ś", "s"),
    ("ṣ", "s"),
    ("ṁ", "N"),
    ("ṃ", "N"),
    ("ṅ", "n"),
    ("ñ", "n"),
    ("r̥", "ru"),
    ("ṛ", "ru"),
    ("a", "a"),
    ("i", "i"),
    ("u", "u"),
    ("e", "e"),
    ("o", "o"),
    ("k", "k"),
    ("q", "k"),
    ("c", "c"),
    ("g", "g"),
    ("j", "j"),
    ("z", "j"),
    ("t", "t"),
    ("d", "d"),
    ("n", "n"),
    ("p", "p"),
    ("f", "p"),
    ("b", "b"),
    ("m", "m"),
    ("y", "y"),
    ("r", "r"),
    ("l", "l"),
    ("v", "v"),
    ("w", "v"),
    ("s", "s"),
    ("x", "ks"),
    ("h", "h"),
];

/// The broad key of a word typed in Latin letters, comparable with [`Level::Broad`] keys of Kannada words.
pub fn latin_key(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut rest = lower.as_str();
    let mut sounds = Vec::new();
    while !rest.is_empty() {
        let longest = LATIN
            .iter()
            .filter(|(from, _)| rest.starts_with(from))
            .max_by_key(|(from, _)| from.len());
        let Some((from, to)) = longest else {
            let skip = rest.chars().next().map_or(1, char::len_utf8);
            rest = &rest[skip..];
            continue;
        };
        for c in to.chars() {
            sounds.push(match c {
                'N' => Sound::Nasal,
                _ if "aiueo".contains(c) => Sound::Vowel(c),
                _ => Sound::Consonant(place(c, c)),
            });
        }
        rest = &rest[from.len()..];
    }
    key_of_sounds(&sounds, Level::Broad)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(a: &str, b: &str, level: Level) -> bool {
        key(a, level) == key(b, level)
    }

    #[test]
    fn narrow_folds_aspiration_length_doubling_and_anusvara() {
        assert!(same("ಸಂಬಂಧ", "ಸಮ್ಬಂದ", Level::Narrow));
        assert!(same("ದೀಪ", "ದಿಪ", Level::Narrow));
        assert!(same("ಅಮ್ಮ", "ಅಮ", Level::Narrow));
        assert!(same("ಭಾರತ", "ಬಾರತ", Level::Narrow));
        assert!(!same("ಕಟ", "ಕತ", Level::Narrow));
        assert!(!same("ಮರ", "ಮನೆ", Level::Narrow));
    }

    #[test]
    fn medium_folds_retroflex_and_sibilants() {
        assert!(same("ನಿರ್ದಿಷ್ಟ", "ನಿರ್ಧಿಸ್ಟ", Level::Medium));
        assert!(same("ಬಾಳೆ", "ಬಾಲೆ", Level::Medium));
        assert!(same("ವಿಷ", "ವಿಶ", Level::Medium));
        assert!(!same("ವಿಷ", "ವಿಷು", Level::Medium));
    }

    #[test]
    fn broad_keeps_consonants_and_a_leading_vowel() {
        assert_eq!(key("ಅಮ್ಮ", Level::Broad), "am");
        assert_eq!(key("ಮನೆ", Level::Broad), "mn");
        assert!(same("ಕೃಷ್ಣ", "ಕ್ರಿಶ್ನ", Level::Broad));
    }

    #[test]
    fn keys_are_ordered_narrow_to_broad() {
        assert_eq!(keys("ದೀಪ"), ["dipa", "dipa", "dp"]);
    }

    #[test]
    fn latin_matches_kannada_at_broad_level() {
        for (latin, kannada) in [
            ("nirdishta", "ನಿರ್ದಿಷ್ಟ"),
            ("bharata", "ಭಾರತ"),
            ("mane", "ಮನೆ"),
            ("bengaluru", "ಬೆಂಗಳೂರು"),
            ("sambandha", "ಸಂಬಂಧ"),
            ("kṛṣṇa", "ಕೃಷ್ಣ"),
            ("amma", "ಅಮ್ಮ"),
            ("cakra", "ಚಕ್ರ"),
            ("chakra", "ಚಕ್ರ"),
        ] {
            assert_eq!(
                latin_key(latin),
                key(kannada, Level::Broad),
                "{latin} {kannada}"
            );
        }
    }

    #[test]
    fn an_independent_vowel_after_a_consonant_is_its_own_sound() {
        assert_eq!(key("ಕಅ", Level::Narrow), "kaa");
    }

    #[test]
    fn visarga_final_anusvara_and_baraha_jna() {
        for (latin, kannada) in [
            ("duHkha", "ದುಃಖ"),
            ("svataH", "ಸ್ವತಃ"),
            ("hiM", "ಹಿಂ"),
            ("hiṁ", "ಹಿಂ"),
            ("saṁbandha", "ಸಂಬಂಧ"),
            ("praj~je", "ಪ್ರಜ್ಞೆ"),
            ("gūḍha", "ಗೂಢ"),
            ("pāṭha", "ಪಾಠ"),
        ] {
            assert_eq!(
                latin_key(latin),
                key(kannada, Level::Broad),
                "{latin} {kannada}"
            );
        }
    }

    #[test]
    fn ignores_other_characters() {
        assert_eq!(key("ಮನೆ123", Level::Broad), key("ಮನೆ", Level::Broad));
        assert_eq!(latin_key("Mane!"), latin_key("mane"));
    }
}
