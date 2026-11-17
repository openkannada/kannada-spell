# Notice

kannada-spell. Copyright 2026 The Open Kannada Authors. The code is under the MIT License or the Apache License 2.0, at your option. Docs and data written for this project are under CC BY 4.0.

This repository includes or is built from the third-party material below. [DATA-LICENSE.md](DATA-LICENSE.md) says which license applies to each dictionary file, and `REUSE.toml` covers every file.

| Material | License | How it is used | In this repository |
| --- | --- | --- | --- |
| [Alar](https://github.com/alar-dict/data) by V. Krishna, commit `8651ccf8e92184ca17e234eeb6c947d8d52dd5c4` | ODbL 1.0 | Headwords and parts of speech become the stems of `dict/kn_IN.dic` | Yes, as `dict/kn_IN.dic` (ODbL 1.0) |
| [Kannada Wikipedia](https://kn.wikipedia.org), dump of 1 November 2026, by its contributors | CC BY-SA 4.0 | Word counts choose inflection classes and order the `TRY` letters; induced words become `dict/kn_IN-wikipedia.dic`; word types are sampled for `data/gold/` | Yes, as `dict/kn_IN-wikipedia.dic` and `data/gold/*.tsv` (CC BY-SA 4.0). The dump itself is not stored. |
| Fedora `hunspell-kn` 1.0.3 and [kn-hunspell](https://github.com/mgharish/kn-hunspell) | Their own licenses | Benchmark comparison only | No |
| Rust crates in `Cargo.lock` | MIT, Apache-2.0 and other permissive licenses | Build dependencies | No |

Contains information from Alar by V. Krishna (https://alar.ink), made available under the Open Database License 1.0 (https://opendatacommons.org/licenses/odbl/1-0/).

Includes words counted in Kannada Wikipedia (https://kn.wikipedia.org), written by its contributors, licensed under CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/).
