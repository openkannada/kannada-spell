//! Morphological analysis by affix stripping: every way a word splits into a known stem and a rule.

use std::collections::HashMap;
use std::fmt;

use crate::class::Class;
use crate::lexicon::{Entry, Lexicon};
use crate::rules::{Rule, all_rules};
use crate::script::{akshara_count, normalize};

/// 1 reading of a word, written `lemma+pos+tags`, as in ಮರ+n+pl+loc.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Analysis {
    pub lemma: String,
    pub pos: String,
    pub tags: String,
    /// The dictionary stem the word was built on.
    pub stem: String,
    pub class: Class,
    /// True when the stem is not in the lexicon and was guessed from the ending alone.
    pub guessed: bool,
}

impl fmt::Display for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}{}", self.lemma, self.pos, self.tags)
    }
}

/// Classes the guesser may assign to an unknown stem: open noun classes. Unknown words are
/// mostly names and loanwords; guessing verbs turned names such as ಕಜ್ವಿನಿ into participles.
const GUESSABLE: &[Class] = &[
    Class::Noun(crate::noun::Declension::Tree),
    Class::Noun(crate::noun::Declension::House),
    Class::Noun(crate::noun::Declension::Forest),
    Class::Noun(crate::noun::Declension::Thing),
];

pub struct Analyzer {
    lexicon: Lexicon,
    rules: Vec<Rule>,
    by_add: HashMap<String, Vec<usize>>,
}

impl Analyzer {
    pub fn new(lexicon: Lexicon) -> Self {
        let rules = all_rules();
        let mut by_add: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, rule) in rules.iter().enumerate() {
            by_add.entry(rule.add.clone()).or_default().push(i);
        }
        Self {
            lexicon,
            rules,
            by_add,
        }
    }

    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    /// Every form of every stem whose lemma is `lemma`, with the analysis `analyze` gives it,
    /// sorted. Readings that differ only in the stem or class they came from are kept once.
    pub fn paradigm(&self, lemma: &str) -> Vec<(String, Analysis)> {
        let lemma = normalize(lemma);
        let mut forms: Vec<(String, Analysis)> = self
            .lexicon
            .entries()
            .into_iter()
            .filter(|e| e.lemma == lemma)
            .flat_map(|e| {
                e.class
                    .forms(&e.stem)
                    .into_iter()
                    .map(move |f| (f.text, analysis(e, &f.tags, false)))
            })
            .collect();
        forms.sort_by_cached_key(|(word, a)| (word.clone(), a.to_string()));
        forms.dedup_by(|a, b| a.0 == b.0 && a.1.to_string() == b.1.to_string());
        forms
    }

    /// Every analysis of `word` from known stems, sorted. Empty for an unknown word.
    pub fn analyze(&self, word: &str) -> Vec<Analysis> {
        let word = normalize(word);
        let mut found: Vec<Analysis> = self
            .lexicon
            .get(&word)
            .iter()
            .filter(|e| e.class == Class::Fixed)
            .map(|e| analysis(e, "", false))
            .collect();
        for (stem, rule) in self.splits(&word) {
            for entry in self
                .lexicon
                .get(&stem)
                .iter()
                .filter(|e| e.class == rule.class)
            {
                found.push(analysis(entry, &rule.tags, false));
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// True when the word has at least 1 analysis: the spelling check.
    pub fn accepts(&self, word: &str) -> bool {
        !self.analyze(word).is_empty()
    }

    /// Analyses of an unknown word from its ending alone: stems of 2 or more
    /// aksharas in an open class, with the longest matching ending.
    pub fn guess(&self, word: &str) -> Vec<Analysis> {
        let word = normalize(word);
        let mut best: Vec<Analysis> = Vec::new();
        let mut best_len = 0;
        for (stem, rule) in self.splits(&word) {
            if rule.add.is_empty() || !GUESSABLE.contains(&rule.class) || akshara_count(&stem) < 2 {
                continue;
            }
            let len = rule.add.chars().count();
            if len < best_len {
                continue;
            }
            if len > best_len {
                best.clear();
                best_len = len;
            }
            best.push(analysis(&Entry::new(&stem, rule.class), &rule.tags, true));
        }
        best.sort();
        best.dedup();
        best
    }

    /// The lemma of `word`: from the analysis with the longest stem, or a guess, or the word itself.
    pub fn stem(&self, word: &str) -> String {
        let mut analyses = self.analyze(word);
        if analyses.is_empty() {
            analyses = self.guess(word);
        }
        analyses
            .into_iter()
            .max_by_key(|a| (a.stem.chars().count(), std::cmp::Reverse(a.tags.len())))
            .map_or_else(|| normalize(word), |a| a.lemma)
    }

    /// Every (stem, rule) whose rule's ending matches the end of `word` and whose stem has an allowed final.
    /// The stem need not be in the lexicon.
    pub fn splits<'a>(&'a self, word: &'a str) -> impl Iterator<Item = (String, &'a Rule)> + 'a {
        word.char_indices()
            .skip(1)
            .map(|(i, _)| i)
            .chain([word.len()])
            .flat_map(move |i| {
                let (head, add) = word.split_at(i);
                self.by_add
                    .get(add)
                    .into_iter()
                    .flatten()
                    .filter_map(move |&r| {
                        let rule = &self.rules[r];
                        let stem = format!("{head}{}", rule.strip);
                        rule.finals
                            .iter()
                            .any(|f| f.matches(&stem))
                            .then_some((stem, rule))
                    })
            })
    }
}

fn analysis(entry: &Entry, rule_tags: &str, guessed: bool) -> Analysis {
    Analysis {
        lemma: entry.lemma.clone(),
        pos: entry.pos.clone(),
        tags: format!("{}{rule_tags}", entry.tags),
        stem: entry.stem.clone(),
        class: entry.class,
        guessed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noun::Declension;

    fn analyzer() -> Analyzer {
        Analyzer::new(Lexicon::from_entries([
            Entry::new("ಮರ", Class::Noun(Declension::Tree)),
            Entry::new("ಮನೆ", Class::Noun(Declension::House)),
            Entry::new("ಮಾಡು", Class::Verb),
            Entry {
                lemma: "ಬರು".into(),
                ..Entry::new("ಬಂದ", Class::VerbPast)
            },
            Entry {
                lemma: "ಬರು".into(),
                pos: "v".into(),
                tags: "+past.3sn".into(),
                ..Entry::new("ಬಂತು", Class::Fixed)
            },
        ]))
    }

    fn readings(analyzer: &Analyzer, word: &str) -> Vec<String> {
        analyzer
            .analyze(word)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn paradigm_lists_forms_of_every_stem_of_a_lemma() {
        let a = analyzer();
        let forms = a.paradigm("ಬರು");
        let has = |word: &str, reading: &str| {
            forms
                .iter()
                .any(|(w, r)| w == word && r.to_string() == reading)
        };
        assert!(has("ಬಂತು", "ಬರು+v+past.3sn"));
        assert!(has("ಬಂದನು", "ಬರು+v+past.3sm"));
        assert!(
            a.paradigm("ಮರ")
                .iter()
                .any(|(w, r)| w == "ಮರಕ್ಕೆ" && r.to_string() == "ಮರ+n+dat")
        );
    }

    #[test]
    fn paradigm_prints_each_reading_once_and_normalizes_the_lemma() {
        let a = Analyzer::new(Lexicon::from_entries([
            Entry::new("ಹೋಗು", Class::Verb),
            Entry::new("ಮನೆ", Class::Noun(Declension::House)),
            Entry {
                class: Class::Noun(Declension::Thing),
                ..Entry::new("ಮನೆ", Class::Noun(Declension::House))
            },
        ]));
        let printed: Vec<String> = a
            .paradigm("ಮನೆ")
            .iter()
            .map(|(w, r)| format!("{w}\t{r}"))
            .collect();
        let mut unique = printed.clone();
        unique.dedup();
        assert_eq!(printed, unique);
        assert!(printed.windows(2).all(|p| p[0] <= p[1]));
        let decomposed = "\u{0CB9}\u{0CCA}\u{0CD5}\u{0C97}\u{0CC1}";
        assert!(!a.paradigm("ಹೋಗು").is_empty());
        assert_eq!(a.paradigm(decomposed), a.paradigm("ಹೋಗು"));
        assert!(a.paradigm("ಝಝಝ").is_empty());
    }

    #[test]
    fn every_paradigm_form_analyzes_back_to_its_reading() {
        let a = analyzer();
        for lemma in ["ಮರ", "ಮನೆ", "ಮಾಡು", "ಬರು"] {
            for (word, reading) in a.paradigm(lemma) {
                assert!(a.analyze(&word).contains(&reading), "{word} {reading}");
            }
        }
    }

    #[test]
    fn analyzes_noun_and_verb_forms() {
        let a = analyzer();
        assert_eq!(readings(&a, "ಮರಗಳಲ್ಲಿ"), ["ಮರ+n+pl+loc"]);
        assert_eq!(readings(&a, "ಮನೆಯಲ್ಲಿದೆ"), ["ಮನೆ+n+loc|ಇರು+v+pres.3sn"]);
        assert_eq!(
            readings(&a, "ಮಾಡಿದೆ"),
            [
                "ಮಾಡು+v+cp|ಇರು+v+pres.3sn",
                "ಮಾಡು+v+past.1s",
                "ಮಾಡು+v+past.2s"
            ]
        );
        assert_eq!(readings(&a, "ಬಂದರು"), ["ಬರು+v+past.3p"]);
        assert_eq!(readings(&a, "ಬಂತು"), ["ಬರು+v+past.3sn"]);
        assert_eq!(readings(&a, "ಮರ"), ["ಮರ+n"]);
    }

    #[test]
    fn rejects_unknown_and_misspelled_words() {
        let a = analyzer();
        assert!(!a.accepts("ಮರನನ್ನು"));
        assert!(!a.accepts("ಮನೆವನ್ನು"));
        assert!(!a.accepts("ಕಲ್ಲು"));
    }

    #[test]
    fn guesses_and_stems_unknown_words() {
        let a = analyzer();
        let guesses: Vec<String> = a
            .guess("ಕಲ್ಲುಗಳಲ್ಲಿ")
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            guesses.contains(&"ಕಲ್ಲು+n+pl+loc".to_string()),
            "{guesses:?}"
        );
        assert_eq!(a.stem("ಮರಗಳಿಗೆ"), "ಮರ");
        assert_eq!(a.stem("ಬಂದರು"), "ಬರು");
        assert_eq!(a.stem("ಕಲ್ಲುಗಳಲ್ಲಿ"), "ಕಲ್ಲು");
        assert!(
            a.guess("ಕವನ್ನು").iter().all(|g| g.stem != "ಕ"),
            "1 akshara stems are not guessed"
        );
        assert!(
            a.guess("ಕಜ್ವಿನಿ").iter().all(|g| g.pos != "v"),
            "verbs are not guessed"
        );
    }
}
