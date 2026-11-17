# Contributing to kannada-spell

Thank you for helping. Wrong spellings, missing words, bugs and ideas go in [GitHub issues](https://github.com/openkannada/kannada-spell/issues). Include the word, what you expected and what the dictionary or `kspell` did. For a security problem, follow [SECURITY.md](SECURITY.md) instead.

## Sign off every commit (DCO)

Open Kannada uses the [Developer Certificate of Origin](https://developercertificate.org/) (DCO), not a contributor license agreement. Every commit needs a `Signed-off-by` line with your name and email. It certifies that you wrote the change, or otherwise have the right to submit it under this repository's licenses.

`git commit -s` adds the line from your Git settings:

```text
Signed-off-by: Your Name <you@example.com>
```

If you forgot, `git commit --amend -s` signs off the last commit, and `git rebase --signoff origin/main` signs off every commit on your branch.

You keep the copyright in your work. It is licensed under the licenses of the files it changes, as the README's License section describes, and you join "The Open Kannada Authors".

CI checks every commit in a pull request for the line.

## Making a change

[AGENTS.md](AGENTS.md) has the commands, the layout and the rules, for people and AI agents alike. Before you open a pull request, run:

```sh
cargo fmt --all && cargo clippy --all-targets -- -D warnings
cargo test
scripts/hunspell-smoke.sh && scripts/agreement.sh
```

Keep Alar entries and Wikipedia-derived entries in separate files: they have different share-alike licenses (see [DATA-LICENSE.md](DATA-LICENSE.md)).
