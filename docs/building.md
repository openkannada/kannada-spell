# How the dictionary is built

## Method

1. **Paradigms.** `crates/kannada-morph` describes 10 noun declensions (ಮರ, ಹುಡುಗ, ಮನೆ, ಹುಡುಗಿ, ಕಾಡು, ವಸ್ತು, ಇದು, plurals, and 2 for loanwords ending in a consonant such as ಬ್ಯಾಂಕ್ and ಬಸ್), regular verbs ending in ು, ಿ and ೆ, 42 irregular verbs and the compounds built on them (ಒಳಗೊಳ್ಳು like ಕೊಳ್ಳು), and the forms that fuse onto a word: the copula ಆಗು, ಇರು, the quotative ಎನ್ನು, ಅಲ್ಲ, the auxiliaries ಕೊಳ್ಳು and ಬಿಡು, and the clitics ಊ and ಏ. Sandhi is 1 function: elision, the ಯ and ವ glides, and the virama.
2. **Rules.** Each class's affix rules are read off its paradigm by inflecting a sample stem and comparing each form with it. The analyzer and the Hunspell file use these same rules.
3. **Stems from Alar.** Alar's single-word headwords become stems. Its part of speech gives noun, verb, adjective or particle. Old Kannada letters (ೞ, ಱ), Old Kannada endings not used in modern text, and headwords damaged by font conversion are left out.
4. **Declension from the corpus.** Alar does not say whether ಕಾಡು declines as ಕಾಡನ್ನು or ಕಾಡುವನ್ನು, or whether ಹುಡುಗ takes ಹುಡುಗನನ್ನು. Each candidate declension is scored by how often its own forms occur in the training split, and the best one wins.
5. **Induced stems** (the `kn_IN-wikipedia` add-on only). A name or loanword that Alar lacks is added when 3 or more distinct training words are its forms in 1 noun class. Frequent unexplained words are added whole. A word 1 confusion away from an accepted word 3 or more times as frequent is treated as a misspelling and left out, so ಹಾಗು (for ಹಾಗೂ) is not learned from Wikipedia's own errors.

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
target/release/kspell hunspell --lexicon data/build/lexicon.tsv --base data/build/lexicon-alar.tsv --out dict/kn_IN-wikipedia
```

## Measure

```sh
target/release/kspell eval --dictionary dict/kn_IN,dict/kn_IN-wikipedia --reference dict/kn_IN
target/release/kspell suggest --dictionary dict/kn_IN,dict/kn_IN-wikipedia
target/release/kspell suggest --dictionary dict/kn_IN,dict/kn_IN-wikipedia --misspellings data/build/typos.tsv
target/release/kspell gold --dictionary dict/kn_IN,dict/kn_IN-wikipedia --gold data/gold/analyses-test.tsv
target/release/kspell phonetic-eval
```

`eval` writes the misspellings it tries to `data/build/misspellings.tsv` (confusions) and `data/build/typos.tsv` (typing slips); `suggest` and `phonetic-eval` read them. Keep `--reference dict/kn_IN` fixed so that every dictionary is tested on the same misspellings. Drop `--dictionary` for the `kn_IN` alone column. [results.md](results.md) has the figures.

Other commands: `kspell analyze`, `kspell stem`, `kspell paradigm`, `kspell phonetic`, `kspell phonetic-vectors`, `kspell sample`, `kspell rules`. The tags are described in [tags.md](tags.md).

## Packages

`scripts/package.sh` builds both packages from `dict/kn_IN` and `dict/kn_IN-wikipedia` into `dist/`, and `scripts/package-check.sh` checks them:

- `kannada-spell-kn_IN-<version>.oxt`, a LibreOffice extension. It registers both word lists for Kannada (India) as 2 dictionaries, and LibreOffice accepts a word that either one accepts. Tested with LibreOffice 26.8.
- `kannada-spell-kn_IN-<version>.xpi`, a Firefox dictionary add-on for Firefox 140 and later. Firefox takes 1 dictionary file per language, so `package.sh` merges the 2 word lists into its `kn-IN.dic`. That file is derived from Alar, so it is licensed under the ODbL 1.0 as a whole, with credit to Alar and Kannada Wikipedia.

Each package's `NOTICE.md` credits Alar and Kannada Wikipedia and holds both license texts. `package-check.sh` fails if a package covers fewer held-out words than `kn_IN` with `kn_IN-wikipedia`: it compares them on the words of `data/gold/sample.tsv`, weighted by their held-out counts.
