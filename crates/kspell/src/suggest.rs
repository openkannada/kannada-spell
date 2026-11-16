//! Measures Hunspell's suggestions: how often the intended word is offered first, or in the top 5.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub struct SuggestionScore {
    pub tried: usize,
    pub first: usize,
    pub top5: usize,
}

/// Runs `hunspell -a` on the misspelled words and parses its suggestion lines.
fn hunspell_suggestions(dictionary: &Path, words: &[&str]) -> Result<Vec<Vec<String>>> {
    let mut child = Command::new("hunspell")
        .arg("-a")
        .arg("-d")
        .arg(dictionary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("running hunspell; is it installed?")?;
    let mut stdin = child.stdin.take().context("hunspell stdin")?;
    let input: String = words.iter().map(|w| format!("^{w}\n")).collect();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let output = child.wait_with_output()?;
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("writer thread"))??;
    if !output.status.success() {
        bail!("hunspell failed");
    }
    let text = String::from_utf8(output.stdout)?;
    let mut results = Vec::new();
    for block in text
        .split("\n\n")
        .skip_while(|b| b.starts_with("@(#)") && !b.contains('\n'))
    {
        let line = block.lines().find(|l| !l.starts_with("@(#)")).unwrap_or("");
        let suggestions = match line.split_once(": ") {
            Some((_, list)) if line.starts_with('&') => {
                list.split(", ").map(str::to_string).collect()
            }
            _ => Vec::new(),
        };
        if !line.is_empty() {
            results.push(suggestions);
        }
    }
    Ok(results)
}

/// Scores the suggestions for each `(correct, misspelled)` pair.
pub fn score(dictionary: &Path, pairs: &[(String, String)]) -> Result<SuggestionScore> {
    let words: Vec<&str> = pairs.iter().map(|(_, bad)| bad.as_str()).collect();
    let suggestions = hunspell_suggestions(dictionary, &words)?;
    if suggestions.len() != pairs.len() {
        bail!(
            "hunspell answered {} of {} words",
            suggestions.len(),
            pairs.len()
        );
    }
    let mut s = SuggestionScore {
        tried: pairs.len(),
        first: 0,
        top5: 0,
    };
    for ((good, _), offered) in pairs.iter().zip(&suggestions) {
        s.first += usize::from(offered.first() == Some(good));
        s.top5 += usize::from(offered.iter().take(5).any(|o| o == good));
    }
    Ok(s)
}
