# Publier une version (ADR 0008, 0009)

1. Numéro suivant : `tool/version_suivante.py $(cat VERSION)` (chaque chiffre de 0 à 9). Le mettre dans `VERSION` et
   `Cargo.toml` (`[workspace.package] version`) et `apps/fanoratana-android/Cargo.toml`.
2. `CHANGELOG.md` : renommer `## [Non publié]` en `## [X.Y.Z] — date`, écrire `### Message fr` et `### Message en` (2 à 4 lignes sur
   ce que la version permet de faire). Le bandeau de mise à jour de l'app affiche ce message.
3. `cargo test --workspace`, captures, essai sur la tablette `8df629a` **par-dessus la version précédente** (les cours restent).
4. `scripts/release.sh` → `release-out/` : `fanoratana-linux-x86_64`, `fanoratana-arm64-v8a.apk`, `release.json` (+ `.sig`, vérifiée par
   le test `signature_de_la_release`), `SHA256SUMS.txt`, `notes.md`, l'icône et le `.desktop`.
5. Publication (action publique, avec l'accord d'Antsa) :
   `git tag vX.Y.Z && git push origin main vX.Y.Z && cd release-out && gh release create vX.Y.Z fanoratana-* release.json release.json.sig SHA256SUMS.txt fanoratana.svg fanoratana.desktop --title "Fanoratana X.Y.Z" --notes-file notes.md`.
   Les apps lisent `releases/latest/download/release.json` : pas de préversion pour une version à proposer.

Clés (`release-keys/`, jamais dans Git, à sauvegarder ailleurs) : `fanoratana.jks` + `keystore.properties` (signature Android),
`maj-ed25519.pem` (signature des mises à jour ; publique : `brand/maj-ed25519.pub`). **Les perdre empêche toute mise à jour.**
