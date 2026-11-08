# Conception visuelle (ADR 0006)

**Référence d'Antsa** : une main écrit au stylo, une ligne d'écriture sort de la pointe ; trait noir uniforme, bouts ronds, fond blanc.
« Sobre, monochrome, mais smart et bien présentée. »

## Jetons

| Nom | Valeur | Rôle |
|---|---|---|
| Noir | `#000000` | texte, icônes, outil actif (pastille noire, icône blanche), encre par défaut |
| Gris doux | `#5C5C5C` | textes secondaires (6,9:1 sur blanc) |
| Filet | `#D6D6D6` | séparation rail / feuille, bords |
| Bureau | `#F1F1F1` | autour des feuilles |
| Papier | `#FFFFFF` | feuille, rail, fenêtres |
| Lignes du cahier | `#CFCFCF` / `#E8E8E8` | Seyès simplifié, carreaux, points |
| Danger | `#B3261E` | « Supprimer » seulement |

Polices : **Caveat** (OFL) pour tout ce qui est écrit sur la feuille ; **Nunito** (OFL) pour l'interface ; Noto Sans Math (OFL) en repli
pour les symboles. Corps de l'interface 17 px, cibles ≥ 48 dp.

## Écran (refonte d'après Fikajiana, 2026-11-08 — remplace le rail)

> Antsa : « base-toi sur l'interface de Fikajiana ; organise les outils de façon très intelligente ».
> Principe de Fikajiana (ADR 0018 là-bas) : la feuille occupe tout l'écran, sans barre ni rail ; quelques commandes flottent
> aux quatre coins ; chaque commande porte son nom ; un seul bouton Outils à la couleur du stylo.

```
┌──────────────────────────────────────────────────────────────┐
│ (≡) Seconde › Histoire — La Révolution        [+ Insérer] (⋯) │  ← haut : où je suis · ajouter · tout le reste
│                                                              │
│                      la feuille, tout l'écran                │
│                                                              │
│ [⌨ Taper un texte]                   1 / 3      (↶)(↷) (●✎) │  ← bas : clavier · page · annuler · OUTILS
└──────────────────────────────────────────────────────────────┘
```

Ordre de fréquence → place :

| Fréquence | Commande | Où |
|---|---|---|
| à chaque geste | écrire, dessiner, gommer (le stylet / le doigt) | la feuille elle-même |
| très souvent | stylo, couleur, épaisseur, gomme, formes, sélection | **Outils** (bas droite, pouce de la main droite) |
| très souvent | annuler, rétablir | deux ronds à côté d'Outils, un toucher |
| souvent | taper au clavier | pastille bas gauche |
| souvent | insérer : texte, tableau, courbe, image, page | **+ Insérer** (haut droite) |
| parfois | mes cours (niveau › chapitre › leçon), chercher | **≡** et le titre (haut gauche) |
| parfois | présenter, exporter, fond de page | **⋯ Menu** (haut droite) |
| rarement | réglages (doigt, mise au propre, lecture, langue, mises à jour, modèle des formules) | Menu › Réglages, écran à part, en sections |

**Carte Outils** (≤ 320 dp, défile) : stylos en pastilles + épaisseur ; puis lignes nommées : Stylo · Surligneur · Formes ·
Sélection · Gomme ; interrupteur « Le doigt écrit » ; « Lire comme » (Deviner / Texte / Formule) ; Mettre au propre ;
Zoom + / Zoom − / Recentrer (alternative aux deux doigts, WCAG 2.5.7). Choisir un outil referme la carte.

- **Contexte** : une sélection ou un tableau fait apparaître sa barre en haut au centre (± ligne, ± colonne, Tracer, Supprimer).
- **Monochrome** (ADR 0006) : boutons ronds et pastilles gris très clair `#EFEFEF`, ombre douce, icônes au trait noir ;
  l'outil actif en noir plein. La couleur n'apparaît que sur la pastille du stylo.
- **Le seul moment fort** : l'écriture qui se met au propre.

## Captures

`FANO_CAPTURES=docs/captures cargo test -p fano-ui --test captures` (paysage, portrait, Mes cours).
