#!/bin/sh
# SPDX-FileCopyrightText: 2026 The Open Kannada Authors
# SPDX-License-Identifier: MIT OR Apache-2.0
# Builds the LibreOffice extension (.oxt) and the Firefox dictionary add-on (.xpi) from dict/kn_IN and
# dict/kn_IN-wikipedia into dist/.
set -eu
export TZ=UTC
umask 022
cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
lo_id=${LO_ID:-io.github.openkannada.kannada-spell}
firefox_id=${FIREFOX_ID:-kannada-spell@openkannada.github.io}
fill() { sed -e "s/@VERSION@/$version/g" -e "s/@LO_ID@/$lo_id/g" -e "s/@FIREFOX_ID@/$firefox_id/g" "$1"; }

notice() {
  {
    sed -E 's/\[([^]]*)\]\(\.\.\/[^)]*\)/\1/g' dict/README.md
    printf '\n---\n\n'
    cat DATA-LICENSE.md
    for license in LICENSES/ODbL-1.0.txt LICENSES/CC-BY-4.0.txt LICENSE-MIT LICENSE-APACHE; do
      printf '\n---\n\n'
      cat "$license"
    done
  } > "$1"
}

# Fixed modes and times, so the archives have the same bytes on any machine.
normalize() {
  find "$1" -type d -exec chmod 755 {} +
  find "$1" -type f -exec chmod 644 {} +
  find "$1" -exec touch -t 202601010000 {} +
}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p dist

oxt="$work/oxt"
mkdir -p "$oxt/META-INF"
cp dict/kn_IN.aff dict/kn_IN.dic dict/kn_IN-wikipedia.dic packaging/libreoffice/dictionaries.xcu packaging/libreoffice/description-en.txt "$oxt/"
# LibreOffice finds each .dic's rules in an .aff of the same name.
cp dict/kn_IN.aff "$oxt/kn_IN-wikipedia.aff"
cp packaging/libreoffice/META-INF/manifest.xml "$oxt/META-INF/"
fill packaging/libreoffice/description.xml > "$oxt/description.xml"
notice "$oxt/NOTICE.md"
rm -f "dist/kannada-spell-kn_IN-$version.oxt"
normalize "$oxt"
(cd "$oxt" && zip -qXD "$OLDPWD/dist/kannada-spell-kn_IN-$version.oxt" META-INF/manifest.xml description.xml description-en.txt dictionaries.xcu kn_IN.aff kn_IN.dic kn_IN-wikipedia.aff kn_IN-wikipedia.dic NOTICE.md)

xpi="$work/xpi"
mkdir -p "$xpi/dictionaries"
cp dict/kn_IN.aff "$xpi/dictionaries/kn-IN.aff"
# Firefox takes 1 .dic per language, so the add-on gets both word lists merged into 1 file.
{ tail -n +2 dict/kn_IN.dic; tail -n +2 dict/kn_IN-wikipedia.dic; } | LC_ALL=C sort > "$work/entries"
{ wc -l < "$work/entries" | tr -d ' '; cat "$work/entries"; } > "$xpi/dictionaries/kn-IN.dic"
fill packaging/firefox/manifest.json > "$xpi/manifest.json"
notice "$xpi/NOTICE.md"
rm -f "dist/kannada-spell-kn_IN-$version.xpi"
normalize "$xpi"
(cd "$xpi" && zip -qXD "$OLDPWD/dist/kannada-spell-kn_IN-$version.xpi" manifest.json dictionaries/kn-IN.aff dictionaries/kn-IN.dic NOTICE.md)

echo "dist/kannada-spell-kn_IN-$version.oxt"
echo "dist/kannada-spell-kn_IN-$version.xpi"
