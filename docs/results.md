# Results

Every figure here is reproducible with the commands named in each section, after the rebuild in [building.md](building.md). The build uses the Kannada Wikipedia dump of 1 November 2026; a newer dump gives slightly different figures.

## Test set

Every 10th article of the Kannada Wikipedia dump of 1 November 2026 is held out: 2.2 million words (2,201,300 tokens), 331,936 distinct word types. The dictionaries are built from the other articles.

That separation is not perfect, for 2 reasons:

- Some general fixes (suffixes written on numbers, loanwords ending in a virama, month names, compound verbs) were chosen after reading the held-out words rejected most often.
- A few pronouns and irregular plurals (ಮಕ್ಕಳು, ಆತ, ಕೆಲವರು) were added to the closed-class list after the development gold set showed them missing, and that set is drawn from held-out articles.

So the coverage figures may be slightly optimistic.

## Coverage and suggestions

Measured with `kspell eval` and `kspell suggest`; `--dictionary dict/kn_IN,dict/kn_IN-wikipedia` gives the packaged column.

| Measure | Packaged, `kn_IN` with `kn_IN-wikipedia` | `kn_IN` alone |
| --- | --- | --- |
| Held-out tokens accepted | 88.9% | 70.8% |
| Held-out word types seen 5 or more times, accepted | 91.2% | 61.7% |
| Confusion misspellings accepted (ದ for ಧ, ಿ for ೀ, ಳ for ಲ), of 5,000 | 3.7% | 3.6% |
| Typing slips accepted, of 5,000 | 3.6% | 3.4% |
| Confusions: intended word suggested first, in top 5, of 2,000 | 81.7%, 97.5% | 83.4%, 97.7% |
| Typing slips: intended word suggested first, in top 5, of 2,000 | 70.2%, 95.8% | 72.4%, 96.3% |

The LibreOffice and Firefox packages carry both word lists, so the packaged column is what users get.

Hunspell and `kspell analyze` accept exactly the same words: they agree on every one of the 331,936 held-out word types, and CI checks a sample of generated forms and typos on every change (`scripts/agreement.sh`).

Both setups are tested on the same misspellings. They are made from frequent held-out words that `kn_IN` accepts, with 1 edit each, kept only when the edited spelling occurs nowhere in the corpus. A confusion swaps letters that sound alike. A typing slip deletes, doubles or swaps a character, or replaces a vowel sign, and is kept only when the result is well-formed Kannada script. Suggestions are scored on the first 2,000 misspellings that each dictionary rejects. Some edits produce rare real words that Alar lists (ತಿವಿ, ರಜನ), so part of the accepted share is correct behavior.

Hunspell stops looking for suggestions after a time limit, so on a heavily loaded machine the top-5 figures can come out 0.1 to 0.2 points lower.

## Compared with earlier Kannada dictionaries

The same held-out words and misspellings, checked with Hunspell 1.7.6:

| Dictionary | Tokens accepted | Types seen 5+ times accepted | Confusions accepted | Typing slips accepted |
| --- | --- | --- | --- | --- |
| Packaged, `kn_IN` with `kn_IN-wikipedia` (this project) | 88.9% | 91.2% | 3.7% | 3.6% |
| `kn_IN` alone (this project) | 70.8% | 61.7% | 3.6% | 3.4% |
| Fedora `hunspell-kn` 1.0.3 (2009) | 65.7% | 40.0% | 0.2% | 0.1% |
| [kn-hunspell](https://github.com/mgharish/kn-hunspell) (2013) | 49.4% | 30.1% | 0.1% | 0.5% |

The earlier dictionaries are mostly plain word lists. They accept almost no misspellings because they accept few inflected forms at all: 6 in 10 frequent words are flagged as errors by the 2009 dictionary.

Most rejected held-out words are rare names and loanwords, and misspellings in Wikipedia itself such as ವಿರುದ್ದ for ವಿರುದ್ಧ and ನಿರ್ಧಿಷ್ಟ for ನಿರ್ದಿಷ್ಟ.

## Analyzer and stemmer

Scored with `kspell gold` against gold analyses of held-out Wikipedia words. Each set samples word types evenly from 3 frequency bands: the 1,000 most frequent, the next 19,000, and the rest seen 3 or more times. Precision and recall are over readings of word types, not weighted by how often a word occurs.

The gold analyses were machine-annotated from [tags.md](tags.md) alone, without access to the analyzer or the dictionaries. A Kannada reader has not checked them yet.

The sets sample 300 and 200 words. The annotation marks 8 and 4 of them as not words, which leaves the 292 and 196 in the table. Those non-words count against precision when the analyzer accepts them.

| Gold set | Dictionary | Readings precision | Readings recall | Lemma and part of speech, P / R | Stem correct |
| --- | --- | --- | --- | --- | --- |
| Development, 292 words (`data/gold/analyses.tsv`) | packaged | 74.9% | 74.9% | 75.1% / 75.3% | 92.1% |
| Development | `kn_IN` alone | 86.8% | 67.8% | 87.3% / 68.1% | 92.1% |
| Test, 196 words (`data/gold/analyses-test.tsv`), first scoring | packaged | 64.7% | 71.7% | 67.3% / 71.5% | 88.3% |
| Test, current | packaged | 67.5% | 72.6% | 68.2% / 72.5% | 90.3% |
| Test, current | `kn_IN` alone | 76.0% | 64.2% | 76.7% / 63.8% | 89.3% |

The development set was used to find and fix gaps. The test set was not looked at until its first scoring, which is the held-out measure. That first scoring used an earlier version of the scorer, which did not count accepted non-words and cut lemma and part of speech in the wrong place. After it, 3 changes came from the test set: 2 bugs it exposed (Old Kannada words given a loanword declension, adjectives taking every form of the copula) and the ಪಡು passive (ಸ್ವೀಕರಿಸಲ್ಪಟ್ಟಿತು), which appears only in the test set. The current rows reflect those changes, so they are not a clean held-out number.

The stem column uses the guesser for words the dictionary does not know; the other columns do not. `kn_IN` alone gives more precise readings but analyzes fewer words: 138 of the 196 test words, against 184 with both lists. With both lists the analyzer accepts 2 of the 4 test-set non-words (the stray suffix ನಷ್ಟು and ಕಿ from ಕಿ.ಮೀ.), because both occur often enough in Wikipedia to be induced.

Most remaining differences are parts of speech: Alar lists many adjectives as nouns too, and names induced from Wikipedia have no part of speech (`x`) where the gold says noun. The rest are missing names and compounds, and a few constructions the paradigms lack, such as the postposition ಅಡಿ ("under").

## Phonetic keys

Measured with `kspell phonetic-eval`:

| Measure | Narrow | Medium | Broad |
| --- | --- | --- | --- |
| Confusion misspellings that keep the intended word's key, of 5,000 | 57.0% | 94.9% | 95.8% |
| Typing slips that keep the key, of 5,000 | 5.5% | 5.8% | 85.0% |
| Other dictionary stems sharing a stem's key, mean (130,485 stems) | 0.22 | 0.41 | 9.5 |
| Informal Latin spellings with the Kannada word's key, of 60 (`informal.tsv`) | | 96.7% | 98.3% |

- The confusion row is high by design: the confusions are the substitutions the medium key folds. The typing slips are random edits and the fairer test of the broad key.
- `informal.tsv` was written for this project, so it is small and not independent. The ISO 15919 and Baraha spellings of 3,712 dictionary words, made by machine transliteration from the Kannada, all get the Kannada word's key; that shows only that the Latin rules invert standard romanization.
- Some different words share keys. Folding vowel length and doubled consonants merges ಹಾಲು and ಹಲ್ಲು, and ಕಾಲು and ಕಲ್ಲು, even at narrow level; the medium key also merges ಕಟ್ಟು and ಕತ್ತು, and ಹಳ್ಳಿ and ಹಲ್ಲಿ. A search should rank exact matches first, then medium, then broad.
- Informal spellings the keys do not handle: "gyana" for ಜ್ಞಾನ, and "y" for ಐ as in "mysuru". Latin "jn" always reads as ಜ್ಞ, so "rajni" does not find ರಜನಿ.
- Before a sibilant or ಹ, a nasal written ಂ, ನ್ or ಮ್ and typed "n" or "m" gets 1 code at medium and broad level, so "sanskrit" and "samskrutha" both find ಸಂಸ್ಕೃತ.
