//! Dessiner une ligne d'encre en image pour le modèle : 48 px de haut, marges blanches, encre noire.

use fano_core::{Rect, Trait};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::HAUTEUR;

/// Rend `(gris, largeur)` : une image en niveaux de gris, `HAUTEUR` lignes de `largeur` px.
pub fn rasteriser(traits: &[Trait]) -> Option<(Vec<u8>, usize)> {
    let bbox = traits.iter().fold(Rect::NOTHING, |r, t| r.union(t.bbox()));
    if !bbox.is_valid() {
        return None;
    }
    // L'écriture occupe ~75 % de la hauteur, comme les lignes d'entraînement de PP-OCR.
    let h_encre = bbox.height().max(8.0);
    let echelle = (HAUTEUR as f32 * 0.75) / h_encre;
    let marge = HAUTEUR as f32 * 0.125;
    let largeur = ((bbox.width() * echelle + 2.0 * marge).ceil() as u32).clamp(16, 2560);
    let mut pix = Pixmap::new(largeur, HAUTEUR)?;
    pix.fill(tiny_skia::Color::WHITE);
    let mut peinture = Paint::default();
    peinture.set_color_rgba8(0, 0, 0, 255);
    peinture.anti_alias = true;
    let tr = Transform::from_translate(marge - bbox.min.x * echelle, marge - bbox.min.y * echelle)
        .pre_scale(echelle, echelle);
    for t in traits.iter().filter(|t| !t.surligneur) {
        // Épaisseur minimale lisible à cette échelle (~2 px).
        let ep = t.epaisseur.max(2.2 / echelle);
        let reel = t.points.iter().any(|p| (p.p - t.points[0].p).abs() > 1e-3);
        let poly = fano_ink::contour(&t.points, ep, reel);
        let mut pb = PathBuilder::new();
        for (i, p) in poly.iter().enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }
        pb.close();
        if let Some(path) = pb.finish() {
            pix.fill_path(&path, &peinture, FillRule::Winding, tr, None);
        }
    }
    let gris = pix.pixels().iter().map(|p| p.demultiply().red()).collect();
    Some((gris, largeur as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fano_core::{Couleur, PointEncre};

    #[test]
    fn un_trait_noircit_l_image() {
        let t = Trait {
            points: (0..20)
                .map(|i| PointEncre {
                    x: i as f32 * 5.0,
                    y: 10.0 + (i % 3) as f32 * 8.0,
                    p: 0.5,
                    t: 0,
                })
                .collect(),
            couleur: Couleur::ENCRE,
            epaisseur: 2.0,
            surligneur: false,
        };
        let (g, w) = rasteriser(&[t]).unwrap();
        assert_eq!(g.len(), w * HAUTEUR as usize);
        assert!(g.iter().filter(|&&v| v < 128).count() > 50);
    }
}
