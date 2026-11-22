# kannada-spell

A Kannada spelling dictionary for Hunspell, LibreOffice and Firefox, with a morphological analyzer, a stemmer and sound-alike keys built from the same rules.

It accepts 88.9% of the words in held-out Kannada Wikipedia text, where the dictionary Fedora ships (`hunspell-kn` 1.0.3, from 2009) accepts 65.7%. Kannada joins case endings, auxiliaries and the copula to the word before them, so 1 noun has hundreds of written forms (ಮರ, ಮರವನ್ನು, ಮರಗಳಲ್ಲಿ, ಮರವಾಗಿದೆ); this project describes that inflection once, as paradigms with sandhi rules, and generates both the Hunspell affix file and the analyzer from it.

## Quick start

```sh
git clone https://github.com/openkannada/kannada-spell && cd kannada-spell
hunspell -d dict/kn_IN,dict/kn_IN-wikipedia file.txt   # check a file
cargo install --path crates/kspell
kspell analyze ಮರಗಳಲ್ಲಿ ಬಂದಿದ್ದಾರೆ                         # ಮರ+n+pl+loc, ಬರು+v+cp|ಇರು+v+pres.3p
kspell phonetic ಸಂಬಂಧ                                    # sambanda sambanda smbnd
```

## Results

Test set: every 10th article of the Kannada Wikipedia dump of 1 November 2026, held out from the build: 2.2 million words, 331,936 distinct word types. Misspellings are 1-edit changes of frequent held-out words that occur nowhere in the corpus.

| Measure | Packaged (`kn_IN` with `kn_IN-wikipedia`) | `kn_IN` alone | Fedora `hunspell-kn` 1.0.3 |
| --- | --- | --- | --- |
| Held-out tokens accepted | 88.9% | 70.8% | 65.7% |
| Held-out word types seen 5 or more times, accepted | 91.2% | 61.7% | 40.0% |
| Sound-alike misspellings accepted, of 5,000 | 3.7% | 3.6% | 0.2% |
| Typing slips accepted, of 5,000 | 3.6% | 3.4% | 0.1% |
| Sound-alike misspellings: intended word first, in top 5, of 2,000 | 81.7%, 97.5% | 83.4%, 97.7% | |
| Typing slips: intended word first, in top 5, of 2,000 | 70.2%, 95.8% | 72.4%, 96.3% | |

The analyzer, on the 196-word gold test set (`data/gold/analyses-test.tsv`) with the packaged dictionary: readings precision 67.5%, recall 72.6%, stem correct 90.3%. The gold analyses were machine-annotated from [docs/tags.md](docs/tags.md) alone, without access to the analyzer or the dictionaries, and a Kannada reader has not checked them yet.

Some coverage fixes were chosen after reading held-out words the dictionary rejected, so the coverage figures may be slightly optimistic. [docs/results.md](docs/results.md) has the full tables, the development gold set, the phonetic key measures and the method; [docs/building.md](docs/building.md) has the commands that reproduce them.

## What's inside

| Path | What it is |
| --- | --- |
| `dict/kn_IN` | The dictionary: Alar headwords and a closed-class list, with the affix rules (ODbL 1.0) |
| `dict/kn_IN-wikipedia` | Names, loanwords and other words Alar lacks, induced from Kannada Wikipedia (CC BY 4.0); loads with the `kn_IN` affix file |
| `crates/kannada-morph` | Paradigms, sandhi, the affix rules, and the analyzer and stemmer |
| `crates/kannada-phonetic` | Sound-alike keys at 3 levels, for Kannada script and Latin input, with test vectors for ports |
| `crates/kspell` | The command-line tool that builds and measures everything |
| `packaging/`, `scripts/package.sh` | The LibreOffice extension and the Firefox add-on |

Hunspell and `kspell analyze` accept exactly the same words: they agree on all 331,936 held-out word types, and CI checks a sample on every change.

## Install

- **Hunspell:** `hunspell -d dict/kn_IN,dict/kn_IN-wikipedia` is what the packages carry. Use `dict/kn_IN` alone for a word list under the ODbL only.
- **LibreOffice:** build the `.oxt` with `scripts/package.sh`, install it with Tools > Extension Manager > Add, restart, and set the text language to Kannada (India).
- **Firefox:** `scripts/package.sh` also builds the `.xpi`. Until it is listed on addons.mozilla.org, load it from about:debugging > This Firefox > Load Temporary Add-on.

## Analyzer, stemmer and sound-alike keys

- `kspell analyze` returns every reading of a word with its lemma and tags, `kspell stem` its lemma, and `kspell paradigm` every form of a lemma. `--guess` proposes readings for unknown words from the ending alone.
- `kspell phonetic` gives narrow, medium and broad keys. Narrow folds aspiration, vowel length and doubled consonants; medium also folds retroflex and dental, ಳ and ಲ, and the 3 sibilants; broad keeps consonants only. Latin input gets the medium and broad keys, so "neeru" matches ನೀರು.

## Related

- [kannada-spell-rs](https://github.com/openkannada/kannada-spell-rs): the same dictionary compiled for fast suggestions, a CLI and WebAssembly.
- [kannada-dictionary](https://github.com/openkannada/kannada-dictionary): uses the analyzer and the sound-alike keys in its search.
- [kannada-test-corpus](https://github.com/openkannada/kannada-test-corpus): indexes the gold sets and the held-out word counts.
- [openkannada.org/spell-check](https://openkannada.org/spell-check/): the project page.

## License

Code under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). `dict/kn_IN.dic` is derived from [Alar](https://alar.ink) by V. Krishna under the ODbL 1.0; `dict/kn_IN-wikipedia.dic` and `data/gold/` are words and counts from Kannada Wikipedia under CC BY 4.0. [DATA-LICENSE.md](DATA-LICENSE.md) and [NOTICE.md](NOTICE.md) give the details, and [CONTRIBUTING.md](CONTRIBUTING.md) how to contribute.

Part of [Open Kannada](https://github.com/openkannada), free and open-source tools that make Kannada work properly on computers.
