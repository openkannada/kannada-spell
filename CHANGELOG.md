# Changelog

All notable changes to this project are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). No version has been released yet; the packages build as 0.1.0.

## Unreleased

### Added

- `CITATION.cff` and `SECURITY.md`.
- LibreOffice extension and Firefox add-on, built by `scripts/package.sh` and checked by `scripts/package-check.sh`.
- `kspell paradigm`, which lists every form of a lemma with its reading.
- Sound-alike keys at 3 levels in `crates/kannada-phonetic`, for Kannada script and Latin input, with test vectors for ports, measured by `kspell phonetic-eval`.
- Gold analyses of held-out Wikipedia words, a 300-word development set and a 200-word test set, scored by `kspell gold`.
- The Hunspell dictionary, the morphological analyzer and stemmer, and the `kspell` tool that builds and measures them.

### Changed

- The dictionary is split by license: `dict/kn_IN` holds Alar and the closed-class list (ODbL 1.0), and `dict/kn_IN-wikipedia` the words induced from Kannada Wikipedia (CC BY 4.0). Code is under MIT OR Apache-2.0, with REUSE and DCO checks in CI.
- Repository links point to the openkannada organization.
- The README is shorter; the full results and the build steps moved to `docs/results.md` and `docs/building.md`.

### Fixed

- Old Kannada words no longer get a loanword declension, and adjectives no longer take every form of the copula.
