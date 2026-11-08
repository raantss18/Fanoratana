//! Lire une formule écrite à la main et la rendre en LaTeX : Pix2Text-MFR 1.0 (Breezedeus, MIT),
//! encodeur ViT + décodeur TrOCR, exécutés par tract. Même chaîne que Fikajiana
//! (`app/lib/reconnaissance/pix2text.dart`, relevée dans `docs/recherche/outils-cours.md`) :
//!
//! - image : encre noire sur blanc, recadrée, marge 12 % de la hauteur, trait = hauteur/70 ; étirée en
//!   384 × 384 RGB, `(x/255 − 0,5)/0,5` ;
//! - décodage glouton, sans cache : jeton de départ 2 (`</s>`), fin 2, 256 jetons au plus ; ids ≤ 4
//!   ignorés ; jetons BPE ByteLevel reconvertis en UTF-8.
//!
//! Le modèle (117 Mo) n'est pas dans l'app : il se télécharge une fois, à la demande (ADR 0010).

use std::collections::HashMap;
use std::path::Path;

use fano_core::{Rect, Trait};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};
use tract_onnx::prelude::*;

pub const COTE: usize = 384;
const DEBUT: i64 = 2;
const FIN: i64 = 2;
const MAX: usize = 256;

/// Les fichiers du modèle, révision fixée et empreintes (mêmes que Fikajiana).
pub const REVISION: &str = "bea257edb2653f2ae413b084f2ac0e8299d08df0";
pub const FICHIERS: [(&str, u64, &str); 3] = [
    ("encoder_model.onnx", 87_496_990, "bd8d5c322792e9ec45793af5569e9748f82a3d728a9e00213dbfc56c1486f37d"),
    ("decoder_model.onnx", 30_114_937, "fd0f92d7a012f3dae41e1ac79421aea0ea888b5a66cb3f9a004e424f82f3daed"),
    ("tokenizer.json", 39_161, "3e2ab757277d22639bec28c9d7972e352d3d1dba223051fa674002dc5ab64df3"),
];

pub fn url(fichier: &str) -> String {
    format!("https://huggingface.co/breezedeus/pix2text-mfr/resolve/{REVISION}/{fichier}")
}

/// Le modèle est-il complet dans `dossier` ?
pub fn present(dossier: &Path) -> bool {
    FICHIERS.iter().all(|(n, t, _)| std::fs::metadata(dossier.join(n)).is_ok_and(|m| m.len() == *t))
}

type Plan = std::sync::Arc<TypedSimplePlan>;

pub struct ModeleFormules {
    encodeur: Plan,
    decodeur: Plan,
    /// id → jeton BPE.
    vocab: Vec<String>,
    /// caractère ByteLevel → octet.
    octets: HashMap<char, u8>,
}

/// La table GPT-2 « bytes_to_unicode », inversée.
fn table_octets() -> HashMap<char, u8> {
    let mut bs: Vec<u32> = (b'!' as u32..=b'~' as u32).chain(0xA1..=0xAC).chain(0xAE..=0xFF).collect();
    let mut cs = bs.clone();
    let mut n = 0;
    for b in 0..256u32 {
        if !bs.contains(&b) {
            bs.push(b);
            cs.push(256 + n);
            n += 1;
        }
    }
    bs.into_iter().zip(cs).map(|(b, c)| (char::from_u32(c).unwrap(), b as u8)).collect()
}

impl ModeleFormules {
    pub fn charger(dossier: &Path) -> TractResult<ModeleFormules> {
        let encodeur = tract_onnx::onnx()
            .model_for_path(dossier.join("encoder_model.onnx"))?
            .with_input_fact(0, f32::fact([1, 3, COTE, COTE]).into())?
            .into_optimized()?
            .into_runnable()?;
        let mut dec = tract_onnx::onnx().model_for_path(dossier.join("decoder_model.onnx"))?;
        let s = dec.symbols.sym("S");
        // input_ids [1, S] ; encoder_hidden_states [1, 578, 384] (DeiT : 576 morceaux + 2 jetons) (sortie de l'encodeur).
        let noms: Vec<String> = dec.input_outlets()?.iter().map(|o| dec.node(o.node).name.clone()).collect();
        for (i, nom) in noms.iter().enumerate() {
            if nom.contains("input_ids") {
                dec = dec.with_input_fact(i, i64::fact([TDim::from(1), s.clone().into()]).into())?;
            } else {
                dec = dec.with_input_fact(i, f32::fact([1, 578, 384]).into())?;
            }
        }
        let decodeur = dec.into_optimized()?.into_runnable()?;
        let json: serde_json::Value = serde_json::from_slice(&std::fs::read(dossier.join("tokenizer.json"))?)?;
        let table = json["model"]["vocab"].as_object().ok_or_else(|| anyhow::anyhow!("tokenizer.json sans vocabulaire"))?;
        let mut vocab = vec![String::new(); table.len()];
        for (jeton, id) in table {
            if let Some(i) = id.as_u64().map(|i| i as usize).filter(|&i| i < vocab.len()) {
                vocab[i] = jeton.clone();
            }
        }
        // L'ordre des entrées du décodeur, pour les donner dans le bon ordre.
        let _ = noms;
        Ok(ModeleFormules { encodeur, decodeur, vocab, octets: table_octets() })
    }

    /// Lit une image 384 × 384 en niveaux de gris (0 = noir) ; rend le LaTeX.
    pub fn lire_image(&self, gris: &[u8]) -> TractResult<String> {
        let entree: Tensor = tract_ndarray::Array4::from_shape_fn((1, 3, COTE, COTE), |(_, _, y, x)| {
            (f32::from(gris[y * COTE + x]) / 255.0 - 0.5) / 0.5
        })
        .into();
        let etats = self.encodeur.run(tvec!(entree.into()))?.remove(0);
        let mut ids: Vec<i64> = vec![DEBUT];
        for _ in 0..MAX {
            let entree_ids: Tensor = tract_ndarray::Array2::from_shape_vec((1, ids.len()), ids.clone())?.into();
            let sortie = self.decodeur.run(tvec!(entree_ids.into(), etats.clone()))?;
            let logits = sortie[0].to_plain_array_view::<f32>()?;
            let n = logits.shape()[1];
            let v = logits.shape()[2];
            let (mut meilleur, mut val) = (0usize, f32::MIN);
            for k in 0..v {
                let x = logits[[0, n - 1, k]];
                if x > val {
                    val = x;
                    meilleur = k;
                }
            }
            let id = meilleur as i64;
            if id == FIN {
                break;
            }
            ids.push(id);
        }
        Ok(compacter(&self.decoder(&ids[1..])))
    }

    fn decoder(&self, ids: &[i64]) -> String {
        let mut octets = Vec::new();
        for &id in ids {
            if id <= 4 {
                continue;
            }
            if let Some(j) = self.vocab.get(id as usize) {
                for c in j.chars() {
                    if let Some(&b) = self.octets.get(&c) {
                        octets.push(b);
                    }
                }
            }
        }
        String::from_utf8_lossy(&octets).trim().to_owned()
    }

    pub fn lire(&self, traits: &[Trait]) -> TractResult<String> {
        let gris = rasteriser_carre(traits).ok_or_else(|| anyhow::anyhow!("rien à lire"))?;
        self.lire_image(&gris)
    }
}

/// Pix2Text sépare chaque jeton par une espace (`F ( x ) = 2 x`) : on les retire, sauf après une
/// commande suivie d'une lettre (`\alpha x`).
pub fn compacter(latex: &str) -> String {
    let c: Vec<char> = latex.chars().collect();
    let mut res = String::new();
    for (i, &ch) in c.iter().enumerate() {
        if ch == ' ' {
            let suivant_lettre = c.get(i + 1).is_some_and(|n| n.is_ascii_alphabetic());
            // Le mot précédent est-il une commande `\nom` ?
            let mut j = res.len();
            let octets = res.as_bytes();
            while j > 0 && octets[j - 1].is_ascii_alphabetic() {
                j -= 1;
            }
            let commande = j > 0 && octets[j - 1] == b'\\' && j < res.len();
            if suivant_lettre && commande {
                res.push(' ');
            }
            continue;
        }
        res.push(ch);
    }
    res
}

/// L'encre, recadrée avec 12 % de marge, étirée en 384 × 384, trait = hauteur/70.
pub fn rasteriser_carre(traits: &[Trait]) -> Option<Vec<u8>> {
    let b = traits.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
    if !b.is_valid() {
        return None;
    }
    let h = b.height().max(8.0);
    let marge = h * 0.12;
    let (w_src, h_src) = (b.width() + 2.0 * marge, h + 2.0 * marge);
    // Le trait garde la même épaisseur dans les deux sens : on dessine à l'échelle, puis on étire.
    let echelle = 1536.0 / w_src.max(h_src);
    let (wp, hp) = ((w_src * echelle).ceil().max(1.0) as u32, (h_src * echelle).ceil().max(1.0) as u32);
    let mut grand = Pixmap::new(wp, hp)?;
    grand.fill(tiny_skia::Color::WHITE);
    let mut peinture = Paint::default();
    peinture.set_color_rgba8(0, 0, 0, 255);
    peinture.anti_alias = true;
    let tr = Transform::from_translate((marge - b.min.x) * echelle, (marge - b.min.y) * echelle).pre_scale(echelle, echelle);
    let trait_px = (h * echelle / 70.0).max(2.0);
    let stroke = tiny_skia::Stroke { width: trait_px / echelle, line_cap: tiny_skia::LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..Default::default() };
    for t in traits.iter().filter(|t| !t.surligneur) {
        let mut pb = PathBuilder::new();
        for (i, p) in t.points.iter().enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }
        if t.points.len() == 1 {
            pb.line_to(t.points[0].x + 0.01, t.points[0].y);
        }
        if let Some(path) = pb.finish() {
            grand.stroke_path(&path, &peinture, &stroke, tr, None);
        }
    }
    let _ = FillRule::Winding;
    // Étirement en 384 × 384 (sans garder les proportions, comme le prétraitement d'origine).
    let mut carre = Pixmap::new(COTE as u32, COTE as u32)?;
    carre.fill(tiny_skia::Color::WHITE);
    let sx = COTE as f32 / wp as f32;
    let sy = COTE as f32 / hp as f32;
    carre.draw_pixmap(
        0,
        0,
        grand.as_ref(),
        &tiny_skia::PixmapPaint { quality: tiny_skia::FilterQuality::Bicubic, ..Default::default() },
        Transform::from_scale(sx, sy),
        None,
    );
    Some(carre.pixels().iter().map(|p| p.demultiply().red()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compactage() {
        assert_eq!(compacter("F ( x ) = 2 x + 3"), "F(x)=2x+3");
        assert_eq!(compacter("\\alpha x + \\frac { 1 } { 2 }"), "\\alpha x+\\frac{1}{2}");
    }

    #[test]
    fn table_bytelevel() {
        let t = table_octets();
        assert_eq!(t.len(), 256);
        assert_eq!(t[&'Ġ'], b' ');
        assert_eq!(t[&'a'], b'a');
    }
}
