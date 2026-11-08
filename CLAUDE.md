# CLAUDE.md — Fanoratana

> Cahier de cours manuscrit pour enseignants de **toutes les matières** : on écrit (doigt, stylet, clavier), ça se met au propre.
> Linux + tablette Android. Rust + egui (comme mahay). Esprit de Fikajiana. Relire au début de chaque étape.
> Ordre en cas de conflit : ce fichier > `docs/adr/` > le reste.

## Règles

1. **Rien d'inventé** : toute affirmation technique ou légale cite une source ; chaque livrable sépare Vérifié / Supposé / Reste à tester.
2. **Rien casser** : une décision = un ADR ≤ 15 lignes (`docs/adr/NNNN-titre.md`) ; commits courts en français (Conventional Commits),
   chemins explicites (jamais `git add -A`) ; `main` toujours vert (`cargo test --workspace`).
3. **Ne jamais dire « fini » sans l'avoir fait tourner** : tests, captures (`FANO_CAPTURES=<dir> cargo test -p fano-ui --test captures`),
   puis essai sur une **tablette Xiaomi Pad 7** : `a11fafee` (Antsa y écrit au stylet, 2026-11-08) ou `8df629a` (sans stylet) ;
   jamais d'émulateur.
   Ne pas injecter de gestes pendant qu'Antsa s'en sert.
4. **Libre et hors ligne** : GPL-3.0-or-later, avec une seule exception, ML Kit Digital Ink sur Android, choisie par Antsa
   (ADR 0012, `LICENSE-EXCEPTION-MLKIT.md`) : accord demandé à l'utilisateur, révocable, l'app marche sans ; dépendances et poids sous licence compatible, notés dans `THIRD_PARTY_LICENSES.md` ;
   aucun poids appris sur des données « NC » (IAM, RIMES, MathWriting, CROHME). Aucun compte, pub ni télémétrie (sauf les statistiques de ML Kit, avec accord). Réseau : mises à jour et modèles.
5. **Aucun secret dans Git** : `release-keys/` (keystore Android, clé Ed25519 des mises à jour) reste local ; la perdre empêche les mises à jour.
6. **Identité** (ADR 0006) : monochrome, noir sur blanc, icônes au trait uniforme à bouts ronds, Caveat sur la feuille, Nunito pour l'interface.
7. **Interface** : cibles ≥ 48 dp ; contrastes WCAG 2.2 AA ; couleur jamais seule ; textes fr/en de qualité égale (`fano-ui/src/textes.rs`).
8. **Une erreur signalée ne revient jamais** : l'inscrire dans `docs/TODO.md` § 0, corriger la cause, ajouter le test qui la protège.

## Organisation

```
crates/fano-core     modèle : coffre (niveau/chapitre/leçon), pages, éléments
crates/fano-ink      encre : contour à pression, formes (ShortStraw), rature, lignes et mots
crates/fano-reco     lecture : PP-OCRv5 latin par tract (Rust pur), sur un fil
crates/fano-render   rendu au propre (Caveat, formules, trait fait main), export SVG/PNG/PDF
crates/fano-maj      mises à jour dans l'app (release.json signé Ed25519, SHA-256)
crates/fano-ui       l'écran egui (rail, toile, coffre, éditeur clavier, diapos)
crates/fano-android  passerelle Android reprise de mahay (FanoActivity, ApkInstaller)
apps/fanoratana      Linux ; apps/fanoratana-android : cdylib de l'APK ; android/ : Gradle
```

- **`docs/TODO.md`** : seule source d'avancement. `docs/design.md` : identité et écrans. `docs/RELEASING.md` : publier.
- **Versions** (ADR 0009) : chaque chiffre de 0 à 9 (0.0.9 → 0.1.0, 0.9.9 → 1.0.0) ; `tool/version_suivante.py`. Nouveautés sous
  `## [Non publié]` de `CHANGELOG.md`, avec un message fr/en qui dit ce que la version permet de faire.
- **Réservé à Antsa** : les clés, toute action publique hors du dépôt, un grand changement de cap.
