//! Joining a stem or suffix to the next suffix, as Kannada spelling writes it.
//!
//! A suffix that starts with a consonant is written straight after the left
//! part: ಮರ + ಗಳು = ಮರಗಳು. A suffix that starts with a vowel either replaces
//! the final short vowel of the left part (elision: ಕಾಡು + ಅನ್ನು = ಕಾಡನ್ನು) or
//! keeps it and inserts a glide, ಯ after front vowels and ವ after the others
//! (ಮನೆ + ಅನ್ನು = ಮನೆಯನ್ನು, ಮರ + ಅನ್ನು = ಮರವನ್ನು). A left part that ends in
//! a virama takes the vowel directly: ಮರದ್ + ಅಲ್ಲಿ = ಮರದಲ್ಲಿ.

use crate::script::{
    ANUSVARA, VIRAMA, is_consonant, is_front_vowel, is_independent_vowel, vowel_sign,
};

/// How a left part meets a following vowel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Junction {
    /// Keep the final vowel and insert a glide: ಮರ + ಊ = ಮರವೂ.
    Glide,
    /// Drop a final short vowel: ಕಾಡು + ಅನ್ನು = ಕಾಡನ್ನು. Long vowels still glide.
    Elide,
    /// Merge "ಇ" with a following "ಇ", glide before other vowels: ಮಾಡಿ + ಇದೆ = ಮಾಡಿದೆ, ಮಾಡಿ + ಊ = ಮಾಡಿಯೂ.
    GlideMergeI,
    /// Both spellings are standard: ಮರದಲ್ಲಿ + ಊ = ಮರದಲ್ಲೂ or ಮರದಲ್ಲಿಯೂ.
    Both,
}

/// The spellings of `left` followed by `right`.
pub fn join(left: &str, right: &str, junction: Junction) -> Vec<String> {
    let mut rest = right.chars();
    let (Some(vowel), Some(last)) = (rest.next(), left.chars().last()) else {
        return vec![format!("{left}{right}")];
    };
    if !is_independent_vowel(vowel) {
        return vec![format!("{left}{right}")];
    }
    let tail = rest.as_str();
    let attach = |base: &str| {
        let mut out = String::from(base);
        out.extend(vowel_sign(vowel));
        out.push_str(tail);
        out
    };
    let body = &left[..left.len() - last.len_utf8()];
    if last == VIRAMA {
        return vec![attach(body)];
    }
    if last == ANUSVARA {
        return vec![attach(&format!("{body}ಮ"))];
    }

    let glided = attach(&format!(
        "{left}{}",
        if is_front_vowel(last) { 'ಯ' } else { 'ವ' }
    ));
    let short = is_consonant(last) || matches!(last, 'ಿ' | 'ು' | 'ೆ');
    if !short {
        return vec![glided];
    }
    let elided = attach(if is_consonant(last) { left } else { body });
    let merges_i = last == 'ಿ' && matches!(vowel, 'ಇ' | 'ಈ');
    match junction {
        Junction::Glide => vec![glided],
        Junction::Elide => vec![elided],
        Junction::GlideMergeI if merges_i => vec![elided],
        Junction::GlideMergeI => vec![glided],
        Junction::Both if merges_i => vec![elided],
        Junction::Both => vec![elided, glided],
    }
}

/// The junction a suffix ending leaves for the next suffix: final "ಿ" allows both spellings.
pub fn junction_after(suffix_form: &str) -> Junction {
    if suffix_form.ends_with('ಿ') {
        Junction::Both
    } else {
        Junction::Elide
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Junction::*;

    fn one(left: &str, right: &str, junction: Junction) -> String {
        let out = join(left, right, junction);
        assert_eq!(out.len(), 1, "{left} + {right}: {out:?}");
        out[0].clone()
    }

    #[test]
    fn consonant_initial_suffixes_are_appended() {
        assert_eq!(one("ಮರ", "ಗಳು", Glide), "ಮರಗಳು");
        assert_eq!(one("ಮರ", "ಕ್ಕೆ", Glide), "ಮರಕ್ಕೆ");
    }

    #[test]
    fn glides_follow_the_vowel() {
        assert_eq!(one("ಮರ", "ಅನ್ನು", Glide), "ಮರವನ್ನು");
        assert_eq!(one("ಮನೆ", "ಅನ್ನು", Glide), "ಮನೆಯನ್ನು");
        assert_eq!(one("ವಸ್ತು", "ಇಗೆ", Glide), "ವಸ್ತುವಿಗೆ");
        assert_eq!(one("ಹೂ", "ಅನ್ನು", Elide), "ಹೂವನ್ನು");
    }

    #[test]
    fn elision_drops_short_vowels() {
        assert_eq!(one("ಕಾಡು", "ಅನ್ನು", Elide), "ಕಾಡನ್ನು");
        assert_eq!(one("ಮರಕ್ಕೆ", "ಆಗಿ", Elide), "ಮರಕ್ಕಾಗಿ");
        assert_eq!(one("ಮಾಡಿದ", "ಅನು", Elide), "ಮಾಡಿದನು");
    }

    #[test]
    fn virama_takes_the_vowel() {
        assert_eq!(one("ಮರದ್", "ಅಲ್ಲಿ", Glide), "ಮರದಲ್ಲಿ");
        assert_eq!(one("ಮರದ್", "ಅ", Glide), "ಮರದ");
        assert_eq!(one("ಮಾಡುತ್ತ್", "ಆನೆ", Elide), "ಮಾಡುತ್ತಾನೆ");
    }

    #[test]
    fn i_merges_with_i_and_glides_otherwise() {
        assert_eq!(one("ಮಾಡಿ", "ಇದ್ದಾನೆ", GlideMergeI), "ಮಾಡಿದ್ದಾನೆ");
        assert_eq!(one("ಮಾಡಿ", "ಊ", GlideMergeI), "ಮಾಡಿಯೂ");
        assert_eq!(join("ಮರದಲ್ಲಿ", "ಊ", Both), ["ಮರದಲ್ಲೂ", "ಮರದಲ್ಲಿಯೂ"]);
        assert_eq!(join("ಮನೆಯಲ್ಲಿ", "ಇದೆ", Both), ["ಮನೆಯಲ್ಲಿದೆ"]);
    }

    #[test]
    fn junction_after_suffix_depends_on_final_vowel() {
        assert_eq!(junction_after("ದಲ್ಲಿ"), Both);
        assert_eq!(junction_after("ವನ್ನು"), Elide);
    }
}
