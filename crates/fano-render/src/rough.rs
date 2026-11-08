//! Le trait « fait main » des formes, à la manière d'Excalidraw et de rough.js (Preet Shihn, MIT) :
//! deux passages légèrement décalés, une courbure discrète, des bouts qui dépassent un peu.
//! Réécrit ici ; la graine fixe redonne le même dessin à chaque affichage.

use fano_core::Point;

/// Générateur pseudo-aléatoire déterministe (xorshift32).
pub struct Alea(u32);

impl Alea {
    pub fn new(graine: u32) -> Alea {
        Alea(graine.wrapping_mul(2_654_435_761).max(1))
    }
    /// Un nombre dans [-1, 1].
    pub fn sym(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Un segment net, en un seul trait : les bouts exacts, une courbure à peine visible (le trait reste
/// vivant sans paraître doublé — « une ligne en fait deux », Antsa, 2026-11-08).
pub fn segment(a: Point, b: Point, alea: &mut Alea, rugosite: f32) -> Vec<Vec<Point>> {
    let l = a.dist(b);
    let n = (b - a).norm().perp();
    let bombe = alea.sym() * (l * 0.004).min(1.5) * rugosite;
    let pas = ((l / 12.0) as usize).clamp(2, 40);
    vec![
        (0..=pas)
            .map(|i| {
                let t = i as f32 / pas as f32;
                a.lerp(b, t) + n * (bombe * 4.0 * t * (1.0 - t))
            })
            .collect(),
    ]
}

/// Une ellipse (ou un cercle) nette, en un seul trait fermé.
pub fn ellipse(c: Point, rx: f32, ry: f32, _alea: &mut Alea, _rugosite: f32) -> Vec<Vec<Point>> {
    let perim = std::f32::consts::TAU * ((rx * rx + ry * ry) / 2.0).sqrt();
    let n = ((perim / 4.0) as usize).clamp(32, 240);
    vec![
        (0..=n)
            .map(|i| {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                Point::new(c.x + rx * a.cos(), c.y + ry * a.sin())
            })
            .collect(),
    ]
}

/// Les deux traits d'une pointe de flèche au bout `b` d'un segment venant de `a`.
pub fn pointe(a: Point, b: Point, taille: f32) -> [Vec<Point>; 2] {
    let dir = (b - a).norm();
    let n = dir.perp();
    let t = taille.min(a.dist(b) * 0.35);
    let recul = b - dir * t;
    [vec![recul + n * (t * 0.55), b], vec![recul - n * (t * 0.55), b]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meme_graine_meme_dessin() {
        let a = segment(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            &mut Alea::new(7),
            1.0,
        );
        let b = segment(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            &mut Alea::new(7),
            1.0,
        );
        assert_eq!(a, b);
        assert_eq!(
            a.len(),
            1,
            "un seul trait : une ligne ne doit jamais paraître doublée"
        );
        assert!(a.iter().flatten().all(|p| p.y.abs() < 2.0));
        let e = ellipse(Point::new(0.0, 0.0), 50.0, 50.0, &mut Alea::new(1), 1.0);
        assert_eq!(e.len(), 1);
        assert!(
            e[0].iter().all(|p| (p.len() - 50.0).abs() < 0.01),
            "un cercle est un cercle"
        );
    }
}
