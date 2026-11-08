//! Le `$` écrit à la main : la règle donnée à l'utilisateur pour dire « ceci est une formule »
//! (comme en LaTeX). Un `$` = un S tracé de haut en bas, coupé par un trait vertical plus haut que lui
//! (deux traits, dans n'importe quel ordre). Ce qui est entre deux `$` d'une ligne est lu en formule.

use fano_core::{Point, Rect};

/// Un S : de haut en bas, avec deux demi-tours horizontaux (droite → gauche → droite → gauche), plus haut
/// que large, commencé en haut à droite et fini en bas à gauche.
pub fn est_un_s(pts: &[Point]) -> bool {
    if pts.len() < 6 {
        return false;
    }
    let r = Rect::from_points(pts.iter().copied());
    let (w, h) = (r.width(), r.height());
    if h < 1.1 * w || w <= 0.0 {
        return false;
    }
    let (debut, fin) = (pts[0], pts[pts.len() - 1]);
    if fin.y - debut.y < 0.5 * h || debut.x < r.center().x - 0.1 * w || fin.x > r.center().x + 0.1 * w {
        return false;
    }
    // Les demi-tours en x (au moins deux : le haut du S part à gauche, le milieu repart à droite, le bas
    // revient à gauche).
    let seuil = 0.25 * w;
    let (mut retours, mut sens, mut extreme) = (0, 0i8, debut.x);
    for p in &pts[1..] {
        match sens {
            0 if (p.x - extreme).abs() > seuil => {
                sens = if p.x > extreme { 1 } else { -1 };
                extreme = p.x;
            }
            1 if p.x > extreme => extreme = p.x,
            1 if extreme - p.x > seuil => {
                retours += 1;
                sens = -1;
                extreme = p.x;
            }
            -1 if p.x < extreme => extreme = p.x,
            -1 if p.x - extreme > seuil => {
                retours += 1;
                sens = 1;
                extreme = p.x;
            }
            _ => {}
        }
    }
    retours >= 2
}

/// Les `$` d'une ligne : paires (S, trait) d'indices dans `traits`, de gauche à droite.
pub fn dollars(traits: &[Vec<Point>]) -> Vec<(usize, usize)> {
    let boites: Vec<Rect> = traits
        .iter()
        .map(|t| Rect::from_points(t.iter().copied()))
        .collect();
    let mut res: Vec<(usize, usize)> = Vec::new();
    let mut pris = vec![false; traits.len()];
    for s in 0..traits.len() {
        if pris[s] || !est_un_s(&traits[s]) {
            continue;
        }
        let bs = boites[s];
        let barre = (0..traits.len()).find(|&b| {
            let bb = boites[b];
            b != s
                && !pris[b]
                && bb.width() <= 0.35 * bb.height().max(1.0)
                && bb.height() >= 1.05 * bs.height()
                && (bb.center().x - bs.center().x).abs() < 0.35 * bs.width().max(bb.width())
                && bb.min.y <= bs.min.y + 0.15 * bs.height()
                && bb.max.y >= bs.max.y - 0.15 * bs.height()
        });
        if let Some(b) = barre {
            pris[s] = true;
            pris[b] = true;
            res.push((s, b));
        }
    }
    res.sort_by(|a, b| boites[a.0].min.x.total_cmp(&boites[b.0].min.x));
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un S tracé de haut en bas, centré en (cx, cy), de demi-hauteur `h`.
    pub(crate) fn s(cx: f32, cy: f32, h: f32) -> Vec<Point> {
        (0..=40)
            .map(|i| {
                let t = i as f32 / 40.0;
                // x : droite → gauche → droite → gauche (sinus) ; y : de haut en bas.
                let x = cx + 0.5 * h * (std::f32::consts::PI * (3.0 * t + 0.5)).sin() * 0.6;
                Point::new(x, cy - h + 2.0 * h * t)
            })
            .collect()
    }

    fn barre(cx: f32, cy: f32, h: f32) -> Vec<Point> {
        vec![Point::new(cx, cy - h * 1.3), Point::new(cx + 1.0, cy + h * 1.3)]
    }

    #[test]
    fn deux_dollars_autour_d_une_formule() {
        let mut t = vec![s(10.0, 50.0, 12.0), barre(10.0, 50.0, 12.0)];
        // Entre les deux : un « x » (deux diagonales) et un « 2 ».
        t.push(vec![Point::new(30.0, 45.0), Point::new(40.0, 58.0)]);
        t.push(vec![Point::new(40.0, 45.0), Point::new(30.0, 58.0)]);
        t.push(barre(80.0, 50.0, 12.0));
        t.push(s(80.0, 50.0, 12.0));
        let d = dollars(&t);
        assert_eq!(d, vec![(0, 1), (5, 4)]);
    }

    #[test]
    fn un_s_seul_ou_un_5_n_est_pas_un_dollar() {
        // Un S sans barre (le mot « Soit »), un 1 seul : rien.
        assert!(dollars(&[s(10.0, 50.0, 12.0)]).is_empty());
        assert!(dollars(&[barre(10.0, 50.0, 12.0)]).is_empty());
        // Une ligne droite n'est pas un S.
        assert!(!est_un_s(&[
            Point::new(0.0, 0.0),
            Point::new(1.0, 10.0),
            Point::new(2.0, 20.0),
            Point::new(3.0, 30.0),
            Point::new(4.0, 40.0),
            Point::new(5.0, 50.0)
        ]));
    }
}
