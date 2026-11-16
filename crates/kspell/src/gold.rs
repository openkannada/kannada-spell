//! Scores the analyzer against gold analyses. Precision and recall are over readings of
//! word types, each word counting once whatever its frequency.

use std::collections::{BTreeMap, BTreeSet};

use kannada_morph::Analyzer;
use kannada_morph::script::normalize;

/// The gold readings of each word, in file order. Words marked `?` (not a word, or misspelled) have none.
pub struct Gold {
    pub words: Vec<(String, BTreeSet<String>)>,
}

impl Gold {
    /// Parses lines of word, analysis and note separated by tabs; lines starting with "#" are comments.
    pub fn parse(text: &str) -> Self {
        let mut order: Vec<String> = Vec::new();
        let mut readings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for line in text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        {
            let mut fields = line.split('\t');
            let word = normalize(fields.next().unwrap_or_default().trim());
            let analysis = fields.next().unwrap_or_default().trim();
            if !readings.contains_key(&word) {
                order.push(word.clone());
            }
            let set = readings.entry(word).or_default();
            if analysis != "?" && !analysis.is_empty() {
                set.insert(normalize(analysis));
            }
        }
        Self {
            words: order
                .into_iter()
                .map(|w| {
                    let r = readings.remove(&w).unwrap_or_default();
                    (w, r)
                })
                .collect(),
        }
    }
}

/// Precision and recall over a set of items: what was predicted, what was expected, what matched.
#[derive(Default, Clone, Copy)]
pub struct Counts {
    pub predicted: usize,
    pub expected: usize,
    pub matched: usize,
}

impl Counts {
    fn add(&mut self, predicted: &BTreeSet<String>, expected: &BTreeSet<String>) {
        self.predicted += predicted.len();
        self.expected += expected.len();
        self.matched += predicted.intersection(expected).count();
    }

    pub fn precision(self) -> f64 {
        ratio(self.matched, self.predicted)
    }

    pub fn recall(self) -> f64 {
        ratio(self.matched, self.expected)
    }
}

fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 { 0.0 } else { a as f64 / b as f64 }
}

pub struct Score {
    /// Words with gold readings.
    pub words: usize,
    /// Gold words marked `?`, not words, and how many of them the analyzer accepted.
    /// Their readings count as predicted readings that match nothing.
    pub non_words: usize,
    pub non_words_accepted: usize,
    /// Words the analyzer gives at least 1 reading from its lexicon.
    pub analyzed: usize,
    /// Full readings: lemma, part of speech and every tag.
    pub full: Counts,
    /// Lemma and part of speech of each reading.
    pub lemma_pos: Counts,
    /// Words whose stemmer output is the lemma of a gold reading.
    pub stem_correct: usize,
    /// Gold words that disagree, for inspection: word, gold readings, predicted readings.
    pub disagreements: Vec<(String, Vec<String>, Vec<String>)>,
}

/// The lemma and part of speech of a reading: "ಮರ+n+pl+loc" gives "ಮರ+n", and
/// "ವಿಶೇಷ+adj|ಆಗು+v+cp" gives "ವಿಶೇಷ+adj".
fn lemma_pos(reading: &str) -> String {
    let (lemma, rest) = reading.split_once('+').unwrap_or((reading, ""));
    let pos = rest.split(['+', '|']).next().unwrap_or_default();
    format!("{lemma}+{pos}")
}

fn lemma(reading: &str) -> &str {
    reading.split('+').next().unwrap_or(reading)
}

/// Scores `analyzer` on `gold`. With `guess`, unknown words get the guesser's readings.
pub fn score(analyzer: &Analyzer, gold: &Gold, guess: bool) -> Score {
    let mut s = Score {
        words: 0,
        non_words: 0,
        non_words_accepted: 0,
        analyzed: 0,
        full: Counts::default(),
        lemma_pos: Counts::default(),
        stem_correct: 0,
        disagreements: Vec::new(),
    };
    for (word, expected) in &gold.words {
        let mut analyses = analyzer.analyze(word);
        if expected.is_empty() {
            s.non_words += 1;
            s.non_words_accepted += usize::from(!analyses.is_empty());
            let predicted: BTreeSet<String> = analyses.iter().map(ToString::to_string).collect();
            s.full.add(&predicted, expected);
            s.lemma_pos
                .add(&predicted.iter().map(|r| lemma_pos(r)).collect(), expected);
            continue;
        }
        s.words += 1;
        s.analyzed += usize::from(!analyses.is_empty());
        if analyses.is_empty() && guess {
            analyses = analyzer.guess(word);
        }
        let predicted: BTreeSet<String> = analyses.iter().map(ToString::to_string).collect();
        s.full.add(&predicted, expected);
        let to_lemma_pos =
            |set: &BTreeSet<String>| set.iter().map(|r| lemma_pos(r)).collect::<BTreeSet<_>>();
        s.lemma_pos
            .add(&to_lemma_pos(&predicted), &to_lemma_pos(expected));
        let stem = analyzer.stem(word);
        s.stem_correct += usize::from(expected.iter().any(|r| lemma(r) == stem));
        if predicted != *expected {
            s.disagreements.push((
                word.clone(),
                expected.iter().cloned().collect(),
                predicted.into_iter().collect(),
            ));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use kannada_morph::noun::Declension;
    use kannada_morph::{Class, Entry, Lexicon};

    #[test]
    fn scores_readings_lemmas_and_stems() {
        let analyzer = Analyzer::new(Lexicon::from_entries([
            Entry::new("ಮರ", Class::Noun(Declension::Tree)),
            Entry::new("ಮಾಡು", Class::Verb),
        ]));
        let gold = Gold::parse(concat!(
            "# header\n",
            "ಮರಗಳಲ್ಲಿ\tಮರ+n+pl+loc\t\n",
            "ಮಾಡಿದೆ\tಮಾಡು+v+past.1s\t\n",
            "ಯಿಂದ\t?\tsuffix\n",
            "ಮರವನ್ನು\t?\ttreated as a non-word here\n",
            "ಕಲ್ಲು\tಕಲ್ಲು+n\t\n",
            "ಮರವಾಗಿದೆ\tಮರ+n|ಆಗು+v+cp|ಇರು+v+pres.3sn\t\n",
        ));
        assert_eq!(gold.words.len(), 6);
        let s = score(&analyzer, &gold, false);
        assert_eq!((s.words, s.analyzed), (4, 3));
        assert_eq!((s.non_words, s.non_words_accepted), (2, 1));
        assert_eq!(
            (s.full.matched, s.full.expected, s.full.predicted),
            (3, 4, 6)
        );
        assert_eq!(
            (
                s.lemma_pos.matched,
                s.lemma_pos.expected,
                s.lemma_pos.predicted
            ),
            (3, 4, 4)
        );
        assert_eq!(s.stem_correct, 4);
    }

    #[test]
    fn lemma_and_part_of_speech_stop_at_the_first_tag_or_joined_verb() {
        assert_eq!(lemma_pos("ಮರ+n+pl+loc"), "ಮರ+n");
        assert_eq!(lemma_pos("ವಿಶೇಷ+adj|ಆಗು+v+cp"), "ವಿಶೇಷ+adj");
    }
}
