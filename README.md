# kannada-spell

A Kannada spelling dictionary for Hunspell, with a morphological analyzer and stemmer built from the same rules.

Kannada joins case endings, auxiliaries and the copula to the word before them, so 1 noun has hundreds of written forms: ಮರ, ಮರವನ್ನು, ಮರಗಳಲ್ಲಿ, ಮರವಾಗಿದೆ, ಮರದಲ್ಲಿದ್ದ. A word list cannot cover that. This project describes Kannada inflection once, as paradigms with sandhi rules, and generates from that description both a Hunspell affix file and an analyzer that returns the lemma and tags of each reading.

```text
$ kspell analyze ಮರಗಳಲ್ಲಿ ಬಂದಿದ್ದಾರೆ ಮಾಡಲಾಗಿದೆ
ಮರಗಳಲ್ಲಿ      ಮರ+n+pl+loc
ಬಂದಿದ್ದಾರೆ    ಬರು+v+cp|ಇರು+v+pres.3p
ಮಾಡಲಾಗಿದೆ     ಮಾಡು+v+pass|ಆಗು+v+cp|ಇರು+v+pres.3sn
```

## Dictionaries

| File | Words from | Held-out tokens accepted | Misspellings accepted |
| --- | --- | --- | --- |
| `dict/kn_IN` | Alar, a closed-class list, and stems induced from Kannada Wikipedia | 89.1% | 4.1% |
| `dict/kn_IN-alar` | Alar and the closed-class list only | 70.9% | 3.9% |

Both work with Hunspell 1.7 and with anything that loads Hunspell dictionaries, such as LibreOffice. Hunspell loads `kn_IN` in about 0.3 seconds and uses about 40 MB. The `.dic` files are under the ODbL 1.0 and the `.aff` files under the MIT License; see [DATA-LICENSE.md](DATA-LICENSE.md).

```sh
hunspell -d dict/kn_IN file.txt
```

## Results

Measured on every 10th article of a Kannada Wikipedia dump: 2.2 million words, 331,936 distinct word types. No word from these articles is used to build the dictionaries. Some general fixes (suffixes written on numbers, loanwords ending in a virama, month names, compound verbs) were chosen after reading the most often rejected held-out words, so the coverage figures may be slightly optimistic. Run `kspell eval` and `kspell suggest` to reproduce them.

| Measure | `kn_IN` | `kn_IN-alar` |
| --- | --- | --- |
| Held-out tokens accepted | 89.1% | 70.9% |
| Held-out word types seen 5 or more times, accepted | 91.3% | 61.6% |
| Confusion misspellings accepted (ದ for ಧ, ಿ for ೀ, ಳ for ಲ), of 5,000 | 4.1% | 3.9% |
| Typing slips accepted, of 5,000 | 3.6% | 3.4% |
| Confusions: intended word suggested first, in top 5, of 2,000 | 82.2%, 97.2% | 83.8%, 97.5% |
| Typing slips: intended word suggested first, in top 5, of 2,000 | 70.1%, 96.0% | 72.2%, 96.8% |

Hunspell and `kspell analyze` accept exactly the same words: they agree on every one of the 331,936 held-out word types, and CI checks a sample of generated forms and typos on every change (`scripts/agreement.sh`).

Both dictionaries are tested on the same misspellings. Suggestions are scored on the first 2,000 of them that each dictionary rejects. They are made from frequent held-out words that `kn_IN-alar` accepts, with 1 edit each, kept only when the edited spelling occurs nowhere in the corpus. A confusion swaps letters that sound alike. A typing slip deletes, doubles or swaps a character, or replaces a vowel sign, and is kept only when the result is well-formed Kannada script. Some edits produce rare real words that Alar lists (ತಿವಿ, ರಜನ), so part of the accepted share is correct behavior.

### Compared with earlier Kannada dictionaries

The same held-out words and misspellings, checked with Hunspell 1.7.6:

| Dictionary | Tokens accepted | Types seen 5+ times accepted | Confusions accepted | Typing slips accepted |
| --- | --- | --- | --- | --- |
| `kn_IN` (this project) | 89.1% | 91.3% | 4.1% | 3.6% |
| `kn_IN-alar` (this project) | 70.9% | 61.6% | 3.9% | 3.4% |
| Fedora `hunspell-kn` 1.0.3 (2009) | 65.7% | 40.0% | 0.1% | 0.2% |
| [kn-hunspell](https://github.com/mgharish/kn-hunspell) (2013) | 49.4% | 30.1% | 0.2% | 0.7% |

The earlier dictionaries are mostly plain word lists. They accept almost no misspellings because they accept few inflected forms at all: 6 in 10 frequent words are flagged as errors by the 2009 dictionary.

Most rejected held-out words are rare names and loanwords, and genuine misspellings in Wikipedia such as ವಿರುದ್ದ for ವಿರುದ್ಧ and ನಿರ್ಧಿಷ್ಟ for ನಿರ್ದಿಷ್ಟ.

## How it is built

1. **Paradigms.** `crates/kannada-morph` describes 10 noun declensions (ಮರ, ಹುಡುಗ, ಮನೆ, ಹುಡುಗಿ, ಕಾಡು, ವಸ್ತು, ಇದು, plurals, and 2 for loanwords ending in a consonant such as ಬ್ಯಾಂಕ್ and ಬಸ್), regular verbs ending in ು, ಿ and ೆ, 42 irregular verbs and the compounds built on them (ಒಳಗೊಳ್ಳು like ಕೊಳ್ಳು), and the forms that fuse onto a word: the copula ಆಗು, ಇರು, the quotative ಎನ್ನು, ಅಲ್ಲ, the auxiliaries ಕೊಳ್ಳು and ಬಿಡು, and the clitics ಊ and ಏ. Sandhi is 1 function: elision, the ಯ and ವ glides, and the virama.
2. **Rules.** Each class's affix rules are read off its paradigm by inflecting a sample stem and comparing each form with it. The analyzer and the Hunspell file use these same rules.
3. **Stems from Alar.** Alar's single-word headwords become stems. Its part of speech gives noun, verb, adjective or particle. Old Kannada letters (ೞ, ಱ), Old Kannada endings not used in modern text, and headwords damaged by font conversion are left out.
4. **Declension from the corpus.** Alar does not say whether ಕಾಡು declines as ಕಾಡನ್ನು or ಕಾಡುವನ್ನು, or whether ಹುಡುಗ takes ಹುಡುಗನನ್ನು. Each candidate declension is scored by how often its own forms occur in the training split, and the best one wins.
5. **Induced stems** (`kn_IN` only). A name or loanword that Alar lacks is added when 3 or more distinct training words are its forms in 1 noun class. Frequent unexplained words are added whole. A word 1 confusion away from an accepted word 3 or more times as frequent is treated as a misspelling and left out, so ಹಾಗು (for ಹಾಗೂ) is not learned from Wikipedia's own errors.

## Build

```sh
cargo build --release
curl -o data/raw/knwiki-latest-pages-articles.xml.bz2 \
  https://dumps.wikimedia.org/knwiki/latest/knwiki-latest-pages-articles.xml.bz2
# data/raw/alar.yml from https://github.com/alar-dict/data at the commit in DATA-LICENSE.md
bzcat data/raw/knwiki-latest-pages-articles.xml.bz2 | target/release/kspell corpus
target/release/kspell lexicon
target/release/kspell induce
target/release/kspell hunspell
target/release/kspell hunspell --lexicon data/build/lexicon-alar.tsv --out dict/kn_IN-alar
target/release/kspell eval
target/release/kspell suggest
```

Other commands: `kspell analyze`, `kspell stem`, `kspell sample`, `kspell rules`. The tags are described in [docs/tags.md](docs/tags.md).

## License

The code is under the MIT License. The `.dic` files are derived from Alar and licensed under the Open Database License 1.0; see [DATA-LICENSE.md](DATA-LICENSE.md).

Contains information from [Alar](https://alar.ink) by V. Krishna, made available under the ODbL 1.0.
