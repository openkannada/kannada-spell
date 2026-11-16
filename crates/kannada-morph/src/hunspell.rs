//! Writes the lexicon and rules as a Hunspell dictionary: a `.aff` and a `.dic` file.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::class::{ALL_CLASSES, Class, Final};
use crate::lexicon::Lexicon;
use crate::rules::{Rule, rules_for};

const CONSONANTS: &str = "ಕಖಗಘಙಚಛಜಝಞಟಠಡಢಣತಥದಧನಪಫಬಭಮಯರಱಲವಶಷಸಹಳೞ";

/// Signs that belong inside a word. Hunspell would otherwise split words at them.
const WORD_CHARS: &str = "ಂಃಾಿೀುೂೃೄೆೇೈೊೋೌ್ೕೖ಼\u{200C}\u{200D}";

/// Zero-width joiners: the lexicon is stored without them, so Hunspell ignores them in text.
const IGNORED: &str = "\u{200C}\u{200D}";

/// Groups of letters that readers confuse, tried first when suggesting corrections.
const SIMILAR: &[&str] = &[
    "ಕಖ",
    "ಗಘ",
    "ಚಛ",
    "ಜಝ",
    "ಟಠ",
    "ಡಢ",
    "ತಥ",
    "ದಧ",
    "ಪಫ",
    "ಬಭ",
    "ನಣ",
    "ಲಳ",
    "ಶಷಸ",
    "ರಱ",
    "ಿೀ",
    "ುೂ",
    "ೆೇ",
    "ೊೋ",
    "ಅಆ",
    "ಇಈ",
    "ಉಊ",
    "ಎಏ",
    "ಒಓ",
];

/// Spellings of 1 sound that differ by more than 1 character.
const REPLACEMENTS: &[(&str, &str)] = &[
    ("ಂ", "ನ್"),
    ("ನ್", "ಂ"),
    ("ಂ", "ಮ್"),
    ("ಮ್", "ಂ"),
    ("ಂ", "ಣ್"),
    ("ಣ್", "ಂ"),
    ("ೃ", "ರು"),
    ("ರು", "ೃ"),
    ("ಕ್ಷ", "ಕ್ಶ"),
    ("ಜ್ಞ", "ಗ್ನ"),
];

/// The `.aff` file. `try_chars` lists letters by frequency, for Hunspell's suggestions.
pub fn affix_file(try_chars: &str, header: &[&str]) -> String {
    let mut out = String::new();
    for line in header {
        let _ = writeln!(out, "# {line}");
    }
    let _ = writeln!(
        out,
        "SET UTF-8\nLANG kn_IN\nFLAG long\nWORDCHARS {WORD_CHARS}\nIGNORE {IGNORED}\nTRY {try_chars}"
    );
    let _ = writeln!(out, "\nMAP {}", SIMILAR.len());
    for group in SIMILAR {
        let _ = writeln!(out, "MAP {group}");
    }
    let _ = writeln!(out, "\nREP {}", REPLACEMENTS.len());
    for (from, to) in REPLACEMENTS {
        let _ = writeln!(out, "REP {from} {to}");
    }
    for &class in ALL_CLASSES {
        let rules: Vec<Rule> = rules_for(class)
            .into_iter()
            .filter(|r| !r.is_identity())
            .collect();
        if rules.is_empty() {
            continue;
        }
        let flag = class.flag();
        let _ = writeln!(out, "\n# {class:?}\nSFX {flag} Y {}", rules.len());
        for rule in &rules {
            let zero_if_empty = |s: &str| {
                if s.is_empty() {
                    "0".to_string()
                } else {
                    s.to_string()
                }
            };
            let (strip, add) = (zero_if_empty(&rule.strip), zero_if_empty(&rule.add));
            let _ = writeln!(
                out,
                "SFX {flag} {strip} {add} {} is:{}",
                condition(rule),
                rule.tags
            );
        }
    }
    out
}

/// The `.dic` file: 1 line per stem and lemma, with its class flags.
pub fn dictionary_file(lexicon: &Lexicon) -> String {
    let mut lines: BTreeMap<(String, String, String, String), Vec<&str>> = BTreeMap::new();
    for e in lexicon.entries() {
        let flags = lines
            .entry((
                e.stem.clone(),
                e.lemma.clone(),
                e.pos.clone(),
                e.tags.clone(),
            ))
            .or_default();
        if e.class != Class::Fixed && !flags.contains(&e.class.flag()) {
            flags.push(e.class.flag());
        }
    }
    let mut out = format!("{}\n", lines.len());
    for ((stem, lemma, pos, tags), flags) in &lines {
        out.push_str(stem);
        if !flags.is_empty() {
            let _ = write!(out, "/{}", flags.concat());
        }
        if lemma != stem {
            let _ = write!(out, " st:{lemma}");
        }
        let _ = write!(out, " po:{pos}");
        if !tags.is_empty() {
            let _ = write!(out, " is:{tags}");
        }
        out.push('\n');
    }
    out
}

/// Reads a `.dic` file written by [`dictionary_file`] back into a lexicon.
pub fn parse_dictionary(text: &str) -> Result<Lexicon, String> {
    let mut lexicon = Lexicon::default();
    for (number, line) in text.lines().enumerate().skip(1) {
        let mut fields = line.split(' ');
        let head = fields.next().unwrap_or_default();
        let (stem, flags) = head.split_once('/').unwrap_or((head, ""));
        let (mut lemma, mut pos, mut tags) = (stem, "", "");
        for field in fields {
            match field.split_once(':') {
                Some(("st", v)) => lemma = v,
                Some(("po", v)) => pos = v,
                Some(("is", v)) => tags = v,
                _ => return Err(format!("line {}: unexpected field {field:?}", number + 1)),
            }
        }
        let flag_list: Vec<&str> = if flags.is_empty() {
            vec![Class::Fixed.flag()]
        } else {
            (0..flags.len())
                .step_by(2)
                .map(|i| &flags[i..i + 2])
                .collect()
        };
        for flag in flag_list {
            let class = Class::from_flag(flag)
                .ok_or_else(|| format!("line {}: unknown flag {flag}", number + 1))?;
            lexicon.insert(crate::lexicon::Entry {
                stem: stem.to_string(),
                class,
                lemma: lemma.to_string(),
                pos: pos.to_string(),
                tags: tags.to_string(),
            });
        }
    }
    Ok(lexicon)
}

/// The Hunspell condition: the stem's last character, or a set of them.
fn condition(rule: &Rule) -> String {
    let mut chars = String::new();
    let mut doubled = String::new();
    for f in &rule.finals {
        match f {
            Final::Consonant => chars.push_str(CONSONANTS),
            Final::Char(c) => chars.push(*c),
            Final::Doubled(c) => doubled.push(*c),
        }
    }
    let class = |set: String| {
        if set.chars().count() == 1 {
            set
        } else {
            format!("[{set}]")
        }
    };
    if doubled.is_empty() {
        class(chars)
    } else {
        format!("{}{}", class(doubled), crate::script::VIRAMA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::Entry;
    use crate::noun::Declension;

    #[test]
    fn affix_file_has_one_block_per_class() {
        let aff = affix_file("ಅರ", &["header"]);
        assert!(aff.starts_with("# header\nSET UTF-8\n"));
        assert!(aff.contains("\nSFX Nu ು ನ್ನು ು is:+acc\n"));
        assert!(aff.contains("\nSFX Na 0 ಗಳು [ಕ"));
        assert!(!aff.contains("SFX Fx"));
        assert!(aff.contains("\nSFX Np ು 0 ು is:+gen\n"));
        assert!(aff.contains("\nIGNORE \u{200C}\u{200D}\n"));
        assert!(!aff.contains("  "));
        let declared: usize = aff
            .lines()
            .filter(|l| l.starts_with("SFX ") && l.split(' ').count() == 4)
            .map(|l| l.rsplit(' ').next().unwrap().parse::<usize>().unwrap())
            .sum();
        let written = aff
            .lines()
            .filter(|l| l.starts_with("SFX ") && l.contains(" is:"))
            .count();
        assert_eq!(declared, written);
    }

    #[test]
    fn dictionary_merges_flags_of_one_lemma() {
        let lexicon = Lexicon::from_entries([
            Entry::new("ಕಾಡು", Class::Noun(Declension::Forest)),
            Entry {
                pos: "n".into(),
                ..Entry::new("ಕಾಡು", Class::Noun(Declension::Thing))
            },
            Entry::new("ಕಾಡು", Class::Verb),
            Entry {
                lemma: "ಬರು".into(),
                pos: "v".into(),
                tags: "+past.3sn".into(),
                ..Entry::new("ಬಂತು", Class::Fixed)
            },
        ]);
        let dic = dictionary_file(&lexicon);
        assert_eq!(
            dic,
            "3\nಕಾಡು/NuNv po:n\nಕಾಡು/Vb po:v\nಬಂತು st:ಬರು po:v is:+past.3sn\n"
        );
        assert_eq!(parse_dictionary(&dic).unwrap().entries(), lexicon.entries());
    }
}
