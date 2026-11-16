//! Builds the lexicon from Alar headwords, the closed-class list and corpus evidence.

use kannada_morph::class::Class;
use kannada_morph::classify::{choose, choose_attested};
use kannada_morph::lexicon::{Entry, Lexicon};
use kannada_morph::script::{ANUSVARA, VIRAMA, akshara_count, is_kannada_word, normalize};
use kannada_morph::verb::{IRREGULAR_VERBS, Stems, irregular_stems};

use crate::alar::AlarEntry;
use crate::counts::Counts;

/// An Old Kannada spelling (final virama or anusvara) is kept only when the corpus uses it this often.
const MIN_OLD_FORM: u64 = 3;

/// Why a headword was left out, for the build summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Skip {
    Phrase,
    OldLetters,
    Damaged,
    OldForm,
    NoClass,
}

/// The headword as a dictionary stem, or why it cannot be one.
pub fn headword(raw: &str) -> Result<String, Skip> {
    let word = normalize(raw.trim());
    if word.contains(' ') || !is_kannada_word(&word) {
        return Err(Skip::Phrase);
    }
    if word.contains(['ೞ', 'ಱ']) {
        return Err(Skip::OldLetters);
    }
    // Font conversion damage in Alar puts the virama of an arkavattu on the wrong side.
    if word.contains("್ರ್") {
        return Err(Skip::Damaged);
    }
    Ok(word)
}

/// A word of 1 akshara ending in a consonant or a short vowel: a letter or prefix, not ಹೂ or ಕೈ.
fn is_fragment(word: &str) -> bool {
    akshara_count(word) == 1 && !word.ends_with(['ಾ', 'ೀ', 'ೂ', 'ೇ', 'ೈ', 'ೋ', 'ೌ'])
}

fn is_old_form(word: &str) -> bool {
    word.ends_with([VIRAMA, ANUSVARA])
}

pub fn irregular_entries(verb: &Stems) -> Vec<Entry> {
    let with_lemma = |stem: &str, class: Class, tags: &str| Entry {
        lemma: verb.root.clone(),
        pos: "v".to_string(),
        tags: tags.to_string(),
        ..Entry::new(stem, class)
    };
    let mut entries = vec![Entry::new(&verb.root, Class::VerbRoot)];
    entries.extend(verb.past.iter().map(|p| with_lemma(p, Class::VerbPast, "")));
    entries.extend(
        verb.participle
            .iter()
            .map(|p| with_lemma(p, Class::VerbParticiple, "")),
    );
    entries.extend(
        verb.past_3sn
            .iter()
            .map(|p| with_lemma(p, Class::Fixed, "+past.3sn")),
    );
    entries.extend(
        verb.extra
            .iter()
            .map(|(form, tags)| with_lemma(form, Class::Fixed, tags)),
    );
    entries.extend(
        verb.nouns
            .iter()
            .map(|(form, tags, d)| with_lemma(form, Class::Noun(*d), tags)),
    );
    entries
}

/// The entries for 1 Alar headword.
pub fn entries_for(alar: &AlarEntry, counts: &Counts) -> Result<Vec<Entry>, Skip> {
    let stem = headword(&alar.entry)?;
    let count = |w: &str| counts.get(w);
    if is_old_form(&stem) {
        if counts.get(&stem) < MIN_OLD_FORM {
            return Err(Skip::OldForm);
        }
        // Alar's consonant-final words are Old Kannada (ಮಕ್ಕಳ್), not loanwords: they take no endings.
        return Ok(vec![Entry {
            pos: pos_of(alar).to_string(),
            ..Entry::new(&stem, Class::Fixed)
        }]);
    }
    // Alar lists letters and prefixes as nouns ("ಮ: the 25th consonant", ಕು, ಬೆ); they take no endings.
    if is_fragment(&stem) {
        return Ok(vec![Entry {
            pos: pos_of(alar).to_string(),
            ..Entry::new(&stem, Class::Fixed)
        }]);
    }
    let mut entries = Vec::new();
    if alar.is("noun") {
        entries.extend(
            choose(&stem, &count)
                .into_iter()
                .map(|d| Entry::new(&stem, Class::Noun(d))),
        );
    }
    if alar.is("adjective") {
        if Class::Adjective.accepts(&stem) {
            entries.push(Entry::new(&stem, Class::Adjective));
        }
        entries.extend(
            choose_attested(&stem, &count)
                .into_iter()
                .map(|d| Entry::new(&stem, Class::Noun(d))),
        );
    }
    if alar.is("verb") {
        if let Some(verb) = irregular_stems(&stem) {
            entries.extend(irregular_entries(&verb));
        } else if Class::Verb.accepts(&stem) {
            entries.push(Entry::new(&stem, Class::Verb));
        }
    }
    for (kind, pos) in [
        ("adverb", "adv"),
        ("interjection", "intj"),
        ("conjunction", "conj"),
        ("preposition", "postp"),
    ] {
        if alar.is(kind) {
            let class = if Class::Particle.accepts(&stem) {
                Class::Particle
            } else {
                Class::Fixed
            };
            entries.push(Entry {
                pos: pos.to_string(),
                ..Entry::new(&stem, class)
            });
        }
    }
    if entries.is_empty() {
        let class = if Class::Particle.accepts(&stem) {
            Class::Particle
        } else {
            Class::Fixed
        };
        if alar.defs.is_empty() || alar.is("pronoun") {
            return Err(Skip::NoClass);
        }
        entries.push(Entry {
            pos: pos_of(alar).to_string(),
            ..Entry::new(&stem, class)
        });
    }
    Ok(entries)
}

fn pos_of(alar: &AlarEntry) -> &'static str {
    [
        ("noun", "n"),
        ("verb", "v"),
        ("adjective", "adj"),
        ("adverb", "adv"),
    ]
    .iter()
    .find(|(kind, _)| alar.is(kind))
    .map_or("x", |&(_, pos)| pos)
}

pub struct Built {
    pub lexicon: Lexicon,
    pub headwords: usize,
    pub skipped: Vec<(Skip, usize)>,
}

/// The lexicon: Alar headwords with corpus-chosen classes, the irregular verbs and the closed-class list.
pub fn build(alar: &[AlarEntry], closed: Lexicon, counts: &Counts) -> Built {
    let closed_stems: std::collections::HashSet<String> =
        closed.entries().iter().map(|e| e.stem.clone()).collect();
    let mut lexicon = closed;
    let mut skipped = std::collections::BTreeMap::new();
    let mut headwords = std::collections::HashSet::new();
    for entry in alar {
        match entries_for(entry, counts) {
            Ok(entries) => {
                // The closed-class list is written for this project and wins over Alar's part of speech.
                for e in entries
                    .into_iter()
                    .filter(|e| !closed_stems.contains(&e.stem))
                {
                    headwords.insert(e.stem.clone());
                    lexicon.insert(e);
                }
            }
            Err(skip) => *skipped.entry(skip).or_insert(0) += 1,
        }
    }
    for verb in IRREGULAR_VERBS
        .iter()
        .filter_map(|v| irregular_stems(v.root))
    {
        for e in irregular_entries(&verb) {
            lexicon.insert(e);
        }
    }
    Built {
        lexicon,
        headwords: headwords.len(),
        skipped: skipped.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alar::AlarDef;
    use kannada_morph::Lexicon;
    use kannada_morph::noun::Declension;

    fn alar(word: &str, kinds: &[&str]) -> AlarEntry {
        AlarEntry {
            entry: word.to_string(),
            defs: kinds
                .iter()
                .map(|k| AlarDef {
                    kind: k.to_string(),
                })
                .collect(),
        }
    }

    fn classes(word: &str, kinds: &[&str], counts: &Counts) -> Vec<Class> {
        entries_for(&alar(word, kinds), counts)
            .unwrap()
            .into_iter()
            .map(|e| e.class)
            .collect()
    }

    #[test]
    fn rejects_phrases_old_letters_and_damage() {
        assert_eq!(headword("ಮರ ಕಡಿ"), Err(Skip::Phrase));
        assert_eq!(headword("ಪೊೞ್ತು"), Err(Skip::OldLetters));
        assert_eq!(headword("ಕ್ರ್ಮ"), Err(Skip::Damaged));
        assert_eq!(headword(" ಮರ "), Ok("ಮರ".to_string()));
    }

    #[test]
    fn keeps_old_forms_only_when_used() {
        let mut counts = Counts::default();
        assert_eq!(
            entries_for(&alar("ಆಯ್", &["verb"]), &counts),
            Err(Skip::OldForm)
        );
        counts.add("ಆಯ್".to_string(), 5);
        assert_eq!(classes("ಆಯ್", &["verb"], &counts), [Class::Fixed]);
        counts.add("ಮಕ್ಕಳ್".to_string(), 5);
        counts.add("ಮಕ್ಕಳಿನ".to_string(), 50);
        assert_eq!(classes("ಮಕ್ಕಳ್", &["noun"], &counts), [Class::Fixed]);
    }

    #[test]
    fn lexicon_entries_of_an_irregular_verb_accept_all_its_forms() {
        use kannada_morph::verb::irregular_verb_forms;
        use kannada_morph::{Analyzer, Lexicon};
        for root in ["ಬರು", "ಇರು", "ಎನ್ನು", "ಒಳಗೊಳ್ಳು", "ಕಲಿ"]
        {
            let stems = irregular_stems(root).unwrap();
            let analyzer = Analyzer::new(Lexicon::from_entries(irregular_entries(&stems)));
            let missing: Vec<String> = irregular_verb_forms(&stems)
                .into_iter()
                .map(|f| f.text)
                .filter(|t| !analyzer.accepts(t))
                .collect();
            assert!(
                missing.is_empty(),
                "{root}: {:?}",
                &missing[..missing.len().min(10)]
            );
        }
    }

    #[test]
    fn the_closed_class_list_replaces_alar_entries() {
        let closed = Lexicon::from_entries([Entry {
            pos: "pron".into(),
            ..Entry::new("ಅವನು", Class::Noun(Declension::Plural))
        }]);
        let built = build(
            &[alar("ಅವನು", &["noun"]), alar("ಮರ", &["noun"])],
            closed,
            &Counts::default(),
        );
        let classes: Vec<(String, String)> = built
            .lexicon
            .entries()
            .iter()
            .filter(|e| ["ಅವನು", "ಮರ"].contains(&e.stem.as_str()))
            .map(|e| (e.stem.clone(), e.pos.clone()))
            .collect();
        assert_eq!(
            classes,
            [
                ("ಅವನು".to_string(), "pron".to_string()),
                ("ಮರ".to_string(), "n".to_string())
            ]
        );
    }

    #[test]
    fn single_letters_take_no_endings() {
        assert_eq!(classes("ಮ", &["noun"], &Counts::default()), [Class::Fixed]);
        assert_eq!(classes("ಕು", &["noun"], &Counts::default()), [Class::Fixed]);
        assert_eq!(classes("ಬೆ", &["noun"], &Counts::default()), [Class::Fixed]);
        assert_eq!(
            classes("ಹೂ", &["noun"], &Counts::default()),
            [Class::Noun(Declension::Thing)]
        );
    }

    #[test]
    fn gives_each_part_of_speech_its_class() {
        let counts = Counts::default();
        assert_eq!(
            classes("ಮರ", &["noun"], &counts),
            [Class::Noun(Declension::Tree)]
        );
        assert_eq!(classes("ಮಾಡು", &["verb"], &counts), [Class::Verb]);
        assert_eq!(
            classes("ಸುಂದರ", &["adjective"], &counts),
            [Class::Adjective]
        );
        assert_eq!(classes("ಬೇಗ", &["adverb"], &counts), [Class::Particle]);
        let bar: Vec<Class> = classes("ಬರು", &["verb"], &counts);
        assert!(
            bar.contains(&Class::VerbRoot)
                && bar.contains(&Class::VerbPast)
                && !bar.contains(&Class::Verb)
        );
    }

    #[test]
    fn adjectives_used_as_nouns_also_decline() {
        let mut counts = Counts::default();
        counts.add("ಪ್ರಸಿದ್ಧರು".to_string(), 30);
        let found = classes("ಪ್ರಸಿದ್ಧ", &["adjective"], &counts);
        assert_eq!(found, [Class::Adjective, Class::Noun(Declension::Boy)]);
    }
}
