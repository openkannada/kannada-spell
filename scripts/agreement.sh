#!/bin/sh
# Checks that real Hunspell and `kspell analyze` accept exactly the same words, on a sample
# of forms built from the dictionary and a typo of each.
set -eu
kspell="${KSPELL:-target/release/kspell}"
dict="${1:-dict/kn_IN}"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
"$kspell" sample --dictionary "$dict" > "$tmp/words"
hunspell -d "$dict" -l < "$tmp/words" | sort -u > "$tmp/hunspell"
"$kspell" analyze --dictionary "$dict" < "$tmp/words" | awk -F'\t' '$2=="?"{print $1}' | sort -u > "$tmp/analyzer"
total=$(sort -u "$tmp/words" | wc -l | tr -d ' ')
rejected=$(wc -l < "$tmp/analyzer" | tr -d ' ')
if ! diff "$tmp/hunspell" "$tmp/analyzer" > "$tmp/diff"; then
  echo "Hunspell and the analyzer disagree on $dict:"
  head -40 "$tmp/diff"
  exit 1
fi
echo "Hunspell and the analyzer agree on $total words from $dict ($rejected rejected)"
