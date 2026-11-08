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
        ranger_mots(boites, l, h_typ);
    }
    lignes.sort_by(|a, b| a.bbox.min.y.total_cmp(&b.bbox.min.y));
    lignes
}

fn ranger_mots(boites: &[Rect], l: &mut Ligne, h_typ: f32) {
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

/// Hauteur typique d'un signe : la médiane des hauteurs des traits ni plats ni minuscules (barres, points).
fn hauteur_typique(boites: &[Rect]) -> f32 {
    let grand = boites
        .iter()
        .filter(|b| b.is_valid())
        .map(|b| b.height().max(b.width()))
        .fold(0.0, f32::max);
    let mut h: Vec<f32> = boites
        .iter()
        .filter(|b| b.is_valid() && b.height() > 0.3 * b.width() && b.height().max(b.width()) > 0.1 * grand)
        .map(|b| b.height().max(2.0))
        .collect();
    h.sort_by(f32::total_cmp);
    h.get(h.len() / 2).copied().unwrap_or(20.0)
}

/// Les barres de fraction : un trait plat, assez large, avec de l'encre (pas seulement d'autres barres)
/// juste au-dessus et juste au-dessous, dans sa largeur. Rend, pour chaque trait, ses voisins du dessus
/// et du dessous s'il est une barre.
pub fn barres_de_fraction(boites: &[Rect]) -> Vec<Option<(Vec<usize>, Vec<usize>)>> {
    let h = hauteur_typique(boites);
    let plat = |b: &Rect| b.width() > 0.0 && b.height() <= 0.3 * b.width();
    (0..boites.len())
        .map(|i| {
            let bar = boites[i];
            // Une barre est fine (un mot cursif allongé est plat, mais il a la hauteur d'une lettre).
            if !bar.is_valid() || !plat(&bar) || bar.width() < 0.8 * h || bar.height() > 0.35 * h {
                return None;
            }
            let dans = |b: &Rect| {
                b.center().x >= bar.min.x - 0.05 * bar.width()
                    && b.center().x <= bar.max.x + 0.05 * bar.width()
            };
            let cy = bar.center().y;
            let dessus: Vec<usize> = (0..boites.len())
                .filter(|&j| {
                    let b = &boites[j];
                    j != i
                        && b.is_valid()
                        && dans(b)
                        && b.max.y <= cy + 0.15 * h
                        && bar.min.y - b.max.y < 0.8 * h
                })
                .collect();
            let dessous: Vec<usize> = (0..boites.len())
                .filter(|&j| {
                    let b = &boites[j];
                    j != i
                        && b.is_valid()
                        && dans(b)
                        && b.min.y >= cy - 0.15 * h
                        && b.min.y - bar.max.y < 0.8 * h
                })
                .collect();
            let encre = |v: &[usize]| {
                v.iter()
                    .any(|&j| !plat(&boites[j]) && boites[j].height() >= 0.3 * h)
            };
            // Un soulignement n'est pas une barre de fraction : sous lui (ou au-dessus), une ligne de texte
            // continue au-delà de ses bouts, sur la même ligne de base — un trait lié qui traverse le bout, ou
            // des mots juste à côté. Un signe qui dépasse seul (le `3` de `3x − 1`) ou un exposant n'en est pas ;
            // une phrase qui passe à la hauteur de la barre non plus (fraction dans le texte).
            let continue_ = |v: &[usize]| {
                if v.is_empty() {
                    return false;
                }
                let bv = v.iter().fold(Rect::NOTHING, |r, &j| r.union(boites[j]));
                // Un mot pris dans la largeur de la barre qui la déborde nettement : du texte.
                if v.iter()
                    .any(|&j| boites[j].min.x < bar.min.x - 0.5 * h || boites[j].max.x > bar.max.x + 0.5 * h)
                {
                    return true;
                }
                let mut largeur = 0.0;
                for k in 0..boites.len() {
                    let b = &boites[k];
                    if k == i || v.contains(&k) || !b.is_valid() || plat(b) || b.height() < 0.5 * h {
                        continue;
                    }
                    let meme_base = (b.max.y - bv.max.y).abs() < 0.35 * h;
                    let a_hauteur_de_barre = b.min.y < cy && b.max.y > cy;
                    let traverse = (b.min.x < bar.min.x && b.max.x > bar.min.x + 0.1 * h)
                        || (b.max.x > bar.max.x && b.min.x < bar.max.x - 0.1 * h);
                    let a_cote = b.max.x > bar.min.x - 1.5 * h && b.min.x < bar.max.x + 1.5 * h;
                    if meme_base && !a_hauteur_de_barre {
                        if traverse {
                            return true;
                        }
                        if a_cote && !dans(b) {
                            largeur += b.width();
                        }
                    }
                }
                largeur >= 1.5 * h
            };
            (encre(&dessus) && encre(&dessous) && !continue_(&dessus) && !continue_(&dessous))
                .then_some((dessus, dessous))
        })
        .collect()
}

/// Les lignes d'une page de cours, en gardant d'un seul tenant ce qui s'écrit sur plusieurs hauteurs :
/// une fraction (numérateur, barre, dénominateur), `lim` et `x → a` dessous, un exposant posé haut, les
/// rangées d'une matrice entre ses grandes parenthèses. Le texte ordinaire reste ligne par ligne.
pub fn grouper_blocs(boites: &[Rect]) -> Vec<Ligne> {
    let mut lignes = grouper_lignes(boites);
    let h = hauteur_typique(boites);
    let ligne_de = |lignes: &[Ligne], t: usize| lignes.iter().position(|l| l.traits.contains(&t));
    let fusionner = |lignes: &mut Vec<Ligne>, a: usize, b: usize| {
        let (a, b) = (a.min(b), a.max(b));
        let l = lignes.remove(b);
        lignes[a].bbox = lignes[a].bbox.union(l.bbox);
        lignes[a].traits.extend(l.traits);
    };
    // 1. Les fractions relient les lignes de leur barre, de leur numérateur et de leur dénominateur.
    for (i, f) in barres_de_fraction(boites).into_iter().enumerate() {
        let Some((dessus, dessous)) = f else { continue };
        for j in dessus.into_iter().chain(dessous) {
            if let (Some(a), Some(b)) = (ligne_de(&lignes, i), ligne_de(&lignes, j)) {
                if a != b {
                    fusionner(&mut lignes, a, b);
                }
            }
        }
    }
    // 2. Une ligne contenue dans la hauteur d'une autre (les rangées d'une matrice entre ses parenthèses),
    //    ou une toute petite ligne collée à une autre (exposant, borne, `x → 0` sous `lim`).
    loop {
        let mut paire = None;
        'p: for a in 0..lignes.len() {
            for b in 0..lignes.len() {
                if a == b {
                    continue;
                }
                let (la, lb) = (&lignes[a].bbox, &lignes[b].bbox);
                let recouvre_x = la.max.x.min(lb.max.x) - la.min.x.max(lb.min.x) >= 0.0;
                if !recouvre_x {
                    continue;
                }
                let recouvre_y = (la.max.y.min(lb.max.y) - la.min.y.max(lb.min.y)).max(0.0);
                let dedans = lb.center().y > la.min.y && lb.center().y < la.max.y;
                let contenue = dedans && recouvre_y >= 0.6 * lb.height() && la.height() > 1.5 * lb.height();
                let ecart = (la.min.y.max(lb.min.y) - la.max.y.min(lb.max.y)).max(0.0);
                // Un exposant ou une borne : un ou deux signes, petits, collés à la ligne (pas un mot court
                // écrit sous une phrase : il est plus large, ou plus loin).
                let petite = lignes[b].traits.len() <= 3
                    && lb.height() < 0.8 * h
                    && lb.width() < 1.5 * h
                    && lb.width() < 0.5 * la.width()
                    && lb.min.x >= la.min.x - 0.5 * h
                    && lb.max.x <= la.max.x + 0.5 * h;
                if contenue || (petite && ecart < 0.4 * h) {
                    paire = Some((a, b));
                    break 'p;
                }
            }
        }
        let Some((a, b)) = paire else { break };
        fusionner(&mut lignes, a, b);
    }
    for l in &mut lignes {
        ranger_mots(boites, l, h);
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
mod tests_page_de_cours {
    use super::*;
    use fano_core::Point;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Point::new(x, y), w, h)
    }

    /// Une ligne de texte cursif : des mots d'un trait, hauteur `h`, à la hauteur `y`.
    fn phrase(y: f32, x0: f32, mots: &[f32]) -> Vec<Rect> {
        let mut x = x0;
        mots.iter()
            .map(|&w| {
                let b = r(x, y, w, 20.0);
                x += w + 12.0;
                b
            })
            .collect()
    }

    /// Une page de cours comme celles d'Antsa : titre souligné collé à la ligne suivante, paragraphe serré,
    /// fraction dans une phrase, fraction seule en display. Le texte reste ligne par ligne ; chaque
    /// fraction reste d'un seul tenant.
    #[test]
    fn une_page_de_cours() {
        let mut b: Vec<Rect> = Vec::new();
        // Titre « 1- Les fractions » et son soulignement, puis une phrase serrée dessous (écart 0,5 h).
        b.extend(phrase(0.0, 0.0, &[30.0, 60.0, 110.0]));
        let soulignement = b.len();
        b.push(r(0.0, 24.0, 210.0, 3.0));
        b.extend(phrase(37.0, 0.0, &[40.0, 90.0, 70.0, 120.0]));
        // Paragraphe serré : deux lignes à 30 pt d'intervalle.
        b.extend(phrase(67.0, 0.0, &[80.0, 50.0, 140.0]));
        b.extend(phrase(97.0, 0.0, &[60.0, 100.0]));
        // Une phrase avec une fraction dedans : « on a 1/2 de la part » (barre au milieu de la ligne).
        let debut = b.len();
        b.extend(phrase(140.0, 0.0, &[30.0, 20.0]));
        b.push(r(70.0, 128.0, 8.0, 14.0)); // 1
        b.push(r(64.0, 145.0, 22.0, 2.0)); // barre
        b.push(r(69.0, 150.0, 10.0, 14.0)); // 2
        b.extend(phrase(140.0, 100.0, &[30.0, 25.0, 60.0]));
        let fin = b.len();
        // Fraction seule (display) : numérateur, barre, dénominateur sur trois hauteurs.
        let display = b.len();
        b.extend([
            r(110.0, 200.0, 40.0, 20.0),
            r(100.0, 226.0, 60.0, 3.0),
            r(105.0, 235.0, 50.0, 20.0),
        ]);
        let lignes = grouper_blocs(&b);
        let ligne_de = |t: usize| lignes.iter().position(|l| l.traits.contains(&t)).unwrap();
        // Le titre (et son soulignement) n'est pas collé à la phrase d'en dessous.
        assert_eq!(ligne_de(0), ligne_de(soulignement));
        assert_ne!(ligne_de(0), ligne_de(soulignement + 1));
        // Le paragraphe reste en deux lignes.
        assert_ne!(ligne_de(soulignement + 5), ligne_de(soulignement + 8));
        // La phrase et sa fraction : une seule ligne.
        for t in debut..fin {
            assert_eq!(ligne_de(t), ligne_de(debut), "trait {t}");
        }
        // La fraction display : d'un seul tenant, à part.
        assert_eq!(ligne_de(display), ligne_de(display + 2));
        assert_ne!(ligne_de(display), ligne_de(debut));
        assert_eq!(
            lignes.len(),
            6,
            "{:#?}",
            lignes.iter().map(|l| l.bbox).collect::<Vec<_>>()
        );
        assert!(
            barres_de_fraction(&b)[soulignement].is_none(),
            "un soulignement n'est pas une barre de fraction"
        );
    }
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
