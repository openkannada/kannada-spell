# Contributing to kannada-spell

Wrong spellings, missing words, bugs and ideas go in [GitHub issues](https://github.com/openkannada/kannada-spell/issues). Include the word, what you expected and what the dictionary or `kspell` did. For a security problem, follow [SECURITY.md](SECURITY.md) instead.

The [Open Kannada contributing guide](https://github.com/openkannada/.github/blob/main/CONTRIBUTING.md) covers the rest: sign off every commit with `git commit -s` under the Developer Certificate of Origin, and your work is licensed under the licenses of the files it changes, as the README's License section describes. CI checks every commit in a pull request for the `Signed-off-by` line.

## Before you open a pull request

```sh
cargo fmt --all && cargo clippy --all-targets -- -D warnings
cargo test
scripts/hunspell-smoke.sh && scripts/agreement.sh
uvx reuse lint
```

- Change inflection in the paradigms (`crates/kannada-morph/src/noun.rs` and `verb.rs`) and rebuild; never edit the `.aff` file itself.
- Keep Alar entries and Wikipedia-derived entries in separate files: they have different licenses (see [DATA-LICENSE.md](DATA-LICENSE.md)).
- Tune the analyzer on `data/gold/analyses.tsv` only; `analyses-test.tsv` is for scoring.
