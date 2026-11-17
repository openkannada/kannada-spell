# Working on kannada-spell

Instructions for anyone changing this repository, human or AI agent.

## What this is

A Kannada spelling dictionary for Hunspell, a morphological analyzer and stemmer, sound-alike keys for search, and the tools that build and measure them. 1 description of Kannada inflection, the paradigms in `crates/kannada-morph`, drives both the Hunspell files and the analyzer, so they accept exactly the same words.

## Commands

```sh
cargo test                                         # fast unit tests, under 5 seconds
cargo fmt --all && cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
scripts/hunspell-smoke.sh                          # the committed dictionary with real Hunspell
scripts/agreement.sh                               # Hunspell and the analyzer accept the same words
scripts/package.sh && scripts/package-check.sh    # the LibreOffice and Firefox packages, into dist/
```

Rebuilding the dictionaries needs `data/raw/alar.yml` and a Kannada Wikipedia dump; see the README.

## Architecture

```text
crates/kannada-morph/      the library
  script.rs                Kannada character classes, normalization, word tokens
  sandhi.rs                how a suffix joins: elision, glides, virama
  form.rs                  a generated form and the step that adds 1 suffix
  noun.rs                  declensions, plurals, clitics, fused predicates
  verb.rs                  conjugation, irregular verbs, auxiliaries
  class.rs                 inflection classes and their Hunspell flags
  rules.rs                 affix rules read off the paradigms
  classify.rs              choosing a declension from corpus evidence
  lexicon.rs               stems with classes, lemmas and tags
  analyzer.rs              analysis, guessing and stemming by affix stripping
  hunspell.rs              writing and reading the .aff and .dic files
crates/kannada-phonetic/   sound-alike keys at 3 levels, for Kannada script and Latin input
  vectors.tsv              test vectors for ports, rewritten by `kspell phonetic-vectors`
crates/kspell/             the command-line tool
  corpus.rs                Wikipedia dump to training and held-out counts
  alar.rs                  Alar headwords and parts of speech
  build_lexicon.rs         Alar plus the closed-class list to a lexicon
  induce.rs                stems and words induced from the training counts
  misspell.rs, eval.rs     synthetic misspellings and the coverage measures
  suggest.rs               scoring Hunspell's suggestions
  gold.rs                  scoring the analyzer against gold analyses
  phonetic_eval.rs         measuring the phonetic keys
data/closed-class.tsv      pronouns, numerals and function words, written by hand
dict/                      the built dictionaries (ODbL, see DATA-LICENSE.md); kn_IN is the published one
packaging/                 templates for the LibreOffice extension and the Firefox add-on
data/gold/                 gold analyses: a development set and a test set
docs/tags.md               the analysis tags
```

## Rules

- Inflection is described once, in `noun.rs` and `verb.rs`. Do not hand-edit affix rules or the `.aff` file; change the paradigm and rebuild.
- Every behavior change gets a test. For each new rule, remove it, confirm a test fails, then restore it. Run each such check under a 5 second timeout.
- Measure on held-out text only. The test split (every 10th Wikipedia article) is never used to build or tune the dictionary.
- Tune the analyzer on `data/gold/analyses.tsv` only. `analyses-test.tsv` is for scoring; report its first score for any change you made after looking at it.
- A new gold test set is committed, with its first score, before any change it inspires. Draw new development sets from the training split, not the held-out articles.
- Keep the ODbL notice on the dictionaries and the credit to Alar and V. Krishna.
- Comments only for what the code cannot say.

## Commits

Sign your commits (`git commit -S`). Add no AI attribution lines such as Co-Authored-By trailers.

## Writing

Plain short sentences, American spelling, digits for numbers, no em dashes, no hype.
