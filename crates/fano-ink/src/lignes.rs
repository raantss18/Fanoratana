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

/// La bande des minuscules d'une ligne d'écriture : (haut, ligne de base), en coordonnées de page.
///
/// Les points sont pris à pas régulier le long des traits (un trait lent n'a pas plus de poids), puis
/// on garde la bande où passe l'essentiel de l'encre : du 20ᵉ au 80ᵉ centile en hauteur. Les hampes
/// (l, J, d) et les jambages (p, j) en débordent ; la bande donne la hauteur d'x et la ligne de base.
pub fn bande_minuscules(traits: &[fano_core::Trait]) -> Option<(f32, f32)> {
    let mut ys: Vec<f32> = Vec::new();
    for t in traits {
        let pas = (t.bbox().height().max(t.bbox().width()) / 60.0).max(0.5);
        let mut reste = 0.0;
        for w in t.points.windows(2) {
            let (a, b) = (w[0].pos(), w[1].pos());
            let d = a.dist(b);
            let mut s = reste;
            while s < d {
                ys.push(a.lerp(b, s / d.max(1e-6)).y);
                s += pas;
            }
            reste = s - d;
        }
        if t.points.len() == 1 {
            ys.push(t.points[0].y);
        }
    }
    if ys.len() < 4 {
        return None;
    }
    ys.sort_by(f32::total_cmp);
    let q = |p: f32| ys[((ys.len() - 1) as f32 * p) as usize];
    Some((q(0.2), q(0.8)))
}

#[cfg(test)]
mod tests_bande {
    use super::*;
    use fano_core::{Couleur, PointEncre, Trait};

    fn trait_(pts: &[(f32, f32)]) -> Trait {
        Trait {
            points: pts
                .iter()
                .map(|&(x, y)| PointEncre { x, y, p: 0.5, t: 0 })
                .collect(),
            couleur: Couleur::NOIR,
            epaisseur: 2.0,
            surligneur: false,
        }
    }

    #[test]
    fn une_hampe_ne_fait_pas_la_taille() {
        // « ele » : des minuscules entre y=80 et y=100, et un l qui monte à y=40.
        let mut v = Vec::new();
        for k in 0..6 {
            let x = k as f32 * 20.0;
            v.push(trait_(&[
                (x, 100.0),
                (x + 5.0, 80.0),
                (x + 10.0, 100.0),
                (x + 15.0, 80.0),
                (x + 20.0, 100.0),
            ]));
        }
        v.push(trait_(&[(130.0, 100.0), (132.0, 40.0)]));
        let (haut, base) = bande_minuscules(&v).unwrap();
        assert!(base > 95.0 && base <= 100.0, "ligne de base {base}");
        assert!(haut > 75.0 && haut < 90.0, "haut des minuscules {haut}");
    }
}
