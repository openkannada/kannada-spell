//! Kannada words from a Wikipedia XML dump, split by article into training and test counts.

use std::io::BufRead;

use anyhow::Result;
use kannada_morph::script::words;

use crate::counts::Counts;

/// 1 article in this many goes to the test counts.
pub const TEST_EVERY: usize = 10;

pub struct Split {
    pub train: Counts,
    pub test: Counts,
    pub articles: usize,
}

/// Reads a MediaWiki XML dump. Only articles (namespace 0) count; redirects are skipped.
/// Every [`TEST_EVERY`]th article goes to `test`, the others to `train`.
pub fn split_dump(reader: impl BufRead) -> Result<Split> {
    let mut split = Split {
        train: Counts::default(),
        test: Counts::default(),
        articles: 0,
    };
    let mut page: Vec<String> = Vec::new();
    let (mut article, mut redirect, mut in_text) = (false, false, false);
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim_start();
        if trimmed.starts_with("<page>") {
            page.clear();
            (article, redirect, in_text) = (false, false, false);
        } else if trimmed.starts_with("<ns>") {
            article = trimmed.starts_with("<ns>0</ns>");
        } else if trimmed.starts_with("<redirect") {
            redirect = true;
        } else if trimmed.starts_with("</page>") {
            if article && !redirect {
                split.articles += 1;
                let counts = if split.articles.is_multiple_of(TEST_EVERY) {
                    &mut split.test
                } else {
                    &mut split.train
                };
                for word in page.drain(..) {
                    counts.add(word, 1);
                }
            }
            continue;
        }
        let mut text = line.as_str();
        if let Some(start) = text.find("<text") {
            in_text = true;
            text = text[start..].split_once('>').map_or("", |(_, rest)| rest);
        }
        if !in_text {
            continue;
        }
        if let Some(end) = text.find("</text>") {
            text = &text[..end];
            in_text = false;
        }
        page.extend(words(text));
    }
    Ok(split)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump(pages: &[(&str, bool, &str)]) -> String {
        let mut out = String::from("<mediawiki>\n");
        for (ns, redirect, text) in pages {
            out.push_str(&format!("  <page>\n    <ns>{ns}</ns>\n"));
            if *redirect {
                out.push_str("    <redirect title=\"x\" />\n");
            }
            out.push_str(&format!(
                "    <text bytes=\"1\" xml:space=\"preserve\">{text}</text>\n  </page>\n"
            ));
        }
        out
    }

    #[test]
    fn counts_articles_only_and_holds_out_every_tenth() {
        let mut pages = vec![("0", false, "ಮರ ಮನೆ"); 9];
        pages.push(("0", false, "ಕಾಡು\nಕಾಡು"));
        pages.push(("1", false, "ಚರ್ಚೆ"));
        pages.push(("0", true, "ಮರುನಿರ್ದೇಶನ"));
        let split = split_dump(dump(&pages).as_bytes()).unwrap();
        assert_eq!(split.articles, 10);
        assert_eq!(split.train.get("ಮರ"), 9);
        assert_eq!(split.test.get("ಕಾಡು"), 2);
        assert_eq!(split.train.get("ಚರ್ಚೆ") + split.test.get("ಮರುನಿರ್ದೇಶನ"), 0);
    }
}
