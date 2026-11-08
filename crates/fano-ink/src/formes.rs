//! Les formes tracées à main levée : segment, flèche, cercle ou ellipse, triangle, rectangle,
//! losange, polygone.
//!
//! Méthode classique de reconnaissance de croquis, réécrite ici :
//! - rééchantillonnage à pas fixe, puis coins par la méthode ShortStraw (Wolin, Eoff, Hammond 2008,
//!   https://doi.org/10.2312/SBM/SBM08/033-040) : un coin est un point où la « corde » sur ±3 pas
//!   est nettement plus courte que la médiane ;
//! - un trait fermé sans coin net qui colle à une ellipse ajustée sur ses moments → ellipse ;
//! - un trait fermé à 3 ou 4 coins → triangle, rectangle (angles proches de 90°) ou losange ;
//! - un trait ouvert presque droit → segment ; un segment terminé par un « V » → flèche.
//!
//! Le doute ne construit rien : `None`, et l'encre reste telle quelle.

use fano_core::{FormeKind, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct FormeReconnue {
    pub kind: FormeKind,
    /// Comme dans `fano_core::Forme::points`.
    pub points: Vec<Point>,
}

const PAS_MIN: f32 = 2.0;

/// Reconnaît la forme d'un trait unique.
pub fn reconnaitre_forme(trace: &[Point]) -> Option<FormeReconnue> {
    if trace.len() < 3 {
        return None;
    }
    let longueur: f32 = trace.windows(2).map(|w| w[0].dist(w[1])).sum();
    let bbox = fano_core::Rect::from_points(trace.iter().copied());
    let diag = bbox.width().hypot(bbox.height());
    if longueur < 12.0 || diag < 8.0 {
        return None;
    }
    let pas = (diag / 40.0).max(PAS_MIN);
    let pts = reechantillonner(trace, pas);
    if pts.len() < 5 {
        return None;
    }
    // Une flèche en un trait fait demi-tour à sa pointe : reconnue avant le filtre ci-dessous.
    if pts[0].dist(*pts.last().unwrap()) >= diag * 0.2 {
        if let Some(f) = fleche_un_trait(&pts, &[]) {
            return Some(f);
        }
    }
    // Un rond (même irrégulier, même s'il dépasse son départ) avant le filtre des tracés libres, s'il
    // n'a pas de coins nets : un rectangle aux coins arrondis reste un rectangle.
    if let Some(e) = ellipse(&pts) {
        let coins = coins_shortstraw(&pts);
        let poly = sommets_fermes(&pts, &coins);
        let poly_net = poly.len() >= 3 && poly.len() <= 6 && ecart_polygone(&pts, &poly) < diag * 0.04;
        if !poly_net {
            return Some(e);
        }
    }
    // Une signature, un mot en cursive ou un dessin libre ne deviennent jamais une forme.
    if est_trace_libre(&pts) {
        return None;
    }
    let debut = pts[0];
    let fin = *pts.last().unwrap();
    let ferme = debut.dist(fin) < diag * 0.2;

    // Presque droit : le chemin ne fait pas plus que 8 % de plus que la corde.
    let corde = debut.dist(fin);
    if !ferme && longueur <= corde * 1.08 {
        return Some(FormeReconnue {
            kind: FormeKind::Segment,
            points: vec![debut, fin],
        });
    }

    let coins = coins_shortstraw(&pts);
    // Un rond qui dépasse son point de départ a ses bouts éloignés, mais il fait le tour : c'est un rond.
    if !ferme {
        if let Some(e) = ellipse(&pts) {
            return Some(e);
        }
    }
    if !ferme {
        // Flèche en un trait : un long segment, puis un aller-retour court (la pointe).
        if let Some(f) = fleche_un_trait(&pts, &coins) {
            return Some(f);
        }
        // Deux bras partant d'un sommet (« < », branche d'arbre), même un peu courbés.
        if let Some(f) = deux_bras(&pts) {
            return Some(f);
        }
        // Ligne brisée nette.
        if coins.len() >= 3 && coins.len() <= 8 {
            let sommets: Vec<Point> = coins.iter().map(|&i| pts[i]).collect();
            if ecart_polyligne(&pts, &sommets) < diag * 0.05 {
                return Some(FormeReconnue {
                    kind: FormeKind::Polyligne,
                    points: sommets,
                });
            }
        }
        return None;
    }

    // Fermé : ellipse ?
    if let Some(e) = ellipse(&pts) {
        // Un polygone net a des coins nets ; une ellipse colle à son ajustement.
        let poly = sommets_fermes(&pts, &coins);
        let poly_net = poly.len() >= 3 && poly.len() <= 6 && ecart_polygone(&pts, &poly) < diag * 0.04;
        if !poly_net {
            return Some(e);
        }
    }
    let sommets = sommets_fermes(&pts, &coins);
    if sommets.len() < 3 || ecart_polygone(&pts, &sommets) > diag * 0.08 {
        return None;
    }
    let kind = match sommets.len() {
        3 => FormeKind::Triangle,
        4 => {
            let droits = (0..4).all(|i| {
                let a = sommets[(i + 3) % 4];
                let b = sommets[i];
                let c = sommets[(i + 1) % 4];
                let cos = (a - b).norm().dot((c - b).norm());
                cos.abs() < 0.26 // ±15° autour de 90°
            });
            if droits {
                // Rectangle redressé s'il est presque aligné aux axes.
                let cote = sommets[1] - sommets[0];
                let angle = cote.y.atan2(cote.x).to_degrees().rem_euclid(90.0);
                if !(10.0..=80.0).contains(&angle) {
                    let r = fano_core::Rect::from_points(sommets.iter().copied());
                    return Some(FormeReconnue {
                        kind: FormeKind::Rectangle,
                        points: vec![
                            r.min,
                            Point::new(r.max.x, r.min.y),
                            r.max,
                            Point::new(r.min.x, r.max.y),
                        ],
                    });
                }
                FormeKind::Rectangle
            } else if est_losange(&sommets) {
                FormeKind::Losange
            } else {
                FormeKind::Polygone
            }
        }
        _ => FormeKind::Polygone,
    };
    Some(FormeReconnue {
        kind,
        points: sommets,
    })
}

/// Signature ou dessin libre plutôt que forme géométrique ?
///
/// Critères classiques de classement encre/forme (cf. PaleoSketch, Paulson et Hammond 2008,
/// https://doi.org/10.1145/1378773.1378775) : une forme simple tourne au plus d'un tour complet
/// (≈ 2π), se recoupe au plus une fois (le bout qui ferme), et a peu de coins. Une signature
/// enchaîne les boucles : beaucoup de tours, plusieurs croisements, beaucoup de coins.
pub fn est_trace_libre(pts: &[Point]) -> bool {
    let (tours, croisements, inversions) = mesures(pts);
    let _ = PAS_MIN;
    tours > 1.6 || croisements >= 2 || inversions >= 3
}

/// (tours cumulés de la direction, auto-intersections, changements de sens de rotation marqués).
pub fn mesures(pts: &[Point]) -> (f32, usize, usize) {
    // Sur un tracé grossier (1/20 de la diagonale) : les tremblements de la main ne comptent pas.
    let r = fano_core::Rect::from_points(pts.iter().copied());
    let grossier = reechantillonner(pts, (r.width().hypot(r.height()) / 20.0).max(PAS_MIN));
    let pts = &grossier[..];
    let mut total = 0.0;
    // Les virages, regroupés par sens de rotation ; une ondulation de la main ne fait pas un virage :
    // seuls comptent les virages d'au moins 0,8 rad (une boucle dans l'autre sens).
    let mut virages: Vec<f32> = Vec::new();
    for w in pts.windows(3) {
        let (a, b) = (w[1] - w[0], w[2] - w[1]);
        if a.len() < 1e-3 || b.len() < 1e-3 {
            continue;
        }
        let ang = a.cross(b).atan2(a.dot(b));
        total += ang.abs();
        if ang.abs() < 0.05 {
            continue;
        }
        match virages.last_mut() {
            Some(v) if v.signum() == ang.signum() => *v += ang,
            _ => virages.push(ang),
        }
    }
    let mut grands: Vec<f32> = Vec::new();
    for v in virages.into_iter().filter(|v| v.abs() >= 0.8) {
        match grands.last_mut() {
            Some(g) if g.signum() == v.signum() => *g += v,
            _ => grands.push(v),
        }
    }
    let inversions = grands.len().saturating_sub(1);
    let n = pts.len();
    let mut croisements = 0;
    for i in 0..n.saturating_sub(1) {
        for j in (i + 2)..n.saturating_sub(1) {
            // Le bout qui rejoint le début (forme fermée) ne compte pas.
            if i * 8 < n && j * 8 > 7 * n {
                continue;
            }
            if secants(pts[i], pts[i + 1], pts[j], pts[j + 1]) {
                croisements += 1;
            }
        }
    }
    (total / std::f32::consts::TAU, croisements, inversions)
}

fn secants(a: Point, b: Point, c: Point, d: Point) -> bool {
    let o = |p: Point, q: Point, r: Point| (q - p).cross(r - p);
    let (d1, d2, d3, d4) = (o(a, b, c), o(a, b, d), o(c, d, a), o(c, d, b));
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

/// Deux bras à peu près droits qui partent d'un même sommet (« < », « > », « ∧ ») : une branche
/// d'arbre. Le sommet est le point le plus éloigné de la corde ; chaque bras peut être un peu courbé
/// (chemin ≤ 1,2 × sa corde), l'angle entre eux entre 20° et 150°.
fn deux_bras(pts: &[Point]) -> Option<FormeReconnue> {
    let (a, b) = (pts[0], *pts.last()?);
    let corde = b - a;
    let (iv, _) = pts
        .iter()
        .enumerate()
        .map(|(i, p)| (i, corde.norm().cross(*p - a).abs()))
        .max_by(|x, y| x.1.total_cmp(&y.1))?;
    if iv < 2 || iv + 2 >= pts.len() {
        return None;
    }
    let v = pts[iv];
    let chemin = |s: &[Point]| s.windows(2).map(|w| w[0].dist(w[1])).sum::<f32>();
    let (l1, l2) = (a.dist(v), v.dist(b));
    if l1 < 20.0 || l2 < 20.0 || chemin(&pts[..=iv]) > l1 * 1.2 || chemin(&pts[iv..]) > l2 * 1.2 {
        return None;
    }
    let cos = (a - v).norm().dot((b - v).norm());
    let angle = cos.clamp(-1.0, 1.0).acos().to_degrees();
    (20.0..=150.0).contains(&angle).then(|| FormeReconnue {
        kind: FormeKind::Polyligne,
        points: vec![a, v, b],
    })
}

/// Une pointe de flèche dessinée en un second trait près du bout d'un segment.
/// Rend vrai si `pointe` (un « V » ou un « > ») touche `bout`.
pub fn est_pointe(pointe: &[Point], bout: Point, longueur_segment: f32) -> bool {
    if pointe.len() < 3 {
        return false;
    }
    let bbox = fano_core::Rect::from_points(pointe.iter().copied());
    let taille = bbox.width().hypot(bbox.height());
    let proche = pointe.iter().any(|p| p.dist(bout) < (taille * 0.5).max(10.0));
    proche && taille < longueur_segment * 0.6 && taille > 4.0
}

fn reechantillonner(trace: &[Point], pas: f32) -> Vec<Point> {
    let mut res = vec![trace[0]];
    let mut reste = 0.0;
    let mut prec = trace[0];
    for &p in &trace[1..] {
        let mut d = prec.dist(p);
        while reste + d >= pas {
            let t = (pas - reste) / d;
            let q = prec.lerp(p, t);
            res.push(q);
            d -= pas - reste;
            prec = q;
            reste = 0.0;
        }
        reste += d;
        prec = p;
    }
    if res
        .last()
        .is_some_and(|l| l.dist(*trace.last().unwrap()) > pas * 0.3)
    {
        res.push(*trace.last().unwrap());
    }
    res
}

fn coins_shortstraw(pts: &[Point]) -> Vec<usize> {
    const W: usize = 3;
    let n = pts.len();
    let mut coins = vec![0];
    if n <= 2 * W {
        coins.push(n - 1);
        return coins;
    }
    let pailles: Vec<f32> = (0..n)
        .map(|i| {
            if i < W || i + W >= n {
                f32::INFINITY
            } else {
                pts[i - W].dist(pts[i + W])
            }
        })
        .collect();
    let mut tri: Vec<f32> = pailles.iter().copied().filter(|v| v.is_finite()).collect();
    tri.sort_by(f32::total_cmp);
    let seuil = tri[tri.len() / 2] * 0.95;
    let mut i = W;
    while i + W < n {
        if pailles[i] < seuil {
            let mut j = i;
            let mut min = i;
            while j + W < n && pailles[j] < seuil {
                if pailles[j] < pailles[min] {
                    min = j;
                }
                j += 1;
            }
            coins.push(min);
            i = j;
        } else {
            i += 1;
        }
    }
    coins.push(n - 1);
    // Retire les coins presque alignés avec leurs voisins.
    let mut k = 1;
    while k + 1 < coins.len() {
        let (a, b, c) = (pts[coins[k - 1]], pts[coins[k]], pts[coins[k + 1]]);
        let cos = (a - b).norm().dot((c - b).norm());
        if cos < -0.88 {
            coins.remove(k);
        } else {
            k += 1;
        }
    }
    coins
}

fn sommets_fermes(pts: &[Point], coins: &[usize]) -> Vec<Point> {
    let mut s: Vec<Point> = coins.iter().map(|&i| pts[i]).collect();
    // Le début et la fin d'un trait fermé se rejoignent : un seul sommet, s'ils sont un coin.
    if s.len() >= 2 {
        let fin = s.pop().unwrap();
        let debut = s[0];
        s[0] = debut.lerp(fin, 0.5);
        // Le point de départ n'est pas toujours un coin : retire-le s'il est aligné.
        if s.len() >= 4 {
            let (a, b, c) = (s[s.len() - 1], s[0], s[1]);
            if (a - b).norm().dot((c - b).norm()) < -0.85 {
                s.remove(0);
            }
        }
    }
    // Fusionne les sommets trop proches.
    let diag = {
        let r = fano_core::Rect::from_points(pts.iter().copied());
        r.width().hypot(r.height())
    };
    let mut i = 0;
    while s.len() > 3 && i < s.len() {
        let j = (i + 1) % s.len();
        if s[i].dist(s[j]) < diag * 0.12 {
            let m = s[i].lerp(s[j], 0.5);
            s[i] = m;
            s.remove(j);
        } else {
            i += 1;
        }
    }
    s
}

fn dist_segment(p: Point, a: Point, b: Point) -> f32 {
    let ab = b - a;
    let l2 = ab.dot(ab);
    if l2 < 1e-9 {
        return p.dist(a);
    }
    let t = ((p - a).dot(ab) / l2).clamp(0.0, 1.0);
    p.dist(a + ab * t)
}

fn ecart_polygone(pts: &[Point], sommets: &[Point]) -> f32 {
    let n = sommets.len();
    let total: f32 = pts
        .iter()
        .map(|&p| {
            (0..n)
                .map(|i| dist_segment(p, sommets[i], sommets[(i + 1) % n]))
                .fold(f32::INFINITY, f32::min)
        })
        .sum();
    total / pts.len() as f32
}

fn ecart_polyligne(pts: &[Point], sommets: &[Point]) -> f32 {
    let total: f32 = pts
        .iter()
        .map(|&p| {
            sommets
                .windows(2)
                .map(|w| dist_segment(p, w[0], w[1]))
                .fold(f32::INFINITY, f32::min)
        })
        .sum();
    total / pts.len() as f32
}

fn est_losange(s: &[Point]) -> bool {
    let cotes: Vec<f32> = (0..4).map(|i| s[i].dist(s[(i + 1) % 4])).collect();
    let moy = cotes.iter().sum::<f32>() / 4.0;
    cotes.iter().all(|c| (c - moy).abs() < moy * 0.2)
}

/// Ajuste une ellipse alignée aux axes (ou un cercle) sur les moments du tracé ; `None` si l'écart
/// relatif moyen dépasse 12 %.
fn ellipse(pts: &[Point]) -> Option<FormeReconnue> {
    let n = pts.len() as f32;
    let c = pts.iter().fold(Point::new(0.0, 0.0), |a, &p| a + p) * (1.0 / n);
    let r = fano_core::Rect::from_points(pts.iter().copied());
    let (mut rx, mut ry) = (r.width() / 2.0, r.height() / 2.0);
    if rx < 2.0 || ry < 2.0 {
        return None;
    }
    // Presque rond → cercle (un rond fait à main levée est rarement parfait : jusqu'à 25 % d'écart).
    if (rx - ry).abs() < 0.25 * rx.max(ry) {
        let m = (rx + ry) / 2.0;
        rx = m;
        ry = m;
    }
    let c = Point::new(
        (r.min.x + r.max.x) / 2.0 * 0.5 + c.x * 0.5,
        (r.min.y + r.max.y) / 2.0 * 0.5 + c.y * 0.5,
    );
    let ecart: f32 = pts
        .iter()
        .map(|p| {
            let d = ((p.x - c.x) / rx).hypot((p.y - c.y) / ry);
            (d - 1.0).abs()
        })
        .sum::<f32>()
        / n;
    // Le tour doit être presque complet.
    let mut angle_total = 0.0;
    for w in pts.windows(2) {
        let a = (w[0] - c).y.atan2((w[0] - c).x);
        let b = (w[1] - c).y.atan2((w[1] - c).x);
        let mut d = b - a;
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        if d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        angle_total += d;
    }
    (ecart < 0.18 && angle_total.abs() > std::f32::consts::TAU * 0.8).then(|| FormeReconnue {
        kind: FormeKind::Ellipse,
        points: vec![c, Point::new(rx, ry)],
    })
}

fn fleche_un_trait(pts: &[Point], _coins: &[usize]) -> Option<FormeReconnue> {
    // Segment principal = le plus long début presque droit ; puis la pointe revient en arrière.
    let mut chemin = 0.0;
    let mut corps_fin = 0;
    for k in 1..pts.len() {
        chemin += pts[k - 1].dist(pts[k]);
        let corde = pts[0].dist(pts[k]);
        if chemin <= corde * 1.06 + 2.0 {
            corps_fin = k;
        } else if chemin > corde * 1.3 {
            break;
        }
    }
    if corps_fin < 3 || corps_fin + 2 >= pts.len() {
        return None;
    }
    let a = pts[0];
    let b = pts[corps_fin];
    let corps = a.dist(b);
    let chemin_corps: f32 = pts[..=corps_fin].windows(2).map(|w| w[0].dist(w[1])).sum();
    if chemin_corps > corps * 1.12 {
        return None;
    }
    let reste = &pts[corps_fin..];
    let reste_taille = {
        let r = fano_core::Rect::from_points(reste.iter().copied());
        r.width().hypot(r.height())
    };
    if reste_taille < corps * 0.08 || reste_taille > corps * 0.6 {
        return None;
    }
    // La pointe reste près du bout du corps.
    if reste.iter().any(|p| p.dist(b) > corps * 0.45) {
        return None;
    }
    // Et elle part vers l'arrière (contre la direction du corps).
    let dir = (b - a).norm();
    let recul = reste
        .iter()
        .map(|p| (*p - b).dot(dir))
        .fold(f32::INFINITY, f32::min);
    (recul < -corps * 0.05).then(|| FormeReconnue {
        kind: FormeKind::Fleche,
        points: vec![a, b],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(sommets: &[(f32, f32)], par_cote: usize, ferme: bool) -> Vec<Point> {
        let s: Vec<Point> = sommets.iter().map(|&(x, y)| Point::new(x, y)).collect();
        let n = if ferme { s.len() } else { s.len() - 1 };
        let mut v = Vec::new();
        for i in 0..n {
            let a = s[i];
            let b = s[(i + 1) % s.len()];
            for k in 0..par_cote {
                // Petit tremblement de la main.
                let t = k as f32 / par_cote as f32;
                let j = ((i * 31 + k * 17) % 7) as f32 * 0.15 - 0.45;
                v.push(a.lerp(b, t) + Point::new(j, -j));
            }
        }
        v.push(if ferme { s[0] } else { *s.last().unwrap() });
        v
    }

    #[test]
    fn segment() {
        let f = reconnaitre_forme(&poly(&[(0.0, 0.0), (200.0, 30.0)], 40, false)).unwrap();
        assert_eq!(f.kind, FormeKind::Segment);
    }

    #[test]
    fn triangle() {
        let f = reconnaitre_forme(&poly(&[(0.0, 0.0), (200.0, 10.0), (90.0, 160.0)], 30, true)).unwrap();
        assert_eq!(f.kind, FormeKind::Triangle);
        assert_eq!(f.points.len(), 3);
    }

    #[test]
    fn rectangle_redresse() {
        let f = reconnaitre_forme(&poly(
            &[(0.0, 0.0), (220.0, 4.0), (218.0, 120.0), (-2.0, 118.0)],
            30,
            true,
        ))
        .unwrap();
        assert_eq!(f.kind, FormeKind::Rectangle);
        assert!((f.points[0].y - f.points[1].y).abs() < 1e-3);
    }

    #[test]
    fn cercle() {
        let pts: Vec<Point> = (0..=80)
            .map(|i| {
                let a = i as f32 / 80.0 * std::f32::consts::TAU;
                let r = 60.0 + (i % 5) as f32 * 0.6;
                Point::new(100.0 + r * a.cos(), 100.0 + r * a.sin())
            })
            .collect();
        let f = reconnaitre_forme(&pts).unwrap();
        assert_eq!(f.kind, FormeKind::Ellipse);
        assert!(
            (f.points[1].x - f.points[1].y).abs() < 1e-3,
            "un cercle : rx = ry"
        );
    }

    #[test]
    fn fleche_en_un_trait() {
        let mut pts = poly(&[(0.0, 100.0), (200.0, 100.0)], 50, false);
        pts.extend(poly(&[(200.0, 100.0), (180.0, 85.0)], 6, false));
        pts.extend(poly(&[(180.0, 85.0), (200.0, 100.0), (180.0, 115.0)], 6, false));
        let f = reconnaitre_forme(&pts).unwrap();
        assert_eq!(f.kind, FormeKind::Fleche);
    }

    #[test]
    fn gribouillis_rien() {
        let pts: Vec<Point> = (0..60)
            .map(|i| Point::new((i * 37 % 50) as f32 * 3.0, (i * 53 % 41) as f32 * 3.0))
            .collect();
        assert!(
            reconnaitre_forme(&pts)
                .is_none_or(|f| f.kind == FormeKind::Polygone || f.kind == FormeKind::Polyligne)
        );
    }
}

#[cfg(test)]
mod tests_signature {
    use super::*;

    /// Une signature : boucles et lettres liées, sur une ligne.
    fn signature() -> Vec<Point> {
        (0..=300)
            .map(|i| {
                let t = i as f32 / 300.0 * 9.0;
                // Boucles cursives (trochoïde) qui avancent, avec un paraphe.
                Point::new(
                    30.0 * t - 22.0 * (2.0 * t).sin(),
                    40.0 * (2.0 * t).cos() + 10.0 * (0.7 * t).sin(),
                )
            })
            .collect()
    }

    #[test]
    fn une_signature_n_est_pas_une_forme() {
        let s = signature();
        let (tours, crois, inv) = mesures(&reechantillonner(&s, 4.0));
        assert!(
            est_trace_libre(&reechantillonner(&s, 4.0)),
            "tours {tours} croisements {crois} inversions {inv}"
        );
        assert!(reconnaitre_forme(&s).is_none());
    }

    #[test]
    fn un_gribouillis_en_s_n_est_pas_une_forme() {
        let s: Vec<Point> = (0..=120)
            .map(|i| {
                let t = i as f32 / 120.0 * std::f32::consts::TAU * 1.5;
                Point::new(60.0 * t.sin(), 30.0 * t)
            })
            .collect();
        assert!(
            reconnaitre_forme(&s)
                .is_none_or(|f| f.kind != FormeKind::Ellipse && f.kind != FormeKind::Triangle)
        );
    }

    #[test]
    fn les_formes_restent_des_formes() {
        for (nom, pts) in [
            (
                "cercle",
                (0..=80)
                    .map(|i| {
                        let a = i as f32 / 80.0 * std::f32::consts::TAU;
                        Point::new(100.0 + 60.0 * a.cos(), 100.0 + 60.0 * a.sin())
                    })
                    .collect::<Vec<_>>(),
            ),
            (
                "triangle",
                vec![
                    Point::new(0.0, 0.0),
                    Point::new(100.0, 5.0),
                    Point::new(40.0, 90.0),
                    Point::new(1.0, 1.0),
                ],
            ),
        ] {
            let r = reechantillonner(&pts, 3.0);
            assert!(!est_trace_libre(&r), "{nom} : {:?}", mesures(&r));
            assert!(reconnaitre_forme(&pts).is_some(), "{nom}");
        }
    }
}

#[cfg(test)]
mod tests_branches {
    use super::*;

    #[test]
    fn branche_un_peu_courbee() {
        // La branche de B relevée sur le Pad 7 (2026-11-08) : deux bras légèrement concaves.
        let mut pts = Vec::new();
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            pts.push(Point::new(
                300.0 - 110.0 * t,
                100.0 + 70.0 * t + 8.0 * (t * std::f32::consts::PI).sin(),
            ));
        }
        for i in 1..=20 {
            let t = i as f32 / 20.0;
            pts.push(Point::new(
                190.0 + 115.0 * t,
                170.0 + 60.0 * t - 8.0 * (t * std::f32::consts::PI).sin(),
            ));
        }
        let f = reconnaitre_forme(&pts).expect("une branche");
        assert_eq!(f.kind, FormeKind::Polyligne);
        assert_eq!(f.points.len(), 3);
    }
}

#[cfg(test)]
mod tests_ronds {
    use super::*;

    /// Un rond à main levée : rayon irrégulier (±8 %), légèrement ovale, qui dépasse son départ.
    fn rond(ovale: f32, tours: f32) -> Vec<Point> {
        (0..=120)
            .map(|i| {
                let a = i as f32 / 120.0 * std::f32::consts::TAU * tours;
                let r = 80.0 * (1.0 + 0.08 * (3.0 * a).sin() + 0.03 * (7.0 * a).cos());
                Point::new(200.0 + r * ovale * a.cos(), 200.0 + r * a.sin())
            })
            .collect()
    }

    #[test]
    fn un_rond_devient_un_cercle() {
        for (ovale, tours) in [(1.0, 1.0), (1.18, 1.0), (0.85, 1.12), (1.1, 1.2)] {
            let f =
                reconnaitre_forme(&rond(ovale, tours)).unwrap_or_else(|| panic!("rien pour {ovale} {tours}"));
            assert_eq!(f.kind, FormeKind::Ellipse, "{ovale} {tours}");
            assert!(
                (f.points[1].x - f.points[1].y).abs() < 1e-3,
                "cercle attendu pour {ovale} {tours}"
            );
        }
    }

    #[test]
    fn un_ovale_franc_reste_une_ellipse() {
        let f = reconnaitre_forme(&rond(1.6, 1.0)).unwrap();
        assert_eq!(f.kind, FormeKind::Ellipse);
        assert!((f.points[1].x - f.points[1].y).abs() > 10.0);
    }
}

#[cfg(test)]
mod tests_peu_de_points {
    use super::*;

    /// Un rectangle tracé avec très peu de points (événements espacés) : relevé sur la tablette, 2026-11-08.
    #[test]
    fn rectangle_cinq_points() {
        let pts: Vec<Point> = [
            (600.0, 1500.0),
            (1300.0, 1505.0),
            (1302.0, 1900.0),
            (598.0, 1898.0),
            (600.0, 1502.0),
        ]
        .iter()
        .map(|&(x, y)| Point::new(x * 0.5, y * 0.5))
        .collect();
        let f = reconnaitre_forme(&pts).expect("une forme");
        assert_eq!(f.kind, FormeKind::Rectangle, "{f:?}");
    }
}

#[cfg(test)]
mod tests_tablette {
    use super::*;

    /// Le triangle tracé sur le Pad 7 (2026-11-08) : 3 côtés de 10 points, fermé.
    #[test]
    fn triangle_de_la_tablette() {
        let tri = [
            (700.0, 1750.0),
            (1150.0, 1760.0),
            (780.0, 1350.0),
            (700.0, 1748.0),
        ];
        let mut pts = Vec::new();
        for i in 0..3 {
            let ((x1, y1), (x2, y2)) = (tri[i], tri[i + 1]);
            for k in 0..10 {
                let t = k as f32 / 10.0;
                pts.push(Point::new((x1 + (x2 - x1) * t) / 2.5, (y1 + (y2 - y1) * t) / 2.5));
            }
        }
        pts.push(Point::new(700.0 / 2.5, 1748.0 / 2.5));
        let r = reechantillonner(&pts, 3.0);
        eprintln!("mesures {:?}", mesures(&r));
        let f = reconnaitre_forme(&pts);
        assert_eq!(f.map(|f| f.kind), Some(FormeKind::Triangle));
    }
}
