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

/// Un segment tremblé : rend deux polylignes (les deux passages).
pub fn segment(a: Point, b: Point, alea: &mut Alea, rugosite: f32) -> Vec<Vec<Point>> {
    let l = a.dist(b);
    let amp = (l * 0.012 + 0.6).min(3.0) * rugosite;
    (0..2)
        .map(|passe| {
            let ecart = if passe == 0 { amp } else { amp * 1.4 };
            let a2 = a + Point::new(alea.sym(), alea.sym()) * ecart;
            let b2 = b + Point::new(alea.sym(), alea.sym()) * ecart;
            let n = (b2 - a2).norm().perp();
            let bombe = alea.sym() * (l * 0.012).min(4.0) * rugosite;
            let pas = ((l / 12.0) as usize).clamp(4, 40);
            (0..=pas)
                .map(|i| {
                    let t = i as f32 / pas as f32;
                    a2.lerp(b2, t) + n * (bombe * 4.0 * t * (1.0 - t))
                })
                .collect()
        })
        .collect()
}

/// Une ellipse tremblée (centre, rayons), en deux passages qui se chevauchent un peu.
pub fn ellipse(c: Point, rx: f32, ry: f32, alea: &mut Alea, rugosite: f32) -> Vec<Vec<Point>> {
    let perim = std::f32::consts::TAU * ((rx * rx + ry * ry) / 2.0).sqrt();
    let n = ((perim / 6.0) as usize).clamp(24, 160);
    (0..2)
        .map(|passe| {
            let depart = alea.sym() * 0.6;
            let tour = std::f32::consts::TAU * (1.0 + 0.06 * (1.0 + alea.sym()));
            let (drx, dry) = (
                alea.sym() * rx * 0.02 * rugosite,
                alea.sym() * ry * 0.02 * rugosite,
            );
            let ph = alea.sym() * 3.0;
            let amp = (0.8 + passe as f32 * 0.5) * rugosite;
            (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32;
                    let a = depart + tour * t;
                    let wobble = (a * 3.0 + ph).sin() * amp;
                    Point::new(
                        c.x + (rx + drx + wobble) * a.cos(),
                        c.y + (ry + dry + wobble) * a.sin(),
                    )
                })
                .collect()
        })
        .collect()
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
        assert!(a.iter().flatten().all(|p| p.y.abs() < 6.0));
    }
}
