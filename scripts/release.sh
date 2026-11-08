#!/usr/bin/env bash
# Prépare une version (docs/RELEASING.md) : programme Linux, APK, release.json signé, notes.
#   scripts/release.sh            → release-out/
# Ne publie rien : la publication (gh release create) est une étape à part, faite à la main.
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(cat VERSION)
cle=release-keys/maj-ed25519.pem
[ -f "$cle" ] || { echo "clé de mise à jour absente : $cle" >&2; exit 1; }
grep -q "^version = \"$version\"" Cargo.toml || { echo "Cargo.toml n'est pas en $version" >&2; exit 1; }

cargo build --release -p fanoratana
scripts/android-build.sh

rm -rf release-out && mkdir -p release-out
cp target/release/fanoratana release-out/fanoratana-linux-x86_64
cp "dist/fanoratana-$version-arm64-v8a.apk" release-out/fanoratana-arm64-v8a.apk
cp brand/icone.svg release-out/fanoratana.svg
cp packaging/fanoratana.desktop release-out/

python3 -I - "$version" <<'PY'
import hashlib, json, os, re, sys
v = sys.argv[1]
d = "release-out"
def f(nom):
    b = open(os.path.join(d, nom), "rb").read()
    return {"nom": nom, "sha256": hashlib.sha256(b).hexdigest(), "taille": len(b)}
journal = open("CHANGELOG.md", encoding="utf-8").read()
bloc = journal.split(f"## [{v}]", 1)[1].split("\n## [", 1)[0]
def message(langue):
    m = re.search(rf"### Message {langue}\n(.*?)(\n### |\Z)", bloc, re.S)
    return m.group(1).strip() if m else ""
rel = {"version": v, "notes_fr": message("fr"), "notes_en": message("en"),
       "fichiers": {"android-arm64": f("fanoratana-arm64-v8a.apk"), "linux-x86_64": f("fanoratana-linux-x86_64")}}
json.dump(rel, open(os.path.join(d, "release.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
notes = f"""# Fanoratana {v}

{message('fr')}

**Installer** : sur Android, télécharge `fanoratana-arm64-v8a.apk` et ouvre-le. Sur Linux (x86_64), télécharge
`fanoratana-linux-x86_64`, rends-le exécutable (`chmod +x`) et lance-le. Les versions suivantes s'installent depuis l'app.

---

{message('en')}

<details><summary>Journal</summary>

{bloc.split('### Message fr')[0].strip()}

</details>
"""
open(os.path.join(d, "notes.md"), "w", encoding="utf-8").write(notes)
PY

openssl pkeyutl -sign -rawin -inkey "$cle" -in release-out/release.json | base64 -w0 > release-out/release.json.sig
(cd release-out && sha256sum fanoratana-* release.json > SHA256SUMS.txt)
FANO_RELEASE="$PWD/release-out" cargo test -q -p fano-maj signature_de_la_release
ls -la release-out
