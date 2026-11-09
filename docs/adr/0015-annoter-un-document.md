# 0015 — Annoter un PDF ou une image : le document en fond de page

- **Date** : 2026-11-09 · **Statut** : accepté (demande d'Antsa du 2026-11-08, TODO § 3, étapes 1 à 3)
- **Choix** : « Insérer › Annoter un PDF ou une image » crée, dans le chapitre de la leçon ouverte, une leçon au format
  `Libre { largeur, hauteur }` (taille de la première page du PDF, ou 595/842 pt de large pour une photo) dont chaque page a
  le document en `fond_image` : une image verrouillée, **pas un élément**, donc ni gomme ni rature ni sélection n'y touchent.
- **PDF** : pages rendues par **hayro 0.7** (Rust pur, MIT OR Apache-2.0, déjà utilisé par mahay) à 2× (144 dpi), sur un
  fil avec la progression ; une page d'une autre taille est centrée sur du blanc ; au plus 200 pages (`PAGES_MAX`).
- **Export** : le fond passe par la même scène que l'écran (`scene::fond_de_page`) ; PDF et PNG gardent le document et
  l'écriture (`pdf_annote_puis_exporte`). Le PDF exporté contient les pages en image, pas le texte d'origine.
- **Refusé pour l'instant** : garder le PDF d'origine en vectoriel sous les annotations (plus net, plus léger) ; à faire
  ensuite (TODO § 3.3). Recherche dans le texte du PDF : plus tard.
- **Reste** : « Ouvrir avec » / « Partager vers » sur Android, glisser-déposer sur Linux, gros PDF rendus à la demande.
