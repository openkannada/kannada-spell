#!/bin/sh
# SPDX-FileCopyrightText: 2026 The Open Kannada Authors
# SPDX-License-Identifier: MIT OR Apache-2.0
# Checks the packages from package.sh: well-formed manifests, the dictionary unchanged, and Hunspell accepting it.
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
grep -q '%origin%/kn_IN.aff %origin%/kn_IN.dic' "$work/oxt/dictionaries.xcu" || fail "dictionaries.xcu does not point at kn_IN"
for f in META-INF/manifest.xml description.xml; do
  ! grep -q '@[A-Z_]*@' "$work/oxt/$f" || fail "unfilled placeholder in $f"
done
cmp -s dict/kn_IN.aff "$work/oxt/kn_IN.aff" || fail "the .oxt kn_IN.aff differs from dict/"
cmp -s dict/kn_IN.dic "$work/oxt/kn_IN.dic" || fail "the .oxt kn_IN.dic differs from dict/"
stems() { tail -n +2 "$1" | cut -d' ' -f1 | cut -d/ -f1 | LC_ALL=C sort -u; }
stems dict/kn_IN-wikipedia.dic > "$work/add-on"
stems "$work/oxt/kn_IN.dic" > "$work/packaged"
[ -z "$(LC_ALL=C comm -12 "$work/add-on" "$work/packaged")" ] || fail "the .oxt kn_IN.dic holds stems of the CC BY-SA kn_IN-wikipedia add-on"
grep -q '<name xlink:href="https://github.com/openkannada/kannada-spell" lang="en">Open Kannada</name>' "$work/oxt/description.xml" || fail "the .oxt publisher is not Open Kannada"
for f in META-INF/manifest.xml description.xml description-en.txt dictionaries.xcu NOTICE.md; do
  [ -f "$work/oxt/$f" ] || fail "the .oxt has no $f"
done
for text in "Open Database License (ODbL)" "opendatacommons.org/licenses/odbl" "Permission is hereby granted" "Apache License"; do
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
cmp -s dict/kn_IN.dic "$work/xpi/dictionaries/kn-IN.dic" || fail "the .xpi kn-IN.dic differs from dict/"

scripts/hunspell-smoke.sh "$work/oxt/kn_IN" core || status=1
[ "$status" -eq 0 ] && echo "package check passed for $version"
exit "$status"
