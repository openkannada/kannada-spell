// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The stems the analyzer and the spelling dictionary know, with their classes.
//!
//! The lexicon file has 1 stem per line, tab-separated: stem, class flag, lemma,
//! part of speech, and tags that every form of the stem carries (often empty).
//! Lines starting with "#" are comments.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::class::Class;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entry {
    pub stem: String,
    pub class: Class,
    pub lemma: String,
    pub pos: String,
    /// Tags put before the tags of each form, such as "+dat" for ನನಗೆ of ನಾನು.
    pub tags: String,
}

impl Entry {
    /// An entry whose lemma is the stem itself and whose part of speech is the class's own.
    pub fn new(stem: &str, class: Class) -> Self {
        Self {
            stem: stem.to_string(),
            class,
            lemma: stem.to_string(),
            pos: class.pos().to_string(),
            tags: String::new(),
        }
    }
}

#[derive(Default)]
pub struct Lexicon {
    by_stem: HashMap<String, Vec<Entry>>,
}

impl Lexicon {
    pub fn from_entries(entries: impl IntoIterator<Item = Entry>) -> Self {
        let mut lexicon = Lexicon::default();
        for entry in entries {
            lexicon.insert(entry);
        }
        lexicon
    }

    pub fn insert(&mut self, entry: Entry) {
        let entries = self.by_stem.entry(entry.stem.clone()).or_default();
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }

    /// The entries for `stem`, empty when it is unknown.
    pub fn get(&self, stem: &str) -> &[Entry] {
        self.by_stem.get(stem).map_or(&[], Vec::as_slice)
    }

    pub fn contains_stem(&self, stem: &str) -> bool {
        self.by_stem.contains_key(stem)
    }

    pub fn len(&self) -> usize {
        self.by_stem.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.by_stem.is_empty()
    }

    /// All entries, sorted by stem and class.
    pub fn entries(&self) -> Vec<&Entry> {
        let mut all: Vec<&Entry> = self.by_stem.values().flatten().collect();
        all.sort();
        all
    }

    /// Parses the tab-separated lexicon format.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lexicon = Lexicon::default();
        for (number, line) in text.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            let [stem, flag, lemma, pos, rest @ ..] = fields.as_slice() else {
                return Err(format!("line {}: expected at least 4 fields", number + 1));
            };
            let class = Class::from_flag(flag)
                .ok_or_else(|| format!("line {}: unknown class {flag}", number + 1))?;
            let tags = rest.first().copied().unwrap_or_default();
            lexicon.insert(Entry {
                stem: stem.to_string(),
                class,
                lemma: lemma.to_string(),
                pos: pos.to_string(),
                tags: tags.to_string(),
            });
        }
        Ok(lexicon)
    }

    /// Writes the tab-separated lexicon format, with `header` lines as comments.
    pub fn to_tsv(&self, header: &[&str]) -> String {
        let mut out = String::new();
        for line in header {
            let _ = writeln!(out, "# {line}");
        }
        for e in self.entries() {
            let _ = writeln!(
                out,
                "{}\t{}\t{}\t{}\t{}",
                e.stem,
                e.class.flag(),
                e.lemma,
                e.pos,
                e.tags
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noun::Declension;

    #[test]
    fn round_trips_through_tsv() {
        let lexicon = Lexicon::from_entries([
            Entry::new("ಮರ", Class::Noun(Declension::Tree)),
            Entry {
                stem: "ನನಗೆ".into(),
                class: Class::Particle,
                lemma: "ನಾನು".into(),
                pos: "pron".into(),
                tags: "+dat".into(),
            },
        ]);
        let text = lexicon.to_tsv(&["test"]);
        assert!(text.starts_with("# test\n"));
        let back = Lexicon::parse(&text).unwrap();
        assert_eq!(back.entries(), lexicon.entries());
    }

    #[test]
    fn rejects_unknown_classes() {
        assert!(Lexicon::parse("ಮರ\tZz\tಮರ\tn\t").is_err());
        assert!(Lexicon::parse("ಮರ\tNa").is_err());
    }
}
