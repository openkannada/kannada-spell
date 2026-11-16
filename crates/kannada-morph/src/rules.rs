//! Affix rules read off the paradigms: what a class strips from a stem and adds to it.
//!
//! The paradigms in [`crate::noun`] and [`crate::verb`] are the only description
//! of Kannada inflection. A rule is read off by inflecting a sample stem of 1
//! akshara and comparing each form with the stem: ಕು and ಕನ್ನು give "strip ು,
//! add ನ್ನು". The analyzer and the Hunspell export both use these rules, so
//! they accept exactly the same words.

use std::collections::BTreeMap;

use crate::class::{ALL_CLASSES, Class, Final};

/// 1 affix rule: `stem - strip + add`, for stems with one of `finals`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub class: Class,
    pub strip: String,
    pub add: String,
    pub finals: Vec<Final>,
    pub tags: String,
}

impl Rule {
    /// True when this rule leaves the stem as it is: the stem's own analysis.
    pub fn is_identity(&self) -> bool {
        self.strip.is_empty() && self.add.is_empty()
    }
}

/// The rules of every class, in a stable order.
pub fn all_rules() -> Vec<Rule> {
    ALL_CLASSES
        .iter()
        .flat_map(|&class| rules_for(class))
        .collect()
}

/// The rules of 1 class.
pub fn rules_for(class: Class) -> Vec<Rule> {
    let mut grouped: BTreeMap<(String, String, String), Vec<Final>> = BTreeMap::new();
    for (final_char, sample) in class.samples() {
        for form in class.forms(&sample) {
            let (strip, add) = difference(&sample, &form.text);
            let finals = grouped.entry((strip, add, form.tags)).or_default();
            if !finals.contains(&final_char) {
                finals.push(final_char);
            }
        }
    }
    grouped
        .into_iter()
        .map(|((strip, add, tags), mut finals)| {
            finals.sort();
            Rule {
                class,
                strip,
                add,
                finals,
                tags,
            }
        })
        .collect()
}

/// What turns `stem` into `form`: the characters to strip from the end and the ones to add.
fn difference(stem: &str, form: &str) -> (String, String) {
    let common: usize = stem
        .char_indices()
        .zip(form.chars())
        .take_while(|((_, a), b)| a == b)
        .map(|((_, a), _)| a.len_utf8())
        .sum();
    (stem[common..].to_string(), form[common..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noun::Declension;

    fn rule<'a>(rules: &'a [Rule], tags: &str) -> &'a Rule {
        rules
            .iter()
            .find(|r| r.tags == tags)
            .unwrap_or_else(|| panic!("no rule for {tags}"))
    }

    #[test]
    fn forest_rules_strip_the_final_u() {
        let rules = rules_for(Class::Noun(Declension::Forest));
        let acc = rule(&rules, "+acc");
        assert_eq!((acc.strip.as_str(), acc.add.as_str()), ("ು", "ನ್ನು"));
        let plural = rule(&rules, "+pl");
        assert_eq!((plural.strip.as_str(), plural.add.as_str()), ("", "ಗಳು"));
    }

    #[test]
    fn house_rules_cover_every_front_vowel() {
        let rules = rules_for(Class::Noun(Declension::House));
        let acc = rule(&rules, "+acc");
        assert_eq!(acc.add, "ಯನ್ನು");
        assert_eq!(acc.finals.len(), 5);
    }

    #[test]
    fn every_class_has_an_identity_rule_except_fixed_words() {
        for &class in ALL_CLASSES {
            let identity = rules_for(class).iter().any(Rule::is_identity);
            assert_eq!(identity, class != Class::Fixed, "{class:?}");
        }
    }

    #[test]
    fn difference_compares_whole_characters() {
        assert_eq!(difference("ಕು", "ಕಿದ"), ("ು".to_string(), "ಿದ".to_string()));
        assert_eq!(
            difference("ಕ", "ಕವನ್ನು"),
            (String::new(), "ವನ್ನು".to_string())
        );
    }
}
