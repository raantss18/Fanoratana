# 0014 — Texte ou formule : la règle des `$` et l'aiguillage automatique

- **Date** : 2026-11-09 · **Statut** : accepté (Antsa : « je veux que ça distingue automatiquement les formules latex des textes de lettres… ou plus subtilement, trouve une règle pour l'utilisateur pour intégrer à la main des formules »)
- **Constat** : `f(x)=mx+p` lu « pa fo= moto » par Google (il cherche des mots français) ; « sur R » resté une lettre.
- **Règle donnée à l'utilisateur** : une formule écrite entre deux `$` à la main (un S coupé d'un trait vertical, comme en
  LaTeX) est toujours lue en formule (`fano_ink::dollars`, `toile::decouper_dollars`) ; les `$` sont effacés ; un `$`
  ouvert sans son second fait attendre la ligne. Rappelé dans Réglages.
- **Sans `$`** (`fano_reco::aiguillage`) : une ligne est une formule si elle a des signes d'opération (ou `f(x)`) et une
  majorité de non-mots du lexique ; après Google, deux lecteurs d'accord → sûr (0,95) ; sinon structure ≥ 0,6 ; sinon un
  candidat de Google bien formé (parenthèses équilibrées, un signe, pas de mot de plus de trois lettres) ; sinon la
  meilleure formule à vérifier. Dans une phrase : lettre seule → `$f$`, `R N Z Q C` après « sur, dans, de, ∈ » → `$\mathbb{R}$`,
  expression collée → `$f(x)=2x$` ; la lecture brute reste en lecture voisine.
- **Reste** : mesurer sur les pages d'Antsa ; `$` d'un seul trait.
