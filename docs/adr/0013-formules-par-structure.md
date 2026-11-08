# 0013 — Lire les formules à leur structure, signe par signe

- **Date** : 2026-11-08 · **Statut** : accepté (Antsa : « les maths dans tout ça ? … le moyen le plus sûr » ; choix « Lecture des formules »)
- **Constat** : Pix2Text-MFR lit la ligne en image et invente (`\overbrace` sur un arbre) ; ML Kit lit des caractères, pas une
  fraction ni un exposant. Fikajiana (même autrice, GPL) lit au tracé : un petit modèle par signe (ADR 0038, 0040, 0041 là-bas).
- **Décision** : `fano-reco::signes` reprend ce modèle (contrat v1, 1,6 Mo, 109 classes, poids entraînés sur six polices OFL et
  des signes d'Antsa ; SHA-256 `a0e7a6bc…c82370`), rendu identique à `rendu.py` (écart moyen 0,4 %). `fano-reco::structure`
  lit la mise en page à la géométrie : fraction, racine, exposant, indice, `lim` repéré à sa flèche `x → a`, `Σ`/`∫` et
  leurs bornes, noms de fonctions, `x` en deux arcs, contexte (`×`/`x`, `0`/`o`, `1`/`l`, parenthèses). LaTeX exact par
  construction ; matrice ou système : `None`.
- **Décision de confiance** : structure ≥ 0,6 (le signe le moins sûr) → prise ; sinon Pix2Text en second avis : d'accord → 0,9 ;
  en désaccord → reste à valider (l'encre reste, « Mettre au propre » force, l'autre lecture en voisine). Après Google,
  la lecture de Google revient si la formule doute.
- **Mesure** (banc de 23 lignes réelles d'Antsa, Pad 7 et téléphone) : 14 justes, **aucune fausse sûre**, 2 à 30 ms par ligne
  sur PC. Les erreurs restantes sont des signes (`y`, `(`, `;`, `e` du téléphone) : modèle entraîné sans son écriture.
- **Reste** : mesurer sur le Pad 7 ; apprendre des corrections sur l'appareil ; matrices et systèmes ; `\vec`.
