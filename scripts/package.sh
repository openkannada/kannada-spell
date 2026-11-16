#!/bin/sh
# Builds the LibreOffice extension (.oxt) and the Firefox dictionary add-on (.xpi) from dict/kn_IN into dist/.
set -eu
export TZ=UTC
cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
lo_id=${LO_ID:-io.github.prasadus92.kannada-spell}
firefox_id=${FIREFOX_ID:-kannada-spell@prasadus92.github.io}
fill() { sed -e "s/@VERSION@/$version/g" -e "s/@LO_ID@/$lo_id/g" -e "s/@FIREFOX_ID@/$firefox_id/g" "$1"; }

notice() {
  {
    cat dict/README.md
    printf '\n---\n\n'
    cat DATA-LICENSE.md
    printf '\n---\n\n'
    cat LICENSE
  } > "$1"
}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p dist

oxt="$work/oxt"
mkdir -p "$oxt/META-INF"
cp dict/kn_IN.aff dict/kn_IN.dic packaging/libreoffice/dictionaries.xcu packaging/libreoffice/description-en.txt "$oxt/"
cp packaging/libreoffice/META-INF/manifest.xml "$oxt/META-INF/"
fill packaging/libreoffice/description.xml > "$oxt/description.xml"
notice "$oxt/NOTICE.md"
rm -f "dist/kannada-spell-kn_IN-$version.oxt"
(cd "$oxt" && find . -exec touch -t 202601010000 {} + && zip -qX -r "$OLDPWD/dist/kannada-spell-kn_IN-$version.oxt" META-INF description.xml description-en.txt dictionaries.xcu kn_IN.aff kn_IN.dic NOTICE.md)

xpi="$work/xpi"
mkdir -p "$xpi/dictionaries"
cp dict/kn_IN.aff "$xpi/dictionaries/kn-IN.aff"
cp dict/kn_IN.dic "$xpi/dictionaries/kn-IN.dic"
fill packaging/firefox/manifest.json > "$xpi/manifest.json"
notice "$xpi/NOTICE.md"
rm -f "dist/kannada-spell-kn_IN-$version.xpi"
(cd "$xpi" && find . -exec touch -t 202601010000 {} + && zip -qX -r "$OLDPWD/dist/kannada-spell-kn_IN-$version.xpi" manifest.json dictionaries NOTICE.md)

echo "dist/kannada-spell-kn_IN-$version.oxt"
echo "dist/kannada-spell-kn_IN-$version.xpi"
