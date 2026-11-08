//! Le contour d'un trait, plus épais quand on appuie, effilé aux bouts.
//!
//! Même idée que perfect-freehand (Steve Ruiz, MIT, https://github.com/steveruizok/perfect-freehand) :
//! lisser les points, donner à chacun un rayon tiré de la pression (ou de la vitesse au doigt),
//! puis relier les décalages à gauche et à droite et fermer par deux demi-cercles. Réécrit ici.

use fano_core::{Point, PointEncre};

/// Le squelette du trait : points lissés et rayon de chacun. L'écran le dessine en segments et
/// disques (rapide, sans triangulation) ; l'export en tire le contour ([`contour`]).
pub fn squelette(points: &[PointEncre], epaisseur: f32, pression_reelle: bool) -> Vec<(Point, f32)> {
    if points.is_empty() {
        return Vec::new();
    }
    // 1. Lissage (moyenne mobile pondérée vers le point courant, « streamline »).
    let mut lisses: Vec<(Point, f32)> = Vec::with_capacity(points.len());
    let mut prec = points[0].pos();
    for (i, p) in points.iter().enumerate() {
        let cur = if i == 0 { p.pos() } else { prec.lerp(p.pos(), 0.6) };
        prec = cur;
        lisses.push((cur, p.p));
    }
    // Le dernier point exact : le trait finit où le doigt s'est levé.
    if let (Some(l), Some(p)) = (lisses.last_mut(), points.last()) {
        l.0 = p.pos();
    }
    // Pas de doublons serrés : ils cassent les normales.
    lisses.dedup_by(|b, a| a.0.dist(b.0) < 0.25);
    if lisses.len() == 1 {
        return vec![(lisses[0].0, rayon(lisses[0].1, epaisseur))];
    }
    // 2. Rayon par point.
    let n = lisses.len();
    let longueur_totale: f32 = lisses.windows(2).map(|w| w[0].0.dist(w[1].0)).sum();
    let effile = (epaisseur * 3.0).min(longueur_totale / 3.0);
    let mut cumul = 0.0;
    let mut pression_sim = 0.5;
    let mut res = Vec::with_capacity(n);
    for i in 0..n {
        if i > 0 {
            let d = lisses[i].0.dist(lisses[i - 1].0);
            cumul += d;
            // Plus on va vite, plus le trait s'affine.
            let cible = (1.0 - (d / epaisseur).min(1.0) * 0.6).clamp(0.25, 1.0);
            pression_sim += (cible - pression_sim) * 0.3;
        }
        let pr = if pression_reelle {
            lisses[i].1
        } else {
            pression_sim
        };
        let mut r = rayon(pr, epaisseur);
        // Effilement des deux bouts.
        if effile > 0.0 {
            let debut = (cumul / effile).min(1.0);
            let fin = ((longueur_totale - cumul) / effile).min(1.0);
            r *= 0.35 + 0.65 * debut.min(fin).max(0.0).sqrt();
        }
        res.push((lisses[i].0, r));
    }
    res
}

/// Rend le polygone (fermé, sens quelconque) qui dessine le trait.
/// `pression_reelle` : faux au doigt ou à la souris, la pression est alors simulée par la vitesse.
pub fn contour(points: &[PointEncre], epaisseur: f32, pression_reelle: bool) -> Vec<Point> {
    let sq = squelette(points, epaisseur, pression_reelle);
    if sq.is_empty() {
        return Vec::new();
    }
    if sq.len() == 1 {
        return cercle(sq[0].0, sq[0].1, 12);
    }
    let n = sq.len();
    let lisses: Vec<(Point, f32)> = sq.clone();
    let rayons: Vec<f32> = sq.iter().map(|s| s.1).collect();
    // 3. Décalages gauche et droite.
    let mut gauche = Vec::with_capacity(n);
    let mut droite = Vec::with_capacity(n);
    for i in 0..n {
        let a = lisses[i.saturating_sub(1)].0;
        let b = lisses[(i + 1).min(n - 1)].0;
        let normale = (b - a).norm().perp();
        gauche.push(lisses[i].0 + normale * rayons[i]);
        droite.push(lisses[i].0 - normale * rayons[i]);
    }
    // 4. Assemblage avec bouts ronds.
    let mut poly = gauche;
    let fin = lisses[n - 1].0;
    let dir_fin = (fin - lisses[n - 2].0).norm();
    poly.extend(demi_cercle(fin, dir_fin, rayons[n - 1]));
    poly.extend(droite.into_iter().rev());
    let debut = lisses[0].0;
    let dir_debut = (debut - lisses[1].0).norm();
    poly.extend(demi_cercle(debut, dir_debut, rayons[0]));
    poly
}

fn rayon(pression: f32, epaisseur: f32) -> f32 {
    (epaisseur / 2.0) * (0.45 + 0.9 * pression.clamp(0.0, 1.0))
}

fn cercle(c: Point, r: f32, n: usize) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            Point::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

/// Demi-cercle autour de `c`, du côté `dir`, de la normale gauche à la normale droite.
fn demi_cercle(c: Point, dir: Point, r: f32) -> Vec<Point> {
    let n = dir.perp();
    (1..8)
        .map(|i| {
            let a = i as f32 / 8.0 * std::f32::consts::PI;
            c + n * (r * a.cos()) + dir * (r * a.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(v: &[(f32, f32)]) -> Vec<PointEncre> {
        v.iter()
            .enumerate()
            .map(|(i, &(x, y))| PointEncre {
                x,
                y,
                p: 0.5,
                t: i as u32 * 8,
            })
            .collect()
    }

    #[test]
    fn un_point_fait_un_rond() {
        let c = contour(&pts(&[(10.0, 10.0)]), 4.0, true);
        assert_eq!(c.len(), 12);
        assert!(
            c.iter().all(|p| {
                (p.dist(Point::new(10.0, 10.0)) - c[0].dist(Point::new(10.0, 10.0))).abs() < 1e-3
            })
        );
    }

    #[test]
    fn un_trait_reste_dans_son_epaisseur() {
        let ligne: Vec<(f32, f32)> = (0..50).map(|i| (i as f32 * 2.0, 0.0)).collect();
        let c = contour(&pts(&ligne), 4.0, false);
        assert!(c.len() > 50);
        assert!(c.iter().all(|p| p.y.abs() <= 4.0 && p.x >= -4.0 && p.x <= 102.0));
    }
}
