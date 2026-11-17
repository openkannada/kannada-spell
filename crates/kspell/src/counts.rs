// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Word counts: loading, saving and looking up a frequency list.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// How often each word occurs in a corpus.
#[derive(Default)]
pub struct Counts {
    map: HashMap<String, u64>,
}

impl Counts {
    pub fn add(&mut self, word: String, n: u64) {
        *self.map.entry(word).or_default() += n;
    }

    pub fn get(&self, word: &str) -> u64 {
        self.map.get(word).copied().unwrap_or(0)
    }

    pub fn contains(&self, word: &str) -> bool {
        self.map.contains_key(word)
    }

    pub fn types(&self) -> usize {
        self.map.len()
    }

    pub fn tokens(&self) -> u64 {
        self.map.values().sum()
    }

    /// Words and counts, most frequent first, ties in code point order.
    pub fn sorted(&self) -> Vec<(&str, u64)> {
        let mut all: Vec<(&str, u64)> = self.map.iter().map(|(w, &n)| (w.as_str(), n)).collect();
        all.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        all
    }

    /// Reads a file of words and counts separated by a tab. Lines starting with "#" are comments.
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let mut counts = Counts::default();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let Some((word, n)) = line.split_once('\t') else {
                continue;
            };
            counts.add(
                word.to_string(),
                n.parse()
                    .with_context(|| format!("bad count in {line:?}"))?,
            );
        }
        Ok(counts)
    }

    pub fn save(&self, path: &Path, header: &str) -> Result<()> {
        let mut out = format!("# {header}\n");
        for (word, n) in self.sorted() {
            out.push_str(&format!("{word}\t{n}\n"));
        }
        fs::write(path, out).with_context(|| format!("writing {}", path.display()))
    }
}
