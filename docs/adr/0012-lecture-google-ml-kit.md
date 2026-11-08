# 0012 — Lecture de l'écriture par ML Kit (Gboard) sur Android

- **Date** : 2026-11-08 · **Statut** : accepté (Antsa : « un modèle comme le handwriting de Google keyboard… Oui, ML Kit »)
- **Constat** : PP-OCRv5 (image) lit mal la cursive d'Antsa au stylet (« Probabhte », « thionime », arbre illisible).
  Gboard lit les **traits** (en ligne) : Carbune et al. 2020 (https://arxiv.org/abs/1902.10525). Ses poids ne sont pas
  publiés ; les conditions de ML Kit interdisent d'en extraire code ou modèles (https://developers.google.com/ml-kit/terms).
- **Décision** : sur Android, **ML Kit Digital Ink Recognition 19.0.0** (« la même technologie que Gboard »), langues `fr`,
  `en`, `mg` (https://developers.google.com/ml-kit/vision/digital-ink-recognition/base-models), hors ligne une fois le modèle
  téléchargé (~20 Mo par langue). Contexte de la ligne précédente et zone d'écriture transmis. Une ligne qui ressemble à
  une formule est relue en LaTeX par Pix2Text-MFR s'il est là. Linux : la lecture libre (PP-OCRv5 + lexique).
- **Accord** : demandé une fois, révocable dans Réglages ; le texte dit que le modèle vient de Google et que ML Kit envoie
  des statistiques d'utilisation (jamais l'écriture). Refus : lecture libre.
- **Licence** : exception à la GPL (section 7) accordée par l'autrice pour lier ML Kit et ses dépendances Google
  (`LICENSE-EXCEPTION-MLKIT.md`). Le reste du code reste libre ; l'app se construit et marche sans ML Kit (Linux).
- **Mises à jour des modèles** : ML Kit, rafraîchi chaque mois en arrière-plan (`download` sans suppression préalable : le
  modèle reste utilisable) et bouton « Mettre à jour le modèle ». Modèles de Fanoratana : révision et empreinte fixées
  par version de l'app ; une nouvelle révision fait proposer le téléchargement.
- **Fluidité** : lecture hors du fil de l'écran (tâches ML Kit, fil de lecture libre) ; état du modèle relu au plus chaque
  seconde ; modèles lourds chargés seulement s'ils servent.
- **Reste à vérifier** : justesse sur l'écriture d'Antsa (fr, mg), latence sur le Pad 7, taille de l'APK.
