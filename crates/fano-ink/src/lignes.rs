//! Regrouper les traits d'encre en lignes d'écriture, puis en mots.
//!
//! Une ligne : des traits dont les bandes verticales se recouvrent assez, triés de gauche à droite.
//! Un mot : des traits séparés par un blanc plus petit que ~0,45 fois la hauteur de ligne.

use fano_core::Rect;

#[derive(Clone, Debug, PartialEq)]
pub struct Ligne {
    /// Index des traits (dans la liste donnée), de gauche à droite.
    pub traits: Vec<usize>,
    pub bbox: Rect,
    /// Les mots : groupes d'index de traits.
    pub mots: Vec<Vec<usize>>,
}

/// Regroupe des traits (donnés par leur rectangle) en lignes, de haut en bas.
pub fn grouper_lignes(boites: &[Rect]) -> Vec<Ligne> {
    let mut ordre: Vec<usize> = (0..boites.len()).filter(|&i| boites[i].is_valid()).collect();
    ordre.sort_by(|&a, &b| boites[a].center().y.total_cmp(&boites[b].center().y));
    // Hauteur typique d'un signe : médiane des hauteurs.
    let mut hauteurs: Vec<f32> = ordre.iter().map(|&i| boites[i].height().max(2.0)).collect();
    hauteurs.sort_by(f32::total_cmp);
    let h_typ = hauteurs.get(hauteurs.len() / 2).copied().unwrap_or(20.0);

    // Les grands signes d'abord : ils font les lignes ; les petits (points, accents) les rejoignent ensuite.
    let (grands, petits): (Vec<usize>, Vec<usize>) = ordre
        .into_iter()
        .partition(|&i| boites[i].height() >= h_typ * 0.4);
    let mut lignes: Vec<Ligne> = Vec::new();
    for i in grands.into_iter().chain(petits) {
        let b = boites[i];
        // Un point, un accent, une barre de fraction appartiennent à la ligne dont le centre est le plus proche.
        let meilleure = lignes
            .iter()
            .enumerate()
            .filter_map(|(k, l)| {
                let haut = l.bbox.min.y.max(b.min.y);
                let bas = l.bbox.max.y.min(b.max.y);
                let recouvre = (bas - haut).max(0.0) / b.height().max(1.0).min(l.bbox.height().max(1.0));
                let ecart_centre = (l.bbox.center().y - b.center().y).abs();
                (recouvre > 0.3
                    || ecart_centre < h_typ * 0.6
                    || (b.height() < h_typ * 0.4 && ecart_centre < h_typ * 1.2))
                    .then_some((k, ecart_centre))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        match meilleure {
            Some((k, _)) => {
                lignes[k].traits.push(i);
                lignes[k].bbox = lignes[k].bbox.union(b);
            }
            None => lignes.push(Ligne {
                traits: vec![i],
                bbox: b,
                mots: vec![],
            }),
        }
    }
    for l in &mut lignes {
        l.traits
            .sort_by(|&a, &b| boites[a].min.x.total_cmp(&boites[b].min.x));
        let seuil = l.bbox.height().min(h_typ * 1.6) * 0.45;
        let mut mots: Vec<Vec<usize>> = Vec::new();
        let mut droite = f32::NEG_INFINITY;
        for &t in &l.traits {
            if boites[t].min.x - droite > seuil || mots.is_empty() {
                mots.push(vec![t]);
            } else {
                mots.last_mut().unwrap().push(t);
            }
            droite = droite.max(boites[t].max.x);
        }
        l.mots = mots;
    }
    lignes.sort_by(|a, b| a.bbox.min.y.total_cmp(&b.bbox.min.y));
    lignes
}

#[cfg(test)]
mod tests {
    use super::*;
    use fano_core::Point;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Point::new(x, y), w, h)
    }

    #[test]
    fn deux_lignes_trois_mots() {
        let b = [
            r(0.0, 0.0, 10.0, 20.0),
            r(12.0, 2.0, 10.0, 18.0),
            r(50.0, 0.0, 10.0, 20.0), // deuxième mot
            r(0.0, 60.0, 30.0, 20.0), // deuxième ligne
            r(13.0, -6.0, 3.0, 3.0),  // un accent au-dessus du 2e signe
        ];
        let l = grouper_lignes(&b);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].mots, vec![vec![0, 1, 4], vec![2]]);
        assert_eq!(l[1].traits, vec![3]);
    }
}
