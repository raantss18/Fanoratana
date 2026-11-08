# 0002 — Rust, egui, comme mahay

- **Date** : 2026-11-08 · **Statut** : accepté (choix d'Antsa : « Rust egui (mahay) »)
- **Décision** : Rust 2024, egui/eframe 0.36 + wgpu ; une seule interface (`fano-ui`) pour Linux et Android ; la passerelle Android
  (`fano-android`, activité Java `FanoActivity`, `ApkInstaller`) est reprise de mahay (MIT/Apache-2.0) et allégée de Mahay HUB.
- **App autonome** (Antsa) : son dépôt, ses versions, ses mises à jour ; pas une app de la suite mahay.
- **Découpage** : `fano-core` (modèle, coffre), `fano-ink` (encre, formes, rature), `fano-reco` (lecture), `fano-render` (rendu, export),
  `fano-maj` (mises à jour), `fano-ui` (écran), `apps/fanoratana` (Linux), `apps/fanoratana-android` (APK).
