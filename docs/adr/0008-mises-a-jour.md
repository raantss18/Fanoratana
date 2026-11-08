# 0008 — Mises à jour dans l'app

- **Date** : 2026-11-08 · **Statut** : accepté (Antsa : « les mises à jour en interne sans passer par GitHub »)
- **Décision** : l'app lit `release.json` (+ `release.json.sig`, Ed25519) aux pièces jointes de la dernière version, une fois par jour
  (réglage) ou à la demande ; un bandeau propose « Installer » ; le fichier est téléchargé, son SHA-256 vérifié, puis installé
  (installeur d'Android ; sur Linux, remplacement du programme, actif au prochain lancement). Personne n'ouvre de page web.
- **Clés** : `release-keys/` (jamais dans Git) ; clé publique dans `brand/maj-ed25519.pub`, embarquée.
- **Réseau** : seul usage du réseau ; tout le reste marche hors ligne.
