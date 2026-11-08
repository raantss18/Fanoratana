//! Graphes et arbres tracés à main levée : des ronds (les nœuds) reliés par des traits ou des flèches
//! (les arêtes). Une fois les formes reconnues, on les met au propre comme on le ferait à la règle :
//!
//! 1. les ronds de taille voisine (± 30 %) prennent la même taille ;
//! 2. les ronds presque à la même hauteur sont alignés (les niveaux d'un arbre) ;
//! 3. chaque bout d'arête proche d'un rond est accroché à son bord, dans l'axe des deux centres.
//!
//! Rien ne bouge au-delà de ce qui est presque juste : un dessin volontairement irrégulier le reste.

use fano_core::{Forme, FormeKind, Point};

fn est_rond(f: &Forme) -> bool {
    matches!(f.kind, FormeKind::Ellipse | FormeKind::Noeud) && f.points.len() >= 2 && {
        let r = f.points[1];
        (r.x - r.y).abs() < 0.25 * r.x.max(r.y)
    }
}

fn est_arete(f: &Forme) -> bool {
    matches!(
        f.kind,
        FormeKind::Segment | FormeKind::Fleche | FormeKind::DoubleFleche
    ) && f.points.len() == 2
}

/// Met au propre les graphes et arbres d'une page (formes modifiées sur place). Rend vrai si
/// quelque chose a bougé.
pub fn ajuster(formes: &mut [&mut Forme]) -> bool {
    let ronds: Vec<usize> = (0..formes.len()).filter(|&i| est_rond(formes[i])).collect();
    if ronds.is_empty() {
        return false;
    }
    let mut change = false;
    // 1. Même taille pour les ronds voisins en taille : groupes (rayons triés, ± 30 % du plus petit),
    //    puis la moyenne du groupe pour chacun.
    let mut tri: Vec<usize> = ronds.clone();
    tri.sort_by(|&i, &j| formes[i].points[1].x.total_cmp(&formes[j].points[1].x));
    let mut groupe: Vec<usize> = Vec::new();
    let mut groupes: Vec<Vec<usize>> = Vec::new();
    for i in tri {
        let r = formes[i].points[1].x;
        if groupe.first().is_some_and(|&g| r > formes[g].points[1].x * 1.3) {
            groupes.push(std::mem::take(&mut groupe));
        }
        groupe.push(i);
    }
    groupes.push(groupe);
    for g in groupes.into_iter().filter(|g| g.len() >= 2) {
        let m = g.iter().map(|&i| formes[i].points[1].x).sum::<f32>() / g.len() as f32;
        for i in g {
            if (formes[i].points[1].x - m).abs() > 1e-3 {
                formes[i].points[1] = Point::new(m, m);
                change = true;
            }
        }
    }
    // 2. Niveaux : les centres à moins d'un demi-rayon de hauteur l'un de l'autre sont alignés.
    let mut faits = vec![false; formes.len()];
    for &i in &ronds {
        if faits[i] {
            continue;
        }
        let (yi, ri) = (formes[i].points[0].y, formes[i].points[1].x);
        let niveau: Vec<usize> = ronds
            .iter()
            .copied()
            .filter(|&j| !faits[j] && (formes[j].points[0].y - yi).abs() < ri * 0.5)
            .collect();
        if niveau.len() >= 2 {
            let m = niveau.iter().map(|&j| formes[j].points[0].y).sum::<f32>() / niveau.len() as f32;
            for &j in &niveau {
                if (formes[j].points[0].y - m).abs() > 1e-3 {
                    formes[j].points[0].y = m;
                    change = true;
                }
                faits[j] = true;
            }
        }
    }
    // 3. Arêtes accrochées au bord des ronds, dans l'axe des centres.
    let noeuds: Vec<(Point, f32)> = ronds
        .iter()
        .map(|&i| (formes[i].points[0], formes[i].points[1].x))
        .collect();
    let proche = |p: Point| -> Option<(Point, f32)> {
        noeuds
            .iter()
            .copied()
            .filter(|(c, r)| c.dist(p) <= r * 1.6 + 8.0)
            .min_by(|a, b| (a.0.dist(p) - a.1).abs().total_cmp(&(b.0.dist(p) - b.1).abs()))
    };
    for f in formes.iter_mut().filter(|f| est_arete(f)) {
        let (a, b) = (f.points[0], f.points[1]);
        let (na, nb) = (proche(a), proche(b));
        // Le point visé de chaque bout : le centre de son nœud s'il en a un, sinon le bout lui-même.
        let cible_a = na.map_or(a, |n| n.0);
        let cible_b = nb.map_or(b, |n| n.0);
        if na.is_some() && nb.is_some() && na.map(|n| n.0) == nb.map(|n| n.0) {
            continue; // les deux bouts sur le même rond : ce n'est pas une arête
        }
        let mut nouveau = [a, b];
        if let Some((c, r)) = na {
            nouveau[0] = c + (cible_b - c).norm() * r;
        }
        if let Some((c, r)) = nb {
            nouveau[1] = c + (cible_a - c).norm() * r;
        }
        if nouveau[0].dist(a) > 1e-3 || nouveau[1].dist(b) > 1e-3 {
            f.points = nouveau.to_vec();
            change = true;
        }
    }
    change
}

/// Le rond qui contient `p` (pour y mettre une étiquette).
pub fn rond_contenant(formes: &[&Forme], p: Point) -> Option<usize> {
    formes
        .iter()
        .position(|f| est_rond(f) && f.points[0].dist(p) < f.points[1].x * 0.8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fano_core::Couleur;

    fn rond(x: f32, y: f32, r: f32) -> Forme {
        Forme {
            kind: FormeKind::Ellipse,
            points: vec![Point::new(x, y), Point::new(r, r)],
            couleur: Couleur::NOIR,
            epaisseur: 2.0,
            rempli: false,
            graine: 1,
            etiquette: String::new(),
            source: vec![],
        }
    }

    fn trait_(a: (f32, f32), b: (f32, f32)) -> Forme {
        Forme {
            kind: FormeKind::Segment,
            points: vec![Point::new(a.0, a.1), Point::new(b.0, b.1)],
            ..rond(0.0, 0.0, 1.0)
        }
    }

    #[test]
    fn un_arbre_a_main_levee() {
        // Racine en haut, deux enfants à peu près au même niveau, de tailles voisines ; des traits qui
        // touchent à peu près les ronds.
        let mut v = vec![
            rond(200.0, 50.0, 22.0),
            rond(120.0, 150.0, 20.0),
            rond(285.0, 160.0, 25.0),
            trait_((188.0, 72.0), (130.0, 128.0)),
            trait_((214.0, 70.0), (275.0, 140.0)),
        ];
        let mut refs: Vec<&mut Forme> = v.iter_mut().collect();
        assert!(ajuster(&mut refs));
        // Mêmes tailles, enfants alignés.
        let r = v[0].points[1].x;
        assert!(v[1..3].iter().all(|f| (f.points[1].x - r).abs() < 1e-3));
        assert!((v[1].points[0].y - v[2].points[0].y).abs() < 1e-3);
        // Chaque arête part exactement du bord d'un rond et arrive au bord de l'autre.
        for (arete, (n1, n2)) in [(3, (0, 1)), (4, (0, 2))] {
            let (a, b) = (v[arete].points[0], v[arete].points[1]);
            assert!((a.dist(v[n1].points[0]) - v[n1].points[1].x).abs() < 1e-2);
            assert!((b.dist(v[n2].points[0]) - v[n2].points[1].x).abs() < 1e-2);
            // Dans l'axe des centres.
            let axe = (v[n2].points[0] - v[n1].points[0]).norm();
            assert!(((b - a).norm().dot(axe) - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn des_formes_eloignees_ne_bougent_pas() {
        let mut v = vec![rond(100.0, 100.0, 20.0), trait_((300.0, 300.0), (400.0, 300.0))];
        let avant = v.clone();
        let mut refs: Vec<&mut Forme> = v.iter_mut().collect();
        assert!(!ajuster(&mut refs));
        assert_eq!(v, avant);
    }
}
