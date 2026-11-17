// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Headwords and parts of speech from Alar, V. Krishna's Kannada-English dictionary (ODbL 1.0).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// The Alar commit this project is built from.
pub const ALAR_COMMIT: &str = "8651ccf8e92184ca17e234eeb6c947d8d52dd5c4";

#[derive(Deserialize)]
pub struct AlarEntry {
    pub entry: String,
    #[serde(default)]
    pub defs: Vec<AlarDef>,
}

#[derive(Deserialize)]
pub struct AlarDef {
    #[serde(rename = "type", default)]
    pub kind: String,
}

impl AlarEntry {
    /// True when any definition has this part of speech, as Alar writes it ("noun", "verb").
    pub fn is(&self, kind: &str) -> bool {
        self.defs.iter().any(|d| d.kind == kind)
    }
}

pub fn load(path: &Path) -> Result<Vec<AlarEntry>> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    serde_yaml_ng::from_reader(BufReader::new(file)).context("parsing Alar YAML")
}
