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
| `dict/kn_IN` | Alar, a closed-class list, and stems induced from Kannada Wikipedia | 89.0% | 3.2% |
| `dict/kn_IN-alar` | Alar and the closed-class list only | 70.9% | 4.0% |

Both work with Hunspell 1.7 and with anything that loads Hunspell dictionaries, such as LibreOffice. Hunspell loads `kn_IN` in about 0.3 seconds and uses about 40 MB.

```sh
hunspell -d dict/kn_IN file.txt
```

## Results

Measured on every 10th article of a Kannada Wikipedia dump (2.2 million words), which is never used to build the dictionary. Run `kspell eval` and `kspell suggest` to reproduce them.

| Measure | `kn_IN` | `kn_IN-alar` |
| --- | --- | --- |
| Held-out tokens accepted | 89.0% | 70.9% |
| Held-out word types seen 5 or more times, accepted | 91.3% | 61.6% |
| Confusion misspellings accepted (ದ for ಧ, ಿ for ೀ, ಳ for ಲ) | 3.2% | 4.0% |
| Random typos accepted (deleted, doubled or swapped character) | 2.3% | 2.3% |
| Confusion misspellings: intended word suggested first, in top 5 | 82.8%, 96.6% | 86.8%, 98.4% |
| Random typos: intended word suggested first, in top 5 | 75.2%, 96.4% | not measured |

Hunspell and `kspell analyze` accept exactly the same words: on all 331,936 held-out word types, they agree on every one.

The misspellings are 5,000 frequent held-out words with 1 edit each, kept only when the edited spelling occurs nowhere in the corpus. Some edits produce rare real words that Alar lists (ತಿವಿ, ರಜನ), so part of the accepted share is correct behavior.

Most rejected held-out words are rare names and loanwords, and genuine misspellings in Wikipedia such as ವಿರುದ್ದ for ವಿರುದ್ಧ and ನಿರ್ಧಿಷ್ಟ for ನಿರ್ದಿಷ್ಟ.

## How it is built

1. **Paradigms.** `crates/kannada-morph` describes 10 noun declensions (ಮರ, ಹುಡುಗ, ಮನೆ, ಹುಡುಗಿ, ಕಾಡು, ವಸ್ತು, ಇದು, plurals, and 2 for loanwords ending in a consonant such as ಬ್ಯಾಂಕ್ and ಬಸ್), regular verbs ending in ು, ಿ and ೆ, 35 irregular verbs and the compounds built on them (ಒಳಗೊಳ್ಳು like ಕೊಳ್ಳು), and the forms that fuse onto a word: the copula ಆಗು, ಇರು, the quotative ಎನ್ನು, ಅಲ್ಲ, the auxiliaries ಕೊಳ್ಳು and ಬಿಡು, and the clitics ಊ and ಏ. Sandhi is 1 function: elision, the ಯ and ವ glides, and the virama.
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

Other commands: `kspell analyze`, `kspell stem`, `kspell rules`. The tags are described in [docs/tags.md](docs/tags.md).

## License

The code is under the MIT License. The `.dic` files are derived from Alar and licensed under the Open Database License 1.0; see [DATA-LICENSE.md](DATA-LICENSE.md).

Contains information from [Alar](https://alar.ink) by V. Krishna, made available under the ODbL 1.0.
