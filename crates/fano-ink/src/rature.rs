//! La rature : on gribouille sur ce qu'on veut effacer (geste repris de Fikajiana, ADR 0021 là-bas).
//!
//! Un trait est une rature s'il fait de nombreux allers-retours sur un axe (chemin bien plus long
//! que sa place) **et** s'il couvre des éléments existants. Sans rien dessous, c'est de l'écriture.

use fano_core::Point;

/// Nombre de demi-tours le long de l'axe principal du trait, et rapport chemin / étendue.
pub fn allers_retours(pts: &[Point]) -> (usize, f32) {
    if pts.len() < 4 {
        return (0, 0.0);
    }
    let r = fano_core::Rect::from_points(pts.iter().copied());
    allers_retours_sur(pts, r.width() >= r.height())
}

/// Demi-tours en x (`horizontal`) ou en y, et rapport chemin / étendue sur cet axe.
fn allers_retours_sur(pts: &[Point], horizontal: bool) -> (usize, f32) {
    if pts.len() < 4 {
        return (0, 0.0);
    }
    let r = fano_core::Rect::from_points(pts.iter().copied());
    let coord = |p: &Point| if horizontal { p.x } else { p.y };
    let etendue = if horizontal { r.width() } else { r.height() }.max(1.0);
    let seuil = etendue * 0.15;
    let mut retours = 0;
    let mut sens = 0i8;
    let mut extreme = coord(&pts[0]);
    for p in &pts[1..] {
        let v = coord(p);
        match sens {
            0 => {
                if (v - extreme).abs() > seuil {
                    sens = if v > extreme { 1 } else { -1 };
                    extreme = v;
                }
            }
            1 => {
                if v > extreme {
                    extreme = v;
                } else if extreme - v > seuil {
                    retours += 1;
                    sens = -1;
                    extreme = v;
                }
            }
            _ => {
                if v < extreme {
                    extreme = v;
                } else if v - extreme > seuil {
                    retours += 1;
                    sens = 1;
                    extreme = v;
                }
            }
        }
    }
    let chemin: f32 = pts.windows(2).map(|w| w[0].dist(w[1])).sum();
    (retours, chemin / etendue)
}

/// Seuils relevés au vrai stylet sur Fikajiana (Pad 7) : 6 retours et 2,2 fois plus de chemin, ou
/// 4 et 3,2. `couvre` : part (0..1) des éléments dessous couverte par le trait.
///
/// On raye aussi un mot de va-et-vient verticaux qui avancent vers la droite (« mmmm ») : le trait est plus
/// large que haut, sans demi-tour en x ; on compte alors les demi-tours en y (une rature lue « m » sur le
/// Pad 7, 2026-11-09). Un « m » seul en a 4 : il en faut 6, serrés, et de l'encre dessous.
pub fn est_rature(pts: &[Point], couvre: f32) -> bool {
    let (r, rapport) = allers_retours(pts);
    let forme = (r >= 8 && rapport >= 1.7) || (r >= 6 && rapport >= 2.2) || (r >= 4 && rapport >= 3.2);
    let b = fano_core::Rect::from_points(pts.iter().copied());
    let travers = if pts.len() >= 4 {
        let (r2, rapport2) = allers_retours_sur(pts, b.width() < b.height());
        // Serrés : chaque aller-retour avance de moins que la hauteur du gribouillis.
        let serre = b.width().min(b.height()) * r2 as f32 >= 0.8 * b.width().max(b.height());
        r2 >= 6 && rapport2 >= 4.5 && serre
    } else {
        false
    };
    (forme || travers) && couvre >= 0.35
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_sur_quelque_chose() {
        let pts: Vec<Point> = (0..40)
            .map(|i| Point::new(if i % 2 == 0 { 0.0 } else { 60.0 }, i as f32 * 0.8))
            .collect();
        assert!(est_rature(&pts, 0.8));
        assert!(
            !est_rature(&pts, 0.0),
            "sans rien dessous, ce n'est pas une rature"
        );
    }

    /// Un mot rayé de va-et-vient verticaux en avançant vers la droite.
    fn va_et_vient(n: usize, largeur: f32, hauteur: f32) -> Vec<Point> {
        (0..=n)
            .flat_map(|i| {
                let x = largeur * i as f32 / n as f32;
                let y = if i % 2 == 0 { 0.0 } else { hauteur };
                // Quelques points par segment, comme un vrai tracé.
                (0..4)
                    .map(move |k| Point::new(x + k as f32, y + if y > 0.0 { -(k as f32) } else { k as f32 }))
            })
            .collect()
    }

    #[test]
    fn mot_raye_de_haut_en_bas() {
        // 10 va-et-vient de 24 de haut sur un mot de 90 de large.
        assert!(est_rature(&va_et_vient(10, 90.0, 24.0), 1.0));
        assert!(
            !est_rature(&va_et_vient(10, 90.0, 24.0), 0.0),
            "sans rien dessous, c'est de l'écriture"
        );
    }

    #[test]
    fn des_lettres_ne_sont_pas_une_rature() {
        // « m », « nm », « mn » : trop peu de demi-tours.
        for n in [4, 5] {
            assert!(
                !est_rature(&va_et_vient(n, 12.0 * n as f32, 20.0), 1.0),
                "{n} jambages"
            );
        }
        // Une écriture large (« uuuu » bien espacés) n'est pas serrée.
        assert!(!est_rature(&va_et_vient(8, 300.0, 20.0), 1.0));
    }

    #[test]
    fn un_m_n_est_pas_une_rature() {
        let m = [(0.0, 20.0), (5.0, 0.0), (10.0, 20.0), (15.0, 0.0), (20.0, 20.0)];
        let pts: Vec<Point> = m.iter().map(|&(x, y)| Point::new(x, y)).collect();
        assert!(!est_rature(&pts, 1.0));
    }
}
