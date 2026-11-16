//! Choosing a stem's declension from the forms attested in a corpus.
//!
//! A dictionary headword such as ಕಾಡು could decline as ಕಾಡನ್ನು or as
//! ಕಾಡುವನ್ನು, and ಹುಡುಗ as ಹುಡುಗನನ್ನು or ಹುಡುಗವನ್ನು. Each candidate declension
//! is scored by how often its own forms (those no other candidate produces)
//! occur in the corpus. The best one wins; a second one is kept when the corpus
//! shows it clearly too.

use std::collections::HashSet;

use crate::class::Class;
use crate::noun::{Declension, case_forms};

/// A second declension is kept when it has at least this many attestations...
const MIN_SECOND: u64 = 5;
/// ...and at least this share of the best declension's attestations.
const SECOND_SHARE: f64 = 0.2;

/// The declensions a dictionary stem may take, the default for each ending first.
const OPEN_DECLENSIONS: &[Declension] = &[
    Declension::Tree,
    Declension::Boy,
    Declension::House,
    Declension::Girl,
    Declension::Forest,
    Declension::Thing,
    Declension::This,
    Declension::Loan,
    Declension::LoanDoubled,
];

/// The declensions whose class allows the stem's ending, the default first.
pub fn candidates(stem: &str) -> Vec<Declension> {
    OPEN_DECLENSIONS
        .iter()
        .copied()
        .filter(|&d| Class::Noun(d).accepts(stem))
        .collect()
}

/// How often each candidate's own forms occur, by `count`.
pub fn evidence(stem: &str, count: &dyn Fn(&str) -> u64) -> Vec<(Declension, u64)> {
    let options = candidates(stem);
    let forms: Vec<HashSet<String>> = options
        .iter()
        .map(|&d| case_forms(stem, d).into_iter().map(|f| f.text).collect())
        .collect();
    options
        .iter()
        .enumerate()
        .map(|(i, &declension)| {
            let own = forms[i].iter().filter(|f| {
                *f != stem
                    && forms
                        .iter()
                        .enumerate()
                        .all(|(j, o)| j == i || !o.contains(*f))
            });
            (declension, own.map(|f| count(f)).sum())
        })
        .collect()
}

/// The declensions to give `stem`: the best attested, or the default with no evidence,
/// plus a clearly attested second one. Empty when the ending allows none.
pub fn choose(stem: &str, count: &dyn Fn(&str) -> u64) -> Vec<Declension> {
    pick(evidence(stem, count))
}

/// Like [`choose`], but only declensions with corpus evidence: for words Alar lists
/// as adjectives, which take noun endings only when they are used as nouns.
pub fn choose_attested(stem: &str, count: &dyn Fn(&str) -> u64) -> Vec<Declension> {
    let scored = evidence(stem, count);
    if scored.iter().all(|&(_, s)| s == 0) {
        return Vec::new();
    }
    pick(scored)
}

fn pick(mut scored: Vec<(Declension, u64)>) -> Vec<Declension> {
    let Some(&(default, _)) = scored.first() else {
        return Vec::new();
    };
    scored.sort_by_key(|s| std::cmp::Reverse(s.1));
    let (best, best_score) = scored[0];
    if best_score == 0 {
        return vec![default];
    }
    let mut chosen = vec![best];
    if let Some(&(second, score)) = scored.get(1)
        && score >= MIN_SECOND
        && score as f64 >= SECOND_SHARE * best_score as f64
    {
        chosen.push(second);
    }
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn corpus(words: &[(&str, u64)]) -> impl Fn(&str) -> u64 {
        let map: HashMap<String, u64> = words.iter().map(|&(w, n)| (w.to_string(), n)).collect();
        move |w| map.get(w).copied().unwrap_or(0)
    }

    #[test]
    fn picks_the_attested_declension() {
        let count = corpus(&[("ಹುಡುಗನನ್ನು", 10), ("ಹುಡುಗನಿಗೆ", 4), ("ಹುಡುಗರು", 30)]);
        assert_eq!(choose("ಹುಡುಗ", &count), [Declension::Boy]);
        let count = corpus(&[("ವಸ್ತುವನ್ನು", 50), ("ವಸ್ತುವಿನ", 20)]);
        assert_eq!(choose("ವಸ್ತು", &count), [Declension::Thing]);
    }

    #[test]
    fn defaults_without_evidence() {
        let none = corpus(&[]);
        assert_eq!(choose("ಮರ", &none), [Declension::Tree]);
        assert_eq!(choose("ಕಾಡು", &none), [Declension::Forest]);
        assert_eq!(choose("ಬ್ಯಾಂಕ್", &none), [Declension::Loan]);
        assert!(choose("ಪಿತೃ", &none).is_empty());
        assert!(choose_attested("ಸುಂದರ", &none).is_empty());
    }

    #[test]
    fn keeps_a_second_declension_only_when_clearly_attested() {
        let count = corpus(&[("ಗುರುವನ್ನು", 40), ("ಗುರುವಿಗೆ", 10), ("ಗುರನ್ನು", 2)]);
        assert_eq!(choose("ಗುರು", &count), [Declension::Thing]);
        let count = corpus(&[("ಹಸುವನ್ನು", 40), ("ಹಸಿನ", 12)]);
        assert_eq!(
            choose("ಹಸು", &count),
            [Declension::Thing, Declension::Forest]
        );
    }
}
