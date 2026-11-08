//! Le modèle de Fanoratana.
//!
//! - [`page`] : une page de leçon, ses traits d'encre et ses éléments au propre (texte, formes, images).
//! - [`coffre`] : le coffre de cours sur disque, façon Obsidian : un dossier par niveau, un dossier
//!   par chapitre, un fichier `.fano` (JSON) par leçon. Rien n'est caché : on peut tout copier,
//!   synchroniser ou versionner avec les outils de son choix (ADR 0004).

pub mod coffre;
pub mod geom;
pub mod page;

pub use coffre::{Coffre, Entree, Erreur};
pub use geom::{Point, Rect, Vec2};
pub use page::{
    Alignement, Couleur, Element, Encadre, Fond, Format, Forme, FormeKind, Graphe, ImageEl, Lecon, Liste,
    Page, PointEncre, Tableau, Texte, Trait,
};
