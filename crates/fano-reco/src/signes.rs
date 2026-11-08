//! Le petit modèle par signe, repris de Fikajiana (même autrice, GPL ; ADR 0038, 0040, 0041 là-bas ;
//! ADR 0013 ici) : un réseau convolutif lit **un signe isolé** (chiffre, lettre, opérateur, grec) à partir
//! de son image 32×32 et de cinq mesures du tracé. 1,6 Mo, ~1 ms par signe, embarqué.
//!
//! Le rendu suit exactement le contrat v1 de Fikajiana (`entrainement/CONTRAT.md`, `rendu.py`) : toile
//! 128×128 sans anticrénelage, chemin quadratique par les milieux (8 pas), trait de 8 px à bouts ronds,
//! moyenne des blocs 4×4. Le test `rendu_comme_fikajiana` le compare aux images de référence.

use std::sync::Arc;

use fano_core::Point;
use tiny_skia::{LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};
use tract_onnx::prelude::*;

static MODELE: &[u8] = include_bytes!("../../../assets/modeles/signes.onnx");
static CLASSES: &str = include_str!("../../../assets/modeles/signes_classes.txt");

const TAILLE: usize = 32;
const SUR: usize = 4;
const MARGE_PX: f32 = 2.0;
const EPAISSEUR_PX: f32 = 2.0;
const COTE_MINIMAL: f32 = 0.15;
const PAS_QUADRATIQUE: usize = 8;

/// Une classe du modèle : son identifiant stable (`par_g`, `x`, `somme`…), le signe et son LaTeX.
#[derive(Clone, Debug)]
pub struct Classe {
    pub id: String,
    pub signe: String,
    pub latex: String,
}

/// La case de référence d'un signe : le haut et la hauteur de sa ligne d'écriture.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub haut: f32,
    pub hauteur: f32,
}

pub struct ClasseurSignes {
    plan: Arc<TypedSimplePlan>,
    pub classes: Vec<Classe>,
}

/// Les meilleures classes d'un signe, de la plus probable à la moins probable : (indice, probabilité).
pub type Choix = Vec<(usize, f32)>;

impl ClasseurSignes {
    pub fn charger() -> TractResult<ClasseurSignes> {
        let classes: Vec<Classe> = CLASSES
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .map(|l| {
                let mut c = l.split('\t');
                let id = c.next().unwrap_or_default().to_owned();
                let signe = c.next().unwrap_or_default().to_owned();
                let latex = c.next().unwrap_or(&signe).to_owned();
                Classe { id, signe, latex }
            })
            .collect();
        let plan = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(MODELE))?
            .with_input_fact(0, f32::fact([1, 1, TAILLE, TAILLE]).into())?
            .with_input_fact(1, f32::fact([1, 5]).into())?
            .into_optimized()?
            .into_runnable()?;
        Ok(ClasseurSignes { plan, classes })
    }

    /// Classe un signe (ses traits, en points de la page) dans sa case ; rend les `n` meilleures classes.
    pub fn classer(&self, traits: &[&[Point]], case: Case, n: usize) -> TractResult<Choix> {
        let image = rendre(traits, case);
        let mesures = mesurer(traits, case);
        let image: Tensor =
            tract_ndarray::Array4::from_shape_vec((1, 1, TAILLE, TAILLE), image.to_vec())?.into();
        let mesures: Tensor = tract_ndarray::Array2::from_shape_vec((1, 5), mesures.to_vec())?.into();
        let sortie = self.plan.run(tvec!(image.into(), mesures.into()))?;
        let logits = sortie[0].to_plain_array_view::<f32>()?;
        let l: Vec<f32> = logits.iter().copied().collect();
        let max = l.iter().copied().fold(f32::MIN, f32::max);
        let e: Vec<f32> = l.iter().map(|v| (v - max).exp()).collect();
        let somme: f32 = e.iter().sum();
        let mut choix: Choix = e.iter().enumerate().map(|(i, v)| (i, v / somme)).collect();
        choix.sort_by(|a, b| b.1.total_cmp(&a.1));
        choix.truncate(n);
        Ok(choix)
    }

    /// L'indice de la classe `id` (`plus`, `x`…).
    pub fn indice(&self, id: &str) -> Option<usize> {
        self.classes.iter().position(|c| c.id == id)
    }
}

fn boite(traits: &[&[Point]]) -> (f32, f32, f32, f32) {
    let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in traits.iter().flat_map(|t| t.iter()) {
        b = (b.0.min(p.x), b.1.min(p.y), b.2.max(p.x), b.3.max(p.y));
    }
    b
}

/// Le chemin d'un trait aplati en segments, comme `_chemin` de `rendu.py`.
fn chemin(points: &[Point]) -> Vec<Point> {
    if points.len() <= 1 {
        return points.to_vec();
    }
    let mut sortie = vec![points[0]];
    let mut courant = points[0];
    for i in 1..points.len() - 1 {
        let (a, b) = (points[i], points[i + 1]);
        let fin = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        for j in 1..=PAS_QUADRATIQUE {
            let t = j as f32 / PAS_QUADRATIQUE as f32;
            let u = 1.0 - t;
            sortie.push(Point::new(
                u * u * courant.x + 2.0 * u * t * a.x + t * t * fin.x,
                u * u * courant.y + 2.0 * u * t * a.y + t * t * fin.y,
            ));
        }
        courant = fin;
    }
    sortie.push(points[points.len() - 1]);
    sortie
}

/// L'image 32×32 du signe (0 fond, 1 encre), rangée ligne par ligne.
pub fn rendre(traits: &[&[Point]], case: Case) -> [f32; TAILLE * TAILLE] {
    let mut res = [0f32; TAILLE * TAILLE];
    let traits: Vec<&[Point]> = traits.iter().copied().filter(|t| !t.is_empty()).collect();
    if traits.is_empty() {
        return res;
    }
    let (x0, y0, x1, y1) = boite(&traits);
    let mut cote = (x1 - x0).max(y1 - y0).max(COTE_MINIMAL * case.hauteur);
    if cote <= 0.0 {
        cote = 1.0;
    }
    let n = (TAILLE * SUR) as f32;
    let k = (TAILLE as f32 - 2.0 * MARGE_PX) * SUR as f32 / cote;
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    // Pillow trace une ligne de 8 px un peu plus large qu'un trait de 8 px : 9 px l'égale (écart moyen 0,004).
    let largeur = EPAISSEUR_PX * SUR as f32 + 1.0;
    let Some(mut pix) = Pixmap::new(n as u32, n as u32) else {
        return res;
    };
    let mut peinture = Paint::default();
    peinture.set_color_rgba8(255, 255, 255, 255);
    peinture.anti_alias = false;
    let trait_ = Stroke {
        width: largeur,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    for t in &traits {
        let pts: Vec<Point> = chemin(t)
            .into_iter()
            .map(|p| Point::new((p.x - cx) * k + n / 2.0, (p.y - cy) * k + n / 2.0))
            .collect();
        let mut pb = PathBuilder::new();
        if pts.len() == 1 {
            pb.push_circle(pts[0].x, pts[0].y, largeur / 2.0);
            if let Some(p) = pb.finish() {
                pix.fill_path(
                    &p,
                    &peinture,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
            continue;
        }
        pb.move_to(pts[0].x, pts[0].y);
        for p in &pts[1..] {
            pb.line_to(p.x, p.y);
        }
        if let Some(p) = pb.finish() {
            pix.stroke_path(&p, &peinture, &trait_, Transform::identity(), None);
        }
    }
    let data = pix.data();
    let ligne = TAILLE * SUR;
    for y in 0..TAILLE {
        for x in 0..TAILLE {
            let mut s = 0u32;
            for dy in 0..SUR {
                for dx in 0..SUR {
                    s += u32::from(data[((y * SUR + dy) * ligne + x * SUR + dx) * 4]);
                }
            }
            res[y * TAILLE + x] = s as f32 / (255.0 * (SUR * SUR) as f32);
        }
    }
    res
}

/// Les cinq mesures du tracé : rapport, traits, hauteur, largeur, centre.
pub fn mesurer(traits: &[&[Point]], case: Case) -> [f32; 5] {
    let traits: Vec<&[Point]> = traits.iter().copied().filter(|t| !t.is_empty()).collect();
    if traits.is_empty() {
        return [0.0; 5];
    }
    let (x0, y0, x1, y1) = boite(&traits);
    let (w, h) = (x1 - x0, y1 - y0);
    let hr = case.hauteur.max(1e-6);
    [
        ((w + 0.02 * hr) / (h + 0.02 * hr)).ln().clamp(-3.0, 3.0),
        traits.len().min(4) as f32 / 4.0,
        (h / hr).clamp(0.0, 2.0),
        (w / hr).clamp(0.0, 3.0),
        (((y0 + y1) / 2.0 - case.haut) / hr).clamp(-0.5, 1.5),
    ]
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Un trait de Fikajiana (fixture JSON) en points.
    pub(crate) fn traits_fixture(nom: &str) -> Vec<Vec<Point>> {
        let chemin = format!("{}/tests/fikajiana/{nom}.json", env!("CARGO_MANIFEST_DIR"));
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(chemin).unwrap()).unwrap();
        v.as_array()
            .unwrap()
            .iter()
            .map(|t| {
                t["points"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| Point::new(p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32))
                    .collect()
            })
            .collect()
    }

    /// Le rendu et les mesures égalent ceux de `rendu.py` (images de référence de Fikajiana).
    #[test]
    fn rendu_comme_fikajiana() {
        let chemin = format!("{}/tests/fikajiana/rendu_signes.json", env!("CARGO_MANIFEST_DIR"));
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(chemin).unwrap()).unwrap();
        let mut ecarts = Vec::new();
        for s in v["signes"].as_array().unwrap() {
            let traits = traits_fixture(s["fixture"].as_str().unwrap());
            let choisis: Vec<&[Point]> = s["traits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| traits[i.as_u64().unwrap() as usize].as_slice())
                .collect();
            let c: Vec<f32> = s["case"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect();
            let case = Case {
                haut: c[1],
                hauteur: c[3],
            };
            let image = rendre(&choisis, case);
            let attendu: Vec<f32> = s["image16"]
                .as_str()
                .unwrap()
                .chars()
                .map(|ch| ch.to_digit(17).unwrap() as f32 / 16.0)
                .collect();
            let ecart = image
                .iter()
                .zip(&attendu)
                .map(|(a, b)| (a - b).abs())
                .sum::<f32>()
                / 1024.0;
            ecarts.push(ecart);

            let m = mesurer(&choisis, case);
            for (a, b) in m.iter().zip(s["mesures"].as_array().unwrap()) {
                assert!(
                    (a - b.as_f64().unwrap() as f32).abs() < 1e-3,
                    "{m:?} {}",
                    s["mesures"]
                );
            }
        }
        let moyen = ecarts.iter().sum::<f32>() / ecarts.len() as f32;
        let pire = ecarts.iter().copied().fold(0.0, f32::max);
        assert!(
            moyen < 0.006 && pire < 0.015,
            "écart moyen {moyen}, pire {pire} sur {}",
            ecarts.len()
        );
    }

    /// Les signes sûrs des fixtures (relevés à l'œil par Antsa) : le modèle en lit la plupart.
    #[test]
    fn signes_de_la_main_d_antsa() {
        let m = ClasseurSignes::charger().unwrap();
        let chemin = format!("{}/tests/fikajiana/rendu_signes.json", env!("CARGO_MANIFEST_DIR"));
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(chemin).unwrap()).unwrap();
        let (mut justes, mut total) = (0, 0);
        for s in v["signes"].as_array().unwrap() {
            let traits = traits_fixture(s["fixture"].as_str().unwrap());
            let choisis: Vec<&[Point]> = s["traits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| traits[i.as_u64().unwrap() as usize].as_slice())
                .collect();
            let c: Vec<f32> = s["case"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect();
            let choix = m
                .classer(
                    &choisis,
                    Case {
                        haut: c[1],
                        hauteur: c[3],
                    },
                    3,
                )
                .unwrap();
            total += 1;
            if m.classes[choix[0].0].id == s["classe"].as_str().unwrap() {
                justes += 1;
            }
        }
        // Une partie de ces signes a servi à l'entraînement : la justesse doit être haute.
        assert!(justes * 100 >= total * 85, "{justes}/{total}");
    }
}
