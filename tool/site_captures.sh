#!/usr/bin/env bash
# Refait les images du site (site/img/) à partir de l'app elle-même : les captures des tests d'interface.
# À relancer quand l'interface change ; le site suit alors l'app.
set -euo pipefail
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
FANO_CAPTURES="$tmp" cargo test -q -p fano-ui --test captures
mkdir -p site/img
cp "$tmp/maths.png" site/img/maths.png
cp "$tmp/tablette-portrait.png" site/img/cours.png
cp "$tmp/document-annote.png" site/img/annoter.png
cp "$tmp/coffre.png" site/img/ranger.png
ls -la site/img
