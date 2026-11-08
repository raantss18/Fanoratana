# 0010 — Formules manuscrites en LaTeX, saisie GeoGebra

- **Date** : 2026-11-08 · **Statut** : accepté (Antsa : « entrée LaTeX et comme dans GeoGebra doivent être bien reconnues »)
- **Lecture** : Pix2Text-MFR 1.0 (MIT, révision `bea257ed`), même chaîne que Fikajiana (`docs/recherche/outils-cours.md`), exécuté par
  tract ; **non embarqué** (117 Mo, provenance des poids non publiée) : téléchargé à la demande (Plus › Réglages), SHA-256 vérifiés.
- **Aiguillage** : « Lire comme » Deviner / Texte / Formule. Deviner : la lecture texte d'abord ; si elle ressemble à une formule
  (signe d'opération, au plus un vrai mot), la formule est relue en LaTeX et écrite `$…$`.
- **GeoGebra** : `f(x)=…`, `y=…`, `A=(1;2)` en syntaxe GeoGebra ou LaTeX (fractions, racines, puissances, multiplication implicite) ;
  « Tracer la courbe » crée un repère (`fano-render::expr`, `scene::graphe`). Commandes de construction (Cercle, Segment…) : plus tard.
- **Mesuré** (Pad 7) : `x²+1=5` écrit au fil → `$x^{2}+1=5$` ; `f(x)=2x+1` → tracé. PC : ~0,5 s par formule.
