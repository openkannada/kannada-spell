// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A generated word form and the step that adds 1 suffix to it.

use crate::sandhi::{Junction, join, junction_after};

/// A word form: its spelling, the tags it adds to the lemma, and how it meets a following vowel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Form {
    pub text: String,
    /// Tags such as "+pl+loc". An auxiliary starts a new group: "+cp|ಇರು+v+pres.3sm".
    pub tags: String,
    pub next: Junction,
}

impl Form {
    /// A stem with no tags yet.
    pub fn stem(text: &str, next: Junction) -> Self {
        Self {
            text: text.to_string(),
            tags: String::new(),
            next,
        }
    }

    /// This form followed by `suffix`, adding `tags`. A form ending in "ಿ" allows both junctions after it.
    pub fn then(&self, suffix: &str, tags: &str) -> Vec<Form> {
        join(&self.text, suffix, self.next)
            .into_iter()
            .map(|text| Form {
                next: junction_after(&text),
                tags: format!("{}{tags}", self.tags),
                text,
            })
            .collect()
    }

    /// Like [`Form::then`], with an explicit junction for what follows.
    pub fn then_with(&self, suffix: &str, tags: &str, next: Junction) -> Vec<Form> {
        let mut forms = self.then(suffix, tags);
        for form in &mut forms {
            form.next = next;
        }
        forms
    }

    /// This form followed by every form of an auxiliary verb, as in ಮಾಡಿ + ಇದ್ದಾನೆ = ಮಾಡಿದ್ದಾನೆ.
    pub fn compose(&self, aux_lemma: &str, aux_forms: &[Form]) -> Vec<Form> {
        aux_forms
            .iter()
            .flat_map(|aux| {
                join(&self.text, &aux.text, self.next)
                    .into_iter()
                    .map(move |text| Form {
                        text,
                        tags: format!("{}|{aux_lemma}+v{}", self.tags, aux.tags),
                        next: aux.next,
                    })
            })
            .collect()
    }

    /// The same form with `prefix` put before its tags.
    pub fn tagged(mut self, prefix: &str) -> Self {
        self.tags.insert_str(0, prefix);
        self
    }
}

/// Every form in `hosts` followed by each `(suffix, tags)` ending.
pub fn extend(hosts: &[Form], endings: &[(&str, &str)]) -> Vec<Form> {
    hosts
        .iter()
        .flat_map(|host| {
            endings
                .iter()
                .flat_map(move |(suffix, tags)| host.then(suffix, tags))
        })
        .collect()
}
