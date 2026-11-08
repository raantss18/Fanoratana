# Reconnaissance de l'écriture, des formes et du code, hors ligne et libre (étude du 2026-11-08)

> Complète Fikajiana (`docs/recherche/ocr-manuscrit.md`, `reverification.md`, ADR 0003/0035/0038) sans refaire les
> formules (Pix2Text-MFR, $P, CNN par signe). Licences lues sur les pages sources (API Hugging Face, API GitHub,
> Zenodo, pub.dev) le 2026-11-08. Aucune latence n'a été mesurée sur le Pad 7 : les temps cités sont ceux des éditeurs.
> **Règle reprise de Fikajiana** : un poids est refusé si sa licence n'est pas libre **ou** s'il est entraîné sur des
> données non commerciales (NC). Ce second filtre élimine presque tous les modèles « français manuscrit » publiés.

## 1. Texte manuscrit sur image (HTR)

### 1a. Petits modèles « ligne » (candidats pour l'appareil)

| Modèle (date) | Code / poids | Données d'entraînement (licence) | Taille | Langues | Précision publiée | Mobile | Verdict |
|---|---|---|---|---|---|---|---|
| **PyLaia-Belfort** (Teklia, 2024) | MIT / MIT | Belfort, actes d'état civil XIXᵉ (**CC BY 4.0**) | ckpt 42,8 Mo + LM 6-g 11,3 Mo | fr | CER 9,52 %, WER 23,73 % (avec LM) ; 10,54 / 28,12 sans | CRNN+CTC : export ONNX supposé simple | **retenu comme base à affiner** |
| PyLaia-POPP (Teklia, 2024) | MIT / MIT | recensements français (**CC BY 4.0**) | ckpt 42,7 Mo | fr | CER 16,09 %, WER 34,52 % | idem | base secondaire |
| PyLaia-RIMES (Teklia, 2024) | MIT / MIT | RIMES : **CC BY-NC 4.0** sur Zenodo (le miroir HF dit MIT) | ≈ 43 Mo | fr | CER 3,47 %, WER 10,20 % (LM) | idem | **exclu (données NC)** ; sert de référence de justesse |
| latin_PP-OCRv5_mobile_rec (PaddleOCR 3.x, ONNX officiel 2026-05-26) | Apache-2.0 / Apache-2.0 | non publiées | **ONNX 8,0 Mo** (Paddle 14 Mo) | latin dont fr (exemple de la fiche en français) | 84,7 % ligne exacte (général, imprimé surtout) ; manuscrit anglais : 0,4944 (mobile), 0,5806 (server) | CPU 21,2 ms/ligne (machine serveur de l'éditeur) | **retenu comme témoin immédiat** ; faible sur le manuscrit |
| TrOCR small/base/large-handwritten (Microsoft, 2021 ; fiches 2024-25) | MIT (unilm) / MIT sur *base*, aucune licence sur *small*, *large* | IAM (**« non-commercial research purpose only »**) | small : 246 Mo ; ONNX int8 23 + 41 Mo (Xenova) ; base 333 M param. | en | — | ONNX existant | **exclu (IAM NC)** |
| agomberto/trocr-large-handwritten-fr (2023) | MIT | trocr-large-handwritten (IAM) + POPP + jeu privé | large | fr | CER 9,42 %, WER 23,49 % (test) | lourd | exclu (hérite d'IAM, jeu privé) |
| dj0w/trocr-french-handwriting-v5 (2026-01) | MIT | RIMES (NC) + « custom » | 1,34 Go | fr | CER 0,20 % (validation, non comparable) | lourd | exclu |
| personalizedrefrigerator/trocr-small-fr (2024-12) | **aucune licence** | « custom dataset » + IAM (base) | 246 Mo | fr | CER 1,38 % (éval. interne) | ONNX possible | exclu |
| HTR-Flor (arthurflor23) | MIT / **pas de poids publiés** | à réentraîner | — | — | — | — | code réutilisable seulement |
| Kraken 7.x (Apache-2.0) + CATMuS | Apache / CC BY 4.0 | imprimé (CATMuS-Print) ou médiéval | — | fr ancien | — | — | aucun modèle **manuscrit moderne** français trouvé |

### 1b. Petits VLM « page » (texte + formules mêlés ; recours sur PC ou en arrière-plan)

| Modèle (date) | Licence poids | Taille | Français | Texte+formules | Format local | Verdict |
|---|---|---|---|---|---|---|
| **GLM-OCR** (zai-org, fiche 2026-09) | **MIT** | 0,9 B ; GGUF Q8 950 + 484 Mo | listé | oui (mode `Formula Recognition`) ; OmniDocBench v1.5 94,62 | GGUF (llama.cpp), Ollama | **retenu pour le PC** |
| LightOnOCR-3 (0,8 B / 1 B / 4 B, 2026-10-07) | Apache-2.0 | 0,85 / 1,0 / 4 B | 1 B : oui (v2 « stronger French ») | « handwriting, and math notation » annoncés, chiffres non relevés | v2-1B : GGUF Q8 639 + 449 Mo ; ONNX q4 ≈ 755 Mo | alternative PC ; données distillées d'un VLM enseignant (licence « other ») |
| PaddleOCR-VL-1.6 (2026-05-28) | Apache-2.0 | 0,9 B ; GGUF 936 + 882 Mo | « multilingual » | oui ; OmniDocBench v1.6 96,33 % | GGUF officiel, ONNX communautaire | alternative PC |
| Florence-2-base | MIT | 0,23 B | non annoncé | non | ONNX communautaire | à essayer en affinage seulement |
| Granite-Docling-258M / SmolDocling-256M | Apache-2.0 / CDLA-Permissive-2.0 | 0,26 B | en | oui (doc imprimés) | ONNX communautaire | hors sujet (imprimé, anglais) |
| GOT-OCR2.0 | Apache-2.0 | 0,7 B | non annoncé | oui | — | fine-tune fr « skander-bs » : MIT sans fiche → provenance inconnue |
| Qwen3-VL-2B-Instruct | Apache-2.0 | 2,1 B | généraliste | oui | GGUF | trop lent pour l'instantané (supposé) |
| dots.ocr / DeepSeek-OCR | MIT / MIT | 3,0 B / 3,3 B | multilingue | oui | — | trop lourds pour la tablette |
| olmOCR-2-7B | Apache-2.0 | 8,3 B | — | oui | — | trop lourd |
| Qwen2.5-VL-3B, Nanonets-OCR2-3B | **qwen-research** / dérivé sans licence | 3,8 B | — | — | — | **exclus** |
| MinerU2.5-1.2B | AGPL-3.0 | 1,2 B | — | oui | — | compatible GPL-3 par §13, mais hors liste MIT/Apache/BSD : écarté |
| Chandra, HunyuanOCR | OpenRAIL / Tencent community | 8,8 B / 1,1 B | — | — | — | **exclus** |

Texte + formules mêlés : seuls les VLM page (GLM-OCR, PaddleOCR-VL, LightOnOCR) le font en une passe. Aucun petit
modèle « ligne » ne le fait ; d'où la segmentation au tracé (§ 5). Malgache : **aucun modèle trouvé** ; l'alphabet
latin suffit, il faudra un lexique et des données maison.

## 2. Reconnaissance en ligne (à partir des traits)

| Projet | Licence code / poids | Données | État | Verdict |
|---|---|---|---|---|
| OnlineHTR (Lellep ; Carbune et al. 2020, LSTM-CTC sur dx, dy, dt, n) | MIT / poids sur blog, sans licence | IAM-OnDB (NC) | maintenu (2026-06) ; aucun CER publié | **code réutilisable**, poids exclus |
| xournalpp_htr (plugin Xournal++) | GPL-2.0 | modèles HF « simple-htr », « word-detector » | actif (2026-11) | exemple d'intégration ; poids à vérifier un par un |
| Google InkSight Small-p (2024-25) | Apache-2.0 / Apache-2.0 | IAM, **RIMES**, IMGUR5K, DeepWriting… | TF SavedModel ≈ 690 Mo ; image → encre (sens inverse) | exclu (données NC, lourd, hors besoin) |
| InkFM (Google, arXiv 2025-03) | — | — | aucun poids trouvé | — |
| Carbune et al. « Fast multi-language LSTM » (2019-20) | article seul | — | SOTA IAM-OnDB revendiqué, chiffre non relevé | architecture à reprendre |
| Lipi Toolkit (HP) | MIT | — | ancien, caractères isolés | idée seulement |
| Zinnia | BSD (fichier COPYING) | — | SVM par caractère (CJK), dernier commit 2022 | utilisable pour signes isolés |
| Tegaki | GPL-2 (tegaki-python) | — | inactif depuis 2022 | inutile |

Jeux de données : **IAM et IAM-OnDB** « may be used for non-commercial research purpose only » ; **RIMES** CC BY-NC 4.0
(Zenodo 10812725, 10805048) ; **BRUSH** « non-commercial research purposes » ; **DeepWriting** étend IAM-OnDB (licence
propre non lue → supposée NC) ; **POPP** et **Belfort** CC BY 4.0 (images, pas d'encre) ; **Quick, Draw!** CC BY 4.0.
Conclusion : **aucun jeu d'encre manuscrite latine sous licence libre** n'a été trouvé. Un modèle en ligne libre exige une
collecte maison CC0 (comme Fikajiana ADR 0042) et/ou de l'encre synthétique (polices OFL rendues en traits + bruit).

## 3. Formes dessinées (géométrie, flèches, graphes, arbres)

| Méthode | Licence | Ce qu'elle fait | Verdict |
|---|---|---|---|
| **Xournal++ `ShapeRecognizer`** (hérité de Xournal) | **GPLv2 or later** (en-tête des fichiers) → reprenable en GPL-3 | moments d'inertie, découpe en segments ; segment, triangle, rectangle, cercle/ellipse | **retenu, porté en Dart** |
| **$Q** (ACE Lab) | New BSD | gabarits multi-traits, « 142× » plus rapide que $P | **retenu** pour flèches, symboles, ✓/✗ |
| $P / $1 | New BSD ($P) ; $1 non relu | idem, déjà utilisé dans Fikajiana | déjà en place |
| ShortStraw, PaleoSketch | articles 2008 (non relus ici) | coins / primitives (arc, spirale, hélice) | idées pour les coins |
| Quick, Draw! | données CC BY 4.0 (50 M dessins, 345 classes) | CNN de croquis | seulement si un objet libre est à classer |
| Excalidraw (MIT) | — | **pas de reconnaissance** dans le cœur (ticket #5510 ouvert depuis 2022-07-29) | rien à reprendre |
| tldraw | licence tldraw (non MIT, NOASSERTION) | ticket #7187 « autodraw shape recognition » clos « completed » 2025-11-25 | **ne pas copier** |

Graphes/arbres : après reconnaissance, une passe géométrique en Dart relie formes et traits : cercle/rectangle =
nœud (texte intérieur lu par le HTR), trait dont les extrémités touchent deux nœuds = arête, pointe en V à une
extrémité ($Q) = arête orientée ; arbre = graphe connexe sans cycle, mis en page par niveaux.

## 4. Code manuscrit

Référence : Islam, Doumbouya, Manning, Piech, « Handwritten Code Recognition for Pen-and-Paper CS Education » (L@S 2024,
code et jeux **MIT**, github.com/mdoumbouya/codeocr) : chaîne **OCR → niveau d'indentation discret → correction par
modèle de langue**, erreur ramenée de 30 % à 5 % (leur mesure). Approche réaliste pour Fanoratana : bloc de code déclaré
par l'utilisateur (geste ou cadre), lecture ligne par ligne par le même HTR avec un **lexique de mots-clés** du langage,
**indentation tirée de l'abscisse au tracé** (regroupement des débuts de ligne), puis vérification syntaxique et
coloration (`syntax_highlight`, BSD-3). Correction par LLM : seulement sur PC et en option.

## 5. Pipeline recommandé pour l'« instantané »

1. **Tracé d'abord (< 10 ms, Dart)** : chaque trait est classé à la levée du stylet — forme (Xournal++ / $Q), signe
   mathématique ($P + CNN de Fikajiana), ou encre « texte » (largeur, densité, trait cursif long).
2. **Segmentation au tracé** : lignes par la ligne de base, mots par les écarts horizontaux ; zones « formule » (signes
   reconnus, exposants, barres de fraction), « dessin » (formes fermées, traits longs), « code » (cadre déclaré).
3. **Mot par mot (< 100 ms visé)** : quand l'écart après un mot dépasse un seuil, le mot rastérisé (hauteur 128 px)
   passe dans le CRNN texte (ONNX int8, `flutter_onnxruntime`) + lexique fr/mg ; résultat affiché en surimpression.
4. **Ligne entière (< 1 s visé)** : à la fin de ligne, relecture de la ligne avec le LM 6-grammes (PyLaia) et
   formules via Fikajiana (Pix2Text-MFR en recours seulement).
5. **File d'arrière-plan** (isolate) : priorité au mot courant, puis lignes en attente ; annulation si l'encre change.
6. **PC (optionnel, hors ligne)** : relecture de page par GLM-OCR (GGUF, ~1,4 Go Q8) pour proposer des corrections ;
   jamais requis, l'appareil reste autonome.

## 6. Rendu « au propre »

| Élément | Choix | Licence (vérifiée) | Remarque |
|---|---|---|---|
| Polices manuscrites | Caveat, Kalam, Patrick Hand, Architects Daughter, Gochi Hand, Indie Flower | **OFL** (dossier `ofl/` de google/fonts) | embarquer les .ttf (pas de téléchargement à l'exécution) |
| Excalifont / Virgil | Excalidraw | **OFL-1.1** (page Excalifont ; dépôt excalidraw/virgil) | couverture des accents français non confirmée |
| Synthèse d'écriture (Graves) | hardmaru/write-rnn-tensorflow, sjvasquez/handwriting-synthesis | **aucune licence** ; entraînés sur IAM-OnDB (NC) | **exclus** ; à défaut : glyphes de police OFL + gigue légère |
| Traits lissés | `perfect_freehand` 2.5.2+1 | MIT | rendu du tracé à la pression |
| Figures façon Excalidraw | `rough_flutter` 0.1.2 (2025-09) ; rough.js d'origine | MIT / MIT | `rough` 0.1.1 (2020) est abandonné |
| Formules | `flutter_math_fork` 0.7.4 (2025-05-21) | Apache-2.0 (simpleclub/flutter_math) | rendu LaTeX natif |
| Code | `syntax_highlight` 0.5.0 | BSD-3-Clause | coloration |

## 7. Export et format de fichier

| Besoin | Choix | Licence | Remarque |
|---|---|---|---|
| PDF / impression | `pdf` 3.13.1 + `printing` 5.15.1 (2026-09-19) | Apache-2.0 (DavBfr/dart_pdf) | diapos = pages 16:9 du même moteur |
| PNG | `RenderRepaintBoundary.toImage` (Flutter) | BSD (Flutter) | page ou sélection |
| Diapos texte | Markdown compatible Marp | Marp CLI MIT | export facultatif, hors appareil |
| Format natif | dossier `.fanoratana` : `cours.md` (texte, LaTeX `$…$`, blocs code) + `traits.json` (traits bruts x, y, t, pression, id) + `formes.json` (style Excalidraw, MIT) | à nous (GPL-3) | lisible sans l'app ; liens Markdown ↔ id de traits |
| Import/export Xournal++ | `.xopp` = XML (`xournal`, `fileversion`, `stroke` lus dans `SaveHandler.cpp`), gzippé (supposé) | GPL-2+ | paquet `xml` 7.1.0 (MIT) |
| Obsidian | format `.excalidraw.md` du greffon | greffon AGPL-3.0 | reprendre le **format**, pas le code |

## Recommandation

- **Texte (modèle retenu)** : un **CRNN-CTC PyLaia** partant de **PyLaia-Belfort** (MIT, données CC BY 4.0 ; CER 9,52 %
  sur son propre jeu), **affiné** sur (a) des lignes synthétiques françaises et malgaches rendues avec les polices OFL
  ci-dessus, (b) l'écriture réelle donnée en CC0 depuis l'app, exporté en ONNX int8 (≈ 11 Mo attendus pour 42 Mo fp32,
  supposé). **Témoin immédiat** pour mesurer dès la semaine 1 : `latin_PP-OCRv5_mobile_rec` ONNX (8 Mo, Apache-2.0).
  Seuil d'adoption proposé : CER ≤ 10 % sur un jeu de test maison jamais vu, < 100 ms par mot sur le Pad 7.
- **Formules** : reprendre la chaîne Fikajiana telle quelle (tracé + CNN par signe, Pix2Text-MFR en recours, avec sa
  réserve de provenance).
- **Formes** : port Dart du `ShapeRecognizer` de Xournal++ (GPL-2+) + **$Q** (BSD) pour flèches/symboles + passe
  géométrique nœuds/arêtes ; Quick, Draw! (CC BY 4.0) seulement si un classeur de croquis devient utile.
- **Plus tard (en ligne)** : modèle LSTM-CTC sur traits (architecture Carbune, code OnlineHTR MIT), entraîné uniquement
  sur encre CC0 maison + encre synthétique ; il remplacera le CRNN pour l'instantané s'il le bat.
- **PC** : GLM-OCR (MIT, 0,9 B) en relecture de page facultative ; LightOnOCR-3-1B (Apache) en alternative.
- **Refusés** : TrOCR-handwritten et dérivés (IAM), tout modèle RIMES, InkSight, OnlineHTR pré-entraîné, Qwen2.5-VL,
  Nanonets, Chandra, HunyuanOCR, ML Kit, MyScript, tout service distant.

## Vérifié / Supposé / Reste à tester

- **Vérifié** (pages sources, 2026-11-08) : licences et tailles de tous les modèles des tableaux (API HF : `cardData`,
  arborescence de fichiers) ; CER/WER PyLaia (fiches Teklia) ; licences Zenodo RIMES (CC BY-NC 4.0), POPP, Belfort,
  CATMuS (CC BY 4.0) ; conditions IAM / IAM-OnDB (pages FKI, certificat expiré, lues par curl) ; BRUSH (README) ;
  Quick, Draw! (README) ; PP-OCRv5 (docs PaddleOCR) ; $Q (page ACE Lab) ; en-tête GPLv2+ du recognizer Xournal++ ;
  licences GitHub (Kraken, PyLaia, HTR-Flor, OnlineHTR, xournalpp_htr, InkSight, Zinnia, Tegaki, rough, perfect_freehand,
  dart_pdf, flutter_math, Marp, codeocr) ; versions pub.dev ; jeux d'entraînement d'InkSight Small-p (article §4.1).
- **Supposé** : export ONNX simple de PyLaia (CRNN) et ≈ 4× de réduction en int8 ; latences sur le Pad 7 ; provenance
  « propre » des données PP-OCRv5 (non publiée, même réserve que Pix2Text) ; DeepWriting NC par héritage d'IAM-OnDB ;
  `.xopp` gzippé ; accents français dans Excalifont ; justesse des VLM sur l'écriture (aucun chiffre manuscrit publié).
- **Non vérifié** : licence propre de DeepWriting (page ETH non ouverte) ; licence de $1 ; poids des modèles HF de
  xournalpp_htr ; CER de Carbune et al. ; contenu de ShortStraw/PaleoSketch ; licence des données LightOnOCR (« other »).
- **Reste à tester** : (1) CER de PP-OCRv5 latin et PyLaia-Belfort sur 200 lignes d'écriture réelle rastérisée ;
  (2) latence mot/ligne sur Pad 7 et PC ; (3) gain de l'affinage synthétique OFL ; (4) recognizer Xournal++ porté sur
  des figures de cours ; (5) GLM-OCR sur une page mêlant texte et formules ; (6) taille APK finale.

## Sources

Modèles texte : https://huggingface.co/Teklia/pylaia-belfort · https://huggingface.co/Teklia/pylaia-popp ·
https://huggingface.co/Teklia/pylaia-rimes · https://zenodo.org/records/8041668 · https://zenodo.org/records/6581158 ·
https://zenodo.org/records/10812725 · https://zenodo.org/records/10805048 · https://github.com/jpuigcerver/PyLaia ·
https://huggingface.co/PaddlePaddle/latin_PP-OCRv5_mobile_rec · https://huggingface.co/PaddlePaddle/latin_PP-OCRv5_mobile_rec_onnx ·
https://www.paddleocr.ai/latest/en/version3.x/module_usage/text_recognition.html ·
https://www.paddleocr.ai/latest/en/version3.x/algorithm/PP-OCRv5/PP-OCRv5.html ·
https://huggingface.co/microsoft/trocr-small-handwritten · https://huggingface.co/microsoft/trocr-base-handwritten ·
https://huggingface.co/Xenova/trocr-small-handwritten · https://github.com/microsoft/unilm ·
https://huggingface.co/agomberto/trocr-large-handwritten-fr · https://huggingface.co/dj0w/trocr-french-handwriting-v5 ·
https://huggingface.co/personalizedrefrigerator/trocr-small-fr · https://github.com/arthurflor23/handwritten-text-recognition ·
https://github.com/mittagessen/kraken · https://zenodo.org/records/10592716 · https://zenodo.org/records/15030337
VLM : https://huggingface.co/zai-org/GLM-OCR · https://huggingface.co/ggml-org/GLM-OCR-GGUF ·
https://huggingface.co/lightonai/LightOnOCR-3-1B · https://huggingface.co/lightonai/LightOnOCR-2-1B ·
https://huggingface.co/datasets/lightonai/LightOnOCR-mix-0126 · https://huggingface.co/onnx-community/LightOnOCR-2-1B-ONNX ·
https://huggingface.co/PaddlePaddle/PaddleOCR-VL-1.6 · https://huggingface.co/PaddlePaddle/PaddleOCR-VL-1.6-GGUF ·
https://huggingface.co/microsoft/Florence-2-base · https://huggingface.co/ibm-granite/granite-docling-258M ·
https://huggingface.co/docling-project/SmolDocling-256M-preview · https://huggingface.co/stepfun-ai/GOT-OCR2_0 ·
https://huggingface.co/Qwen/Qwen3-VL-2B-Instruct · https://huggingface.co/Qwen/Qwen2.5-VL-3B-Instruct ·
https://huggingface.co/dots-studio/dots.ocr · https://huggingface.co/deepseek-ai/DeepSeek-OCR ·
https://huggingface.co/allenai/olmOCR-2-7B-1025 · https://huggingface.co/opendatalab/MinerU2.5-2509-1.2B ·
https://huggingface.co/nanonets/Nanonets-OCR2-3B · https://huggingface.co/datalab-to/chandra · https://huggingface.co/tencent/HunyuanOCR
En ligne et données : https://github.com/PellelNitram/OnlineHTR · https://github.com/PellelNitram/xournalpp_htr ·
https://github.com/google-research/inksight · https://huggingface.co/Derendering/InkSight-Small-p ·
https://arxiv.org/html/2402.05804v4 · https://arxiv.org/abs/2503.23081 · https://arxiv.org/abs/1902.10525 ·
https://lipitk.sourceforge.net/faq.htm · https://github.com/taku910/zinnia · https://github.com/tegaki/tegaki ·
https://fki.tic.heia-fr.ch/databases/iam-handwriting-database · https://fki.tic.heia-fr.ch/databases/iam-on-line-handwriting-database ·
https://github.com/brownvc/decoupled-style-descriptors · https://github.com/emreaksan/deepwriting ·
https://github.com/googlecreativelab/quickdraw-dataset
Formes : https://depts.washington.edu/acelab/proj/dollar/qdollar.html · https://depts.washington.edu/acelab/proj/dollar/pdollar.html ·
https://github.com/xournalpp/xournalpp/tree/master/src/core/control/shaperecognizer ·
https://github.com/excalidraw/excalidraw/issues/5510 · https://github.com/tldraw/tldraw/issues/7187
Code : https://arxiv.org/abs/2408.07220 · https://github.com/mdoumbouya/codeocr
Rendu et export : https://github.com/google/fonts/tree/main/ofl · https://plus.excalidraw.com/excalifont ·
https://github.com/excalidraw/virgil · https://github.com/hardmaru/write-rnn-tensorflow · https://github.com/sjvasquez/handwriting-synthesis ·
https://pub.dev/packages/perfect_freehand · https://pub.dev/packages/rough_flutter · https://github.com/rough-stuff/rough ·
https://pub.dev/packages/flutter_math_fork · https://github.com/simpleclub/flutter_math · https://pub.dev/packages/syntax_highlight ·
https://pub.dev/packages/pdf · https://pub.dev/packages/printing · https://github.com/DavBfr/dart_pdf ·
https://github.com/marp-team/marp-cli · https://github.com/zsviczian/obsidian-excalidraw-plugin · https://pub.dev/packages/xml ·
https://pub.dev/packages/flutter_onnxruntime
