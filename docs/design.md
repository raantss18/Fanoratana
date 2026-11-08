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

## Écran

```
┌────┬──────────────────────────────────────────┐
│ ▤  │  Niveau › Chapitre — Leçon               │  ← fil discret, gris
│ ✎● │   ┌────────── feuille ──────────┐        │
│ ○△ │   │  Titre souligné à la main   │        │
│ ⌨  │   │  texte au propre (Caveat)   │        │
│ ⬚  │   │  ┌ Définition ───────────┐  │        │
│ ⌫  │   │  └───────────────────────┘  │        │
│ ●  │   │  [1789] ───→ [1792]         │        │
│ ✦  │   └─────────────────────────────┘        │
│ ▶  │                                          │
│ …  │                                          │
│ ↶↷ │                                          │
└────┴──────────────────────────────────────────┘
```

- Le **rail** (68 dp) porte tout : Mes cours, les cinq outils, la pastille couleur/épaisseur, Mettre au propre, Présenter, Plus,
  Annuler/Rétablir en bas. Le reste de l'écran est à la feuille.
- **Mes cours** s'ouvre en panneau à gauche ; **l'éditeur clavier** en carte en haut ; les **messages** en bandeau noir en bas.
- **Le seul moment fort** : l'écriture qui se met au propre. Pas d'autre animation.
- **Encadrés** monochromes : l'étiquette et le cadre les distinguent (À retenir : cadre épais et léger fond).

## Captures

`FANO_CAPTURES=docs/captures cargo test -p fano-ui --test captures` (paysage, portrait, Mes cours).
