//! Lire l'écriture, hors ligne (ADR 0003).
//!
//! 1. Les traits d'une ligne sont dessinés en image (48 px de haut, encre sombre sur blanc), avec le
//!    même contour que l'écran ([`raster`]).
//! 2. L'image est lue par `latin_PP-OCRv5_mobile_rec` (PaddleOCR, Apache-2.0, 8 Mo, embarqué), exécuté
//!    par tract (Rust pur, MIT/Apache) : aucune bibliothèque native, même code sur Linux et Android.
//! 3. Décodage CTC glouton, avec une confiance par signe et des lectures voisines pour corriger d'un
//!    toucher ([`Lecture::alternatives`]).
//!
//! La lecture tourne sur un fil à part ([`Lecteur`]) : l'écriture n'attend jamais.

pub mod raster;

use std::sync::mpsc::{Receiver, Sender, channel};

use tract_onnx::prelude::*;

pub use raster::rasteriser;

static MODELE: &[u8] = include_bytes!("../../../assets/modeles/latin_PP-OCRv5_mobile_rec.onnx");
static DICO: &str = include_str!("../../../assets/modeles/latin_PP-OCRv5_dict.txt");

/// Hauteur d'entrée du modèle.
pub const HAUTEUR: u32 = 48;
/// Largeurs préparées : une image est complétée à la première qui la contient.
const LARGEURS: [usize; 4] = [320, 640, 1280, 2560];

type Plan = std::sync::Arc<TypedSimplePlan>;

/// Le résultat d'une lecture.
#[derive(Clone, Debug, PartialEq)]
pub struct Lecture {
    pub texte: String,
    /// Confiance moyenne 0..1.
    pub confiance: f32,
    /// D'autres lectures plausibles, de la plus probable à la moins probable (sans `texte`).
    pub alternatives: Vec<String>,
}

pub struct Modele {
    plans: Vec<(usize, Plan)>,
    dico: Vec<String>,
}

impl Modele {
    /// Charge le modèle embarqué (≈ 0,2 s par largeur sur PC ; fait une fois, sur le fil de lecture).
    pub fn charger() -> TractResult<Modele> {
        let mut dico: Vec<String> = DICO.lines().map(String::from).collect();
        dico.push(" ".into());
        Ok(Modele {
            plans: Vec::new(),
            dico,
        })
    }

    fn plan(&mut self, largeur: usize) -> TractResult<&Plan> {
        if let Some(i) = self.plans.iter().position(|(w, _)| *w == largeur) {
            return Ok(&self.plans[i].1);
        }
        let m = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(MODELE))?
            .with_input_fact(0, f32::fact([1, 3, HAUTEUR as usize, largeur]).into())?
            .into_optimized()?
            .into_runnable()?;
        self.plans.push((largeur, m));
        Ok(&self.plans.last().unwrap().1)
    }

    /// Lit une image en niveaux de gris (0 = noir), `HAUTEUR` px de haut, `largeur` px de large.
    pub fn lire_image(&mut self, gris: &[u8], largeur: usize) -> TractResult<Lecture> {
        let w = LARGEURS
            .iter()
            .copied()
            .find(|&w| w >= largeur)
            .unwrap_or(*LARGEURS.last().unwrap());
        let h = HAUTEUR as usize;
        let entree: Tensor = tract_ndarray::Array4::from_shape_fn((1, 3, h, w), |(_, _, y, x)| {
            if x >= largeur.min(w) {
                return 0.0; // remplissage de PaddleOCR (gris moyen après normalisation)
            }
            (f32::from(gris[y * largeur + x]) / 255.0 - 0.5) / 0.5
        })
        .into();
        let plan = self.plan(w)?;
        let sortie = plan.run(tvec!(entree.into()))?;
        let o = sortie[0].to_plain_array_view::<f32>()?;
        let (t_max, classes) = (o.shape()[1], o.shape()[2]);
        // Décodage CTC glouton ; on garde, par signe, sa probabilité et son second choix.
        let mut signes: Vec<(usize, f32, usize, f32)> = Vec::new();
        let mut prec = 0;
        for t in 0..t_max {
            let (mut b1, mut v1, mut b2, mut v2) = (0, f32::MIN, 0, f32::MIN);
            for c in 0..classes {
                let v = o[[0, t, c]];
                if v > v1 {
                    (b2, v2) = (b1, v1);
                    (b1, v1) = (c, v);
                } else if v > v2 {
                    (b2, v2) = (c, v);
                }
            }
            // Un signe trop incertain n'est pas écrit (une page blanche ne doit rien lire).
            if b1 != 0 && b1 != prec && v1 >= 0.35 {
                signes.push((b1, v1, b2, v2));
            } else if b1 != 0 && b1 == prec {
                if let Some(s) = signes.last_mut() {
                    s.1 = s.1.max(v1);
                }
            }
            prec = b1;
        }
        let car = |c: usize| self.dico.get(c - 1).cloned().unwrap_or_default();
        let texte: String = signes.iter().map(|s| car(s.0)).collect();
        let confiance = if signes.is_empty() {
            0.0
        } else {
            signes.iter().map(|s| s.1).sum::<f32>() / signes.len() as f32
        };
        // Alternatives : on remplace l'un des trois signes les moins sûrs par son second choix.
        let mut doutes: Vec<usize> = (0..signes.len())
            .filter(|&i| signes[i].2 != 0 && signes[i].3 > 0.05)
            .collect();
        doutes.sort_by(|&a, &b| signes[b].3.total_cmp(&signes[a].3));
        let alternatives = doutes
            .into_iter()
            .take(3)
            .map(|i| {
                signes
                    .iter()
                    .enumerate()
                    .map(|(j, s)| if j == i { car(s.2) } else { car(s.0) })
                    .collect::<String>()
            })
            .filter(|a| *a != texte)
            .collect();
        Ok(Lecture {
            texte: texte.trim().to_owned(),
            confiance,
            alternatives,
        })
    }
}

/// Une demande de lecture : un identifiant choisi par l'appelant, et l'image.
pub struct Demande {
    pub id: u64,
    pub gris: Vec<u8>,
    pub largeur: usize,
}

/// Le fil de lecture : on lui envoie des images, il rend les lectures dans l'ordre.
pub struct Lecteur {
    envoi: Sender<Demande>,
    pub resultats: Receiver<(u64, Result<Lecture, String>)>,
}

impl Lecteur {
    /// Démarre le fil ; `reveil` est appelé après chaque lecture (pour redessiner l'écran).
    pub fn demarrer(reveil: impl Fn() + Send + 'static) -> Lecteur {
        let (envoi, demandes) = channel::<Demande>();
        let (rendre, resultats) = channel();
        std::thread::Builder::new()
            .name("lecture".into())
            .spawn(move || {
                let mut modele = match Modele::charger() {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("modèle de lecture illisible : {e}");
                        return;
                    }
                };
                // Prépare la largeur la plus courante pendant qu'on écrit.
                let _ = modele.plan(LARGEURS[1]);
                for d in demandes {
                    let r = modele.lire_image(&d.gris, d.largeur).map_err(|e| e.to_string());
                    if rendre.send((d.id, r)).is_err() {
                        break;
                    }
                    reveil();
                }
            })
            .expect("fil de lecture");
        Lecteur { envoi, resultats }
    }

    pub fn lire(&self, id: u64, traits: &[fano_core::Trait]) {
        if let Some((gris, largeur)) = rasteriser(traits) {
            let _ = self.envoi.send(Demande { id, gris, largeur });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionnaire_complet() {
        let m = Modele::charger().unwrap();
        assert_eq!(
            m.dico.len(),
            837,
            "836 signes + l'espace ; la classe 0 est le blanc CTC"
        );
        assert!(m.dico.iter().any(|c| c == "é") && m.dico.iter().any(|c| c == "ç"));
    }

    #[test]
    fn une_image_blanche_ne_lit_rien() {
        let mut m = Modele::charger().unwrap();
        let gris = vec![255u8; HAUTEUR as usize * 200];
        let l = m.lire_image(&gris, 200).unwrap();
        assert!(l.texte.is_empty(), "lu {:?} ({})", l.texte, l.confiance);
    }
}
