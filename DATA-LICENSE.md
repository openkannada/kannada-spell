# Data license

The code in this repository is under the MIT License (see `LICENSE`). The dictionaries are not.

## `dict/kn_IN.dic` and `dict/kn_IN-alar.dic`

These word lists are derived from [Alar](https://github.com/alar-dict/data), the Kannada-English dictionary by V. Krishna, which is published under the [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/) (ODbL). The ODbL is share-alike for databases: a database derived from Alar must also be offered under the ODbL. Both `.dic` files are therefore licensed under the ODbL 1.0.

Built from Alar commit `8651ccf8e92184ca17e234eeb6c947d8d52dd5c4`. The Alar headwords were filtered, normalized and given inflection classes; see `README.md`.

`dict/kn_IN.dic` also contains words and stems induced from word counts of Kannada Wikipedia (text under CC BY-SA 4.0). Only single words and the inflection class each one takes are used: no sentences or longer passages. `dict/kn_IN-alar.dic` leaves them out, for users who want a word list drawn from Alar alone.

## `dict/kn_IN.aff` and `dict/kn_IN-alar.aff`

The affix files hold the inflection rules generated from the paradigms in `crates/kannada-morph` and contain no Alar data. Their `TRY` line lists Kannada letters in order of frequency in Kannada Wikipedia. They are under the MIT License, like the code.

## Attribution

When you use or redistribute the dictionaries, keep this notice and credit:

> Contains information from Alar by V. Krishna (https://alar.ink), made available under the Open Database License 1.0.
