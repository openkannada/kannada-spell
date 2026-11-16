//! Inflection classes: which paradigm a dictionary stem takes.

use crate::form::{Form, extend};
use crate::noun::{CLITICS, Declension, copular_predicates, noun_forms};
use crate::sandhi::Junction;
use crate::script::{ANUSVARA, VIRAMA, is_consonant};
use crate::verb::{irregular_root_forms, participle_stem_forms, past_stem_forms, regular_forms};

/// The paradigm a stem takes. Each class has a 2 letter Hunspell flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    Noun(Declension),
    /// Adjectives and other words that take the copula and clitics but no case: ಸುಂದರವಾದ.
    Adjective,
    /// Adverbs, conjunctions and interjections: clitics only.
    Particle,
    /// A form listed with its own tags, such as ಬಂತು "+past.3sn" of ಬರು. Takes no affixes.
    Fixed,
    /// A regular verb root such as ಮಾಡು or ಬರೆ.
    Verb,
    /// The root of an irregular verb such as ಬರು: only the forms built on the root.
    VerbRoot,
    /// An irregular past stem such as ಬಂದ.
    VerbPast,
    /// An irregular conjunctive participle such as ಬಂದು.
    VerbParticiple,
}

/// The final character a class requires of its stems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Final {
    /// A consonant letter with its inherent vowel: ಮರ.
    Consonant,
    Char(char),
    /// This consonant and a virama, for classes that double the consonant: ಬಸ್, ಬಸ್ಸು.
    Doubled(char),
}

impl Final {
    /// The last character of `stem` as a final: a consonant letter, or the character itself.
    pub fn of(stem: &str) -> Option<Final> {
        let last = stem.chars().last()?;
        Some(if is_consonant(last) {
            Final::Consonant
        } else {
            Final::Char(last)
        })
    }

    /// True when `stem` ends this way.
    pub fn matches(self, stem: &str) -> bool {
        match self {
            Final::Doubled(c) => stem.strip_suffix(VIRAMA).is_some_and(|s| s.ends_with(c)),
            _ => Final::of(stem) == Some(self),
        }
    }

    /// A short stem with this ending, used to read off a class's affix rules.
    fn sample_stem(self) -> String {
        match self {
            Final::Consonant => "ಕ".to_string(),
            Final::Char(c) => format!("ಕ{c}"),
            Final::Doubled(c) => format!("ಅ{c}{VIRAMA}"),
        }
    }
}

pub const ALL_CLASSES: &[Class] = &[
    Class::Noun(Declension::Tree),
    Class::Noun(Declension::Boy),
    Class::Noun(Declension::House),
    Class::Noun(Declension::Girl),
    Class::Noun(Declension::Forest),
    Class::Noun(Declension::Thing),
    Class::Noun(Declension::This),
    Class::Noun(Declension::Plural),
    Class::Noun(Declension::Loan),
    Class::Noun(Declension::LoanDoubled),
    Class::Adjective,
    Class::Particle,
    Class::Fixed,
    Class::Verb,
    Class::VerbRoot,
    Class::VerbPast,
    Class::VerbParticiple,
];

const FRONT: &[Final] = &[
    Final::Char('ಿ'),
    Final::Char('ೆ'),
    Final::Char('ೀ'),
    Final::Char('ೇ'),
    Final::Char('ೈ'),
];
const LONG_BACK: &[Final] = &[
    Final::Char('ು'),
    Final::Char('ಾ'),
    Final::Char('ೂ'),
    Final::Char('ೋ'),
    Final::Char('ೌ'),
];
const U: &[Final] = &[Final::Char('ು')];
const DOUBLED: &[Final] = &[
    Final::Doubled('ಕ'),
    Final::Doubled('ಖ'),
    Final::Doubled('ಗ'),
    Final::Doubled('ಘ'),
    Final::Doubled('ಙ'),
    Final::Doubled('ಚ'),
    Final::Doubled('ಛ'),
    Final::Doubled('ಜ'),
    Final::Doubled('ಝ'),
    Final::Doubled('ಞ'),
    Final::Doubled('ಟ'),
    Final::Doubled('ಠ'),
    Final::Doubled('ಡ'),
    Final::Doubled('ಢ'),
    Final::Doubled('ಣ'),
    Final::Doubled('ತ'),
    Final::Doubled('ಥ'),
    Final::Doubled('ದ'),
    Final::Doubled('ಧ'),
    Final::Doubled('ನ'),
    Final::Doubled('ಪ'),
    Final::Doubled('ಫ'),
    Final::Doubled('ಬ'),
    Final::Doubled('ಭ'),
    Final::Doubled('ಮ'),
    Final::Doubled('ಯ'),
    Final::Doubled('ರ'),
    Final::Doubled('ಲ'),
    Final::Doubled('ವ'),
    Final::Doubled('ಶ'),
    Final::Doubled('ಷ'),
    Final::Doubled('ಸ'),
    Final::Doubled('ಹ'),
    Final::Doubled('ಳ'),
];
const OPEN: &[Final] = &[
    Final::Consonant,
    Final::Char('ಿ'),
    Final::Char('ೆ'),
    Final::Char('ು'),
    Final::Char('ಾ'),
    Final::Char('ೀ'),
    Final::Char('ೂ'),
    Final::Char('ೇ'),
    Final::Char('ೋ'),
];

impl Class {
    /// The Hunspell flag, also used as the class name in data files.
    pub fn flag(self) -> &'static str {
        match self {
            Class::Noun(Declension::Tree) => "Na",
            Class::Noun(Declension::Boy) => "Nm",
            Class::Noun(Declension::House) => "Ne",
            Class::Noun(Declension::Girl) => "Nf",
            Class::Noun(Declension::Forest) => "Nu",
            Class::Noun(Declension::Thing) => "Nv",
            Class::Noun(Declension::This) => "Nd",
            Class::Noun(Declension::Plural) => "Np",
            Class::Noun(Declension::Loan) => "Nl",
            Class::Noun(Declension::LoanDoubled) => "Nk",
            Class::Adjective => "Aj",
            Class::Particle => "Px",
            Class::Fixed => "Fx",
            Class::Verb => "Vb",
            Class::VerbRoot => "Vr",
            Class::VerbPast => "Vp",
            Class::VerbParticiple => "Vc",
        }
    }

    pub fn from_flag(flag: &str) -> Option<Class> {
        ALL_CLASSES.iter().copied().find(|c| c.flag() == flag)
    }

    /// The part of speech written after the lemma in an analysis.
    pub fn pos(self) -> &'static str {
        match self {
            Class::Noun(_) => "n",
            Class::Adjective => "adj",
            Class::Particle | Class::Fixed => "x",
            Class::Verb | Class::VerbRoot | Class::VerbPast | Class::VerbParticiple => "v",
        }
    }

    /// The stem endings this class allows. An empty list allows any ending.
    pub fn finals(self) -> &'static [Final] {
        match self {
            Class::Noun(Declension::Tree | Declension::Boy) | Class::VerbPast => {
                &[Final::Consonant]
            }
            Class::Noun(Declension::House | Declension::Girl) => FRONT,
            Class::Noun(Declension::Thing) => LONG_BACK,
            Class::Noun(Declension::Forest | Declension::This | Declension::Plural) => U,
            Class::Noun(Declension::Loan) => &[Final::Char(VIRAMA)],
            Class::Noun(Declension::LoanDoubled) => DOUBLED,
            Class::Verb | Class::VerbRoot => &[Final::Char('ು'), Final::Char('ಿ'), Final::Char('ೆ')],
            Class::VerbParticiple => &[Final::Char('ು'), Final::Char('ಿ')],
            Class::Adjective | Class::Particle => OPEN,
            Class::Fixed => &[],
        }
    }

    /// True when `stem` has an ending this class allows.
    pub fn accepts(self, stem: &str) -> bool {
        let finals = self.finals();
        finals.is_empty() || finals.iter().any(|f| f.matches(stem))
    }

    /// Every form of `stem` in this class, including the stem itself.
    pub fn forms(self, stem: &str) -> Vec<Form> {
        match self {
            Class::Noun(declension) => noun_forms(stem, declension),
            Class::Adjective => adjective_forms(stem),
            Class::Particle => {
                let host = Form::stem(stem, open_junction(stem));
                let mut forms = extend(std::slice::from_ref(&host), CLITICS);
                forms.push(host);
                forms
            }
            Class::Fixed => vec![Form::stem(stem, Junction::Elide)],
            Class::Verb => regular_forms(stem),
            Class::VerbRoot => irregular_root_forms(stem),
            Class::VerbPast => past_stem_forms(stem),
            Class::VerbParticiple => participle_stem_forms(stem),
        }
    }

    /// The forms of 1 sample stem per allowed ending, from which affix rules are read.
    pub(crate) fn samples(self) -> Vec<(Final, String)> {
        self.finals()
            .iter()
            .map(|&f| (f, f.sample_stem()))
            .collect()
    }
}

/// A stem ending in "ು" drops it before a vowel; other endings take a glide.
fn open_junction(stem: &str) -> Junction {
    match stem.chars().last() {
        Some('ು') => Junction::Elide,
        Some(VIRAMA | ANUSVARA) => Junction::Elide,
        _ => Junction::Glide,
    }
}

fn adjective_forms(stem: &str) -> Vec<Form> {
    let host = Form::stem(stem, open_junction(stem));
    let mut forms = extend(std::slice::from_ref(&host), CLITICS);
    forms.extend(copular_predicates(&host));
    forms.push(host);
    forms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_are_unique_and_round_trip() {
        for &class in ALL_CLASSES {
            assert_eq!(Class::from_flag(class.flag()), Some(class));
        }
        let mut flags: Vec<&str> = ALL_CLASSES.iter().map(|c| c.flag()).collect();
        flags.sort();
        flags.dedup();
        assert_eq!(flags.len(), ALL_CLASSES.len());
    }

    #[test]
    fn classes_check_the_final_character() {
        assert!(Class::Noun(Declension::Tree).accepts("ಮರ"));
        assert!(!Class::Noun(Declension::Tree).accepts("ಮನೆ"));
        assert!(Class::Noun(Declension::House).accepts("ಮನೆ"));
        assert!(Class::Verb.accepts("ಬರೆ") && !Class::Verb.accepts("ಮರ"));
        assert!(Class::Fixed.accepts("ಆಯ್"));
    }

    #[test]
    fn doubled_finals_name_the_consonant() {
        assert!(Final::Doubled('ಸ').matches("ಬಸ್"));
        assert!(!Final::Doubled('ಸ').matches("ಬ್ಯಾಂಕ್"));
        assert!(!Final::Doubled('ಸ').matches("ಬಸ"));
        assert!(Class::Noun(Declension::LoanDoubled).accepts("ಬಸ್"));
    }

    #[test]
    fn adjectives_take_the_copula() {
        let forms = Class::Adjective.forms("ಸುಂದರ");
        assert!(
            !forms.iter().any(|f| f.tags == "|ಆಗು+v+imp.2p"),
            "only fused predicate forms"
        );
        assert!(
            forms
                .iter()
                .any(|f| f.text == "ಸುಂದರವಾದ" && f.tags == "|ಆಗು+v+rel.past")
        );
        assert!(forms.iter().any(|f| f.text == "ಸುಂದರವಾಗಿ"));
        let forms = Class::Adjective.forms("ಹೆಚ್ಚು");
        assert!(forms.iter().any(|f| f.text == "ಹೆಚ್ಚಾಗಿ"));
    }
}
