# TODO — Fanoratana

> Seule source d'avancement. ✅ fait et vérifié · 🟡 fait, à vérifier sur la tablette · ⬜ à faire.

## 0. Erreurs signalées (ne reviennent jamais)

- ✅ Un tableau inséré se posait sous le dernier élément, hors de l'écran, puis par-dessus le texte (Pad 7, 2026-11-08) : espace
  libre visible, sinon défilement jusqu'à lui.

- ✅ Page cadrée pendant la fenêtre flottante, jamais recadrée en plein écran (Pad 7, 2026-11-08) : recadrage quand la largeur change.
- ✅ Trait aux points espacés déformé par le lissage (rectangle devenu parallélogramme, Pad 7, 2026-11-08) : `points_espaces_non_deformes`.

- ✅ Le clavier d'Android s'ouvrait et se fermait en boucle dans « Nouvelle leçon » (focus redemandé à chaque image) : des lettres
  se perdaient (« Essai du cours » → « Esa dcours ») et l'app a fini par se fermer (Pad 7, 2026-11-08). Focus demandé une seule fois.

## 1. Version 0.0.1 (2026-11-08)

- ✅ Conception : questions à Antsa, recherche sourcée (`docs/recherche/reconnaissance.md`), ADR 0001–0009, identité (ADR 0006).
- ✅ Coffre niveau → chapitre → leçon, recherche, corbeille, niveaux type (`fano-core`, tests).
- ✅ Encre à pression (simulée au doigt), formes ShortStraw, flèches, rature, lignes et mots (`fano-ink`, tests).
- ✅ Lecture hors ligne PP-OCRv5 latin par tract, sur un fil (`fano-reco`, test bout en bout : 4/4 sur traits Caveat).
- ✅ Rendu au propre : Caveat, formules `$…$`, trait fait main, encadrés ; export SVG/PNG/PDF (`fano-render`, tests).
- ✅ Interface egui : rail, toile, coffre, éditeur clavier, menu Plus, diapos, raccourcis ; captures headless.
- ✅ APK arm64 (21,7 Mo) installé et lancé sur le Pad 7 sans stylet ; « Bonjour » injecté lu juste.
- ✅ Mises à jour dans l'app : release.json signé Ed25519, SHA-256, installeur Android / remplacement Linux.
- ✅ Cours écrit sur la tablette (gestes injectés, nouvelle leçon « Essai du cours », 2026-11-08) : titre, phrase et `f(x)=2x+1` lus
  justes ; triangle tenu → forme ; signature → reste de l'encre ; rature d'une ligne → effacée ; `f(x)=2x+1` → courbe tracée ;
  modèle des formules téléchargé (SHA-256 vérifiés) ; `x²+1=5` écrit au fil → `$x^{2}+1=5$` ; tableau inséré, « Date » et « 1789 »
  écrits à la main dans les cases.
- ✅ Tableaux, mise en forme (listes, alignement, souligné, surligné), courbes GeoGebra, formules LaTeX, menus en listes (ADR 0010, 0011).
- ✅ Gras reconnu à l'épaisseur (≥ 1,45 × l'écriture habituelle de la page) ; fin de page → « + Ajouter une page » (essayé sur le
  Pad 7) ; page ajustée à la largeur et bloquée sur ses bords sauf zoom.
- 🟡 Gras : à mesurer sur de vraies écritures (le seuil 1,45 est une hypothèse ; repasser deux fois sur un mot pas encore pris en compte).
- 🟡 Essai complet au doigt par Antsa : rature, gomme, formes tenues, sélection, diapos, export PDF vers Documents.
- 🟡 Mise à jour réelle de 0.0.1 vers 0.0.2 sur la tablette (la 0.0.1 est la première : rien à tester avant la 0.0.2).

## 2. Lecture (priorité)

- ⬜ Mesurer la justesse sur l'écriture réelle (200 lignes d'Antsa, au doigt) : CER, mots justes, latence sur le Pad 7.
- ⬜ PyLaia (MIT) affiné sur des lignes synthétiques françaises et malgaches (polices OFL) puis sur l'écriture donnée en CC0 ; seuil
  d'adoption : CER ≤ 10 %, < 100 ms par mot.
- ⬜ Apprendre des corrections au clavier, sur l'appareil (comme Fikajiana, ADR 0038 là-bas).
- ⬜ Formules : tester Pix2Text-MFR 1.5 (MIT, jeton de départ 1, vocabulaire 1 868) ; versions int8 (24 + 9 Mo) pour aller plus vite.
- ⬜ Commandes GeoGebra de construction (Cercle, Segment, Droite, Polygone, Vecteur, Intersection) dans le repère.
- ⬜ Échap ne ferme pas un menu ouvert sur Android (clavier physique) : à vérifier.
- ⬜ Code manuscrit : lexique de mots-clés, indentation par la position du tracé, coloration.
- ⬜ Lecture page entière sur PC en option (GLM-OCR, MIT).

## 3. Cours

- ⬜ Nœuds et arêtes : un cercle avec une étiquette relié par des flèches = un graphe ; arbres rangés automatiquement.
- ⬜ Tableaux à main levée ; listes à puces reconnues ; liens entre leçons `[[…]]` façon Obsidian.
- ⬜ Redimensionner et faire pivoter une sélection ; copier-coller ; aligner.
- ⬜ Modèles de leçon (cours, fiche d'exercices, évaluation) ; numérotation automatique des exercices.
- ⬜ Interface en malgache.

## 4. Plateformes

- ⬜ Paquets Linux (AppImage, .deb) ; icône et entrée de menu installées.
- ⬜ Accessibilité : lecteur d'écran (TalkBack) sur le rail et le coffre ; vérifier les contrastes.
- ⬜ Mode sombre de l'interface (la feuille reste blanche).
