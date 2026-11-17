#!/bin/sh
# SPDX-FileCopyrightText: 2026 The Open Kannada Authors
# SPDX-License-Identifier: MIT OR Apache-2.0
# Checks the packages from package.sh: well-formed manifests, both word lists carried unchanged, both
# licenses credited, and packaged coverage no lower than kn_IN with the kn_IN-wikipedia add-on.
set -eu
cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
status=0
fail() { echo "$1"; status=1; }

unzip -q "dist/kannada-spell-kn_IN-$version.oxt" -d "$work/oxt"
unzip -q "dist/kannada-spell-kn_IN-$version.xpi" -d "$work/xpi"
for f in META-INF/manifest.xml description.xml dictionaries.xcu; do
  xmllint --noout "$work/oxt/$f" || fail "malformed $f in the .oxt"
done
grep -q "<version value=\"$version\"/>" "$work/oxt/description.xml" || fail "the .oxt version is not $version"
grep -q '%origin%/kn_IN.aff %origin%/kn_IN.dic' "$work/oxt/dictionaries.xcu" || fail "dictionaries.xcu does not register kn_IN"
grep -q '%origin%/kn_IN-wikipedia.aff %origin%/kn_IN-wikipedia.dic' "$work/oxt/dictionaries.xcu" || fail "dictionaries.xcu does not register kn_IN-wikipedia"
[ "$(grep -c '<value>kn-IN</value>' "$work/oxt/dictionaries.xcu")" -eq 2 ] || fail "dictionaries.xcu does not give both dictionaries the kn-IN locale"
for f in META-INF/manifest.xml description.xml; do
  ! grep -q '@[A-Z_]*@' "$work/oxt/$f" || fail "unfilled placeholder in $f"
done
cmp -s dict/kn_IN.aff "$work/oxt/kn_IN.aff" || fail "the .oxt kn_IN.aff differs from dict/"
cmp -s dict/kn_IN.aff "$work/oxt/kn_IN-wikipedia.aff" || fail "the .oxt kn_IN-wikipedia.aff differs from dict/kn_IN.aff"
cmp -s dict/kn_IN.dic "$work/oxt/kn_IN.dic" || fail "the .oxt kn_IN.dic differs from dict/"
cmp -s dict/kn_IN-wikipedia.dic "$work/oxt/kn_IN-wikipedia.dic" || fail "the .oxt kn_IN-wikipedia.dic differs from dict/"

# The repository keeps the Alar and Wikipedia word lists in separate files; a stem may be in both with other classes.
entries() { tail -n +2 "$1" | LC_ALL=C sort -u; }
entries dict/kn_IN-wikipedia.dic > "$work/add-on-entries"
entries dict/kn_IN.dic > "$work/alar-entries"
[ -z "$(LC_ALL=C comm -12 "$work/add-on-entries" "$work/alar-entries")" ] || fail "dict/kn_IN.dic holds entries of the kn_IN-wikipedia add-on"

grep -q '<name xlink:href="https://github.com/openkannada/kannada-spell" lang="en">Open Kannada</name>' "$work/oxt/description.xml" || fail "the .oxt publisher is not Open Kannada"
for f in META-INF/manifest.xml description.xml description-en.txt dictionaries.xcu NOTICE.md; do
  [ -f "$work/oxt/$f" ] || fail "the .oxt has no $f"
done
for text in "Open Database License (ODbL)" "opendatacommons.org/licenses/odbl" "Attribution 4.0 International" "Kannada Wikipedia" "Permission is hereby granted" "Apache License"; do
  grep -q "$text" "$work/oxt/NOTICE.md" || fail "the .oxt NOTICE.md lacks $text"
  grep -q "$text" "$work/xpi/NOTICE.md" || fail "the .xpi NOTICE.md lacks $text"
done

python3 - "$work/xpi/manifest.json" "$version" <<'PY' || fail "bad Firefox manifest"
import json, sys
m = json.load(open(sys.argv[1], encoding="utf-8"))
assert m["version"] == sys.argv[2], m["version"]
assert "@VERSION@" not in json.dumps(m) and "@FIREFOX_ID@" not in json.dumps(m)
assert m["dictionaries"] == {"kn-IN": "dictionaries/kn-IN.dic"}, m["dictionaries"]
gecko = m["browser_specific_settings"]["gecko"]
assert gecko["id"].count("@") == 1 and gecko["id"].split("@")[1], gecko["id"]
assert m["author"] == "Open Kannada", m["author"]
assert gecko["strict_min_version"] == "140.0", gecko["strict_min_version"]
PY
cmp -s dict/kn_IN.aff "$work/xpi/dictionaries/kn-IN.aff" || fail "the .xpi kn-IN.aff differs from dict/"
{ tail -n +2 dict/kn_IN.dic; tail -n +2 dict/kn_IN-wikipedia.dic; } | LC_ALL=C sort > "$work/union"
tail -n +2 "$work/xpi/dictionaries/kn-IN.dic" | cmp -s - "$work/union" || fail "the .xpi kn-IN.dic is not exactly kn_IN.dic and kn_IN-wikipedia.dic merged"
[ "$(head -n 1 "$work/xpi/dictionaries/kn-IN.dic")" -eq "$(wc -l < "$work/union")" ] || fail "the .xpi kn-IN.dic has a wrong entry count"

# Coverage on the held-out words of data/gold/sample.tsv, weighted by their held-out counts. LibreOffice
# accepts a word that either registered dictionary accepts; Firefox reads the merged file.
sample=data/gold/sample.tsv
grep -v '^#' "$sample" | cut -f1 > "$work/words"
accepted() { awk -F'\t' 'NR == FNR { bad[$1] = 1; next } !/^#/ && !($1 in bad) { n += $2 } END { print n + 0 }' "$1" "$sample"; }
hunspell -d dict/kn_IN,dict/kn_IN-wikipedia -l < "$work/words" | sort -u > "$work/rejected-combined"
hunspell -d dict/kn_IN -l < "$work/words" | sort -u > "$work/rejected-alar"
hunspell -d "$work/oxt/kn_IN" -l < "$work/words" | hunspell -d "$work/oxt/kn_IN-wikipedia" -l | sort -u > "$work/rejected-oxt"
hunspell -d "$work/xpi/dictionaries/kn-IN" -l < "$work/words" | sort -u > "$work/rejected-xpi"
total=$(grep -v '^#' "$sample" | awk -F'\t' '{ n += $2 } END { print n }')
combined=$(accepted "$work/rejected-combined")
for package in oxt xpi; do
  got=$(accepted "$work/rejected-$package")
  echo "the .$package accepts $got of $total held-out sample tokens; kn_IN with kn_IN-wikipedia accepts $combined, kn_IN alone $(accepted "$work/rejected-alar")"
  [ "$got" -ge "$combined" ] || fail "the .$package covers less than kn_IN with the kn_IN-wikipedia add-on"
done

scripts/hunspell-smoke.sh "$work/xpi/dictionaries/kn-IN" || status=1
scripts/hunspell-smoke.sh "$work/oxt/kn_IN,$work/oxt/kn_IN-wikipedia" || status=1
[ "$status" -eq 0 ] && echo "package check passed for $version"
exit "$status"
