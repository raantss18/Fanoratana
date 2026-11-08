//! Ce qui change d'une plateforme à l'autre : choisir un fichier, enregistrer un export, savoir si
//! c'est un stylet qui touche l'écran, montrer le clavier.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointeur {
    Doigt,
    Stylet,
    Souris,
}

pub trait Plateforme {
    /// Le dossier des données de l'app (le coffre par défaut s'y trouve).
    fn dossier_donnees(&self) -> PathBuf;
    /// Demande une image à l'utilisateur ; la réponse arrive par [`Plateforme::image_choisie`].
    fn demander_image(&mut self);
    /// L'image choisie (nom, octets), une seule fois.
    fn image_choisie(&mut self) -> Option<(String, Vec<u8>)>;
    /// Enregistre un export ; rend l'endroit où il a été mis, ou `None` si la personne a renoncé.
    fn enregistrer(&mut self, nom: &str, mime: &str, octets: Vec<u8>) -> Result<Option<String>, String>;
    /// Ce qui touche l'écran pour ce contact (identifiant egui de toucher), `None` à la souris.
    fn pointeur(&self, toucher: Option<u64>) -> Pointeur {
        if toucher.is_some() {
            Pointeur::Doigt
        } else {
            Pointeur::Souris
        }
    }
    /// Montrer ou cacher le clavier virtuel (Android).
    fn clavier(&mut self, _visible: bool) {}
    /// Installe une mise à jour téléchargée et vérifiée ; rend ce que la personne doit savoir.
    fn installer_maj(&mut self, fichier: &std::path::Path) -> Result<String, String> {
        fano_maj::remplacer_programme(fichier).map(|_| String::new())
    }
    /// Les marges du système (barre d'état, navigation, encoche, clavier), en pixels physiques :
    /// [gauche, haut, droite, bas]. Les commandes flottantes restent dedans.
    fn marges(&mut self) -> [f32; 4] {
        [0.0; 4]
    }
    /// La langue du système (`fr_FR`, `en-US`…).
    fn langue(&self) -> String {
        std::env::var("LANG").unwrap_or_default()
    }
}
