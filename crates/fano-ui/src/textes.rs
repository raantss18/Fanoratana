//! Les textes de l'interface, en français et en anglais (de qualité égale).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Langue {
    Fr,
    En,
}

impl Langue {
    /// D'après une étiquette de langue (`fr_FR.UTF-8`, `en-US`…) ; le français par défaut.
    pub fn depuis(tag: &str) -> Langue {
        if tag.to_lowercase().starts_with("en") {
            Langue::En
        } else {
            Langue::Fr
        }
    }
}

macro_rules! textes {
    ($($cle:ident => $fr:literal, $en:literal;)*) => {
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        pub enum T { $($cle),* }
        pub fn t(l: Langue, c: T) -> &'static str {
            match (c, l) {
                $((T::$cle, Langue::Fr) => $fr, (T::$cle, Langue::En) => $en,)*
            }
        }
    };
}

textes! {
    Coffre => "Mes cours", "My courses";
    Ecrire => "Écrire", "Write";
    Formes => "Formes", "Shapes";
    Gomme => "Gomme", "Eraser";
    Selection => "Sélection", "Select";
    Clavier => "Clavier", "Keyboard";
    AuPropre => "Mettre au propre", "Tidy up";
    Diapos => "Présenter", "Present";
    Plus => "Plus", "More";
    Annuler => "Annuler", "Undo";
    Retablir => "Rétablir", "Redo";
    Couleur => "Couleur", "Colour";
    Epaisseur => "Épaisseur", "Thickness";
    Surligneur => "Surligneur", "Highlighter";
    NouveauNiveau => "Nouveau niveau", "New level";
    NouveauChapitre => "Nouveau chapitre", "New chapter";
    NouvelleLecon => "Nouvelle leçon", "New lesson";
    NouvelleDiapo => "Nouvelles diapos", "New slides";
    NiveauxType => "Ajouter les niveaux 6e … M2", "Add the levels 6e … M2";
    Renommer => "Renommer", "Rename";
    Supprimer => "Supprimer", "Delete";
    MettreCorbeille => "Mettre à la corbeille du coffre ?", "Move to the course trash?";
    Chercher => "Chercher dans les cours", "Search the courses";
    Nom => "Nom", "Name";
    Creer => "Créer", "Create";
    Fermer => "Fermer", "Close";
    Valider => "Valider", "Done";
    PageSuivante => "Page suivante", "Next page";
    PagePrecedente => "Page précédente", "Previous page";
    AjouterPage => "Ajouter une page", "Add a page";
    SupprimerPage => "Supprimer cette page", "Delete this page";
    InsererImage => "Insérer une image", "Insert an image";
    ExporterPdf => "Exporter en PDF", "Export as PDF";
    ExporterPng => "Exporter en images PNG", "Export as PNG images";
    ExporterMd => "Exporter en Markdown (Obsidian)", "Export as Markdown (Obsidian)";
    Fond => "Fond de page", "Page background";
    FondBlanc => "Blanc", "Plain";
    FondLignes => "Lignes", "Lined";
    FondCarreaux => "Carreaux", "Squared";
    FondPoints => "Points", "Dotted";
    Reglages => "Réglages", "Settings";
    DoigtEcrit => "Le doigt écrit (sinon il déplace la page)", "Finger writes (otherwise it moves the page)";
    AuPropreAuto => "Mettre au propre tout seul après une pause", "Tidy up by itself after a pause";
    Delai => "Pause avant la mise au propre", "Pause before tidying";
    Langue => "Langue", "Language";
    Apropos => "À propos", "About";
    Lu => "Ce qui a été lu", "What was read";
    Autres => "Autres lectures", "Other readings";
    RevenirEncre => "Revenir à l'encre", "Back to ink";
    Titre => "Titre", "Heading";
    Texte => "Texte", "Text";
    Encadre => "Encadré", "Box";
    AucunEncadre => "Aucun", "None";
    Definition => "Définition", "Definition";
    ARetenir => "À retenir", "Key point";
    Exemple => "Exemple", "Example";
    Exercice => "Exercice", "Exercise";
    Citation => "Citation", "Quote";
    AideFormule => "Formule : écrire entre $ … $, ex. $\\frac{a}{b}$, $x^2$, $\\sqrt{2}$, $\\alpha$",
                   "Formula: write between $ … $, e.g. $\\frac{a}{b}$, $x^2$, $\\sqrt{2}$, $\\alpha$";
    Exporte => "Exporté :", "Exported:";
    Erreur => "Impossible :", "Could not:";
    Vide => "Choisis une leçon dans « Mes cours », ou crée-la.", "Pick a lesson in “My courses”, or create one.";
    Bienvenue => "Écris ici avec le doigt, le stylet ou la souris. Après une pause, l'écriture se met au propre. Touche un texte pour le corriger au clavier.",
                 "Write here with your finger, a stylus or the mouse. After a pause, your writing is tidied up. Tap a text to correct it with the keyboard.";
    Lecture => "Lecture…", "Reading…";
    PasLu => "Pas lu avec assez de certitude : l'encre reste. Sélectionne-la et touche « Mettre au propre » pour forcer.",
             "Not read with enough confidence: the ink stays. Select it and tap “Tidy up” to force it.";
    Quitter => "Quitter la présentation", "Leave the presentation";
    Remplir => "Remplir", "Fill";
    Etiquette => "Étiquette", "Label";
    Outil => "Outil", "Tool";
    Inserer => "Insérer", "Insert";
    InsererTexte => "Un texte au clavier", "Typed text";
    InsererTableau => "Un tableau", "A table";
    InsererCourbe => "Une courbe (comme GeoGebra)", "A graph (like GeoGebra)";
    Lignes => "lignes", "rows";
    Colonnes => "colonnes", "columns";
    LireComme => "Lire l'écriture comme", "Read handwriting as";
    LireAuto => "Deviner", "Guess";
    LireFormule => "Formule", "Formula";
    Exporter => "Exporter", "Export";
    AppliquerSelection => "Appliquer à la sélection", "Apply to the selection";
    Liste => "Liste", "List";
    Puces => "Puces", "Bullets";
    Numeros => "Numéros", "Numbers";
    AucuneListe => "Pas de liste", "No list";
    Alignement => "Alignement", "Alignment";
    AGauche => "À gauche", "Left";
    AuCentre => "Au centre", "Centre";
    ADroite => "À droite", "Right";
    Souligne => "Souligné", "Underlined";
    Surligne => "Surligné", "Highlighted";
    Style => "Style", "Style";
    Tracer => "Tracer la courbe", "Plot the graph";
    Definitions => "Une définition par ligne : f(x)=x^2, y=2x+1, A=(1;2)", "One definition per line: f(x)=x^2, y=2x+1, A=(1;2)";
    AjouterLigne => "+ ligne", "+ row";
    AjouterColonne => "+ colonne", "+ column";
    RetirerLigne => "− ligne", "− row";
    RetirerColonne => "− colonne", "− column";
    EnTete => "En-tête", "Header row";
    ModeleManquant => "Pour lire les formules écrites à la main, télécharge le modèle (Plus › Réglages).", "To read handwritten formulas, download the model (More › Settings).";
    TelechargerModele => "Télécharger le modèle des formules (117 Mo)", "Download the formula model (117 MB)";
    ModelePresent => "Modèle des formules installé : les formules écrites à la main sont lues en LaTeX.", "Formula model installed: handwritten formulas are read as LaTeX.";
    AstuceTracer => "C'est une fonction : touche-la puis « Tracer la courbe ».", "This is a function: tap it, then “Plot the graph”.";
    FinDePage => "Bas de la page : touche « + Ajouter une page » sous la feuille pour continuer.", "Bottom of the page: tap “+ Add a page” below the sheet to go on.";
    Gras => "Gras", "Bold";
    ChercherMaj => "Rechercher une mise à jour", "Check for updates";
    MajAuto => "Chercher les mises à jour chaque jour", "Check for updates every day";
    MajRecherche => "Recherche d'une mise à jour…", "Checking for updates…";
    AJour => "Fanoratana est à jour.", "Fanoratana is up to date.";
    MajDispo => "est disponible", "is available";
    Installer => "Installer", "Install";
    PlusTard => "Plus tard", "Later";
    Telechargement => "Téléchargement…", "Downloading…";
    MajLinux => "Mise à jour installée : elle prendra effet au prochain lancement.", "Update installed: it takes effect next time you start Fanoratana.";
    MajAndroid => "Android va demander de confirmer l'installation.", "Android will ask you to confirm the installation.";
    MajErreur => "Mise à jour impossible :", "Could not update:";
}
