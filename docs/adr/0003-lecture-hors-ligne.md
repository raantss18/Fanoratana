# 0003 — Lecture de l'écriture, hors ligne

- **Date** : 2026-11-08 · **Statut** : accepté pour la 0.0.1, à revoir sur mesure (`docs/recherche/reconnaissance.md`)
- **Décision** : l'encre d'une ligne est rastérisée (48 px) et lue par **latin_PP-OCRv5_mobile_rec** (PaddleOCR, Apache-2.0, ONNX 8 Mo,
  embarqué) exécuté par **tract** (Rust pur, MIT/Apache) : même code sur Linux et Android, aucune bibliothèque native.
- **Mesure** (PC, 2026-11-08) : 4 lignes sur 4 justes sur des traits tirés de Caveat (cas optimiste), 20 à 40 ms par ligne après
  préparation. **Pas encore mesuré** sur une vraie main ni sur la tablette.
- **Sûreté** : sous le seuil de confiance (0,6), l'encre reste ; toucher un texte montre ce qui a été lu, propose les lectures voisines
  et permet de corriger au clavier. L'encre reste gardée (« Revenir à l'encre »).
- **Plus tard** : PyLaia (MIT) affiné sur données libres françaises et malgaches ; Pix2Text-MFR pour les formules (cf. Fikajiana) ;
  apprentissage local des corrections. Exclus : TrOCR et tout poids appris sur IAM/RIMES (non commerciaux).
