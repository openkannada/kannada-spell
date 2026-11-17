// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stems induced from a corpus: names and loanwords that Alar does not list.
//!
//! A stem is induced when at least [`MIN_FORMS`] distinct words in the training
//! counts are its inflected forms in 1 noun class, as ಬೆಂಗಳೂರು, ಬೆಂಗಳೂರಿನ and
//! ಬೆಂಗಳೂರಿನಲ್ಲಿ are for ಬೆಂಗಳೂರು. Candidates are taken greedily, the best
//! supported first, and each word supports only 1 stem. Frequent words that
//! are still unexplained are added whole. A word 1 confusion away from a much
//! more frequent accepted word is treated as a misspelling and left out: ಹಾಗು
//! for ಹಾಗೂ.

use std::collections::HashMap;

use kannada_morph::Analyzer;
use kannada_morph::class::Class;
use kannada_morph::lexicon::Entry;
use kannada_morph::noun::Declension;
use kannada_morph::script::akshara_count;

use crate::counts::Counts;
use crate::misspell::confusions;

/// Words seen fewer times than this are not used as evidence.
pub const MIN_COUNT: u64 = 3;
/// A stem needs this many distinct attested forms.
pub const MIN_FORMS: usize = 3;
/// An unexplained word seen this often is added as it is.
pub const MIN_WHOLE_WORD: u64 = 20;
/// A word is a likely misspelling when a confusable accepted word is this many times more frequent.
pub const MISSPELLING_RATIO: u64 = 3;

const INDUCIBLE: &[Class] = &[
    Class::Noun(Declension::Tree),
    Class::Noun(Declension::Boy),
    Class::Noun(Declension::House),
    Class::Noun(Declension::Girl),
    Class::Noun(Declension::Forest),
    Class::Noun(Declension::Thing),
    Class::Noun(Declension::Loan),
    Class::Noun(Declension::LoanDoubled),
];

pub struct Induced {
    pub stems: Vec<Entry>,
    pub whole_words: Vec<Entry>,
    pub likely_misspellings: usize,
}

/// True when `word` is 1 confusion away from an accepted word that is far more frequent.
fn looks_misspelled(word: &str, analyzer: &Analyzer, counts: &Counts) -> bool {
    let threshold = counts.get(word).max(1) * MISSPELLING_RATIO;
    confusions(word)
        .iter()
        .any(|other| counts.get(other) >= threshold && analyzer.accepts(other))
}

/// A consonant-final loan stem must itself occur in the corpus, and must not be a known
/// word with its "ು" dropped: otherwise ಎಂದು and ನಟರು yield the false stems ಎಂದ್ and ನಟರ್.
fn plausible_loan(stem: &str, class: Class, analyzer: &Analyzer, train: &Counts) -> bool {
    if !matches!(
        class,
        Class::Noun(Declension::Loan | Declension::LoanDoubled)
    ) {
        return true;
    }
    let with_u = format!("{}ು", stem.trim_end_matches(kannada_morph::script::VIRAMA));
    train.get(stem) >= MIN_COUNT && !analyzer.accepts(&with_u)
}

pub fn induce(analyzer: &Analyzer, train: &Counts) -> Induced {
    let mut likely_misspellings = 0;
    let unknown: Vec<(&str, u64)> = train
        .sorted()
        .into_iter()
        .filter(|&(w, n)| n >= MIN_COUNT && !analyzer.accepts(w))
        .filter(|&(w, _)| {
            let misspelled = looks_misspelled(w, analyzer, train);
            likely_misspellings += usize::from(misspelled);
            !misspelled
        })
        .collect();

    let mut support: HashMap<(String, Class), Vec<u32>> = HashMap::new();
    for (id, &(word, _)) in unknown.iter().enumerate() {
        for (stem, rule) in analyzer.splits(word) {
            if !INDUCIBLE.contains(&rule.class) || akshara_count(&stem) < 2 {
                continue;
            }
            let ids = support.entry((stem, rule.class)).or_default();
            if ids.last() != Some(&(id as u32)) {
                ids.push(id as u32);
            }
        }
    }
    let mut candidates: Vec<((String, Class), Vec<u32>)> = support
        .into_iter()
        .filter(|(_, ids)| ids.len() >= MIN_FORMS)
        .collect();
    let weight = |ids: &[u32]| ids.iter().map(|&i| unknown[i as usize].1).sum::<u64>();
    candidates.sort_by(|a, b| {
        (b.1.len(), weight(&b.1))
            .cmp(&(a.1.len(), weight(&a.1)))
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut explained = vec![false; unknown.len()];
    let mut induced = Induced {
        stems: Vec::new(),
        whole_words: Vec::new(),
        likely_misspellings,
    };
    for ((stem, class), ids) in candidates {
        let fresh: Vec<u32> = ids
            .into_iter()
            .filter(|&i| !explained[i as usize])
            .collect();
        // A stem that is already a word, such as the verb form ಮಾಡಬಹುದು, is not a new noun.
        if fresh.len() < MIN_FORMS
            || analyzer.accepts(&stem)
            || !plausible_loan(&stem, class, analyzer, train)
        {
            continue;
        }
        if looks_misspelled(&stem, analyzer, train) {
            induced.likely_misspellings += 1;
            continue;
        }
        for i in fresh {
            explained[i as usize] = true;
        }
        induced.stems.push(Entry::new(&stem, class));
    }
    for (i, &(word, n)) in unknown.iter().enumerate() {
        if explained[i] || n < MIN_WHOLE_WORD {
            continue;
        }
        induced.whole_words.push(Entry {
            pos: "x".to_string(),
            ..Entry::new(word, Class::Fixed)
        });
    }
    induced
}

#[cfg(test)]
mod tests {
    use super::*;
    use kannada_morph::Lexicon;

    fn counts(words: &[(&str, u64)]) -> Counts {
        let mut c = Counts::default();
        for &(w, n) in words {
            c.add(w.to_string(), n);
        }
        c
    }

    #[test]
    fn loan_stems_must_occur_and_not_hide_a_known_word() {
        let analyzer = Analyzer::new(Lexicon::from_entries([Entry::new(
            "ನಟ",
            Class::Noun(Declension::Boy),
        )]));
        let train = counts(&[
            ("ನಟರ್", 9),
            ("ನಟರ್ನ", 9),
            ("ನಟರ್ನಲ್ಲಿ", 9),
            ("ನಟರ್ಗೆ", 9),
            ("ಗಿಗ್ನ", 9),
            ("ಗಿಗ್ನಲ್ಲಿ", 9),
            ("ಗಿಗ್ಗೆ", 9),
            ("ಬ್ಯಾಂಕ್", 9),
            ("ಬ್ಯಾಂಕ್ನ", 9),
            ("ಬ್ಯಾಂಕ್ನಲ್ಲಿ", 9),
        ]);
        let stems: Vec<String> = induce(&analyzer, &train)
            .stems
            .into_iter()
            .map(|e| e.stem)
            .collect();
        assert!(stems.contains(&"ಬ್ಯಾಂಕ್".to_string()), "{stems:?}");
        assert!(!stems.contains(&"ನಟರ್".to_string()), "hides ನಟರು: {stems:?}");
        assert!(
            !stems.contains(&"ಗಿಗ್".to_string()),
            "never written alone: {stems:?}"
        );
    }

    #[test]
    fn does_not_induce_a_stem_that_is_already_a_word() {
        let analyzer = Analyzer::new(Lexicon::from_entries([Entry::new("ಮಾಡು", Class::Verb)]));
        let train = counts(&[("ಮಾಡಬಹುದನ್ನು", 9), ("ಮಾಡಬಹುದಿಗೆ", 9), ("ಮಾಡಬಹುದಿನ", 9)]);
        let induced = induce(&analyzer, &train);
        assert!(
            induced.stems.iter().all(|e| e.stem != "ಮಾಡಬಹುದು"),
            "{:?}",
            induced.stems
        );
    }

    #[test]
    fn induces_a_stem_from_its_forms_and_skips_misspellings() {
        let analyzer = Analyzer::new(Lexicon::from_entries([Entry::new("ಹಾಗೂ", Class::Fixed)]));
        let train = counts(&[
            ("ಬೆಂಗಳೂರು", 50),
            ("ಬೆಂಗಳೂರಿನ", 40),
            ("ಬೆಂಗಳೂರಿನಲ್ಲಿ", 30),
            ("ಹಾಗೂ", 1000),
            ("ಹಾಗು", 100),
            ("ಯುನೈಟೆಡ್", 25),
            ("ಅಪರೂಪ", 3),
        ]);
        let induced = induce(&analyzer, &train);
        let stems: Vec<(&str, Class)> = induced
            .stems
            .iter()
            .map(|e| (e.stem.as_str(), e.class))
            .collect();
        assert_eq!(stems, [("ಬೆಂಗಳೂರು", Class::Noun(Declension::Forest))]);
        let whole: Vec<&str> = induced
            .whole_words
            .iter()
            .map(|e| e.stem.as_str())
            .collect();
        assert_eq!(whole, ["ಯುನೈಟೆಡ್"]);
        assert_eq!(induced.likely_misspellings, 1);
    }
}
