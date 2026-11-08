# Les maths dans Fanoratana : lire, afficher, échanger (2026-11-08)

> Demande d'Antsa : « figures, TikZ, amsmath, amssymb, amsthm… le moyen le plus sûr, optimal, fluide et fiable ».

## 1. Lire une formule écrite à la main — fiable et rapide

Aucun modèle seul ne suffit (Pix2Text-MFR invente du LaTeX sur une étiquette ; ML Kit lit des caractères, pas des
structures 2D). Chaîne en trois étages, du plus rapide au plus lourd, chacun vérifiable :

1. **Signes** : chaque signe isolé par la géométrie des traits, puis reconnu par le **petit modèle par signe de Fikajiana**
   (CNN 32×32 + mesures du tracé, < 2 Mo, ADR 0038 là-bas, même autrice, GPL) et, sur Android, par ML Kit en recours.
2. **Structure** : la mise en page 2D se lit à la géométrie, comme dans Fikajiana (`trace_ligne`, ADR 0035, 0037) :
   ligne de base, exposants et indices, barre de fraction, racine, Σ/∫/lim avec bornes, matrices et systèmes.
   Résultat : LaTeX **exact par construction** (rien d'inventé : chaque symbole vient d'un trait).
3. **Recours** : Pix2Text-MFR seulement pour ce que la structure ne sait pas lire, et sa lecture est montrée à valider.

Toujours : ce qui a été compris est affiché (LaTeX + rendu) et se corrige d'un toucher ; les lectures validées
servent d'exemples pour améliorer le modèle par signe (comme dans Fikajiana).

## 2. Afficher tout LaTeX — amsmath, amssymb

- **Typst** (compilateur Rust, Apache-2.0, https://github.com/typst/typst) intégré comme bibliothèque, et **MiTeX**
  (LaTeX → Typst, Apache-2.0, https://github.com/mitex-rs/mitex : environnements `aligned`, `matrix`, `cases`, macros
  `\newcommand` ; 32 500 équations converties en 0,085 s) : rendu typographique complet en SVG, hors ligne.
- Deux styles au choix : **manuscrit** (le moteur Caveat actuel, simple) et **typographié** (Typst, complet).
- Cache par formule : une formule n'est mise en page qu'une fois (fluidité).
- À mesurer avant d'adopter : poids ajouté à l'APK (polices mathématiques comprises), temps de la première formule.

## 3. amsthm — théorèmes et démonstrations

- Les encadrés deviennent les environnements d'amsthm : Définition, **Théorème, Proposition, Lemme, Corollaire**,
  **Démonstration** (finie par ∎), Remarque, Exemple, Exercice ; **numérotés par chapitre** (Théorème 2.3).
- Reconnus à l'écriture : un mot-clé en tête de ligne (« Théorème : », « Déf. », « Preuve ») ouvre l'encadré.

## 4. Figures et TikZ

- Les formes, graphes, arbres et courbes restent des **objets** (pas des images) : chacun s'exporte en **TikZ**
  (`\draw`, `\node`, arbres avec `child`, courbes avec `pgfplots`).
- **Export LaTeX d'une leçon** : un `.tex` complet (amsmath, amssymb, amsthm, tikz, images) qui compile tel quel sur
  Overleaf (le serveur Overleaf d'Antsa) ; et import d'une formule ou d'un bloc LaTeX collé.
- Plus tard : figures de géométrie construites (points nommés, perpendiculaires…) comme la Figure de Fikajiana.

## Ordre proposé

1. Lecture des formules par signes + structure (port de Fikajiana en Rust) — c'est la fiabilité.
2. Rendu Typst + MiTeX (mesuré) — c'est la richesse (amsmath, amssymb).
3. Environnements amsthm numérotés et mots-clés.
4. Export LaTeX/TikZ d'une leçon, import de LaTeX.

## Vérifié / Supposé / Reste à tester

- **Vérifié** : licences Apache-2.0 de Typst et MiTeX (pages GitHub, 2026-11-08) ; environnements de MiTeX cités ci-dessus.
- **Supposé** : Typst embarquable sans réseau sur Android (crates `typst`, `typst-svg`) ; poids raisonnable.
- **Reste à tester** : poids APK, latence de la première formule sur le Pad 7, couverture amssymb de MiTeX
  (le dépôt range la prise en charge des paquets dans « à faire »).
