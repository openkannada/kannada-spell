# Analysis tags

The analyzer writes each reading of a word as `lemma+pos+tags`:

```text
ಮರಗಳಲ್ಲಿ      ಮರ+n+pl+loc
ಬಂದರು         ಬರು+v+past.3p
ಮಾಡಲಾಗಿದೆ     ಮಾಡು+v+pass|ಆಗು+v+cp|ಇರು+v+pres.3sn
ಮುಖ್ಯವಾಗಿದೆ    ಮುಖ್ಯ+adj|ಆಗು+v+cp|ಇರು+v+pres.3sn
```

The lemma is the dictionary form: the nominative singular for nouns, the root (the 2nd person imperative) for verbs, as Alar lists them. Kannada writes auxiliaries and the copula joined to the word before them. Each joined verb starts a new group after `|`, with its own lemma and tags. A word can have several readings; ಮಾಡಿದೆ is `ಮಾಡು+v+past.1s`, `ಮಾಡು+v+past.2s` and `ಮಾಡು+v+cp|ಇರು+v+pres.3sn`.

## Parts of speech

| Tag | Meaning |
| --- | --- |
| `n` | noun |
| `v` | verb |
| `adj` | adjective |
| `adv` | adverb |
| `pron` | pronoun |
| `num` | numeral |
| `det` | determiner (ಈ, ಆ, ಯಾವ) |
| `postp` | postposition (ಬಗ್ಗೆ, ಮೂಲಕ) |
| `conj` | conjunction |
| `intj` | interjection |
| `abbr` | a Latin letter name used as an initial (ಎಸ್, ಕೆ) |
| `x` | other, or a word added from the corpus without a known part of speech |

## Nouns

| Tag | Meaning | Example |
| --- | --- | --- |
| (none) | nominative singular | ಮರ |
| `nom` | nominative with the formal ending ಉ or ನು | ಮರವು, ಹುಡುಗನು |
| `pl` | plural | ಮರಗಳು, ಹುಡುಗರು |
| `acc` | accusative | ಮರವನ್ನು |
| `dat` | dative | ಮರಕ್ಕೆ, ಮನೆಗೆ |
| `gen` | genitive | ಮರದ |
| `loc` | locative | ಮರದಲ್ಲಿ |
| `ins` | instrumental and ablative | ಮರದಿಂದ |
| `loc+attr` | locative used as a modifier | ಮರದಲ್ಲಿನ |
| `ins+caus` | "because of" | ಮರದಿಂದಾಗಿ |
| `dat+purp` | "for" | ಮರಕ್ಕಾಗಿ |
| `dat+comp` | "than" | ಮರಕ್ಕಿಂತ |
| `in` | "inside" | ಮರದೊಳಗೆ |
| `soc` | "with" | ಮರದೊಂದಿಗೆ |
| `sim` | "like", "as" | ಮರದಂತೆ |
| `like` | "such as" | ಮರದಂತಹ |
| `extent` | "as much as" | ಮರದಷ್ಟು |
| `term` | "up to" | ಮರದವರೆಗೆ |
| `also` | clitic ಊ, "also", "even" | ಮರವೂ |
| `emph` | clitic ಏ, "itself", "only" | ಮರವೇ |

## Verbs

Person, number and gender are written after a dot: `1s`, `1p`, `2s`, `2p`, `3sm` (he), `3sf` (she), `3sn` (it), `3p` (they, people), `3pn` (they, things).

| Tag | Meaning | Example |
| --- | --- | --- |
| `imp.2s`, `imp.2p` | imperative | ಮಾಡು, ಮಾಡಿ |
| `npst.*` | non-past (present and future) | ಮಾಡುತ್ತಾನೆ |
| `npst.neg` | non-past negative | ಮಾಡುವುದಿಲ್ಲ |
| `past.*` | past | ಮಾಡಿದನು, ಮಾಡಿತು |
| `past.neg` | past negative | ಮಾಡಲಿಲ್ಲ |
| `pres.*` | present of ಇರು, "is" | ಇದ್ದಾನೆ, ಇದೆ |
| `neg` | negative of ಇರು and ಅಲ್ಲ | ಇಲ್ಲ, ಅಲ್ಲ |
| `cp` | conjunctive participle, "having done" | ಮಾಡಿ, ಬಂದು |
| `simul` | simultaneous participle, "while doing" | ಮಾಡುತ್ತಾ |
| `neg.cp` | "without doing" | ಮಾಡದೆ |
| `rel.past`, `rel.npst`, `rel.neg` | relative participles, "that did", "that does", "that does not" | ಮಾಡಿದ, ಮಾಡುವ, ಮಾಡದ |
| `inf` | infinitive | ಮಾಡಲು |
| `hort`, `hort.1p` | "let him", "let us" | ಮಾಡಲಿ, ಮಾಡೋಣ |
| `deb` | "must" | ಮಾಡಬೇಕು |
| `perm` | "may" | ಮಾಡಬಹುದು |
| `proh`, `proh.2s`, `proh.2p` | "must not", "do not" | ಮಾಡಬಾರದು, ಮಾಡಬೇಡ |
| `abil` | "can" | ಮಾಡಬಲ್ಲ |
| `cond` | "if" | ಮಾಡಿದರೆ |
| `cond.loc` | "if", "in case" | ಮಾಡಿದಲ್ಲಿ |
| `conc` | "even if" | ಮಾಡಿದರೂ |
| `when` | "when" | ಮಾಡುವಾಗ, ಮಾಡಿದಾಗ |
| `after` | "after" | ಮಾಡಿದನಂತರ |
| `vn`, `vn.past` | verbal noun, "doing", "having done" | ಮಾಡುವುದು, ಮಾಡಿದ್ದು |
| `vn+loc` | "in doing" | ಮಾಡುವಲ್ಲಿ |
| `rel.neg+sim` | "so as not to" | ಮಾಡದಂತೆ |
| `ag.*`, `ag.past.*` | participle noun, "the one who does", "the one who did" | ಮಾಡುವವನು, ಮಾಡಿದವರು |
| `pass` | infinitive before the passive ಆಗು or ಪಡು | ಮಾಡಲಾಯಿತು, ಮಾಡಲ್ಪಟ್ಟಿತು |

Verbal and participle nouns take the noun tags after them: `ಮಾಡು+v+vn+acc` is ಮಾಡುವುದನ್ನು.

## Joined verbs

| Group | Meaning | Example |
| --- | --- | --- |
| `|ಇರು+v+…` after `cp` | perfect | ಮಾಡಿದ್ದಾನೆ, ಮಾಡಿದೆ |
| `|ಇರು+v+…` after `simul` | progressive | ಮಾಡುತ್ತಿದ್ದಾರೆ |
| `|ಇರು+v+…` after a locative | "is in", "was in" | ಮನೆಯಲ್ಲಿದೆ, ಮನೆಯಲ್ಲಿದ್ದ |
| `|ಇರು+v+…` after a bare noun | "there is", "there was", "there is no": `pres.3sn`, `pres.3pn`, `past.3sn` and `neg` only | ಮರವಿದೆ, ಮರವಿಲ್ಲ |
| `|ಆಗು+v+…` | copula "become", "be", and the passive | ಮುಖ್ಯವಾದ, ಮಾಡಲಾಗಿದೆ |
| `|ಎನ್ನು+v+…` | quotative "called", "that" | ಮರವೆಂದು, ಮರವೆಂಬ |
| `|ಅಲ್ಲ+v+…` | "is not" | ಮರವಲ್ಲ |
| `|ಕೊಳ್ಳು+v+…` | reflexive, "for oneself"; it takes the perfect itself | ಮಾಡಿಕೊಂಡರು, ಮಾಡಿಕೊಂಡಿದ್ದಾನೆ |
| `|ಪಡು+v+…` after `pass` | formal passive | ಮಾಡಲ್ಪಟ್ಟಿದೆ |
| `|ಬಿಡು+v+…` | completive | ಮಾಡಿಬಿಟ್ಟ |

`kspell rules` prints the number of rules per class, and every rule in `dict/kn_IN.aff` carries its tags in its `is:` field.
