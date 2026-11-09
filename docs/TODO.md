# TODO — Fanoratana

> Seule source d'avancement. ✅ fait et vérifié · 🟡 fait, à vérifier sur la tablette · ⬜ à faire.

## 0. Erreurs signalées (ne reviennent jamais)

- ✅ 0.0.2 publiée mais jamais proposée dans l'app (2026-11-09) : une recherche faite avant la publication (ou sans réseau)
  bloquait la suivante un jour → recherche à chaque ouverture puis toutes les 6 h, seule une recherche réussie compte
  (`une_recherche_ratee_ne_bloque_pas_la_suivante`) ; GitHub répondait 504 sur `releases/download/…` → deux nouveaux essais
  puis secours par l'API (`secours_par_l_api`, `FANO_RESEAU=1`). Vérifié sur émulateur : bandeau à l'ouverture, téléchargement
  vérifié, installeur d'Android ouvert.
- ✅ Écriture d'Antsa au stylet (Pad 7 a11fafee, 2026-11-08) : « Je veux voir » lu « fe voux vor », « 1- Important » lu
  « 1- Impontant » → correction des mots incertains par lexique français, une faute au plus (`lexique::lectures_du_pad7`) ;
  « médiatrice » n'est jamais changé (`un_mot_sur_ou_rare_et_juste_reste`).
- ✅ Texte au propre bien plus grand que l'écriture (« Je ne sais pas lire ») : taille d'après la bande des minuscules
  (`bande_minuscules`, `une_hampe_ne_fait_pas_la_taille`).
- ✅ Le soulignement tracé à la main restait de l'encre : il souligne le texte (en attente ou déjà au propre).
- ✅ Le doigt ne déplaçait pas la page avec Clavier, Sélection ou Gomme quand il n'écrit pas : il la déplace avec tous les outils.
- ✅ Téléchargement du modèle des formules coupé à 60 s (« timeout: global ») : plus de délai global, seulement une connexion muette.
- ✅ « Quand je trace une ligne ça en fait deux » (Antsa, 2026-11-08) : les formes sont un seul trait net (`rough` : test « un seul trait »).
- ✅ « Un rond ne fait pas un cercle » : un rond irrégulier, un peu ovale (≤ 25 %) ou qui dépasse son départ devient un cercle
  (`un_rond_devient_un_cercle`) ; un ovale franc reste une ellipse.
- ✅ Deux ronds lus « 00 » par « Mettre au propre » (Pad 7, 2026-11-08) : un grand tracé isolé qui est une forme nette devient
  une forme (`deux_ronds_ne_font_pas_00`, `un_o_dans_un_mot_reste_une_lettre`).
- ✅ Intégrale ∫₀¹ e^(−x²) dx = √π jamais lue (ligne haute prise pour un dessin) : une ligne de plusieurs traits est toujours lue.
- ✅ Bandeau des mises à jour recouvert par la feuille (bandes noires « Cou… ») : la feuille prend la place qui reste.
- ✅ Arbre de probabilités au stylet (Pad 7, 2026-11-08) : branches « < » lues comme le signe inférieur, étiquettes éloignées
  fusionnées (« XA »), Ā lu « −A », faux gras → branches = formes (1,6 × la médiane des lettres), lignes coupées aux grands
  vides, barre au-dessus = `\overline`, seuil du gras 1,6 (`arbre_de_probabilites`, `etiquettes_eloignees_lues_a_part`, `barre_de_a_barre`).
- ✅ Boutons du haut cachés sous la barre de notification d'Android : les coins respectent les marges du système.
- ✅ Arbre retracé (Pad 7, 2026-11-08) : branche un peu courbée restée dans le texte (« BΣ ») → deux bras courbés
  acceptés (`branche_un_peu_courbee`) ; C seuls lus « l », « ( » → majuscule au bout d'une branche
  (`etiquette_d_arbre_en_majuscule`) ; « Mathimaliques » → « Mathématiques » ; soulignement du titre pris pour une ligne
  → il souligne ; boutons de zoom « □ » → « + » et « − ».
- 🟡 « Calculer cette expression » lu « Calmer cette expetion » : limite du modèle sur la cursive (§ 3, priorité 1).
- 🟡 « Je ne sais pas lire » lu « Je ne sors pas line » : « sors » est un vrai mot ; il faudra un modèle de langue (suite de mots)
  ou un modèle d'écriture affiné sur la cursive scolaire française (§ 3).

- ✅ « f(x)=2x+1 » lu « f(z)=2x+1 » (Pad 7 a11fafee, 2026-11-08) : la variable d'une définition est celle du membre de droite
  (`expr::corriger_definition`, test `variable_mal_lue`).

- ✅ Un tableau inséré se posait sous le dernier élément, hors de l'écran, puis par-dessus le texte (Pad 7, 2026-11-08) : espace
  libre visible, sinon défilement jusqu'à lui.

- ✅ Page cadrée pendant la fenêtre flottante, jamais recadrée en plein écran (Pad 7, 2026-11-08) : recadrage quand la largeur change.
- ✅ Trait aux points espacés déformé par le lissage (rectangle devenu parallélogramme, Pad 7, 2026-11-08) : `points_espaces_non_deformes`.

- ✅ Le clavier d'Android s'ouvrait et se fermait en boucle dans « Nouvelle leçon » (focus redemandé à chaque image) : des lettres
  se perdaient (« Essai du cours » → « Esa dcours ») et l'app a fini par se fermer (Pad 7, 2026-11-08). Focus demandé une seule fois.

- ✅ L'APK avec ML Kit plantait au démarrage (R8 retirait `WorkDatabase_Impl` de WorkManager) : règles de conservation dans
  `android/app/proguard-rules.pro` ; à vérifier sur la tablette après chaque ajout de dépendance Android.

- ✅ Le modèle Google ne se téléchargeait jamais : R8 retirait les registres de ML Kit (constructeurs appelés par réflexion) ;
  règles dans `android/app/proguard-rules.pro`, vérifié sur 8df629a (« Modèle prêt, hors ligne »).
- ✅ « Mise à jour impossible : 404 » quand aucune version n'est publiée : ce n'est plus une erreur.
- ✅ Un √ tracé au-dessus de x devenait une forme : un trait ouvert qui coiffe de l'écriture reste un signe.
- ✅ Réglages en fenêtre étroite : la fenêtre ne déborde plus (capture `reglages-etroit`).
- ⬜ En portrait étroit, le titre de la leçon passe sous « Insérer ».
- ✅ Écran « À propos » (version, licence, coordonnées d'Antsa).

- ✅ Les `$` d'Antsa (S plus large que haut) n'étaient pas reconnus ; les traits après un `$` ouvert étaient perdus au lieu
  d'attendre le second (`tests/ecriture_antsa.rs`, traits du Pad 7 du 2026-11-09).
- ✅ `\mathbb{R}` lu juste par Google mais écrit en texte : une commande LaTeX écrite à la main devient une formule.
- ✅ `f(x)=mx+p` : Pix2Text le lisait juste, la lecture par signes douteuse (« μJc)=m1C+7 ») l'emportait ; Pix2Text gagne.
- ⬜ Un gribouillis de rature lu « m » (lecture 2 du journal du 2026-11-09) : à reproduire.

## 1. Version 0.0.1 (2026-11-08)

- ✅ Conception : questions à Antsa, recherche sourcée (`docs/recherche/reconnaissance.md`), ADR 0001–0009, identité (ADR 0006).
- ✅ Coffre niveau → chapitre → leçon, recherche, corbeille, niveaux type (`fano-core`, tests).
- ✅ Encre à pression (simulée au doigt), formes ShortStraw, flèches, rature, lignes et mots (`fano-ink`, tests).
- ✅ Lecture hors ligne PP-OCRv5 latin par tract, sur un fil (`fano-reco`, test bout en bout : 4/4 sur traits Caveat).
- ✅ Rendu au propre : Caveat, formules `$…$`, trait fait main, encadrés ; export SVG/PNG/PDF (`fano-render`, tests).
- ✅ Interface egui : rail, toile, coffre, éditeur clavier, menu Plus, diapos, raccourcis ; captures headless.
- ✅ APK arm64 (21,7 Mo) installé et lancé sur le Pad 7 sans stylet ; « Bonjour » injecté lu juste.
- ✅ Mises à jour dans l'app : release.json signé Ed25519, SHA-256, installeur Android / remplacement Linux.
- ✅ Cours écrit sur la tablette (gestes injectés, nouvelle leçon « Essai du cours », 2026-11-08) : titre, phrase et `f(x)=2x+1` lus
  justes ; triangle tenu → forme ; signature → reste de l'encre ; rature d'une ligne → effacée ; `f(x)=2x+1` → courbe tracée ;
  modèle des formules téléchargé (SHA-256 vérifiés) ; `x²+1=5` écrit au fil → `$x^{2}+1=5$` ; tableau inséré, « Date » et « 1789 »
  écrits à la main dans les cases.
- ✅ Tableaux, mise en forme (listes, alignement, souligné, surligné), courbes GeoGebra, formules LaTeX, menus en listes (ADR 0010, 0011).
- ✅ Gras reconnu à l'épaisseur (≥ 1,45 × l'écriture habituelle de la page) ; fin de page → « + Ajouter une page » (essayé sur le
  Pad 7) ; page ajustée à la largeur et bloquée sur ses bords sauf zoom.
- ✅ Deuxième tablette (Pad 7 a11fafee, interface en anglais, 2026-11-08) : titre et phrase lus justes, triangle tenu → forme,
  signature → encre, rature d'une seule ligne, « Important » écrit épais → gras, leçon gardée après mise à jour de l'APK.
- 🟡 Gras : à mesurer sur de vraies écritures (le seuil 1,45 est une hypothèse ; repasser deux fois sur un mot pas encore pris en compte).
- 🟡 Essai complet au doigt par Antsa : rature, gomme, formes tenues, sélection, diapos, export PDF vers Documents.
- 🟡 Mise à jour réelle de 0.0.1 vers 0.0.2 sur la tablette (la 0.0.1 est la première : rien à tester avant la 0.0.2).

## 2. Interface d'après Fikajiana (Antsa, 2026-11-08) — en cours

1. ⬜ Feuille plein écran, plus de rail ; quatre coins flottants (docs/design.md § Écran).
2. ⬜ Bouton Outils à la couleur du stylo (bas droite) + Annuler/Rétablir à côté ; carte Outils en lignes nommées.
3. ⬜ « + Insérer » et « ⋯ Menu » en haut à droite ; « ≡ » + titre du cours en haut à gauche ; « Taper un texte » en bas à gauche.
4. ⬜ Réglages en écran à part, en sections (Écriture, Lecture, Langue, Mises à jour, À propos).
5. ⬜ Mes cours en panneau par-dessus (il ne pousse plus la feuille).
6. ⬜ Captures, essai sur la tablette (portrait et paysage), contraste et cibles ≥ 48 dp.

## 3. Annoter un PDF ou une image (demande d'Antsa, 2026-11-08) — prochaine version 0.0.2

Ordre de priorité (chaque étape est utilisable seule) :

1. ⬜ **Image en fond de page** : Mes cours › « Importer une image » (et Insérer › « Image en fond de page ») crée une leçon dont
   chaque page a la taille de l'image, l'image verrouillée dessous ; on écrit, tape, dessine par-dessus ; export PDF/PNG tel quel.
   Peu de risque : l'image, l'encre et l'export existent déjà.
2. ⬜ **Importer un PDF** : chaque page du PDF devient une page de leçon à sa taille, fond verrouillé. Rendu des pages par
   **hayro** (Rust pur, MIT/Apache, déjà vendu dans mahay) à 2× ; licences et poids à mesurer (ADR).
3. ⬜ **Exporter annoté** : PDF (fond en image 200 dpi + annotations vectorielles) et PNG par page. Ensuite seulement :
   garder le PDF d'origine en vectoriel (page d'origine en XObject sous les annotations), plus net et plus léger.
4. ⬜ **Ouvrir depuis Android** : « Ouvrir avec » / « Partager vers Fanoratana » pour un PDF ou une image (intents VIEW/SEND, déjà
   gérés par la passerelle de mahay) ; sur Linux, glisser-déposer et `fanoratana fichier.pdf`.
5. ⬜ **Confort** : gros PDF rendus page par page à la demande, miniatures des pages, aller à une page ; gomme qui ne touche jamais le fond.
6. ⬜ **Plus tard** : recherche dans le texte du PDF, copier une zone du PDF dans un cours, PDF protégés (refus clair).

## 4. Lecture (priorité)

- ✅ Texte ou formule (ADR 0014) : `$` manuscrits, aiguillage automatique, ℝ et variables dans les phrases (« pa fo= moto » pour `f(x)=mx+p`).

- ✅ Page de cours lue par blocs (`fano_ink::grouper_blocs`) : fraction, `lim` et sa borne, exposant, matrice d'un seul tenant ;
  titre souligné, paragraphe serré et fraction dans une phrase restent à part ; barre de fraction jamais prise pour un dessin.
- ⬜ Essai sur le Pad 7 d'une vraie page de cours (texte + formules + figures) ; exporter l'encre de ses leçons pour en faire des tests.

- ✅ Formules lues à leur structure (ADR 0013) : modèle de signes de Fikajiana + géométrie ; banc de 23 lignes d'Antsa : 14 justes,
  aucune fausse sûre (`cargo test -p fano-reco banc -- --nocapture`).
- ⬜ Formules : essai sur le Pad 7 (latence, justesse) ; apprendre des corrections (plus proche voisin, comme Fikajiana ADR 0041) ;
  réentraîner le modèle de signes avec l'écriture d'Antsa (`y`, `(`, `;`, `g`, `e` du téléphone ratés).
- ✅ Matrices et systèmes lus (`pmatrix`, `vmatrix`, `cases`, `si`) et rendus ; forme juste sur les 7 fixtures d'Antsa.
- ⬜ Formules : vecteur `\vec{AB}`, normes, `\sum` avec bornes réelles à tester.
- ⬜ Rendu Typst + MiTeX, environnements amsthm, export LaTeX/TikZ (`docs/recherche/maths.md`).

- ⬜ Mesurer la justesse sur l'écriture réelle (200 lignes d'Antsa, au doigt) : CER, mots justes, latence sur le Pad 7.
- ⬜ PyLaia (MIT) affiné sur des lignes synthétiques françaises et malgaches (polices OFL) puis sur l'écriture donnée en CC0 ; seuil
  d'adoption : CER ≤ 10 %, < 100 ms par mot.
- ⬜ Apprendre des corrections au clavier, sur l'appareil (comme Fikajiana, ADR 0038 là-bas).
- ⬜ Formules : tester Pix2Text-MFR 1.5 (MIT, jeton de départ 1, vocabulaire 1 868) ; versions int8 (24 + 9 Mo) pour aller plus vite.
- ⬜ Commandes GeoGebra de construction (Cercle, Segment, Droite, Polygone, Vecteur, Intersection) dans le repère.
- ⬜ Échap ne ferme pas un menu ouvert sur Android (clavier physique) : à vérifier.
- ⬜ Code manuscrit : lexique de mots-clés, indentation par la position du tracé, coloration.
- ⬜ Lecture page entière sur PC en option (GLM-OCR, MIT).

## 5. Cours

- ⬜ Nœuds et arêtes : un cercle avec une étiquette relié par des flèches = un graphe ; arbres rangés automatiquement.
- ⬜ Tableaux à main levée ; listes à puces reconnues ; liens entre leçons `[[…]]` façon Obsidian.
- ⬜ Redimensionner et faire pivoter une sélection ; copier-coller ; aligner.
- ⬜ Modèles de leçon (cours, fiche d'exercices, évaluation) ; numérotation automatique des exercices.
- ⬜ Interface en malgache.

## 6. Plateformes

- ⬜ Paquets Linux (AppImage, .deb) ; icône et entrée de menu installées.
- ⬜ Accessibilité : lecteur d'écran (TalkBack) sur le rail et le coffre ; vérifier les contrastes.
- ⬜ Mode sombre de l'interface (la feuille reste blanche).
