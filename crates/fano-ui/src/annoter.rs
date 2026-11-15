//! Annoter un PDF ou une image (TODO § 3) : le document devient une leçon dont chaque page a le document
//! en fond verrouillé ; on écrit, tape et dessine par-dessus, et l'export garde tout.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fano_core::Lecon;
use fano_render::document;

use crate::app::Fanoratana;
use crate::textes::{T, t};

/// Où en est la préparation d'un document (sur un fil : un long PDF prend du temps).
#[derive(Default)]
pub(crate) enum Import {
    #[default]
    Rien,
    Pages(usize, usize),
    Pret(Result<Pret, String>),
}

pub(crate) struct Pret {
    titre: String,
    largeur: f32,
    hauteur: f32,
    /// (nom de fichier, octets) de chaque page.
    pages: Vec<(String, Vec<u8>)>,
}

pub(crate) type Partage = Arc<Mutex<Import>>;

/// Le titre de la leçon d'après le nom du fichier (`Sujet bac 2024.pdf` → « Sujet bac 2024 »).
fn titre_de(nom: &str) -> String {
    let base = nom.rsplit_once('.').map_or(nom, |(b, _)| b).trim();
    if base.is_empty() {
        "Document".into()
    } else {
        base.to_owned()
    }
}

/// Prépare le document (PDF ou image) en pages.
fn preparer(nom: &str, octets: &[u8], progres: impl Fn(usize, usize)) -> Result<Pret, String> {
    let titre = titre_de(nom);
    if octets.len() > document::OCTETS_MAX {
        return Err(format!(
            "document de plus de {} Mo",
            document::OCTETS_MAX / (1024 * 1024)
        ));
    }
    if document::est_pdf(octets) {
        let d = document::pdf(octets, progres)?;
        let base = fano_core::coffre::nom_sur(&titre);
        let pages = d
            .pages
            .into_iter()
            .enumerate()
            .map(|(i, p)| (format!("{base}-p{}.png", i + 1), p.png))
            .collect();
        return Ok(Pret {
            titre,
            largeur: d.largeur,
            hauteur: d.hauteur,
            pages,
        });
    }
    let (w, h) = crate::peinture::taille_image(octets, nom).ok_or("ni PDF ni image lisible")?;
    let (largeur, hauteur) = document::image(w, h);
    Ok(Pret {
        titre,
        largeur,
        hauteur,
        pages: vec![(nom.to_owned(), octets.to_vec())],
    })
}

/// Range les pages dans le coffre et écrit la leçon dans `chapitre`. Rend son chemin.
pub(crate) fn ranger(coffre: &fano_core::Coffre, chapitre: &Path, p: Pret) -> Result<PathBuf, String> {
    let mut images = Vec::with_capacity(p.pages.len());
    for (nom, octets) in &p.pages {
        images.push(coffre.importer_media(nom, octets).map_err(|e| e.to_string())?);
    }
    let chemin = coffre
        .creer_lecon(chapitre, &p.titre, fano_core::Format::A4)
        .or_else(|_| coffre.creer_lecon(chapitre, &format!("{} (annoté)", p.titre), fano_core::Format::A4))
        .map_err(|e| e.to_string())?;
    let lecon = Lecon::annotee(&p.titre, p.largeur, p.hauteur, images);
    coffre.ecrire_lecon(&chemin, &lecon).map_err(|e| e.to_string())?;
    Ok(chemin)
}

/// Le chapitre « Documents › À annoter » du coffre, créé s'il manque.
pub(crate) fn chapitre_des_documents(
    coffre: &fano_core::Coffre,
    langue: crate::textes::Langue,
) -> Result<PathBuf, String> {
    let (niveau, chapitre) = (t(langue, T::NiveauDocuments), t(langue, T::ChapitreAAnnoter));
    let trouve = |dans: &Path, nom: &str| {
        std::fs::read_dir(dans)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.is_dir() && fano_core::coffre::nom_montre(p) == nom)
    };
    let n = match trouve(&coffre.racine, niveau) {
        Some(n) => n,
        None => coffre.creer_niveau(niveau).map_err(|e| e.to_string())?,
    };
    match trouve(&n, chapitre) {
        Some(c) => Ok(c),
        None => coffre.creer_chapitre(&n, chapitre).map_err(|e| e.to_string()),
    }
}

impl Fanoratana {
    /// « Annoter un PDF ou une image » : demande le fichier.
    pub(crate) fn demander_document(&mut self) {
        self.document_demande = true;
        self.plateforme.demander_document();
    }

    /// Un document reçu d'ailleurs (« Ouvrir avec », « Partager vers Fanoratana », glisser-déposer, fichier
    /// donné au lancement) : il attend son tour, puis devient une leçon à annoter.
    pub fn recevoir_document(&mut self, nom: String, octets: Vec<u8>) {
        self.documents_recus.push((nom, octets));
    }

    /// Où ranger la leçon annotée : le chapitre de la leçon ouverte ; sans leçon ouverte (document reçu d'une
    /// autre app), le chapitre « Documents › À annoter », créé au besoin.
    fn chapitre_pour_document(&self) -> Result<PathBuf, String> {
        if let Some(c) = self.ouverte.as_ref().and_then(|o| o.chemin.parent()) {
            return Ok(c.to_path_buf());
        }
        chapitre_des_documents(&self.coffre, self.langue)
    }

    /// Le fichier choisi : préparé sur un fil.
    pub(crate) fn importer_document(&mut self, nom: String, octets: Vec<u8>, ctx: &egui::Context) {
        let etat: Partage = Arc::new(Mutex::new(Import::Pages(0, 0)));
        self.import = Some(etat.clone());
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let e2 = etat.clone();
            let c2 = ctx.clone();
            // Un PDF abîmé peut faire paniquer le lecteur : l'import échoue alors proprement, au lieu de rester
            // « en préparation » pour toujours et de bloquer les suivants.
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                preparer(&nom, &octets, move |fait, total| {
                    if let Ok(mut e) = e2.lock() {
                        *e = Import::Pages(fait, total);
                    }
                    c2.request_repaint();
                })
            }))
            .unwrap_or_else(|_| Err("document illisible".into()));
            *etat.lock().unwrap_or_else(|e| e.into_inner()) = Import::Pret(r);
            ctx.request_repaint();
        });
    }

    /// Chaque image : la progression, puis la leçon ouverte.
    pub(crate) fn suivre_import(&mut self, ctx: &egui::Context) {
        // Les fichiers lâchés sur la fenêtre (Linux).
        let laches = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
        for f in laches {
            let nom = f
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "document".into());
            match f.bytes() {
                Ok(o) => self.recevoir_document(nom, o),
                Err(e) => {
                    let m = format!("{} {e}", t(self.langue, T::Erreur));
                    self.dire(m);
                }
            }
        }
        let Some(etat) = self.import.clone() else {
            // Rien en cours : le document reçu suivant.
            if !self.documents_recus.is_empty() {
                let (nom, octets) = self.documents_recus.remove(0);
                self.importer_document(nom, octets, ctx);
            }
            return;
        };
        let fini = std::mem::take(&mut *etat.lock().unwrap());
        match fini {
            Import::Rien => {}
            Import::Pages(fait, total) => {
                let m = if total == 0 {
                    t(self.langue, T::DocumentPrepare).to_owned()
                } else {
                    format!("{} {fait} / {total}", t(self.langue, T::DocumentPrepare))
                };
                self.dire(m);
                *etat.lock().unwrap() = Import::Pages(fait, total);
            }
            Import::Pret(r) => {
                self.import = None;
                let chapitre = self.chapitre_pour_document();
                match r.and_then(|p| ranger(&self.coffre, &chapitre?, p)) {
                    Ok(chemin) => {
                        self.rafraichir_arbre();
                        self.ouvrir(chemin);
                        let m = t(self.langue, T::DocumentPret).to_owned();
                        self.dire(m);
                    }
                    Err(e) => {
                        let m = format!("{} {e}", t(self.langue, T::Erreur));
                        self.dire(m);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titre_d_apres_le_fichier() {
        assert_eq!(titre_de("Sujet bac 2024.pdf"), "Sujet bac 2024");
        assert_eq!(titre_de("photo.JPG"), "photo");
        assert_eq!(titre_de(".pdf"), "Document");
    }

    #[test]
    fn un_document_recu_sans_lecon_ouverte_a_son_chapitre() {
        let dir = tempfile::tempdir().unwrap();
        let coffre = fano_core::Coffre::ouvrir(dir.path()).unwrap();
        let c1 = chapitre_des_documents(&coffre, crate::textes::Langue::Fr).unwrap();
        // La seconde fois, le même chapitre (pas de doublon).
        let c2 = chapitre_des_documents(&coffre, crate::textes::Langue::Fr).unwrap();
        assert_eq!(c1, c2);
        assert_eq!(fano_core::coffre::nom_montre(&c1), "À annoter");
        assert_eq!(fano_core::coffre::nom_montre(c1.parent().unwrap()), "Documents");
    }

    #[test]
    fn une_photo_devient_une_lecon_a_annoter() {
        let dir = tempfile::tempdir().unwrap();
        let coffre = fano_core::Coffre::ouvrir(dir.path()).unwrap();
        let chap = coffre
            .creer_chapitre(&coffre.creer_niveau("Seconde").unwrap(), "Maths")
            .unwrap();
        // Une image PNG de 40 × 30.
        let png = tiny_skia::Pixmap::new(40, 30).unwrap().encode_png().unwrap();
        let p = preparer("exercice.png", &png, |_, _| {}).unwrap();
        let chemin = ranger(&coffre, &chap, p).unwrap();
        let l = coffre.lire_lecon(&chemin).unwrap();
        assert_eq!(l.titre, "exercice");
        assert_eq!(l.format.taille(), (842.0, 632.0));
        let fond = l.pages[0].fond_image.clone().unwrap();
        assert!(dir.path().join(&fond).exists(), "{fond} rangé dans le coffre");
        assert!(l.pages[0].elements.is_empty());
    }
}
