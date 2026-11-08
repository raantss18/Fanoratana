//! Les réglages, gardés dans `reglages.json` du dossier de l'app.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reglages {
    /// Le doigt écrit ; sinon il déplace la page (le stylet écrit toujours).
    pub doigt_ecrit: bool,
    /// Mise au propre toute seule après une pause.
    pub au_propre_auto: bool,
    /// Pause avant la mise au propre, en ms.
    pub delai_ms: u64,
    /// Seuil de confiance (0..1) sous lequel l'encre reste.
    pub seuil: f32,
    /// `fr` ou `en` ; vide = la langue du système.
    pub langue: String,
    /// Le coffre ; vide = `<données>/Cours`.
    pub coffre: String,
    /// La dernière leçon ouverte.
    pub derniere_lecon: String,
    /// Chercher les mises à jour une fois par jour.
    pub maj_auto: bool,
    /// Dernière recherche (secondes depuis 1970).
    pub derniere_verif: u64,
    /// La lecture Google (ML Kit, Android) : `None` tant que la personne n'a pas choisi.
    pub lecture_google: Option<bool>,
    /// La langue de l'écriture : `fr`, `en` ou `mg`.
    pub langue_ecriture: String,
    /// Dernier rafraîchissement du modèle d'écriture (secondes depuis 1970).
    pub maj_modele_encre: u64,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            doigt_ecrit: true,
            au_propre_auto: true,
            delai_ms: 1200,
            seuil: 0.6,
            langue: String::new(),
            coffre: String::new(),
            derniere_lecon: String::new(),
            maj_auto: true,
            derniere_verif: 0,
            lecture_google: None,
            langue_ecriture: "fr".into(),
            maj_modele_encre: 0,
        }
    }
}

impl Reglages {
    fn fichier(dossier: &Path) -> PathBuf {
        dossier.join("reglages.json")
    }
    pub fn lire(dossier: &Path) -> Reglages {
        std::fs::read_to_string(Self::fichier(dossier))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn ecrire(&self, dossier: &Path) {
        let _ = std::fs::create_dir_all(dossier);
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fano_core::coffre::ecrire_atomique(&Self::fichier(dossier), s.as_bytes());
        }
    }
}
