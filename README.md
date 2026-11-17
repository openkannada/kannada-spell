# kannada-spell

A Kannada spelling dictionary for Hunspell, with a morphological analyzer and stemmer built from the same rules, and sound-alike keys for fuzzy search.

Kannada joins case endings, auxiliaries and the copula to the word before them, so a single noun has hundreds of written forms: ಮರ, ಮರವನ್ನು, ಮರಗಳಲ್ಲಿ, ಮರವಾಗಿದೆ, ಮರದಲ್ಲಿದ್ದ. A word list cannot cover that. This project describes Kannada inflection once, as paradigms with sandhi rules, and generates from that description both a Hunspell affix file and an analyzer that returns the lemma and tags of each reading.

```text
$ kspell analyze ಮರಗಳಲ್ಲಿ ಬಂದಿದ್ದಾರೆ ಮಾಡಲಾಗಿದೆ
ಮರಗಳಲ್ಲಿ      ಮರ+n+pl+loc
ಬಂದಿದ್ದಾರೆ    ಬರು+v+cp|ಇರು+v+pres.3p
ಮಾಡಲಾಗಿದೆ     ಮಾಡು+v+pass|ಆಗು+v+cp|ಇರು+v+pres.3sn
```

## Results

| File | Words from | Held-out tokens accepted | Misspellings accepted |
| --- | --- | --- | --- |
| `dict/kn_IN` | Alar, a closed-class list, and stems induced from Kannada Wikipedia | 88.9% | 3.7% |
| `dict/kn_IN-alar` | Alar and the closed-class list only | 70.8% | 3.6% |

Measured on every 10th article of the Kannada Wikipedia dump of 1 November 2026: 2.2 million words, 331,936 distinct word types. The dictionaries are built from the other articles.

That separation is not perfect, for 2 reasons:

- Some general fixes (suffixes written on numbers, loanwords ending in a virama, month names, compound verbs) were chosen after reading the held-out words rejected most often.
- A few pronouns and irregular plurals (ಮಕ್ಕಳು, ಆತ, ಕೆಲವರು) were added to the closed-class list after the development gold set showed them missing, and that set is drawn from held-out articles.

So the coverage figures may be slightly optimistic. Run `kspell eval` and `kspell suggest` to reproduce them; `kspell suggest --dictionary dict/kn_IN-alar` gives the `kn_IN-alar` column. The build below uses the same dated dump; a newer one gives slightly different figures.

| Measure | `kn_IN` | `kn_IN-alar` |
| --- | --- | --- |
| Held-out tokens accepted | 88.9% | 70.8% |
| Held-out word types seen 5 or more times, accepted | 91.2% | 61.7% |
| Confusion misspellings accepted (ದ for ಧ, ಿ for ೀ, ಳ for ಲ), of 5,000 | 3.7% | 3.6% |
| Typing slips accepted, of 5,000 | 3.6% | 3.4% |
| Confusions: intended word suggested first, in top 5, of 2,000 | 81.7%, 97.5% | 83.4%, 97.7% |
| Typing slips: intended word suggested first, in top 5, of 2,000 | 70.2%, 95.8% | 72.4%, 96.3% |

Hunspell and `kspell analyze` accept exactly the same words: they agree on every one of the 331,936 held-out word types, and CI checks a sample of generated forms and typos on every change (`scripts/agreement.sh`).

Both dictionaries are tested on the same misspellings. Suggestions are scored on the first 2,000 of them that each dictionary rejects. They are made from frequent held-out words that `kn_IN-alar` accepts, with 1 edit each, kept only when the edited spelling occurs nowhere in the corpus. A confusion swaps letters that sound alike. A typing slip deletes, doubles or swaps a character, or replaces a vowel sign, and is kept only when the result is well-formed Kannada script. Some edits produce rare real words that Alar lists (ತಿವಿ, ರಜನ), so part of the accepted share is correct behavior.

### Compared with earlier Kannada dictionaries

The same held-out words and misspellings, checked with Hunspell 1.7.6:

| Dictionary | Tokens accepted | Types seen 5+ times accepted | Confusions accepted | Typing slips accepted |
| --- | --- | --- | --- | --- |
| `kn_IN` (this project) | 88.9% | 91.2% | 3.7% | 3.6% |
| `kn_IN-alar` (this project) | 70.8% | 61.7% | 3.6% | 3.4% |
| Fedora `hunspell-kn` 1.0.3 (2009) | 65.7% | 40.0% | 0.2% | 0.1% |
| [kn-hunspell](https://github.com/mgharish/kn-hunspell) (2013) | 49.4% | 30.1% | 0.1% | 0.5% |

The earlier dictionaries are mostly plain word lists. They accept almost no misspellings because they accept few inflected forms at all: 6 in 10 frequent words are flagged as errors by the 2009 dictionary.

Most rejected held-out words are rare names and loanwords, and genuine misspellings in Wikipedia such as ವಿರುದ್ದ for ವಿರುದ್ಧ and ನಿರ್ಧಿಷ್ಟ for ನಿರ್ದಿಷ್ಟ.

## Install

Both dictionaries work with Hunspell 1.7 and with anything that loads Hunspell dictionaries, such as LibreOffice. Hunspell loads `kn_IN` in about 0.3 seconds and uses about 40 MB. The `.dic` files are under the ODbL 1.0 and the `.aff` files under the MIT License; see [DATA-LICENSE.md](DATA-LICENSE.md).

```sh
hunspell -d dict/kn_IN file.txt
```

`kn_IN` is the dictionary this project publishes. The LibreOffice and Firefox packages carry it. `kn_IN-alar` stays in the repository for anyone who wants a word list drawn from Alar alone.

### The kspell command

```sh
cargo install --path crates/kspell      # puts kspell on your PATH, to run it outside the repository
```

### LibreOffice and Firefox

`scripts/package.sh` builds both packages from `dict/kn_IN` into `dist/`, and `scripts/package-check.sh` checks them:

- `kannada-spell-kn_IN-<version>.oxt`, a LibreOffice extension. Install it with Tools > Extension Manager > Add, restart LibreOffice, and set the text language to Kannada (India). Tested with LibreOffice 26.8.
- `kannada-spell-kn_IN-<version>.xpi`, a Firefox dictionary add-on for Firefox 140 and later. Firefox installs only add-ons that Mozilla has signed, so until it is listed on addons.mozilla.org, load it for testing from about:debugging > This Firefox > Load Temporary Add-on.

```sh
scripts/package.sh && scripts/package-check.sh
```

## Analyzer and stemmer

`kspell analyze` returns every reading of a word; `kspell stem` returns its lemma; `kspell paradigm` lists every form of a lemma with its reading. The analyzer strips the same affix rules Hunspell uses, so it reads exactly the words the dictionary accepts. For a word the dictionary does not know, `--guess` proposes readings from the ending alone, for open noun classes only.

Scored against gold analyses of held-out Wikipedia words with `kspell gold`. Each set samples word types evenly from 3 frequency bands: the 1,000 most frequent, the next 19,000, and the rest seen 3 or more times. Precision and recall are therefore over readings of word types, not weighted by how often a word occurs.

The sets sample 300 and 200 words. The annotator marked 8 and 4 of them as not words, which leaves the 292 and 196 in the table. Those non-words count against precision when the analyzer accepts them.

| Gold set | Readings precision | Readings recall | Lemma and part of speech, P / R | Stem correct |
| --- | --- | --- | --- | --- |
| Development, 292 words (`data/gold/analyses.tsv`) | 74.9% | 74.9% | 75.1% / 75.3% | 92.1% |
| Test, 196 words (`data/gold/analyses-test.tsv`), first scoring | 64.7% | 71.7% | 67.3% / 71.5% | 88.3% |
| Test, current | 67.5% | 72.6% | 68.2% / 72.5% | 90.3% |

The development set was used to find and fix gaps. The test set was not looked at until its first scoring, which is the honest measure. That first scoring used an earlier version of the scorer, which did not count accepted non-words and cut lemma and part of speech in the wrong place. After it, 3 changes came from the test set: 2 bugs it exposed (Old Kannada words given a loanword declension, adjectives taking every form of the copula) and the ಪಡು passive (ಸ್ವೀಕರಿಸಲ್ಪಟ್ಟಿತು), which appears only in the test set. The current row reflects those changes, so it is not a clean held-out number.

The stem column uses the guesser for words the dictionary does not know; the other columns do not. With `kn_IN-alar` the readings are more precise (76.0% on the test set) but fewer words are analyzed (138 of 196 against 184). The analyzer with `kn_IN` accepts 2 of the 4 test-set non-words (the stray suffix ನಷ್ಟು and ಕಿ from ಕಿ.ಮೀ.), because both occur often enough in Wikipedia to be induced.

Most remaining differences are parts of speech: Alar lists many adjectives as nouns too, and names induced from Wikipedia have no part of speech (`x`) where the gold says noun. The rest are missing names and compounds, and a few constructions the paradigms lack, such as the postposition ಅಡಿ ("under").

The gold analyses were written by an AI model given only `docs/tags.md`, without access to the analyzer or the dictionaries. A Kannada reader has not checked them yet.

## Phonetic keys

`kannada-phonetic` maps a word to sound-alike keys for fuzzy search, so that spellings a reader would confuse share a key:

| Key | Folds | Example |
| --- | --- | --- |
| Narrow | aspiration, vowel length, doubled consonants, a nasal before a stop written as ಂ or as a letter | ಸಂಬಂಧ and ಸಮ್ಬಂದ: `sambanda` |
| Medium | the narrow folds, plus retroflex and dental, ಳ and ಲ, the 3 sibilants | ನಿರ್ದಿಷ್ಟ and ನಿರ್ಧಿಸ್ಟ: `nirdista` |
| Broad | consonants only, plus a leading vowel | ನಿರ್ದಿಷ್ಟ and "nirdishta": `nrdst` |

Latin input gets the medium and broad keys: "neeru" matches ನೀರು at medium level. The keys also normalize decomposed vowel signs, the nukta and zero-width joiners, and read ಜ್ಞ as "gn", as Kannada speakers say it.

```text
$ kspell phonetic ಸಂಬಂಧ
ಸಂಬಂಧ	sambanda	sambanda	smbnd
$ kspell phonetic --latin neeru
neeru	niru	nr
```

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

`crates/kannada-phonetic/vectors.tsv` holds test vectors for ports to other languages, including nukta, decomposed vowel signs and nasal clusters. `kspell phonetic-vectors` rewrites it, and a test checks it matches the code.

## How it is built

1. **Paradigms.** `crates/kannada-morph` describes 10 noun declensions (ಮರ, ಹುಡುಗ, ಮನೆ, ಹುಡುಗಿ, ಕಾಡು, ವಸ್ತು, ಇದು, plurals, and 2 for loanwords ending in a consonant such as ಬ್ಯಾಂಕ್ and ಬಸ್), regular verbs ending in ು, ಿ and ೆ, 42 irregular verbs and the compounds built on them (ಒಳಗೊಳ್ಳು like ಕೊಳ್ಳು), and the forms that fuse onto a word: the copula ಆಗು, ಇರು, the quotative ಎನ್ನು, ಅಲ್ಲ, the auxiliaries ಕೊಳ್ಳು and ಬಿಡು, and the clitics ಊ and ಏ. Sandhi is 1 function: elision, the ಯ and ವ glides, and the virama.
2. **Rules.** Each class's affix rules are read off its paradigm by inflecting a sample stem and comparing each form with it. The analyzer and the Hunspell file use these same rules.
3. **Stems from Alar.** Alar's single-word headwords become stems. Its part of speech gives noun, verb, adjective or particle. Old Kannada letters (ೞ, ಱ), Old Kannada endings not used in modern text, and headwords damaged by font conversion are left out.
4. **Declension from the corpus.** Alar does not say whether ಕಾಡು declines as ಕಾಡನ್ನು or ಕಾಡುವನ್ನು, or whether ಹುಡುಗ takes ಹುಡುಗನನ್ನು. Each candidate declension is scored by how often its own forms occur in the training split, and the best one wins.
5. **Induced stems** (`kn_IN` only). A name or loanword that Alar lacks is added when 3 or more distinct training words are its forms in 1 noun class. Frequent unexplained words are added whole. A word 1 confusion away from an accepted word 3 or more times as frequent is treated as a misspelling and left out, so ಹಾಗು (for ಹಾಗೂ) is not learned from Wikipedia's own errors.

## Rebuild the dictionaries

```sh
cargo build --release
mkdir -p data/raw
curl -L -o data/raw/knwiki-20261101-pages-articles.xml.bz2 \
  https://dumps.wikimedia.org/knwiki/20261101/knwiki-20261101-pages-articles.xml.bz2
curl -L -o data/raw/alar.yml \
  https://raw.githubusercontent.com/alar-dict/data/8651ccf8e92184ca17e234eeb6c947d8d52dd5c4/alar.yml
bzcat data/raw/knwiki-20261101-pages-articles.xml.bz2 | target/release/kspell corpus
target/release/kspell lexicon
target/release/kspell induce
target/release/kspell hunspell
target/release/kspell hunspell --lexicon data/build/lexicon-alar.tsv --out dict/kn_IN-alar
target/release/kspell eval
target/release/kspell suggest
```

Other commands: `kspell analyze`, `kspell stem`, `kspell paradigm`, `kspell gold`, `kspell phonetic`, `kspell phonetic-eval`, `kspell sample`, `kspell rules`. The tags are described in [docs/tags.md](docs/tags.md).

## Limits

- The coverage figures may be slightly optimistic, because some fixes were chosen after reading held-out words (see Results).
- Accepting inflected forms lets more misspellings through: 3.7% of sound-alike misspellings, against 0.2% for the 2009 dictionary.
- Most held-out words it rejects are rare names and loanwords, besides genuine misspellings in Wikipedia.
- The analyzer's gold analyses were written by an AI model; a Kannada reader has not checked them yet.
- The phonetic keys merge some different words, such as ಹಾಲು and ಹಲ್ಲು, and miss some informal spellings (see Phonetic keys).
- The Firefox add-on is unsigned until it is listed on addons.mozilla.org.

## License

The code is under the MIT License. The `.dic` files are derived from Alar and licensed under the Open Database License 1.0; see [DATA-LICENSE.md](DATA-LICENSE.md).

Contains information from [Alar](https://alar.ink) by V. Krishna, made available under the ODbL 1.0.

Part of [Open Kannada](https://github.com/openkannada), free and open-source tools that make Kannada work properly on computers.
