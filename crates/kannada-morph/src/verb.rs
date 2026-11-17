// SPDX-FileCopyrightText: 2026 The Open Kannada Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Verb conjugation: forms built on the root, the past stem and the conjunctive participle.

use std::sync::OnceLock;

use crate::form::{Form, extend};
use crate::noun::{CLITICS, Declension, cases, fused_existential, predicates};
use crate::sandhi::{Junction, join};

/// Endings on the root, as in ಮಾಡು + ಉತ್ತಾನೆ = ಮಾಡುತ್ತಾನೆ and ಬರೆ + ಅಲು = ಬರೆಯಲು.
const ROOT_ENDINGS: &[(&str, &str)] = &[
    ("ಇರಿ", "+imp.2p"),
    ("ಉತ್ತಾನೆ", "+npst.3sm"),
    ("ಉತ್ತಾಳೆ", "+npst.3sf"),
    ("ಉತ್ತದೆ", "+npst.3sn"),
    ("ಉತ್ತಾರೆ", "+npst.3p"),
    ("ಉತ್ತವೆ", "+npst.3pn"),
    ("ಉತ್ತೇನೆ", "+npst.1s"),
    ("ಉತ್ತೇವೆ", "+npst.1p"),
    ("ಉತ್ತೀಯೆ", "+npst.2s"),
    ("ಉತ್ತೀಯ", "+npst.2s"),
    ("ಉತ್ತೀರಿ", "+npst.2p"),
    ("ಉತ್ತಾ", "+simul"),
    ("ಉತ್ತ", "+simul"),
    ("ಉತ್ತಲೇ", "+simul+emph"),
    ("ಉವ", "+rel.npst"),
    ("ಉವಾಗ", "+when"),
    ("ಉವಂತೆ", "+sim"),
    ("ಉವಂತಹ", "+like"),
    ("ಉವಷ್ಟು", "+extent"),
    ("ಉವವರೆಗೆ", "+term"),
    ("ಉವುದಿಲ್ಲ", "+npst.neg"),
    ("ಉವಲ್ಲಿ", "+vn+loc"),
    ("ಅಲು", "+inf"),
    ("ಅಲೂ", "+inf+also"),
    ("ಅಲೇ", "+inf+emph"),
    ("ಅಲಿ", "+hort"),
    ("ಅಲಿಕ್ಕೆ", "+inf+dat"),
    ("ಅಲಿಲ್ಲ", "+past.neg"),
    ("ಅಬೇಡ", "+proh.2s"),
    ("ಅಬೇಡಿ", "+proh.2p"),
    ("ಅಬಲ್ಲ", "+abil"),
    ("ಅದ", "+rel.neg"),
    ("ಅದಂತೆ", "+rel.neg+sim"),
    ("ಓಣ", "+hort.1p"),
];

/// Modal endings on the root. Each also takes the fused predicates: ಮಾಡಬೇಕಾಗಿದೆ, ಮಾಡಬಹುದೆಂದು.
const MODALS: &[(&str, &str)] = &[
    ("ಅಬೇಕು", "+deb"),
    ("ಅಬಹುದು", "+perm"),
    ("ಅಬಾರದು", "+proh"),
    ("ಅಬಲ್ಲದು", "+abil"),
];

/// Endings on the past stem, as in ಮಾಡಿದ + ಅನು = ಮಾಡಿದನು and ಬಂದ + ಆಗ = ಬಂದಾಗ.
const PAST_ENDINGS: &[(&str, &str)] = &[
    ("", "+rel.past"),
    ("ಅನು", "+past.3sm"),
    ("ಅಳು", "+past.3sf"),
    ("ಅರು", "+past.3p"),
    ("ಎನು", "+past.1s"),
    ("ಎ", "+past.1s"),
    ("ಎ", "+past.2s"),
    ("ಎವು", "+past.1p"),
    ("ಇರಿ", "+past.2p"),
    ("ಅವು", "+past.3pn"),
    ("ಅರೆ", "+cond"),
    ("ಅರೂ", "+conc"),
    ("ಆಗ", "+when"),
    ("ಅಂತೆ", "+sim"),
    ("ಅಂತಹ", "+like"),
    ("ಅನಂತರ", "+after"),
    ("ಅಲ್ಲಿ", "+cond.loc"),
    ("ಅಷ್ಟು", "+extent"),
];

/// Participle nouns: "the one who did", "what was done". Each takes case endings.
const ROOT_NOUNS: &[(&str, &str, Declension)] = &[
    ("ಉವುದು", "+vn", Declension::This),
    ("ಉವವನು", "+ag.3sm", Declension::Plural),
    ("ಉವವಳು", "+ag.3sf", Declension::Plural),
    ("ಉವವರು", "+ag.3p", Declension::Plural),
];
const PAST_NOUNS: &[(&str, &str, Declension)] = &[
    ("ಉದು", "+vn.past", Declension::This),
    ("ಅದ್ದು", "+vn.past", Declension::This),
    ("ಅವನು", "+ag.past.3sm", Declension::Plural),
    ("ಅವಳು", "+ag.past.3sf", Declension::Plural),
    ("ಅವರು", "+ag.past.3p", Declension::Plural),
];

/// A verb whose past stem or participle does not follow the regular pattern.
pub struct IrregularVerb {
    pub root: &'static str,
    pub past: &'static [&'static str],
    pub participle: &'static [&'static str],
    pub past_3sn: &'static [&'static str],
    /// Other forms with their tags, such as ಇದೆ "is" for ಇರು.
    pub extra: &'static [(&'static str, &'static str)],
    /// Irregular participle nouns that take case endings, such as ಎಂಬುದು of ಎನ್ನು.
    pub nouns: &'static [(&'static str, &'static str, Declension)],
}

const fn irregular(
    root: &'static str,
    past: &'static [&'static str],
    participle: &'static [&'static str],
    past_3sn: &'static [&'static str],
) -> IrregularVerb {
    IrregularVerb {
        root,
        past,
        participle,
        past_3sn,
        extra: &[],
        nouns: &[],
    }
}

/// Common verbs with irregular past stems. Every other root follows [`regular_stems`].
pub const IRREGULAR_VERBS: &[IrregularVerb] = &[
    irregular("ಬರು", &["ಬಂದ"], &["ಬಂದು"], &["ಬಂತು", "ಬಂದಿತು"]),
    irregular("ತರು", &["ತಂದ"], &["ತಂದು"], &["ತಂತು", "ತಂದಿತು"]),
    irregular("ಹೋಗು", &["ಹೋದ"], &["ಹೋಗಿ"], &["ಹೋಯಿತು"]),
    irregular("ಆಗು", &["ಆದ"], &["ಆಗಿ"], &["ಆಯಿತು"]),
    IrregularVerb {
        root: "ಇರು",
        past: &["ಇದ್ದ"],
        participle: &["ಇದ್ದು"],
        past_3sn: &["ಇತ್ತು"],
        extra: &[
            ("ಇದ್ದಾನೆ", "+pres.3sm"),
            ("ಇದ್ದಾಳೆ", "+pres.3sf"),
            ("ಇದೆ", "+pres.3sn"),
            ("ಇದ್ದಾರೆ", "+pres.3p"),
            ("ಇವೆ", "+pres.3pn"),
            ("ಇದ್ದೇನೆ", "+pres.1s"),
            ("ಇದ್ದೇವೆ", "+pres.1p"),
            ("ಇದ್ದೀಯೆ", "+pres.2s"),
            ("ಇದ್ದೀರಿ", "+pres.2p"),
            ("ಇಲ್ಲ", "+neg"),
        ],
        nouns: &[],
    },
    irregular("ಕೊಡು", &["ಕೊಟ್ಟ"], &["ಕೊಟ್ಟು"], &["ಕೊಟ್ಟಿತು"]),
    irregular("ಇಡು", &["ಇಟ್ಟ"], &["ಇಟ್ಟು"], &["ಇಟ್ಟಿತು"]),
    irregular("ಬಿಡು", &["ಬಿಟ್ಟ"], &["ಬಿಟ್ಟು"], &["ಬಿಟ್ಟಿತು"]),
    irregular("ಪಡು", &["ಪಟ್ಟ"], &["ಪಟ್ಟು"], &["ಪಟ್ಟಿತು"]),
    irregular("ಸುಡು", &["ಸುಟ್ಟ"], &["ಸುಟ್ಟು"], &["ಸುಟ್ಟಿತು"]),
    irregular("ನೆಡು", &["ನೆಟ್ಟ"], &["ನೆಟ್ಟು"], &["ನೆಟ್ಟಿತು"]),
    irregular("ತೊಡು", &["ತೊಟ್ಟ"], &["ತೊಟ್ಟು"], &["ತೊಟ್ಟಿತು"]),
    irregular("ಕೆಡು", &["ಕೆಟ್ಟ"], &["ಕೆಟ್ಟು"], &["ಕೆಟ್ಟಿತು"]),
    irregular("ಸಾಯು", &["ಸತ್ತ"], &["ಸತ್ತು"], &["ಸತ್ತಿತು"]),
    irregular("ಅಳು", &["ಅತ್ತ"], &["ಅತ್ತು"], &["ಅತ್ತಿತು"]),
    irregular("ಬೀಳು", &["ಬಿದ್ದ"], &["ಬಿದ್ದು"], &["ಬಿದ್ದಿತು"]),
    irregular("ಏಳು", &["ಎದ್ದ"], &["ಎದ್ದು"], &["ಎದ್ದಿತು"]),
    irregular("ಕೀಳು", &["ಕಿತ್ತ"], &["ಕಿತ್ತು"], &["ಕಿತ್ತಿತು"]),
    irregular("ತಿನ್ನು", &["ತಿಂದ"], &["ತಿಂದು"], &["ತಿಂದಿತು"]),
    irregular("ಉಣ್ಣು", &["ಉಂಡ"], &["ಉಂಡು"], &["ಉಂಡಿತು"]),
    irregular("ಕಾಣು", &["ಕಂಡ"], &["ಕಂಡು"], &["ಕಂಡಿತು"]),
    IrregularVerb {
        root: "ಎನ್ನು",
        past: &["ಎಂದ"],
        participle: &["ಎಂದು"],
        past_3sn: &["ಎಂದಿತು"],
        extra: &[("ಎಂಬ", "+rel.npst")],
        nouns: &[
            ("ಎಂಬುದು", "+vn", Declension::This),
            ("ಎಂಬವನು", "+ag.3sm", Declension::Plural),
            ("ಎಂಬವಳು", "+ag.3sf", Declension::Plural),
            ("ಎಂಬವರು", "+ag.3p", Declension::Plural),
        ],
    },
    irregular("ಅನ್ನು", &["ಅಂದ"], &["ಅಂದು"], &["ಅಂದಿತು"]),
    irregular("ಕೊಳ್ಳು", &["ಕೊಂಡ"], &["ಕೊಂಡು"], &["ಕೊಂಡಿತು"]),
    irregular("ನಿಲ್ಲು", &["ನಿಂತ"], &["ನಿಂತು"], &["ನಿಂತಿತು"]),
    irregular("ಗೆಲ್ಲು", &["ಗೆದ್ದ"], &["ಗೆದ್ದು"], &["ಗೆದ್ದಿತು"]),
    irregular("ಕೊಲ್ಲು", &["ಕೊಂದ"], &["ಕೊಂದು"], &["ಕೊಂದಿತು"]),
    irregular("ಕೂರು", &["ಕೂತ"], &["ಕೂತು"], &["ಕೂತಿತು"]),
    irregular("ನೋಯು", &["ನೊಂದ"], &["ನೊಂದು"], &["ನೊಂದಿತು"]),
    irregular("ಬೇಯು", &["ಬೆಂದ"], &["ಬೆಂದು"], &["ಬೆಂದಿತು"]),
    irregular("ಸಿಗು", &["ಸಿಕ್ಕ", "ಸಿಕ್ಕಿದ"], &["ಸಿಕ್ಕಿ", "ಸಿಕ್ಕು"], &["ಸಿಕ್ಕಿತು"]),
    irregular("ಕಲಿ", &["ಕಲಿತ"], &["ಕಲಿತು"], &["ಕಲಿಯಿತು"]),
    irregular("ಅರಿ", &["ಅರಿತ"], &["ಅರಿತು"], &["ಅರಿಯಿತು"]),
    irregular("ಮರೆ", &["ಮರೆತ"], &["ಮರೆತು"], &["ಮರೆಯಿತು"]),
    irregular("ನಗು", &["ನಕ್ಕ"], &["ನಕ್ಕು"], &["ನಕ್ಕಿತು"]),
    irregular("ಸೋಲು", &["ಸೋತ"], &["ಸೋತು"], &["ಸೋತಿತು"]),
    irregular("ಹೊರು", &["ಹೊತ್ತ"], &["ಹೊತ್ತು"], &["ಹೊತ್ತಿತು"]),
    irregular("ಕೊಯ್ಯು", &["ಕೊಯ್ದ"], &["ಕೊಯ್ದು"], &["ಕೊಯ್ದಿತು"]),
    irregular("ಒಯ್ಯು", &["ಒಯ್ದ"], &["ಒಯ್ದು"], &["ಒಯ್ದಿತು"]),
    irregular("ಕಾಯು", &["ಕಾದ"], &["ಕಾದು"], &["ಕಾಯಿತು"]),
    irregular("ಸಲ್ಲು", &["ಸಂದ"], &["ಸಂದು"], &["ಸಂದಿತು"]),
    irregular("ಮೀಯು", &["ಮಿಂದ"], &["ಮಿಂದು"], &["ಮಿಂದಿತು"]),
];

/// The irregular verb with this root, if there is one.
pub fn irregular_verb(root: &str) -> Option<&'static IrregularVerb> {
    IRREGULAR_VERBS.iter().find(|v| v.root == root)
}

/// Endings of compound verbs that conjugate like an irregular verb, and whether
/// the compound voices its first consonant: ಒಳಗೊಳ್ಳು is ಒಳ + ಕೊಳ್ಳು, ಹೊರಬರು is ಹೊರ + ಬರು.
const COMPOUND_HEADS: &[(&str, &str, bool)] = &[
    ("ಕೊಳ್ಳು", "ಕೊಳ್ಳು", false),
    ("ಗೊಳ್ಳು", "ಕೊಳ್ಳು", true),
    ("ಕೊಡು", "ಕೊಡು", false),
    ("ಗೊಡು", "ಕೊಡು", true),
    ("ಬಿಡು", "ಬಿಡು", false),
    ("ಹೋಗು", "ಹೋಗು", false),
    ("ಬರು", "ಬರು", false),
    ("ತರು", "ತರು", false),
    ("ಪಡು", "ಪಡು", false),
    ("ನಿಲ್ಲು", "ನಿಲ್ಲು", false),
];

/// The stems of an irregular verb, or of a compound ending in one, spelled out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stems {
    pub root: String,
    pub past: Vec<String>,
    pub participle: Vec<String>,
    pub past_3sn: Vec<String>,
    pub extra: Vec<(String, &'static str)>,
    pub nouns: Vec<(String, &'static str, Declension)>,
}

/// The irregular stems of `root`: its own, or those of the irregular verb it ends in.
pub fn irregular_stems(root: &str) -> Option<Stems> {
    if let Some(verb) = irregular_verb(root) {
        return Some(spell(verb, "", false));
    }
    COMPOUND_HEADS.iter().find_map(|&(ending, head, voiced)| {
        let prefix = root.strip_suffix(ending).filter(|p| !p.is_empty())?;
        Some(spell(irregular_verb(head)?, prefix, voiced))
    })
}

fn spell(verb: &IrregularVerb, prefix: &str, voiced: bool) -> Stems {
    let stem = |s: &str| {
        let s = if voiced {
            s.replacen('ಕ', "ಗ", 1)
        } else {
            s.to_string()
        };
        format!("{prefix}{s}")
    };
    let all = |list: &[&str]| list.iter().map(|s| stem(s)).collect();
    Stems {
        root: stem(verb.root),
        past: all(verb.past),
        participle: all(verb.participle),
        past_3sn: all(verb.past_3sn),
        extra: verb.extra.iter().map(|(s, t)| (stem(s), *t)).collect(),
        nouns: verb
            .nouns
            .iter()
            .map(|(s, t, d)| (stem(s), *t, *d))
            .collect(),
    }
}

/// Which auxiliaries a verb's forms take. Auxiliaries are built at a lower depth, so fusion stops.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Depth {
    /// No auxiliaries. The forms of ಇರು, ಎನ್ನು, ಕೊಳ್ಳು and ಬಿಡು when they are fused to another word.
    Base,
    /// Only the participle takes ಇರು. Used for the copula ಆಗು, whose fused forms include ಆಗಿದೆ.
    Copula,
    /// Every auxiliary: perfect, progressive, reflexive, completive, passive. The forms of ordinary verbs.
    Full,
}

/// The junction of a verb root before a vowel: "ು" is dropped, "ಿ" and "ೆ" take a glide.
pub fn root_junction(root: &str) -> Junction {
    if root.ends_with('ು') {
        Junction::Elide
    } else {
        Junction::Glide
    }
}

/// The past stem and participle of a regular root: ಮಾಡು gives ಮಾಡಿದ and ಮಾಡಿ, ಬರೆ gives ಬರೆದ and ಬರೆದು.
pub fn regular_stems(root: &str) -> (String, String) {
    if root.ends_with('ು') {
        let first = |suffix| join(root, suffix, Junction::Elide).remove(0);
        (first("ಇದ"), first("ಇ"))
    } else {
        (format!("{root}ದ"), format!("{root}ದು"))
    }
}

fn participle_junction(participle: &str) -> Junction {
    if participle.ends_with('ಿ') {
        Junction::GlideMergeI
    } else {
        Junction::Elide
    }
}

/// Participle nouns built on `host`, with their cases and, at full depth, the fused predicates.
fn nouns(host: &Form, nouns: &[(&str, &str, Declension)], depth: Depth) -> Vec<Form> {
    let mut forms = Vec::new();
    for (suffix, tags, declension) in nouns {
        for noun in host.then(suffix, tags) {
            if depth == Depth::Full {
                forms.extend(predicates(&noun));
            }
            forms.extend(cases(&noun, *declension));
        }
    }
    forms
}

fn root_forms(root: &str, depth: Depth) -> Vec<Form> {
    let host = Form::stem(root, root_junction(root));
    let mut forms = vec![host.clone().tagged("+imp.2s")];
    forms.extend(extend(std::slice::from_ref(&host), ROOT_ENDINGS));
    // The short plural imperative is standard only for roots in "ು": ಮಾಡಿ, but ಬರೆಯಿರಿ.
    if root.ends_with('ು') {
        forms.extend(host.then("ಇ", "+imp.2p"));
    }
    for negative in host.then_with("ಅದೆ", "+neg.cp", Junction::Both) {
        if depth == Depth::Full {
            let elided = Form {
                next: Junction::Elide,
                ..negative.clone()
            };
            forms.extend(elided.compose("ಇರು", fused_existential()));
        }
        forms.push(negative);
    }
    forms.extend(nouns(&host, ROOT_NOUNS, depth));
    for modal in extend(std::slice::from_ref(&host), MODALS) {
        if depth == Depth::Full {
            forms.extend(predicates(&modal));
        }
        forms.extend(extend(std::slice::from_ref(&modal), CLITICS));
        forms.push(modal);
    }
    if depth == Depth::Full {
        for progressive in host.then("ಉತ್ತ್", "+simul") {
            forms.extend(progressive.compose("ಇರು", existential()));
        }
        for infinitive in host.then("ಅಲು", "+pass") {
            forms.extend(infinitive.compose("ಆಗು", copula()));
        }
        for stem in host.then("ಅಲ್", "+pass") {
            forms.extend(stem.compose("ಪಡು", auxiliary_forms("ಪಡು")));
        }
    }
    forms
}

fn past_forms(past: &str, depth: Depth) -> Vec<Form> {
    let host = Form::stem(past, Junction::Elide);
    let mut forms = extend(std::slice::from_ref(&host), PAST_ENDINGS);
    // The bare stem also ends a sentence as "he did" (ಅವನು ಮಾಡಿದ), but not in a fused auxiliary.
    if depth == Depth::Full {
        forms.push(host.clone().tagged("+past.3sm"));
    }
    forms.extend(nouns(&host, PAST_NOUNS, depth));
    forms
}

fn participle_forms(participle: &str, depth: Depth) -> Vec<Form> {
    let host = Form::stem(participle, participle_junction(participle)).tagged("+cp");
    let mut forms = vec![host.clone()];
    forms.extend(extend(std::slice::from_ref(&host), CLITICS));
    if depth != Depth::Base {
        forms.extend(host.compose("ಇರು", existential()));
    }
    if depth == Depth::Full {
        forms.extend(host.compose("ಕೊಳ್ಳು", auxiliary_forms("ಕೊಳ್ಳು")));
        forms.extend(host.compose("ಬಿಡು", auxiliary_forms("ಬಿಡು")));
    }
    forms
}

fn past_3sn_form(text: &str) -> Form {
    Form {
        text: text.to_string(),
        tags: "+past.3sn".to_string(),
        next: Junction::Elide,
    }
}

/// Every form of an irregular verb, built from its spelled-out stems.
fn irregular_forms(verb: &Stems, depth: Depth) -> Vec<Form> {
    let mut forms = root_forms(&verb.root, depth);
    forms.extend(verb.past.iter().flat_map(|p| past_forms(p, depth)));
    forms.extend(
        verb.participle
            .iter()
            .flat_map(|p| participle_forms(p, depth)),
    );
    forms.extend(verb.past_3sn.iter().map(|t| past_3sn_form(t)));
    forms.extend(verb.extra.iter().map(|(text, tags)| Form {
        text: text.to_string(),
        tags: tags.to_string(),
        next: crate::sandhi::junction_after(text),
    }));
    for (text, tags, declension) in &verb.nouns {
        forms.extend(cases(
            &Form::stem(text, Junction::Elide).tagged(tags),
            *declension,
        ));
    }
    forms
}

/// Every form of a regular verb root.
pub fn regular_forms(root: &str) -> Vec<Form> {
    let (past, participle) = regular_stems(root);
    let mut forms = root_forms(root, Depth::Full);
    forms.extend(past_forms(&past, Depth::Full));
    forms.extend(participle_forms(&participle, Depth::Full));
    forms.extend(Form::stem(root, root_junction(root)).then("ಇತು", "+past.3sn"));
    forms
}

/// Every form of an irregular verb, with all auxiliaries.
pub fn irregular_verb_forms(verb: &Stems) -> Vec<Form> {
    irregular_forms(verb, Depth::Full)
}

/// The forms of an irregular root that are built on the root itself.
pub fn irregular_root_forms(root: &str) -> Vec<Form> {
    root_forms(root, Depth::Full)
}

/// The forms built on a past stem such as ಬಂದ.
pub fn past_stem_forms(past: &str) -> Vec<Form> {
    past_forms(past, Depth::Full)
}

/// The forms built on a conjunctive participle such as ಬಂದು.
pub fn participle_stem_forms(participle: &str) -> Vec<Form> {
    participle_forms(participle, Depth::Full)
}

/// The auxiliaries fused to a participle or an infinitive, with the depth each is built to and
/// whether its command forms are left out. ಕೊಳ್ಳು takes the perfect itself
/// (ಹೇಳಿಕೊಂಡಿದ್ದಾನೆ); the passive ಪಡು has no imperative or hortative (no ಮಾಡಲ್ಪಡು).
const AUXILIARIES: &[(&str, Depth, bool)] = &[
    ("ಕೊಳ್ಳು", Depth::Copula, false),
    ("ಬಿಡು", Depth::Base, false),
    ("ಪಡು", Depth::Copula, true),
];

/// Tags of the commands a passive auxiliary does not take: imperatives, hortatives and
/// ಬೇಡ (`+proh.2s`). "Must not" (`+proh`, ಮಾಡಲ್ಪಡಬಾರದು) is not a command.
fn is_command(tags: &str) -> bool {
    ["+imp", "+hort", "+proh."]
        .iter()
        .any(|t| tags.starts_with(t))
}

fn auxiliary_forms(root: &str) -> &'static [Form] {
    static ALL: OnceLock<Vec<(&'static str, Vec<Form>)>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        AUXILIARIES
            .iter()
            .map(|&(aux, depth, no_commands)| {
                let mut forms = irregular_forms(
                    &irregular_stems(aux).expect("auxiliary is irregular"),
                    depth,
                );
                if no_commands {
                    forms.retain(|f| !is_command(&f.tags));
                }
                (aux, forms)
            })
            .collect()
    });
    all.iter()
        .find(|(aux, _)| *aux == root)
        .map(|(_, forms)| forms.as_slice())
        .expect("an auxiliary")
}

/// The forms of ಆಗು that fuse to a noun or an infinitive: ಆಗಿ, ಆದ, ಆಗಿದೆ.
pub fn copula() -> &'static [Form] {
    static FORMS: OnceLock<Vec<Form>> = OnceLock::new();
    FORMS.get_or_init(|| irregular_forms(&irregular_stems("ಆಗು").expect("ಆಗು"), Depth::Copula))
}

/// The forms of ಇರು that fuse to a participle or a locative: ಇದೆ, ಇದ್ದ, ಇರುವ.
pub fn existential() -> &'static [Form] {
    static FORMS: OnceLock<Vec<Form>> = OnceLock::new();
    FORMS.get_or_init(|| irregular_forms(&irregular_stems("ಇರು").expect("ಇರು"), Depth::Base))
}

/// The forms of ಎನ್ನು that fuse to a quoted word: ಎಂದು, ಎಂಬ, ಎಂದರೆ.
pub fn quotative() -> &'static [Form] {
    static FORMS: OnceLock<Vec<Form>> = OnceLock::new();
    FORMS.get_or_init(|| irregular_forms(&irregular_stems("ಎನ್ನು").expect("ಎನ್ನು"), Depth::Base))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(forms: &[Form], text: &str, tags: &str) -> bool {
        forms.iter().any(|f| f.text == text && f.tags == tags)
    }

    #[test]
    fn regular_u_root() {
        let forms = regular_forms("ಮಾಡು");
        for (text, tags) in [
            ("ಮಾಡು", "+imp.2s"),
            ("ಮಾಡುತ್ತಾನೆ", "+npst.3sm"),
            ("ಮಾಡುತ್ತದೆ", "+npst.3sn"),
            ("ಮಾಡಲು", "+inf"),
            ("ಮಾಡಬೇಕು", "+deb"),
            ("ಮಾಡದೆ", "+neg.cp"),
            ("ಮಾಡುವ", "+rel.npst"),
            ("ಮಾಡುವುದನ್ನು", "+vn+acc"),
            ("ಮಾಡುವುದರಿಂದ", "+vn+ins"),
            ("ಮಾಡಿದ", "+rel.past"),
            ("ಮಾಡಿದನು", "+past.3sm"),
            ("ಮಾಡಿದರು", "+past.3p"),
            ("ಮಾಡಿತು", "+past.3sn"),
            ("ಮಾಡಿದರೆ", "+cond"),
            ("ಮಾಡಿದವರು", "+ag.past.3p"),
            ("ಮಾಡಿ", "+cp"),
            ("ಮಾಡಿಯೂ", "+cp+also"),
            ("ಮಾಡಿದ್ದಾನೆ", "+cp|ಇರು+v+pres.3sm"),
            ("ಮಾಡಿದೆ", "+cp|ಇರು+v+pres.3sn"),
            ("ಮಾಡಿಕೊಂಡು", "+cp|ಕೊಳ್ಳು+v+cp"),
            ("ಮಾಡುತ್ತಿದ್ದಾರೆ", "+simul|ಇರು+v+pres.3p"),
            ("ಮಾಡಲಾಗಿದೆ", "+pass|ಆಗು+v+cp|ಇರು+v+pres.3sn"),
            ("ಮಾಡಲಾಯಿತು", "+pass|ಆಗು+v+past.3sn"),
            ("ಮಾಡಬೇಕಾಗಿದೆ", "+deb|ಆಗು+v+cp|ಇರು+v+pres.3sn"),
            ("ಮಾಡಬೇಕಿದೆ", "+deb|ಇರು+v+pres.3sn"),
            ("ಮಾಡಬಹುದೆಂದು", "+perm|ಎನ್ನು+v+cp"),
            ("ಮಾಡುವುದಾಗಿ", "+vn|ಆಗು+v+cp"),
            ("ಮಾಡಿದವರಾಗಿದ್ದರು", "+ag.past.3p|ಆಗು+v+cp|ಇರು+v+past.3p"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
    }

    #[test]
    fn regular_front_root() {
        let forms = regular_forms("ಬರೆ");
        for (text, tags) in [
            ("ಬರೆಯುತ್ತಾರೆ", "+npst.3p"),
            ("ಬರೆಯಲು", "+inf"),
            ("ಬರೆದ", "+rel.past"),
            ("ಬರೆದರು", "+past.3p"),
            ("ಬರೆದು", "+cp"),
            ("ಬರೆದಿದ್ದಾರೆ", "+cp|ಇರು+v+pres.3p"),
            ("ಬರೆಯಿತು", "+past.3sn"),
            ("ಬರೆಯಲಾಗಿದೆ", "+pass|ಆಗು+v+cp|ಇರು+v+pres.3sn"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
    }

    #[test]
    fn irregular_past_and_participle_stems() {
        let past = past_stem_forms("ಬಂದ");
        assert!(has(&past, "ಬಂದರು", "+past.3p"));
        assert!(has(&past, "ಬಂದಾಗ", "+when"));
        assert!(has(&past, "ಬಂದದ್ದು", "+vn.past"));
        let participle = participle_stem_forms("ಬಂದು");
        assert!(has(&participle, "ಬಂದಿದ್ದಾರೆ", "+cp|ಇರು+v+pres.3p"));
        let root = irregular_root_forms("ಬರು");
        assert!(has(&root, "ಬರುತ್ತಾರೆ", "+npst.3p"));
        assert!(!root.iter().any(|f| f.text == "ಬರಿದ" || f.text == "ಬರಿತು"));
    }

    #[test]
    fn front_irregular_roots_keep_their_root_forms() {
        let forms = irregular_root_forms("ಕಲಿ");
        for (text, tags) in [
            ("ಕಲಿಯುತ್ತಾರೆ", "+npst.3p"),
            ("ಕಲಿಯಲು", "+inf"),
            ("ಕಲಿಯಿರಿ", "+imp.2p"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
        assert!(!forms.iter().any(|f| f.text == "ಕಲಿಯಿ"));
    }

    #[test]
    fn newly_listed_irregular_verbs() {
        for (root, past) in [
            ("ನಗು", "ನಕ್ಕರು"),
            ("ಸೋಲು", "ಸೋತರು"),
            ("ಕೊಯ್ಯು", "ಕೊಯ್ದರು"),
            ("ಕಾಯು", "ಕಾದರು"),
        ] {
            let stems = irregular_stems(root).unwrap();
            let forms = past_stem_forms(&stems.past[0]);
            assert!(has(&forms, past, "+past.3p"), "{root}");
            assert!(!regular_forms(root).is_empty());
        }
    }

    #[test]
    fn able_takes_no_fused_copula() {
        let forms = regular_forms("ಮಾಡು");
        assert!(has(&forms, "ಮಾಡಬಲ್ಲ", "+abil"));
        assert!(!forms.iter().any(|f| f.text == "ಮಾಡಬಲ್ಲಾಗಿ"));
        assert!(has(&forms, "ಮಾಡಬಲ್ಲದೆಂದು", "+abil|ಎನ್ನು+v+cp"));
    }

    #[test]
    fn forms_found_missing_by_the_gold_set() {
        let forms = regular_forms("ಮಾಡು");
        for (text, tags) in [
            ("ಮಾಡಿದ", "+past.3sm"),
            ("ಮಾಡುವಲ್ಲಿ", "+vn+loc"),
            ("ಮಾಡದಂತೆ", "+rel.neg+sim"),
            ("ಮಾಡದಿರುವುದು", "+neg.cp|ಇರು+v+vn"),
            ("ಮಾಡಿಕೊಂಡಿದ್ದಾನೆ", "+cp|ಕೊಳ್ಳು+v+cp|ಇರು+v+pres.3sm"),
            ("ಮಾಡಲ್ಪಟ್ಟಿತು", "+pass|ಪಡು+v+past.3sn"),
            ("ಮಾಡಲ್ಪಟ್ಟಿದೆ", "+pass|ಪಡು+v+cp|ಇರು+v+pres.3sn"),
            ("ಮಾಡದಿದ್ದರೆ", "+neg.cp|ಇರು+v+cond"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
        assert!(has(&forms, "ಮಾಡಲ್ಪಡಬಾರದು", "+pass|ಪಡು+v+proh"));
        let wrong = ["ಮಾಡಲ್ಪಡು", "ಮಾಡಲ್ಪಡೋಣ", "ಮಾಡಲ್ಪಡಬೇಡ", "ಮಾಡದೆಯಿದ್ದರೆ", "ಮಾಡದೆಯಿರುವುದು"];
        assert!(!forms.iter().any(|f| wrong.contains(&f.text.as_str())));
        assert!(
            !copula()
                .iter()
                .any(|f| f.tags == "+past.3sm" && f.text == "ಆದ"),
            "no bare 3sm in fused forms"
        );
    }

    #[test]
    fn compounds_conjugate_like_their_head() {
        let stems = irregular_stems("ಒಳಗೊಳ್ಳು").unwrap();
        assert_eq!(stems.past, ["ಒಳಗೊಂಡ"]);
        assert_eq!(stems.participle, ["ಒಳಗೊಂಡು"]);
        let stems = irregular_stems("ಹೊರಬರು").unwrap();
        assert_eq!(stems.past_3sn, ["ಹೊರಬಂತು", "ಹೊರಬಂದಿತು"]);
        assert!(irregular_stems("ಮಾಡು").is_none());
        assert!(irregular_stems("ಬರು").is_some_and(|s| s.root == "ಬರು"));
    }

    #[test]
    fn irregular_verb_count_matches_the_readme() {
        assert_eq!(IRREGULAR_VERBS.len(), 42);
    }

    #[test]
    fn auxiliary_tables_are_complete() {
        assert!(has(existential(), "ಇದೆ", "+pres.3sn"));
        assert!(has(existential(), "ಇರುವ", "+rel.npst"));
        assert!(has(copula(), "ಆಗಿದೆ", "+cp|ಇರು+v+pres.3sn"));
        assert!(has(quotative(), "ಎಂಬ", "+rel.npst"));
        assert!(has(quotative(), "ಎಂದರೆ", "+cond"));
        assert!(has(quotative(), "ಎಂಬುದನ್ನು", "+vn+acc"));
        for verb in IRREGULAR_VERBS {
            assert!(
                !verb.past.is_empty() && !verb.participle.is_empty(),
                "{}",
                verb.root
            );
        }
    }
}
