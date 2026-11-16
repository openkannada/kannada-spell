//! Noun declension: case endings, plurals, clitics, and the predicates that fuse to a noun.

use std::sync::OnceLock;

use crate::form::{Form, extend};
use crate::sandhi::Junction;
use crate::script::VIRAMA;
use crate::verb::{copula, existential, quotative};

/// How a noun stem takes case endings. Each declension is named after an example.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Declension {
    /// ಮರ: ಮರವನ್ನು, ಮರಕ್ಕೆ, ಮರದ, ಮರದಲ್ಲಿ.
    Tree,
    /// ಹುಡುಗ: ಹುಡುಗನನ್ನು, ಹುಡುಗನಿಗೆ, ಹುಡುಗನ, ಹುಡುಗರು.
    Boy,
    /// ಮನೆ: ಮನೆಯನ್ನು, ಮನೆಗೆ, ಮನೆಯ, ಮನೆಗಳು.
    House,
    /// ಹುಡುಗಿ: like ಮನೆ, with the plural ಹುಡುಗಿಯರು.
    Girl,
    /// ಕಾಡು: ಕಾಡನ್ನು, ಕಾಡಿಗೆ, ಕಾಡಿನ. The final "ು" is dropped.
    Forest,
    /// ವಸ್ತು: ವಸ್ತುವನ್ನು, ವಸ್ತುವಿಗೆ, ವಸ್ತುವಿನ. The final vowel stays.
    Thing,
    /// ಇದು: ಇದನ್ನು, ಇದಕ್ಕೆ, ಇದರ. Pronouns, numerals and verbal nouns.
    This,
    /// ಮರಗಳು, ಅವರು: ಮರಗಳನ್ನು, ಮರಗಳಿಗೆ, ಮರಗಳ.
    Plural,
    /// A loanword ending in a consonant, ಬ್ಯಾಂಕ್: ಬ್ಯಾಂಕ್‌ನಲ್ಲಿ, ಬ್ಯಾಂಕ್‌ಗಳು, or with "ು" added, ಬ್ಯಾಂಕಿನ, ಬ್ಯಾಂಕುಗಳು.
    Loan,
    /// Like [`Declension::Loan`], doubling the consonant before "ು": ಬಸ್, ಬಸ್ಸಿನಲ್ಲಿ, ಬಸ್ಸುಗಳು.
    LoanDoubled,
}

impl Declension {
    fn junction(self) -> Junction {
        match self {
            Declension::Tree
            | Declension::Boy
            | Declension::House
            | Declension::Girl
            | Declension::Thing => Junction::Glide,
            Declension::Forest
            | Declension::This
            | Declension::Plural
            | Declension::Loan
            | Declension::LoanDoubled => Junction::Elide,
        }
    }

    fn plural_suffix(self) -> Option<&'static str> {
        match self {
            Declension::Tree
            | Declension::House
            | Declension::Forest
            | Declension::Thing
            | Declension::Loan
            | Declension::LoanDoubled => Some("ಗಳು"),
            Declension::Boy => Some("ರು"),
            Declension::Girl => Some("ಅರು"),
            Declension::This | Declension::Plural => None,
        }
    }

    /// True when the nominative also takes a formal "ಉ": ಮರವು, ಮನೆಯು, ವಸ್ತುವು.
    fn has_formal_nominative(self) -> bool {
        matches!(
            self,
            Declension::Tree | Declension::House | Declension::Girl | Declension::Thing
        )
    }

    /// The loanword with "ು" added, which declines like ಕಾಡು: ಬ್ಯಾಂಕು, ಬಸ್ಸು.
    fn u_extension(self, stem: &str) -> Option<String> {
        let consonant = stem.strip_suffix(VIRAMA)?.chars().last()?;
        match self {
            Declension::Loan => Some(format!("{}ು", &stem[..stem.len() - VIRAMA.len_utf8()])),
            Declension::LoanDoubled => Some(format!("{stem}{consonant}ು")),
            _ => None,
        }
    }
}

const OBLIQUE_ENDINGS: &[(&str, &str)] = &[
    ("ಅ", "+gen"),
    ("ಅಲ್ಲಿ", "+loc"),
    ("ಅಲ್ಲಿನ", "+loc+attr"),
    ("ಇಂದ", "+ins"),
    ("ಇಂದಾಗಿ", "+ins+caus"),
    ("ಇಂದಲೂ", "+ins+also"),
    ("ಇಂದಲೇ", "+ins+emph"),
    ("ಒಳಗೆ", "+in"),
    ("ಒಂದಿಗೆ", "+soc"),
    ("ಅಂತೆ", "+sim"),
    ("ಅಂತಹ", "+like"),
    ("ಅಷ್ಟು", "+extent"),
    ("ಅವರೆಗೆ", "+term"),
];

const DATIVE_ENDINGS: &[(&str, &str)] =
    &[("ಆಗಿ", "+purp"), ("ಇಂತ", "+comp"), ("ಇಂತಲೂ", "+comp+also")];

pub(crate) const CLITICS: &[(&str, &str)] = &[("ಊ", "+also"), ("ಏ", "+emph")];

const NEGATIVE_COPULA: &[(&str, &str)] = &[
    ("ಅಲ್ಲ", "|ಅಲ್ಲ+v+neg"),
    ("ಅಲ್ಲದ", "|ಅಲ್ಲ+v+rel.neg"),
    ("ಅಲ್ಲದೆ", "|ಅಲ್ಲ+v+neg.cp"),
];

/// The case forms of 1 nominal stem, with clitics: no plural and no fused predicates.
pub fn cases(stem: &Form, declension: Declension) -> Vec<Form> {
    let nom = stem.clone();
    // ಹುಡುಗ takes clitics only through ಹುಡುಗನು: ಹುಡುಗನೂ, never ಹುಡುಗವೂ.
    let mut nominatives = if declension == Declension::Boy {
        Vec::new()
    } else {
        vec![nom.clone()]
    };
    let (acc, dat, oblique) = match declension {
        Declension::Tree => (
            nom.then("ಅನ್ನು", "+acc"),
            nom.then("ಕ್ಕೆ", "+dat"),
            nom.then("ದ್", ""),
        ),
        Declension::Boy => {
            nominatives.extend(nom.then("ನು", "+nom"));
            let oblique = nom.then("ನ್", "");
            (
                extend(&oblique, &[("ಅನ್ನು", "+acc")]),
                extend(&oblique, &[("ಇಗೆ", "+dat")]),
                oblique,
            )
        }
        Declension::House | Declension::Girl => (
            nom.then("ಅನ್ನು", "+acc"),
            nom.then("ಗೆ", "+dat"),
            vec![nom.clone()],
        ),
        Declension::Forest | Declension::Thing => (
            nom.then("ಅನ್ನು", "+acc"),
            nom.then("ಇಗೆ", "+dat"),
            nom.then("ಇನ್", ""),
        ),
        Declension::This => (
            nom.then("ಅನ್ನು", "+acc"),
            nom.then("ಅಕ್ಕೆ", "+dat"),
            nom.then("ಅರ್", ""),
        ),
        Declension::Plural => (
            nom.then("ಅನ್ನು", "+acc"),
            nom.then("ಇಗೆ", "+dat"),
            vec![nom.clone()],
        ),
        Declension::Loan | Declension::LoanDoubled => (
            nom.then("ನ್ನು", "+acc"),
            nom.then("ಗೆ", "+dat"),
            nom.then("ನ್", ""),
        ),
    };
    if declension.has_formal_nominative() {
        nominatives.extend(nom.then("ಉ", "+nom"));
    }
    let oblique_cases = extend(&oblique, OBLIQUE_ENDINGS);
    let dative_cases = extend(&dat, DATIVE_ENDINGS);

    let mut takes_clitics = nominatives;
    takes_clitics.extend(acc);
    takes_clitics.extend(dat);
    takes_clitics.extend(
        dative_cases
            .iter()
            .filter(|f| f.tags.ends_with("+purp"))
            .cloned(),
    );
    takes_clitics.extend(
        oblique_cases
            .iter()
            .filter(|f| f.tags.ends_with("+loc"))
            .cloned(),
    );

    let mut forms = extend(&takes_clitics, CLITICS);
    if declension == Declension::Boy {
        forms.push(nom);
    }
    forms.extend(takes_clitics);
    forms.extend(oblique_cases);
    forms.extend(
        dative_cases
            .into_iter()
            .filter(|f| !f.tags.ends_with("+purp")),
    );
    forms
}

fn number_hosts(stem: &str, declension: Declension) -> Vec<(Form, Declension)> {
    let singular = Form::stem(stem, declension.junction());
    let mut hosts = vec![(singular.clone(), declension)];
    if let Some(suffix) = declension.plural_suffix() {
        hosts.extend(
            singular
                .then(suffix, "+pl")
                .into_iter()
                .map(|plural| (plural, Declension::Plural)),
        );
    }
    hosts
}

/// The case forms of a noun stem in the singular and plural, without fused predicates.
pub fn case_forms(stem: &str, declension: Declension) -> Vec<Form> {
    let mut forms: Vec<Form> = number_hosts(stem, declension)
        .iter()
        .flat_map(|(host, d)| cases(host, *d))
        .collect();
    if let Some(extended) = declension.u_extension(stem) {
        forms.extend(case_forms(&extended, Declension::Forest));
    }
    forms
}

/// The forms of a fused predicate that writers join to a nominal. Rarer ones,
/// such as 2nd person forms (ಮರವಾಗುತ್ತೀರಿ), are written apart and left out.
const FUSED_PREDICATE_TAGS: &[&str] = &[
    "+cp",
    "+cp+also",
    "+cp+emph",
    "+rel.past",
    "+rel.npst",
    "+past.3sn",
    "+past.3p",
    "+past.3sm",
    "+past.3sf",
    "+npst.3sn",
    "+npst.3p",
    "+npst.3pn",
    "+cond",
    "+conc",
    "+when",
    "+sim",
    "+inf",
    "+neg",
    "+neg.cp",
    "+vn",
    "+pres.3sn",
    "+pres.3p",
    "+pres.3pn",
    "+pres.3sm",
    "+pres.3sf",
    "+cp|ಇರು+v+pres.3sn",
    "+cp|ಇರು+v+pres.3p",
    "+cp|ಇರು+v+pres.3pn",
    "+cp|ಇರು+v+pres.3sm",
    "+cp|ಇರು+v+pres.3sf",
    "+cp|ಇರು+v+past.3sn",
    "+cp|ಇರು+v+past.3p",
    "+cp|ಇರು+v+rel.npst",
    "+cp|ಇರು+v+rel.past",
    "+cp|ಇರು+v+cp",
    "+cp|ಇರು+v+cond",
    "+cp|ಇರು+v+neg",
];

/// The forms of `all` whose tags are in [`FUSED_PREDICATE_TAGS`], computed once per auxiliary.
fn fused(cell: &'static OnceLock<Vec<Form>>, all: fn() -> &'static [Form]) -> &'static [Form] {
    cell.get_or_init(|| {
        all()
            .iter()
            .filter(|f| FUSED_PREDICATE_TAGS.contains(&f.tags.as_str()))
            .cloned()
            .collect()
    })
}

fn fused_existential() -> &'static [Form] {
    static EXISTENTIAL: OnceLock<Vec<Form>> = OnceLock::new();
    fused(&EXISTENTIAL, existential)
}

/// The predicates written fused to a nominal: ಆಗು, ಇರು, ಎನ್ನು and ಅಲ್ಲ.
pub fn predicates(host: &Form) -> Vec<Form> {
    static COPULA: OnceLock<Vec<Form>> = OnceLock::new();
    static QUOTATIVE: OnceLock<Vec<Form>> = OnceLock::new();
    let mut forms = host.compose("ಆಗು", fused(&COPULA, copula));
    forms.extend(host.compose("ಇರು", fused_existential()));
    forms.extend(host.compose("ಎನ್ನು", fused(&QUOTATIVE, quotative)));
    forms.extend(extend(std::slice::from_ref(host), NEGATIVE_COPULA));
    forms
}

/// Every form of a noun stem: cases in the singular and plural, and the predicates
/// that fuse to a noun in writing (ಮರವಾಗಿದೆ, ಮರವೆಂದು, ಮನೆಯಲ್ಲಿದೆ, ಮರವಲ್ಲ).
pub fn noun_forms(stem: &str, declension: Declension) -> Vec<Form> {
    if matches!(declension, Declension::Loan | Declension::LoanDoubled) {
        return case_forms(stem, declension);
    }
    let mut forms = Vec::new();
    for (host, host_declension) in number_hosts(stem, declension) {
        let host_cases = cases(&host, host_declension);
        let predicate_hosts: Vec<Form> = if host_declension == Declension::Boy {
            host.then("ನು", "")
        } else {
            vec![host.clone()]
        };
        for predicate_host in &predicate_hosts {
            forms.extend(predicates(predicate_host));
        }
        for locative in host_cases.iter().filter(|f| f.tags.ends_with("+loc")) {
            forms.extend(locative.compose("ಇರು", fused_existential()));
        }
        forms.extend(host_cases);
    }
    forms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(forms: &[Form], text: &str, tags: &str) -> bool {
        forms.iter().any(|f| f.text == text && f.tags == tags)
    }

    #[test]
    fn tree_declension() {
        let forms = noun_forms("ಮರ", Declension::Tree);
        for (text, tags) in [
            ("ಮರವನ್ನು", "+acc"),
            ("ಮರಕ್ಕೆ", "+dat"),
            ("ಮರದ", "+gen"),
            ("ಮರದಲ್ಲಿ", "+loc"),
            ("ಮರದಿಂದ", "+ins"),
            ("ಮರಕ್ಕಾಗಿ", "+dat+purp"),
            ("ಮರಕ್ಕಿಂತ", "+dat+comp"),
            ("ಮರವು", "+nom"),
            ("ಮರವೂ", "+also"),
            ("ಮರದಲ್ಲೂ", "+loc+also"),
            ("ಮರದಲ್ಲಿಯೂ", "+loc+also"),
            ("ಮರಗಳು", "+pl"),
            ("ಮರಗಳನ್ನು", "+pl+acc"),
            ("ಮರಗಳಿಗೆ", "+pl+dat"),
            ("ಮರಗಳ", "+pl+gen"),
            ("ಮರಗಳಲ್ಲಿ", "+pl+loc"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
        assert!(
            !forms
                .iter()
                .any(|f| f.text == "ಮರನನ್ನು" || f.text == "ಮರಗಳಿನ")
        );
    }

    #[test]
    fn boy_declension_adds_n() {
        let forms = noun_forms("ಹುಡುಗ", Declension::Boy);
        for (text, tags) in [
            ("ಹುಡುಗನು", "+nom"),
            ("ಹುಡುಗನನ್ನು", "+acc"),
            ("ಹುಡುಗನಿಗೆ", "+dat"),
            ("ಹುಡುಗನ", "+gen"),
            ("ಹುಡುಗರು", "+pl"),
            ("ಹುಡುಗರಿಗೆ", "+pl+dat"),
            ("ಹುಡುಗನಾಗಿ", "|ಆಗು+v+cp"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
        let wrong = ["ಹುಡುಗವನ್ನು", "ಹುಡುಗವಾಗಿ", "ಹುಡುಗವೂ", "ಹುಡುಗವೇ"];
        assert!(!forms.iter().any(|f| wrong.contains(&f.text.as_str())));
        assert!(has(&forms, "ಹುಡುಗನೂ", "+nom+also"));
        assert!(has(&forms, "ಹುಡುಗ", ""));
    }

    #[test]
    fn house_and_girl() {
        let house = noun_forms("ಮನೆ", Declension::House);
        for text in [
            "ಮನೆಯನ್ನು",
            "ಮನೆಗೆ",
            "ಮನೆಯ",
            "ಮನೆಯಲ್ಲಿ",
            "ಮನೆಯಿಂದ",
            "ಮನೆಗಳು",
            "ಮನೆಗೂ",
            "ಮನೆಗಾಗಿ",
        ] {
            assert!(house.iter().any(|f| f.text == text), "{text}");
        }
        let girl = noun_forms("ಹುಡುಗಿ", Declension::Girl);
        assert!(has(&girl, "ಹುಡುಗಿಯರು", "+pl"));
        assert!(has(&girl, "ಹುಡುಗಿಯರನ್ನು", "+pl+acc"));
    }

    #[test]
    fn forest_drops_u_and_thing_keeps_it() {
        let forest = noun_forms("ಕಾಡು", Declension::Forest);
        for text in ["ಕಾಡನ್ನು", "ಕಾಡಿಗೆ", "ಕಾಡಿನ", "ಕಾಡಿನಲ್ಲಿ", "ಕಾಡುಗಳು", "ಕಾಡಾಗಿ"]
        {
            assert!(forest.iter().any(|f| f.text == text), "{text}");
        }
        let thing = noun_forms("ವಸ್ತು", Declension::Thing);
        for text in ["ವಸ್ತುವನ್ನು", "ವಸ್ತುವಿಗೆ", "ವಸ್ತುವಿನ", "ವಸ್ತುಗಳು", "ವಸ್ತುವಾಗಿ"]
        {
            assert!(thing.iter().any(|f| f.text == text), "{text}");
        }
    }

    #[test]
    fn this_declension_has_r_oblique() {
        let forms = noun_forms("ಇದು", Declension::This);
        for (text, tags) in [
            ("ಇದನ್ನು", "+acc"),
            ("ಇದಕ್ಕೆ", "+dat"),
            ("ಇದರ", "+gen"),
            ("ಇದರಿಂದ", "+ins"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
    }

    #[test]
    fn loanwords_decline_with_and_without_u() {
        let bank = noun_forms("ಬ್ಯಾಂಕ್", Declension::Loan);
        for (text, tags) in [
            ("ಬ್ಯಾಂಕ್ನಲ್ಲಿ", "+loc"),
            ("ಬ್ಯಾಂಕ್ನ", "+gen"),
            ("ಬ್ಯಾಂಕ್ಗೆ", "+dat"),
            ("ಬ್ಯಾಂಕ್ಗಳು", "+pl"),
            ("ಬ್ಯಾಂಕಿನ", "+gen"),
            ("ಬ್ಯಾಂಕುಗಳು", "+pl"),
            ("ಬ್ಯಾಂಕಿಗೆ", "+dat"),
        ] {
            assert!(has(&bank, text, tags), "{text} {tags}");
        }
        let bus = noun_forms("ಬಸ್", Declension::LoanDoubled);
        for text in ["ಬಸ್ಸಿನಲ್ಲಿ", "ಬಸ್ಸುಗಳು", "ಬಸ್ಗಳನ್ನು"]
        {
            assert!(bus.iter().any(|f| f.text == text), "{text}");
        }
        assert!(!bank.iter().any(|f| f.text == "ಬ್ಯಾಂಕ್ಕುಗಳು"));
    }

    #[test]
    fn predicates_fuse_to_the_noun() {
        let forms = noun_forms("ಮರ", Declension::Tree);
        for (text, tags) in [
            ("ಮರವಾಗಿದೆ", "|ಆಗು+v+cp|ಇರು+v+pres.3sn"),
            ("ಮರವೆಂದು", "|ಎನ್ನು+v+cp"),
            ("ಮರವಲ್ಲ", "|ಅಲ್ಲ+v+neg"),
            ("ಮರದಲ್ಲಿದೆ", "+loc|ಇರು+v+pres.3sn"),
            ("ಮರಗಳಾಗಿ", "+pl|ಆಗು+v+cp"),
        ] {
            assert!(has(&forms, text, tags), "{text} {tags}");
        }
    }

    #[test]
    fn rare_predicate_forms_are_not_fused() {
        let forms = noun_forms("ಮರ", Declension::Tree);
        assert!(forms.iter().any(|f| f.text == "ಮರವಾಗುತ್ತದೆ"));
        assert!(
            !forms
                .iter()
                .any(|f| f.text == "ಮರವಾಗುತ್ತೀರಿ" || f.text == "ಮರವಾಗಿದ್ದೇನೆ")
        );
    }
}
