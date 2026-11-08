//! L'encre : le contour d'un trait à pression ([`contour`]), la reconnaissance des formes tracées à
//! main levée ([`formes`]), la rature ([`rature`]) et le regroupement des traits en lignes
//! d'écriture ([`lignes`]). Géométrie pure, sans modèle ni dépendance : quelques microsecondes.

pub mod contour;
pub mod formes;
pub mod graphe;
pub mod lignes;
pub mod rature;

pub use contour::{contour, squelette};
pub use formes::{FormeReconnue, reconnaitre_forme};
pub use lignes::{Ligne, bande_minuscules, grouper_lignes};
pub use rature::est_rature;
