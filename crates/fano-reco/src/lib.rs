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

pub mod formules;
pub mod lexique;
pub mod raster;
pub mod signes;
pub mod structure;

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
    /// La confiance de chaque caractère de `texte` (vide pour une formule).
    pub confiances: Vec<f32>,
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
        // Une confiance par caractère (un signe CTC peut faire plusieurs caractères : chacun la reçoit).
        let confiances: Vec<f32> = signes
            .iter()
            .flat_map(|s| std::iter::repeat_n(s.1, car(s.0).chars().count()))
            .collect();
        let debut = texte.chars().take_while(|c| c.is_whitespace()).count();
        let n = texte.trim().chars().count();
        Ok(Lecture {
            texte: texte.trim().to_owned(),
            confiance,
            alternatives,
            confiances: confiances.into_iter().skip(debut).take(n).collect(),
        })
    }
}

/// Une demande de lecture : un identifiant choisi par l'appelant, et l'image.
/// Comment lire : texte, formule (LaTeX), ou deviner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Auto,
    Texte,
    Formule,
}

/// Une demande de lecture : un identifiant choisi par l'appelant, l'encre, et le mode.
pub struct Demande {
    pub id: u64,
    pub traits: Vec<fano_core::Trait>,
    pub mode: Mode,
    /// Une formule demandée après une autre lecture (Google) : rendue telle quelle si la formule n'est
    /// pas lue avec assez de confiance.
    pub repli: Option<Lecture>,
}

/// Une lecture de texte ressemble-t-elle à une formule ? (signes d'opération, peu de vrais mots)
pub fn ressemble_formule(texte: &str) -> bool {
    const MOTS_MATHS: [&str; 8] = ["sin", "cos", "tan", "lim", "exp", "log", "ln", "sqrt"];
    let signes = texte.chars().filter(|c| "=+^<>×÷≤≥∫∑√".contains(*c)).count();
    let mots = texte
        .split(|c: char| !c.is_alphabetic())
        .filter(|m| m.chars().count() >= 3 && !MOTS_MATHS.contains(&m.to_lowercase().as_str()))
        .count();
    signes >= 1 && mots <= 1
}

/// Le fil de lecture : on lui envoie de l'encre, il rend les lectures dans l'ordre.
pub struct Lecteur {
    envoi: Sender<Demande>,
    pub resultats: Receiver<(u64, Result<Lecture, String>)>,
}

impl Lecteur {
    /// Démarre le fil ; `formules` : le dossier du modèle des formules (chargé à la première formule) ;
    /// `reveil` est appelé après chaque lecture (pour redessiner l'écran).
    pub fn demarrer(formules: std::path::PathBuf, reveil: impl Fn() + Send + 'static) -> Lecteur {
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
                let mut mfr: Option<formules::ModeleFormules> = None;
                let signes = signes::ClasseurSignes::charger()
                    .map_err(|e| log::error!("modèle des signes illisible : {e}"))
                    .ok();
                for d in demandes {
                    let r = lire_demande(&mut modele, signes.as_ref(), &mut mfr, &formules, &d);
                    if rendre.send((d.id, r)).is_err() {
                        break;
                    }
                    reveil();
                }
            })
            .expect("fil de lecture");
        Lecteur { envoi, resultats }
    }

    pub fn lire(&self, id: u64, traits: &[fano_core::Trait], mode: Mode) {
        let _ = self.envoi.send(Demande {
            id,
            traits: traits.to_vec(),
            mode,
            repli: None,
        });
    }

    /// Relit en formule une ligne déjà lue (par Google) ; si la formule n'est pas sûre, `repli` revient.
    pub fn lire_formule_sinon(&self, id: u64, traits: &[fano_core::Trait], repli: Lecture) {
        let _ = self.envoi.send(Demande {
            id,
            traits: traits.to_vec(),
            mode: Mode::Formule,
            repli: Some(repli),
        });
    }
}

/// Erreur rendue quand une formule est demandée sans le modèle.
pub const SANS_MODELE: &str = "modele-formules-absent";

/// Confiance à partir de laquelle une formule lue à sa structure est prise sans autre avis.
const STRUCTURE_SURE: f32 = 0.6;

/// La formule lue à sa structure contient-elle de quoi être une formule (pas seulement des lettres) ?
fn a_des_maths(latex: &str) -> bool {
    [
        "\\frac", "^", "_", "\\sqrt", "\\lim", "\\sum", "\\int", "=", "+", "\\times", "\\to", "<", ">",
        "\\leq", "\\geq",
    ]
    .iter()
    .any(|m| latex.contains(m))
}

/// Deux LaTeX égaux, aux espaces près.
fn meme_latex(a: &str, b: &str) -> bool {
    let n = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    n(a) == n(b)
}

/// Une formule (ADR 0013) : d'abord sa structure (signes + géométrie, quelques ms, rien d'inventé) ; si elle
/// doute, Pix2Text en second avis : d'accord, la lecture est sûre ; en désaccord, elle reste à valider
/// (confiance basse, l'autre lecture en voisine).
fn lire_formule(
    signes: Option<&signes::ClasseurSignes>,
    mfr: &mut Option<formules::ModeleFormules>,
    dossier: &std::path::Path,
    traits: &[fano_core::Trait],
    structure: Option<structure::LectureFormule>,
) -> Option<Lecture> {
    let structure = structure.or_else(|| {
        let pts: Vec<Vec<fano_core::Point>> = traits
            .iter()
            .map(|t| t.points.iter().map(|p| fano_core::Point::new(p.x, p.y)).collect())
            .collect();
        signes.and_then(|c| structure::lire_formule(c, &pts))
    });
    let dollars = |s: &str| format!("${s}$");
    if let Some(f) = &structure {
        if f.confiance >= STRUCTURE_SURE {
            return Some(Lecture {
                texte: dollars(&f.latex),
                confiance: f.confiance,
                alternatives: f.alternatives.iter().map(|a| dollars(a)).collect(),
                confiances: Vec::new(),
            });
        }
    }
    if mfr.is_none() && formules::present(dossier) {
        *mfr = formules::ModeleFormules::charger(dossier)
            .map_err(|e| log::error!("formules : {e}"))
            .ok();
    }
    let p2t = mfr
        .as_ref()
        .and_then(|m| m.lire(traits).map_err(|e| log::warn!("formules : {e}")).ok())
        .filter(|l| !l.is_empty());
    match (structure, p2t) {
        (Some(f), Some(p)) if meme_latex(&f.latex, &p) => Some(Lecture {
            texte: dollars(&f.latex),
            confiance: 0.9,
            alternatives: f.alternatives.iter().map(|a| dollars(a)).collect(),
            confiances: Vec::new(),
        }),
        (Some(f), p) => Some(Lecture {
            texte: dollars(&f.latex),
            confiance: f.confiance,
            alternatives: p
                .iter()
                .map(|p| dollars(p))
                .chain(f.alternatives.iter().map(|a| dollars(a)))
                .collect(),
            confiances: Vec::new(),
        }),
        // La structure n'est pas comprise (matrice, système) : Pix2Text seul, comme avant.
        (None, Some(p)) => Some(Lecture {
            texte: dollars(&p),
            confiance: 0.7,
            alternatives: Vec::new(),
            confiances: Vec::new(),
        }),
        (None, None) => None,
    }
}

fn lire_demande(
    modele: &mut Modele,
    signes: Option<&signes::ClasseurSignes>,
    mfr: &mut Option<formules::ModeleFormules>,
    dossier: &std::path::Path,
    d: &Demande,
) -> Result<Lecture, String> {
    if d.mode == Mode::Formule {
        return match (lire_formule(signes, mfr, dossier, &d.traits, None), &d.repli) {
            (Some(f), Some(r)) if f.confiance < STRUCTURE_SURE => {
                // Google a lu la ligne : sa lecture reste, la formule devient une lecture voisine.
                let mut r = r.clone();
                r.alternatives.insert(0, f.texte);
                Ok(r)
            }
            (Some(f), _) => Ok(f),
            (None, Some(r)) => Ok(r.clone()),
            (None, None) => Err(if formules::present(dossier) {
                "formule illisible".into()
            } else {
                SANS_MODELE.into()
            }),
        };
    }
    // Un bloc sur plusieurs rangées (fraction, limite, matrice) n'est pas une ligne de texte : lu en formule,
    // ou laissé à l'encre s'il n'est pas compris (jamais un texte inventé).
    let boites: Vec<fano_core::Rect> = d.traits.iter().map(fano_core::Trait::bbox).collect();
    if d.mode == Mode::Auto && fano_ink::grouper_lignes(&boites).len() > 1 {
        return lire_formule(signes, mfr, dossier, &d.traits, None)
            .ok_or_else(|| "formule illisible".to_owned());
    }
    let (gris, largeur) = rasteriser(&d.traits).ok_or("rien à lire")?;
    let mut l = modele.lire_image(&gris, largeur).map_err(|e| e.to_string())?;
    if d.mode == Mode::Auto {
        // La structure se lit en quelques millisecondes : une ligne qui y est sûre et a des maths est une formule.
        let pts: Vec<Vec<fano_core::Point>> = d
            .traits
            .iter()
            .map(|t| t.points.iter().map(|p| fano_core::Point::new(p.x, p.y)).collect())
            .collect();
        let structure = signes.and_then(|c| structure::lire_formule(c, &pts));
        let sure = structure
            .as_ref()
            .is_some_and(|f| f.confiance >= 0.8 && a_des_maths(&f.latex));
        if sure || ressemble_formule(&l.texte) {
            if let Some(f) = lire_formule(signes, mfr, dossier, &d.traits, structure) {
                if f.confiance >= l.confiance.min(STRUCTURE_SURE) || sure {
                    let mut f = f;
                    f.alternatives.push(l.texte);
                    return Ok(f);
                }
            }
        }
    }
    // Texte : les mots incertains sont corrigés par le lexique français ; la lecture brute devient une
    // lecture voisine, pour revenir en arrière d'un toucher.
    if !ressemble_formule(&l.texte) {
        let corrige = lexique::corriger(&l.texte, &l.confiances);
        if corrige != l.texte {
            let brut = std::mem::replace(&mut l.texte, corrige);
            l.alternatives.insert(0, brut);
        }
    }
    Ok(l)
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

#[cfg(test)]
mod tests_formule {
    use super::ressemble_formule;

    #[test]
    fn texte_ou_formule() {
        assert!(ressemble_formule("x+1=5"));
        assert!(ressemble_formule("f(x)=2x+3"));
        assert!(ressemble_formule("sin(x)=0"));
        assert!(!ressemble_formule("Les fractions égales"));
        assert!(!ressemble_formule("En 1789, les États généraux = Versailles"));
        assert!(!ressemble_formule("1789"));
    }
}
