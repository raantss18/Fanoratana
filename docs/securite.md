# Sécurité et robustesse — audit du 2026-11-15

Relecture indépendante du code (fichiers hostiles, chemins, mises à jour, Android) et `cargo audit` sur les deux
fichiers de verrouillage (`Cargo.lock`, `apps/fanoratana-android/Cargo.lock`).

## Vérifié (corrigé, avec un test)

| Défaut | Correction | Test |
|---|---|---|
| Une leçon reçue pouvait faire lire n'importe quel fichier (`ImageEl.fichier`, `fond_image` absolus ou avec `..`), affiché et exporté | `coffre::chemin_media` : relatif, sous `medias/`, sans `..` ; utilisé à l'écran et à l'export | `une_image_ne_sort_pas_du_coffre` |
| Grille d'un repère sans fin (`x + pas == x` en f32) : app figée, mémoire épuisée | nombre de traits compté (`GRILLE_MAX`), bornes assainies | `un_repere_aux_bornes_folles_se_dessine_quand_meme` |
| Page `Libre` sans limite (fond et export en gigaoctets) | côté ≤ 5000 pt à la lecture | `un_fichier_abime_devient_une_lecon_sure` |
| Leçon sans page, tableau aux cases manquantes : plantage | `Lecon::assainir` à chaque lecture | idem |
| Leçon d'une version plus récente réécrite en perdant des champs | refusée à la lecture | `une_lecon_d_une_version_plus_recente_est_refusee` |
| Enregistrement raté seulement journalisé, puis leçon remplacée : travail perdu | message, et la leçon reste ouverte | `une_lecon_non_enregistree_n_est_jamais_remplacee` |
| Formule très imbriquée (`((((…`, `----…`) : débordement de pile | `expr::trop_profond` avant toute analyse | `une_formule_sans_fond_ne_fait_pas_tomber_l_app` |
| PDF à page géante : gigaoctets ; panique du lecteur : import bloqué pour toujours | page ≤ 3000 pt, document ≤ 200 Mo, `catch_unwind` | `une_page_geante_est_reduite` |
| SVG inséré lisant un fichier du disque (`<image href>`) | seules les images `data:` sont lues, à l'écran et à l'export | `un_svg_ne_lit_aucun_fichier_du_disque` |
| Image décodée en entier pour connaître sa taille, sans limite | en-tête seulement ; ≤ 16 000 px et 512 Mo au décodage | `la_taille_se_lit_sans_decoder_et_le_reste_est_refuse` |
| Écriture « atomique » sans `sync_all` : leçon vide après une coupure | `sync_all` avant le renommage | (pas de test : coupure de courant) |
| Android : flux sans fin envoyé par une autre app (activité exportée) | copie arrêtée à 200 Mo, fichier partiel effacé | (pas de test automatique) |
| Android : sauvegarde automatique saturée par les modèles (117 Mo) et les APK | `fanoratana/modeles/` et `fanoratana/mises-a-jour/` exclus | (à vérifier sur appareil) |
| Nom de fichier de mise à jour pris tel quel (signé) | dernier segment seulement | — |
| `fin.t - avant.t` pouvait déborder | `saturating_sub` | — |

## Vérifié (rien à corriger)

- Mises à jour : `release.json` signé Ed25519, SHA-256 du binaire, lectures bornées, version strictement croissante.
- Modèle des formules : révision, taille et SHA-256 épinglés.
- `nom_sur` / `importer_media` : noms assainis à l'écriture. Android : seuls les URI `content://` sont acceptés.
- `cargo audit` (2026-11-14) : **aucune vulnérabilité connue**. Cinq avis « non maintenu », sans faille déclarée :
  `rustybuzz` 0.20 et `ttf-parser` 0.25 (rendu du texte, via usvg), `zbus_names`, `zvariant`, `zvariant_derive`
  (boîte de dialogue de fichiers sur Linux, via rfd). À remplacer quand leurs utilisateurs (usvg, rfd) migrent.

## Reste à faire

- Copie Android d'un document reçu faite sur le fil de l'interface (bornée à 200 Mo, mais un gros fichier fige l'écran).
- Textures jamais libérées : un PDF de 200 pages parcouru en entier garde environ 8 Mo par page vue.
- Annuler garde jusqu'à 100 copies de toutes les pages ; à réduire à la page touchée.
- Modèle des formules : présence vérifiée à la taille après téléchargement, pas à l'empreinte à chaque lancement.
- Dépendances Java (ML Kit, AndroidX) : pas d'outil d'audit lancé ; versions à suivre à la main.
