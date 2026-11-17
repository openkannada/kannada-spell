# Dictionaries

| File | License |
| --- | --- |
| `kn_IN.dic` | Open Database License 1.0. Derived from Alar by V. Krishna (https://alar.ink). |
| `kn_IN-wikipedia.dic` | CC BY 4.0. Words counted in Kannada Wikipedia (https://kn.wikipedia.org), credited to it as the source. |
| `kn_IN.aff` | MIT OR Apache-2.0. Generated rules; no Alar or Wikipedia text. |

Hunspell's `.dic` format has no room for a comment, so the notices for the word lists are here and in [DATA-LICENSE.md](../DATA-LICENSE.md). Keep them with any copy.

> Contains information from Alar by V. Krishna (https://alar.ink), made available under the Open Database License 1.0 (https://opendatacommons.org/licenses/odbl/1-0/).

`kn_IN` holds Alar and the hand-written closed-class list. `kn_IN-wikipedia` holds names, loanwords and other words induced from Kannada Wikipedia word counts. It loads with the `kn_IN` affix file, as in `hunspell -d kn_IN,kn_IN-wikipedia`. The LibreOffice and Firefox packages carry both; the Firefox add-on merges them into 1 file, licensed under the ODbL 1.0 as a whole, since it is derived from Alar. Credit Kannada Wikipedia with `kn_IN-wikipedia` and with the merged file:

> Includes words counted in Kannada Wikipedia (https://kn.wikipedia.org), written by its contributors. This word list is licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/).
