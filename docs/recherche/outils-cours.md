# Outils d'un cahier de cours manuscrit : gestes, tableaux, mise en forme, formes, GeoGebra, formules (étude du 2026-11-08)

> Pages sources lues le 2026-11-08. « Vérifié » = lu sur la page de l'éditeur, le dépôt ou l'article. « Extrait » = lu
> seulement dans un extrait de recherche (page source illisible ou bloquée). « Non vérifié » = rien trouvé de première
> main. Aucune mesure faite sur appareil : rien ici ne dit ce que donnera Fanoratana.

## 1. Structurer un document sans clavier : gestes des applications

| Geste | Nebo / MyScript Notes | OneNote | Apple Scribble | Goodnotes 6 | Samsung Notes | Notability |
|---|---|---|---|---|---|---|
| Gribouiller dessus = effacer | **oui** (« scratch out », vérifié) | **oui**, gribouillis « en boucles ou zigzag », désactivable (vérifié) | **oui** (vérifié) | **oui**, « Scribble to Erase », désactivable (extrait) | non : bouton du S Pen maintenu + toucher (vérifié) | non vérifié |
| Barrer d'un trait = supprimer | oui, action réglable (vérifié) | — | — | — | — | — |
| Trait vertical **vers le bas** = couper (mot, ligne, nouveau paragraphe) | **oui** (vérifié) | non vérifié | trait vertical = joindre **ou** séparer des caractères (vérifié) | — | — | — |
| Trait vertical **vers le haut** = joindre | **oui** (vérifié) | — | (même geste, voir ci-dessus) | — | — | — |
| Insérer de l'espace | via « couper » (extrait) | non vérifié | **appui long**, puis écrire dans l'espace ouvert (vérifié) | — | — | — |
| Entourer = sélectionner | non : outil lasso (vérifié) | outil Lasso (vérifié) | **entourer ou souligner** (vérifié) | « Circle to Lasso » au stylo (extrait) | mode sélection lasso/rectangle (vérifié) | non vérifié |
| Souligner | trait simple ou double = soulignement (vérifié, page MyScript Notes) | — | souligner = sélectionner | — | — | — |
| Titre | **double soulignement = H1, simple = H2** (extrait, page Nebo plus ancienne) ; aujourd'hui menu du bloc → Titre / Sous-titre (vérifié) | — | — | — | — | — |
| Liste à puces | **tiret en début de ligne** (extrait) ; puces reconnues •, ○, *, → (extrait) ; menu du bloc → Liste à puces / numérotée / cases (vérifié) ; numéros `1.`, `1-`, `a)`, `a.`, `A)` (vérifié) | — | — | — | — | — |
| Retrait | lasso → Indent… → Augmenter / Diminuer (vérifié) ; 5 niveaux (extrait ; une annonce 2020 dit 6) | — | — | — | — | — |
| Surligner | encadrer le texte au stylo (vérifié) | — | — | — | — | — |
| Forme parfaite | **tracer puis garder le stylo immobile** (extrait) ; ou lasso → Convertir | **« Draw and Hold »** : trait unique tenu → forme nette, encore redimensionnable ; flèche, cercle, ellipse, hexagone, ligne, pentagone, rectangle, losange, carré, trapèze, triangle (vérifié) | — | « draw and hold » (forum : erreurs de détection signalées) | non vérifié | maintien du crayon (forum, non officiel) |
| Diagramme | bloc diagramme (menu + ou double-tap dans le vide) ; connexions seulement en mode Document ; double-tap = tout convertir (extrait) | — | — | — | — | — |
| Encre → texte | double-tap / Convertir (vérifié) | stylo « Ink to Text » (Windows 11 23H2, français inclus) (extrait) | écriture directe dans les champs | — | lasso → « T » Convertir en texte ; « Aucun texte à reconnaître » si illisible (vérifié) | — |
| Encre → maths | MyScript Math (app séparée) | Lasso → Dessin → **Math** → corriger en entourant le symbole → « Convertir l'encre en maths » (vérifié) | — | — | — | — |

**Retours d'usage (peu de sources de première main)** : Nebo ne permet pas de désactiver les gestes (extrait, support
MyScript) ; « couper/joindre » ne marchent pas sur l'encre non convertie des Notes et PDF (extrait). Chez Goodnotes, le
« draw and hold » « peut mener à une détection incorrecte » (forum Goodnotes, demandes de choisir la forme avant de
tracer). Microsoft et Goodnotes rendent l'effacement par gribouillis désactivable : signe qu'il produit des faux positifs.
Le motif commun qui revient partout : **gribouiller = effacer, entourer = sélectionner, tenir = forme parfaite** ; le
reste (titres, listes) passe de plus en plus par **un menu du bloc** plutôt que par un geste.

## 2. Tableaux

| Application | Créer | Remplir / agrandir |
|---|---|---|
| Nebo / MyScript Notes | **aucun outil tableau trouvé** dans l'aide ; colonnes en redimensionnant des blocs de texte côte à côte (extrait) | — |
| OneNote | Insérer → Tableau → **grille survolée** pour choisir lignes × colonnes (vérifié) ; au clavier, Tab après une ligne de texte crée un tableau (extrait) | Ctrl+Entrée = ligne dessous (extrait) ; menu Tableau au clic droit (vérifié) |
| Excalidraw | **pas de tableau** : ticket #4847 « Please add a "table" template » ouvert depuis 2022-02-26 (146 commentaires), #3762 (coller des données en tableau) ouvert (vérifié, API GitHub) | — |
| Obsidian | Markdown : `| a | b |` + ligne `| -- | -- |` (au moins deux tirets), alignement `:--`, `:-:`, `--:`, `\|` dans une cellule ; commande « Insert Table » ; clic droit en aperçu = ajouter/supprimer lignes et colonnes, trier, aligner (vérifié) | idem |

Aucune application étudiée ne reconnaît un **tableau dessiné à main levée** de façon documentée. Le chemin le plus sûr
est celui d'OneNote : menu lignes × colonnes, puis chaque case = une petite zone d'écriture. Un geste « rectangle puis
traits intérieurs » reste à inventer et à mesurer (non vérifié ailleurs).

## 3. Mise en forme et interface tactile sobre

Outils utiles à un cours, d'après Nebo (vérifié) : titre / sous-titre / paragraphe, listes à puces, numérotées et à
cocher, retrait ±, souligné, surligné (stylo ou surligneur « libre » / « intelligent »). Samsung Notes ajoute le
**redressement** de l'écriture (aligner, espacer les mots, corriger les lettres) (vérifié).

Repères Material 3 (valeurs lues dans les jetons `androidx.compose.material3`, 2026-11-08) :
- cible tactile **48 dp minimum** (vérifié, developer.android.com) ;
- élément de menu **48 dp** de haut (`MenuListItemContainerHeight`), élément de liste une ligne **56 dp**
  (`ListTokens.ItemOneLineContainerHeight`) ;
- barre d'outils **64 dp** (docked et floating), **4 dp** entre actions (`FloatingToolbarTokens.ContainerBetweenSpace`) ;
- interrupteur : piste **52 × 32 dp**.
- Barre **docked** = actions globales, pleine largeur ; barre **flottante** = actions contextuelles au contenu, peut être
  **verticale** sur grand écran, ne déborde pas de la fenêtre ; surplus → menu de débordement (Google cité par 9to5Google,
  page m3.material.io illisible sans JS).
- Interrupteur = effet immédiat ; cases à cocher pour plusieurs options dans une liste (règle Material **1**, mdui.org ;
  PatternFly dit pareil) — non relu dans M3.

Traduction pour Fanoratana : la mise en forme est **contextuelle** (barre flottante au-dessus de la sélection, ≤ 6
actions + « … »), le reste dans des **menus en liste** de 48 dp ; les réglages persistants en interrupteurs.

## 4. Signature ou dessin libre vs forme géométrique

| Source | Critère | Seuils publiés |
|---|---|---|
| **Windows Ink Analysis** (`InkAnalysisDrawingKind`, vérifié) | un nœud `InkDrawing` porte une forme reconnue ou **`Drawing` = « not recognized as one of the shapes »** | 14 formes : cercle, ellipse, triangle (isocèle, équilatéral, rectangle), quadrilatère, rectangle, carré, losange, trapèze, parallélogramme, pentagone, hexagone ; seuils non publiés |
| **ShortStraw** (Wolin, Eoff, Hammond, SBIM 2008, PDF lu) | coins = minima locaux de la « paille » | rééchantillonnage tous les **diagonale/40** ; fenêtre **W = 3** ; seuil **t = médiane × 0,95** ; segment droit si corde/chemin **> 0,95** ; 0,979 de coins justes (leur mesure) ; manque les angles obtus |
| **Bhat & Hammond**, « Using Entropy to Distinguish Shape versus Text », IJCAI 2009 (PDF lu) | **entropie** des angles : 7 symboles (6 tranches de π/6 sur [0, π) + X aux extrémités), points espacés de **4 px** ; texte = plus d'entropie | regroupement temporel **100 ms** (400 ms si chevauchement) ; confiance `0,5 + arctan(x − b)/π` avec **b = 3,55** (échelle « multipliée par un facteur », non donnée) ; 92,06 % forcé, 95,56 % en laissant ≤ 25 % non classés |
| **PaleoSketch** (Paulson & Hammond, IUI 2008) — article non lu, résumés seulement | **NDDE** (distance entre directions max et min / longueur ; proche de 1 = courbe), **DCR** (plus grand changement de direction / moyen ; élevé = polyligne) ; 8 primitives (ligne, polyligne, cercle, ellipse, arc, courbe, spirale, hélice) | **seuils non vérifiés** ; 98,56 % (blog) |
| Auto-intersections | — | aucun seuil publié trouvé (non vérifié) |

Règle déduite (à mesurer) : une forme est **peu de coins (ShortStraw), faible entropie, bon ajustement** à une primitive ;
une signature a **forte entropie, beaucoup de coins/inversions, auto-intersections**. Laisser une zone « non classé » →
encre gardée telle quelle (comme Bhat & Hammond et `Drawing` chez Microsoft). Ne jamais « redresser » sans geste de maintien.

## 5. GeoGebra : syntaxe à reconnaître en priorité

Saisie (manuel FR, dépôt `geogebra/manual`, vérifié) : point `A = (1, 2)` (polaire `(1; 0°)`), vecteur `u = (0, 5)`,
fonction `f(x) = 3 x^3 – x^2`, restriction `Si(3≤x≤5, x^2)`, nommage `S = Intersection(g, h)`, indices `A_1`. Le manuel
actuel écrit les commandes **avec parenthèses** (l'anglais montre encore `Intersect[g, h]`).

| Français (manuel FR) | Anglais | Formes |
|---|---|---|
| `Segment` | Segment | `(A, B)` |
| `Droite` | Line | `(A, B)`, `(A, d)` parallèle, `(A, v)` |
| `DemiDroite` | Ray | |
| `Cercle` | Circle | `(C, r)`, `(C, A)`, `(C, s)`, `(A, B, C)` |
| `Polygone` | Polygon | `(A, B, C, …)`, `(A, B, n)` régulier |
| `Vecteur` | Vector | `(A)`, `(A, B)` |
| `Intersection` | Intersect | `(objet, objet)` |
| `MilieuCentre` | Midpoint | `(A, B)` ; aussi `M = (A + B)/2` |
| `Médiatrice`, `Perpendiculaire`, `Bissectrice`, `Tangente` | PerpendicularBisector, PerpendicularLine, AngleBisector, Tangent | |
| `Angle`, `Distance`, `Longueur` | Angle, Distance, Length | |
| `Racine`, `Extremum`, `Dérivée`, `Intégrale`, `Résoudre` | Root, Extremum, Derivative, Integral, Solve | |

Commandes anglaises acceptées quelle que soit la langue : affirmé par une ressource communautaire geogebra.org, **non
vérifié** dans le manuel. Priorité pour un cours manuscrit : `A = (x, y)`, `f(x) = …`, puis Segment, Droite, Cercle,
Polygone, Vecteur, Intersection, MilieuCentre ; le reste plus tard.

## 6. Formules manuscrites → LaTeX : ce qui change depuis Fikajiana

**Nouveau** : `breezedeus/pix2text-mfr-1.5` (révision `1cef9f0b…`, 2025-07-25, **MIT**, non restreint) est le modèle par
défaut de Pix2Text (`model_name='mfr-1.5'` dans `latex_ocr.py` ; Pix2Text v1.1.7 du 2026-08-23). Fikajiana utilise
toujours **1.0** (`pix2text-mfr`, révision `bea257ed…`, 2024-05-05). Écarts lus dans les configs HF :

| | 1.0 (Fikajiana) | 1.5 |
|---|---|---|
| `encoder_model.onnx` / `decoder_model.onnx` | 87 496 990 / 30 114 937 o | 87 510 770 / 32 026 253 o |
| `tokenizer.json` (BPE ByteLevel) | 1 200 jetons | **1 868 jetons** |
| `decoder_start_token_id` | **2** (`</s>`) | **1** (`<s>`) |
| `eos_token_id` / pad | 2 / 0 (`config.json`) | 2 / 0 |
| `max_new_tokens` ; positions | 512 ; 512 | 1 024 ; 1 024 |
| Prétraitement | identique : `DeiTImageProcessor`, 384 × 384, bicubique (`resample: 3`), `x/255`, moyenne 0,5, écart 0,5, pas de recadrage | idem |

Jetons spéciaux identiques : 0 `<pad>`, 1 `<s>`, 2 `</s>`, 3 `<unk>`, 4 `<mask>`. **Passer à 1.5 sans changer le jeton
de départ casse le décodage.** Précision manuscrite de 1.5 : graphique de CER non lu → non vérifié. Provenance des
données toujours non publiée (même réserve que Fikajiana).

**Chaîne exacte de Fikajiana** (`app/lib/reconnaissance/{lecteur,rasteriser,pix2text,modele}.dart`) :
1. **Rastérisation** : encre noire sur blanc, cadre de l'encre, hauteur mini **48**, marge **12 %** de la hauteur,
   épaisseur **hauteur/70** (un trait épais fait lire `1` comme `Λ`), bouts et jointures ronds, grand côté **1 536 px**.
   `lireCourt` (case de matrice, nombre seul) élargit l'image à **3 × la hauteur** pour qu'un chiffre ne soit pas étiré.
2. **Tenseur** : l'image est **étirée en carré 384 × 384** sans garder les proportions (comme le processeur HF),
   `FilterQuality.high` ; RGBA → planaire RGB `[1, 3, 384, 384]`, `v/127,5 − 1` (= `(x/255 − 0,5)/0,5`).
3. **Encodeur** : entrée `pixel_values`, sortie 0 = `encoder_hidden_states`.
4. **Décodage glouton** (`lire`) : jetons = `[2]` ; à chaque pas, décodeur **sans cache** sur toute la suite
   (`input_ids` int64 `[1, n]` + `encoder_hidden_states`), argmax de la **dernière ligne** des logits ; arrêt sur **2** ou
   à **256** jetons.
5. **Faisceau** (`lirePlusieurs`, lectures douteuses) : largeur **3**, log-softmax, score **moyenné par la longueur**,
   départ = fin = 2, **96** jetons maximum.
6. **Détokenisation** : ids ≤ 4 ignorés ; table inverse ByteLevel (octets 0x21–0x7E, 0xA1–0xAC, 0xAE–0xFF directs, les
   autres décalés à 256 + k) ; UTF-8 tolérant, `trim()`.
7. **Comparaison** : `normaliserLatex` retire `\,` `\;` `\:` `\!` `\ `, `\left`/`\right`, `\limits`, espaces ;
   `\dfrac`/`\tfrac` → `\frac` ; `\mathrm{x}` → `x`.
8. **Fichiers** : téléchargés à la demande, vérifiés par SHA-256 ; int8 local (MatMul/Gemm) 23 966 185 + 9 305 287 o.

## Recommandations pour la 0.0.1

1. Trois gestes seulement, partout : **gribouiller = effacer** (désactivable), **entourer = sélectionner**, **tenir =
   forme parfaite** ; titres et listes par un **menu du bloc** (pas de double soulignement en 0.0.1).
2. Barre flottante contextuelle de 64 dp (≤ 6 actions + « … »), menus en liste de 48 dp, cibles ≥ 48 dp.
3. Tableau = **menu lignes × colonnes** (grille), chaque case = zone d'écriture lue par `lireCourt` ; « + ligne » en bas.
4. Classeur forme/encre : ShortStraw (W = 3, 0,95 × médiane, diag/40) + entropie Bhat (4 px, 7 symboles) avec une
   zone **non classé = encre brute** ; seuils à régler sur nos traits.
5. GeoGebra : reconnaître d'abord `A = (x, y)`, `f(x) = …`, Segment, Droite, Cercle, Polygone, Vecteur, Intersection,
   MilieuCentre ; écrire en français, exporter avec parenthèses.
6. Formules : porter la chaîne Fikajiana telle quelle en Rust (`ort`), avec **1.0** ; essayer 1.5 seulement en changeant
   le jeton de départ (1) et le vocabulaire, puis comparer sur le même corpus manuscrit.
7. Mesurer sur la tablette avant toute promesse de latence ou de justesse.

## Vérifié / Supposé / Reste à tester

- **Vérifié** : gestes Microsoft, Apple, MyScript Notes, Samsung ; tableaux OneNote (menu), Obsidian, tickets Excalidraw ; jetons
  M3 ; `InkAnalysisDrawingKind` ; seuils ShortStraw et Bhat & Hammond ; manuel GeoGebra FR ; configs HF 1.0 et 1.5 ; code Fikajiana.
- **Supposé / extrait** : gestes Nebo (tiret, double soulignement, maintien), Goodnotes, raccourcis OneNote, guidance toolbars M3.
- **Reste à tester** : seuils PaleoSketch, commandes anglaises en version FR, précision manuscrite de 1.5, tout sur la tablette.

## Sources

help.myscript.com/notes/edit-content/pen-gestures-and-interactions/ · help.myscript.com/notes/customize-content/basic-formatting/ · help.myscript.com/nebo/customize-content/basic-formatting ·
help.myscript.com/nebo/create-content/diagrams · app-support.myscript.com/support/solutions/articles/16000109690-lists-bullets-and-indentation · app-support.myscript.com/support/solutions/articles/16000088325-columns ·
support.microsoft.com/en-us/OneNote/onenote-help-and-learning/ink-faster-with-gestures-for-creating-shapes-deleting-ink-and-more · support.microsoft.com/en-us/onenote/convert-your-ink-to-shapes-and-math-equations ·
support.microsoft.com/en-us/education/onenote/create-math-equations-using-ink-or-text-with-math-assistant-in-onenote · support.microsoft.com/en-us/OneNote/onenote-help-and-learning/insert-a-table ·
support.microsoft.com/help/d865a2ea-9b2d-4738-afe7-157ce309721a · support.apple.com/guide/ipad/aside/ipad355ab2a7/ipados · support.goodnotes.com/hc/en-us/articles/7443501398671 ·
feedback.goodnotes.com/forums/191274-customer-suggestions-for-goodnotes/suggestions/39359695-hold-stroke-for-perfect-shape · www.samsung.com/us/support/answer/ANS10003634 · github.com/excalidraw/excalidraw/issues/4847 ·
github.com/excalidraw/excalidraw/issues/3762 · obsidian.md/help/advanced-syntax · developer.android.com/develop/ui/compose/accessibility/api-defaults · github.com/androidx/androidx (compose/material3/…/tokens, Menu.kt) ·
9to5google.com/2025/05/18/material-3-expressive-toolbars/ · mdui.org/design/components/selection-controls.html · learn.microsoft.com/en-us/uwp/api/windows.ui.input.inking.analysis.inkanalysisdrawingkind ·
www.cs.ucf.edu/courses/cap6105/fall2010/readings/Wolin_Eoff_Hammond_SBIM08.pdf · www.ijcai.org/Proceedings/09/Papers/234.pdf · jasmeet13n.blogspot.com/2015/10/reading-17-paleosketch-accurate.html ·
github.com/geogebra/manual (fr/modules/ROOT/pages) · geogebra.github.io/docs/manual/fr/commands/Commandes_Transformations · geogebra.org/m/w49ynjXj · huggingface.co/breezedeus/pix2text-mfr · huggingface.co/breezedeus/pix2text-mfr-1.5 ·
github.com/breezedeus/Pix2Text (pix2text/latex_ocr.py, utils.py) · ~/GitHub/Fikajiana/app/lib/reconnaissance/{lecteur,rasteriser,pix2text,modele}.dart · ~/GitHub/Fikajiana/models/pix2text-mfr/*.json
